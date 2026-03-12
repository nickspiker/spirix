// spirix_addsub — Combinational add/subtract for Spirix scalars with edge cases
//
// Floor-only (no rounding), single-path design.
// Parameterized: FRAC_BITS in {8..256}, EXP_BITS in {2..256}.
//
// Architecture: edge detect -> left-shift big -> add -> CLZ -> normalize -> extract (floor).
//   Matches the Rust reference model (scalar_add_scalar / scalar_subtract_scalar) exactly.
//   Edge cases handled: undefined passthrough, transfinite+transfinite,
//   vanished+vanished, transfinite+finite, vanished+normal, zero+anything.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction: signed two's complement, N1-normalized (top two bits differ).
// Exponent: signed two's complement. AMBIGUOUS_EXP = -2^(EXP_BITS-1)
// encodes zero/exploded/vanished/infinity/undefined.

module spirix_addsub #(
    parameter FRAC_BITS = 32,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    input  wire                         sub,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    // 2x width intermediate — matches Rust i16/i32/i64 promotion.
    localparam INT_BITS   = 2 * FRAC_BITS;
    localparam SHIFT_BITS = $clog2(INT_BITS);
    localparam AMBIG_EXP  = -(1 <<< (EXP_BITS - 1));

    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};

    // Exponent calc width
    localparam ECW = (SHIFT_BITS + 1 > EXP_BITS + 1) ? SHIFT_BITS + 2 : EXP_BITS + 2;

    // Undefined prefix constants (top 8 bits, zero-padded)
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_TF  = {8'h1F, {UPAD{1'b0}}}; // ℘⬆+⬆
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_TF  = {8'hE0, {UPAD{1'b0}}}; // ℘⬆-⬆
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_P_VAN = {8'h1E, {UPAD{1'b0}}}; // ℘↓+↓
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_M_VAN = {8'hE1, {UPAD{1'b0}}}; // ℘↓-↓
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_FIN  = {8'h1C, {UPAD{1'b0}}}; // ℘⬆+
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_FIN  = {8'hE3, {UPAD{1'b0}}}; // ℘⬆-
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_P_TF  = {8'h18, {UPAD{1'b0}}}; // ℘+⬆
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_M_TF  = {8'hE7, {UPAD{1'b0}}}; // ℘-⬆

    // ========== State detection =================================================

    wire a_is_ambig = (a_exp == AMBIG_EXP[EXP_BITS-1:0]);
    wire b_is_ambig = (b_exp == AMBIG_EXP[EXP_BITS-1:0]);

    wire a_frac_zero = (a_frac == {FRAC_BITS{1'b0}});
    wire a_frac_neg1 = &a_frac;
    wire b_frac_zero = (b_frac == {FRAC_BITS{1'b0}});
    wire b_frac_neg1 = &b_frac;

    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;
    wire a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire b_n1 = (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-2]);
    wire a_n2 = ~a_n1 & (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-3]);
    wire b_n2 = ~b_n1 & (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-3]);
    wire a_top3 = (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire b_top3 = (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                  (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);

    // Rust: is_zero/is_infinite/exploded/is_transfinite check AMBIG exp.
    //       vanished/is_undefined check fraction only (no exp check).
    wire a_is_zero   = a_is_ambig & a_frac_zero;
    wire a_is_inf    = a_is_ambig & a_frac_neg1;
    wire a_exploded  = a_is_ambig & a_n1;
    wire a_transf    = a_is_inf | a_exploded;  // is_transfinite
    wire a_vanished  = a_n2;                    // fraction-only
    wire a_undef     = ~a_n0 & a_top3;          // fraction-only

    wire b_is_zero   = b_is_ambig & b_frac_zero;
    wire b_is_inf    = b_is_ambig & b_frac_neg1;
    wire b_exploded  = b_is_ambig & b_n1;
    wire b_transf    = b_is_inf | b_exploded;
    wire b_vanished  = b_n2;
    wire b_undef     = ~b_n0 & b_top3;

    wire a_is_normal  = ~a_is_ambig & a_n1;
    wire b_is_normal  = ~b_is_ambig & b_n1;
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // ========== Spirix negation of b (for sub edge cases) =======================
    //
    // Normal path: POS_HALF→(NEG_ONE, exp-1) or (NEG_SMALL if exp-1==AMBIG)
    //              NEG_ONE→(POS_HALF, exp+1)
    //              else→(-frac, exp)
    // Non-normal: top3_same (undef/zero/inf)→unchanged
    //             POS_HALF↔NEG_ONE, POS_SMALL↔NEG_SMALL (no exp change)
    //             else→wrapping_neg (no exp change)

    localparam signed [FRAC_BITS-1:0] POS_SMALL = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_SMALL = {2'b11, {(FRAC_BITS-2){1'b0}}};

    wire b_is_pos_half = (b_frac == POS_HALF);
    wire b_is_neg_one  = (b_frac == NEG_ONE);
    wire b_is_pos_small = (b_frac == POS_SMALL);
    wire b_is_neg_small = (b_frac == NEG_SMALL);

    // Normal negation (b has non-AMBIG exp)
    wire signed [EXP_BITS-1:0] b_exp_m1 = b_exp - 1'b1;
    wire b_exp_m1_ambig = (b_exp_m1 == AMBIG_EXP[EXP_BITS-1:0]);

    wire signed [FRAC_BITS-1:0] neg_b_frac_normal =
        b_is_pos_half ? (b_exp_m1_ambig ? NEG_SMALL : NEG_ONE) :
        b_is_neg_one  ? POS_HALF :
                        -b_frac;
    wire signed [EXP_BITS-1:0] neg_b_exp_normal =
        b_is_pos_half ? b_exp_m1 :
        b_is_neg_one  ? (b_exp + 1'b1) :
                        b_exp;

    // Non-normal negation (b has AMBIG exp)
    // top3_same → unchanged (zero, inf, undef don't negate)
    wire b_top3_same = (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                       (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);
    wire b_n0_local = b_frac_zero | b_frac_neg1;
    wire b_nonnorm_nochange = b_n0_local | (~b_n0_local & b_top3_same);

    wire signed [FRAC_BITS-1:0] neg_b_frac_nonnorm =
        b_nonnorm_nochange ? b_frac :
        b_is_pos_half  ? NEG_ONE :
        b_is_neg_one   ? POS_HALF :
        b_is_pos_small ? NEG_SMALL :
        b_is_neg_small ? POS_SMALL :
                         -b_frac;

    // Select normal vs non-normal negation
    wire signed [FRAC_BITS-1:0] neg_b_frac = b_is_ambig ? neg_b_frac_nonnorm : neg_b_frac_normal;
    wire signed [EXP_BITS-1:0]  neg_b_exp  = b_is_ambig ? b_exp               : neg_b_exp_normal;

    // ========== Edge case priority (matches Rust spec) ==========================
    //
    // Add:                                    Sub:
    // 1. a undef    -> a                      1. a undef    -> a
    // 2. b undef    -> b                      2. b undef    -> b
    // 3. a_tf & b_tf -> TF_PLUS_TF            3. a_tf & b_tf -> TF_MINUS_TF
    // 4. a_van & b_van -> VAN_PLUS_VAN        4. a_van & b_van -> VAN_MINUS_VAN
    // 5. a_tf       -> TF_PLUS_FIN            5. a_tf       -> TF_MINUS_FIN
    // 6. b_tf       -> FIN_PLUS_TF            6. b_tf       -> FIN_MINUS_TF
    // 7. a_van      -> b                      7. a_van      -> -b
    // 8. b_van      -> a                      8. b_van      -> a
    // 9. a_zero     -> b                      9. a_zero     -> -b
    // 10. fallback  -> a                      10. fallback  -> a

    wire sc_a_undef   = a_undef;
    wire sc_b_undef   = ~a_undef & b_undef;
    wire sc_tf_tf     = ~a_undef & ~b_undef & a_transf & b_transf;
    wire sc_van_van   = ~a_undef & ~b_undef & ~sc_tf_tf & a_vanished & b_vanished;
    wire sc_a_transf  = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van & a_transf;
    wire sc_b_transf  = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van & ~sc_a_transf & b_transf;
    wire sc_a_van     = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & a_vanished;
    wire sc_b_van     = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_van & b_vanished;
    wire sc_a_zero    = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_van & ~sc_b_van & a_is_zero;
    wire sc_fallback  = any_non_normal & ~sc_a_undef & ~sc_b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_van & ~sc_b_van & ~sc_a_zero;

    wire shortcut = any_non_normal & (sc_a_undef | sc_b_undef | sc_tf_tf | sc_van_van |
                                      sc_a_transf | sc_b_transf | sc_a_van | sc_b_van |
                                      sc_a_zero | sc_fallback);

    // Shortcut fraction (add vs sub selects different undefined prefixes)
    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_tf_tf    ? (sub ? UNDEF_TF_M_TF  : UNDEF_TF_P_TF) :
        sc_van_van  ? (sub ? UNDEF_VAN_M_VAN : UNDEF_VAN_P_VAN) :
        sc_a_transf ? (sub ? UNDEF_TF_M_FIN  : UNDEF_TF_P_FIN) :
        sc_b_transf ? (sub ? UNDEF_FIN_M_TF  : UNDEF_FIN_P_TF) :
        sc_a_van    ? (sub ? neg_b_frac : b_frac) :
        sc_b_van    ? a_frac :
        sc_a_zero   ? (sub ? neg_b_frac : b_frac) :
                      a_frac;  // fallback

    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
        sc_tf_tf    ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_van_van  ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_a_transf ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_b_transf ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_a_van    ? (sub ? neg_b_exp : b_exp) :
        sc_b_van    ? a_exp :
        sc_a_zero   ? (sub ? neg_b_exp : b_exp) :
                      a_exp;

    // ========== Normal arithmetic (unchanged, only used when !shortcut) ==========

    // == Step 1: Exponent difference + swap ===================================

    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                       - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;
    wire signed [EXP_BITS-1:0]  small_exp  = a_is_big ? b_exp  : a_exp;

    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;
    wire negligible = (exp_diff >= FRAC_BITS);

    wire negate_small = sub &  a_is_big;
    wire negate_big   = sub & !a_is_big;

    // == Bypass: negligible operand ===========================================

    wire big_is_pos_half = (big_frac == POS_HALF);
    wire big_is_neg_one  = (big_frac == NEG_ONE);

    wire signed [FRAC_BITS-1:0] neg_big_frac = big_is_pos_half ? NEG_ONE  :
                                                 big_is_neg_one  ? POS_HALF :
                                                 -big_frac;
    wire signed [EXP_BITS-1:0]  neg_big_exp  = big_is_pos_half ? (big_exp - 1) :
                                                 big_is_neg_one  ? (big_exp + 1) :
                                                 big_exp;

    wire bypass_add = negligible & !negate_big;
    wire bypass_sub = negligible &  negate_big;

    // == Step 2: Extend to 2x width ==========================================

    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac);
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac);

    wire [SHIFT_BITS-1:0] shift_amt = (exp_diff >= FRAC_BITS)
                                     ? FRAC_BITS[SHIFT_BITS-1:0] - 1
                                     : exp_diff[SHIFT_BITS-1:0];
    wire signed [INT_BITS-1:0] big_shifted = big_ext <<< shift_amt;

    // == Step 3: Add / subtract ===============================================

    wire signed [INT_BITS-1:0] sum = (big_shifted ^ {INT_BITS{negate_big}})
                                   + (small_ext   ^ {INT_BITS{negate_small}})
                                   + {{(INT_BITS-1){1'b0}}, sub};
    wire is_zero = (sum == 0);

    // == Step 4: CLZ (count leading sign bits - 1) ============================

    wire [INT_BITS-2:0] xor_bits = sum[INT_BITS-1:1] ^ sum[INT_BITS-2:0];

    reg [SHIFT_BITS-1:0] norm_shift;
    integer ci;
    always @(*) begin
        norm_shift = INT_BITS - 1;
        for (ci = 0; ci < INT_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) norm_shift = (INT_BITS - 2) - ci[SHIFT_BITS-1:0];
    end

    // == Step 5: Normalize (left shift) + extract top FRAC_BITS (floor) =======

    wire signed [INT_BITS-1:0] normalized = sum <<< norm_shift;
    wire signed [FRAC_BITS-1:0] out_frac = normalized[INT_BITS-1 -: FRAC_BITS];

    // == Step 6: Exponent =====================================================

    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS){small_exp[EXP_BITS-1]}}, small_exp})
        + FRAC_BITS
        - $signed({{(ECW-SHIFT_BITS){1'b0}}, norm_shift});

    wire signed [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    wire underflow = (exp_calc < (AMBIG_EXP + 1));

    // Vanished fraction: duplicate sign bit -> N2 prefix pattern
    wire signed [FRAC_BITS-1:0] vanished_frac = {normalized[INT_BITS-1],
                                                   normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // ========== Output mux ===================================================
    //
    // Priority: shortcut > bypass > zero > underflow > normal

    assign result_frac = shortcut    ? sc_frac :
                         bypass_add  ? big_frac :
                         bypass_sub  ? neg_big_frac :
                         is_zero     ? {FRAC_BITS{1'b0}} :
                         underflow   ? vanished_frac :
                                       out_frac;

    assign result_exp  = shortcut              ? sc_exp :
                         bypass_add            ? big_exp :
                         bypass_sub            ? neg_big_exp :
                         (is_zero | underflow) ? AMBIG_EXP[EXP_BITS-1:0] :
                                                  out_exp;

endmodule
