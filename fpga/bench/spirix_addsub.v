// spirix_addsub — Combinational add/subtract for Spirix scalars with edge cases
//
// Floor-only (no rounding), single-path design. N0 format.
// Parameterized: FRAC_BITS in {8..256}, EXP_BITS in {2..256}.
//
// N0 storage convention (matches Rust Scalar<Fn, En>):
//   value = inflate(fraction) / 2^FRAC * 2^exponent
//   inflate(x) = sign_ext(x) XOR high_mask, where high_mask = (-1) << FRAC.
//   Equivalently: wide = {{FRAC{~x[FRAC-1]}}, x} (low FRAC bits = stored; high
//   FRAC bits = complement of stored MSB, giving implicit ~MSB sign).
//
// Ambiguous (exponent == AMBIG_EXP == -2^(EXP-1)) encodes non-normal states:
//   all-0 fraction     → Zero
//   all-1 fraction     → Infinity
//   N-1 pattern         → Exploded (top 2 bits differ)
//   N-2 pattern         → Vanished (top 2 bits same, bit 3 differs)
//   N-3+ pattern        → Undefined (top 3+ bits all same, non-uniform)
//
// Edge case precedence matches Rust scalar_add_scalar exactly.
// Undefined-prefix constants match src/core/undefined.rs.

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

    // 2*FRAC intermediate for the inflated sum.
    localparam INT_BITS   = 2 * FRAC_BITS;
    localparam SHIFT_BITS = $clog2(INT_BITS);
    localparam AMBIG_EXP  = -(1 <<< (EXP_BITS - 1));
    localparam signed [EXP_BITS-1:0] MAX_EXP = {1'b0, {(EXP_BITS-1){1'b1}}};
    localparam signed [EXP_BITS-1:0] MIN_EXP = AMBIG_EXP[EXP_BITS-1:0] + 1;

    // Exponent calc width: need enough bits to detect over/underflow without wrapping.
    localparam ECW = EXP_BITS + SHIFT_BITS + 2;

    // N0 normal boundary constants.
    localparam signed [FRAC_BITS-1:0] POS_ONE_NORMAL = {1'b1, {(FRAC_BITS-1){1'b0}}}; // 0x80...0
    localparam signed [FRAC_BITS-1:0] NEG_ONE_NORMAL = {FRAC_BITS{1'b0}};             // 0x00...0

    // Escaped (N-1 / N-2) boundary patterns — same in N0 and N1.
    localparam signed [FRAC_BITS-1:0] POS_ONE_EXPLODED = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}}; // 0x40
    localparam signed [FRAC_BITS-1:0] NEG_ONE_EXPLODED = {1'b1, 1'b0, {(FRAC_BITS-2){1'b0}}}; // 0x80
    localparam signed [FRAC_BITS-1:0] POS_ONE_VANISHED = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}}; // 0x20
    localparam signed [FRAC_BITS-1:0] NEG_ONE_VANISHED = {2'b11, {(FRAC_BITS-2){1'b0}}};       // 0xC0

    // Undefined prefix constants — match src/core/undefined.rs exactly.
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_TF   = {8'h1A, {UPAD{1'b0}}}; // ℘⬆+⬆ TRANSFINITE_PLUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_TF   = {8'hE5, {UPAD{1'b0}}}; // ℘⬆-⬆ TRANSFINITE_MINUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_P_VAN = {8'h1D, {UPAD{1'b0}}}; // ℘↓+↓ VANISHED_PLUS_VANISHED
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_M_VAN = {8'hE2, {UPAD{1'b0}}}; // ℘↓-↓ VANISHED_MINUS_VANISHED
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_FIN  = {8'h1C, {UPAD{1'b0}}}; // ℘⬆+ TRANSFINITE_PLUS_FINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_FIN  = {8'hE3, {UPAD{1'b0}}}; // ℘⬆- TRANSFINITE_MINUS_FINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_P_TF  = {8'h1B, {UPAD{1'b0}}}; // ℘+⬆ FINITE_PLUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_M_TF  = {8'hE4, {UPAD{1'b0}}}; // ℘-⬆ FINITE_MINUS_TRANSFINITE

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
    wire a_n2 = ~a_n1 & ~a_n0 & (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-3]);
    wire b_n2 = ~b_n1 & ~b_n0 & (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-3]);
    wire a_top3 = ~a_n0 & (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire b_top3 = ~b_n0 & (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                  (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);

    wire a_is_zero   = a_is_ambig & a_frac_zero;
    wire a_is_inf    = a_is_ambig & a_frac_neg1;
    wire a_exploded  = a_is_ambig & a_n1;
    wire a_transf    = a_is_inf | a_exploded;
    wire a_vanished  = a_is_ambig & a_n2;
    wire a_undef     = a_is_ambig & a_top3;
    wire a_is_normal = ~a_is_ambig;   // N0: any fraction pattern + non-AMBIG exp

    wire b_is_zero   = b_is_ambig & b_frac_zero;
    wire b_is_inf    = b_is_ambig & b_frac_neg1;
    wire b_exploded  = b_is_ambig & b_n1;
    wire b_transf    = b_is_inf | b_exploded;
    wire b_vanished  = b_is_ambig & b_n2;
    wire b_undef     = b_is_ambig & b_top3;
    wire b_is_normal = ~b_is_ambig;

    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // ========== Spirix negation of b (for sub edge cases) =======================
    //
    // N0 normal negation:
    //   POS_ONE_NORMAL (0x80) → (NEG_ONE_NORMAL, exp-1) or NEG_ONE_VANISHED if exp-1==AMBIG
    //   NEG_ONE_NORMAL (0x00) → (POS_ONE_NORMAL, exp+1) or POS_ONE_EXPLODED if exp+1==AMBIG
    //   else → (-frac, exp) — two's-complement negate in place
    //
    // Non-normal negation:
    //   zero/inf/undef (top3_same) → unchanged (signless)
    //   POS_ONE_EXPLODED ↔ NEG_ONE_EXPLODED
    //   POS_ONE_VANISHED ↔ NEG_ONE_VANISHED
    //   else → wrapping_neg (exp unchanged)

    wire b_is_pos_one_norm  = (b_frac == POS_ONE_NORMAL);
    wire b_is_neg_one_norm  = (b_frac == NEG_ONE_NORMAL);
    wire b_is_pos_one_exp   = (b_frac == POS_ONE_EXPLODED);
    wire b_is_neg_one_exp   = (b_frac == NEG_ONE_EXPLODED);
    wire b_is_pos_one_van   = (b_frac == POS_ONE_VANISHED);
    wire b_is_neg_one_van   = (b_frac == NEG_ONE_VANISHED);

    wire signed [EXP_BITS-1:0] b_exp_m1 = b_exp - 1'b1;
    wire signed [EXP_BITS-1:0] b_exp_p1 = b_exp + 1'b1;
    wire b_exp_m1_ambig = (b_exp_m1 == AMBIG_EXP[EXP_BITS-1:0]);
    wire b_exp_p1_ambig = (b_exp_p1 == AMBIG_EXP[EXP_BITS-1:0]);

    // Normal negation (b_is_normal)
    wire signed [FRAC_BITS-1:0] neg_b_frac_normal =
        b_is_pos_one_norm ? (b_exp_m1_ambig ? NEG_ONE_VANISHED : NEG_ONE_NORMAL) :
        b_is_neg_one_norm ? (b_exp_p1_ambig ? POS_ONE_EXPLODED : POS_ONE_NORMAL) :
                            -b_frac;
    wire signed [EXP_BITS-1:0]  neg_b_exp_normal =
        b_is_pos_one_norm ? (b_exp_m1_ambig ? AMBIG_EXP[EXP_BITS-1:0] : b_exp_m1) :
        b_is_neg_one_norm ? (b_exp_p1_ambig ? AMBIG_EXP[EXP_BITS-1:0] : b_exp_p1) :
                            b_exp;

    // Non-normal negation (b_is_ambig)
    wire b_nonnorm_nochange = b_frac_zero | b_frac_neg1 | b_top3;
    wire signed [FRAC_BITS-1:0] neg_b_frac_nonnorm =
        b_nonnorm_nochange ? b_frac :
        b_is_pos_one_exp  ? NEG_ONE_EXPLODED :
        b_is_neg_one_exp  ? POS_ONE_EXPLODED :
        b_is_pos_one_van  ? NEG_ONE_VANISHED :
        b_is_neg_one_van  ? POS_ONE_VANISHED :
                            -b_frac;

    wire signed [FRAC_BITS-1:0] neg_b_frac = b_is_ambig ? neg_b_frac_nonnorm : neg_b_frac_normal;
    wire signed [EXP_BITS-1:0]  neg_b_exp  = b_is_ambig ? b_exp               : neg_b_exp_normal;

    // ========== Edge case priority (matches Rust scalar_add_scalar) =============
    //
    // Add:                                    Sub:
    // 1. a undef    -> a                      1. a undef    -> a
    // 2. b undef    -> b                      2. b undef    -> b
    // 3. a_tf & b_tf -> TF_P_TF               3. a_tf & b_tf -> TF_M_TF
    // 4. a_van & b_van -> VAN_P_VAN           4. a_van & b_van -> VAN_M_VAN
    // 5. a_tf       -> TF_P_FIN               5. a_tf       -> TF_M_FIN
    // 6. b_tf       -> FIN_P_TF               6. b_tf       -> FIN_M_TF
    // 7. a_zero     -> b                      7. a_zero     -> -b
    // 8. b_zero     -> a                      8. b_zero     -> a
    // 9. a_van      -> b                      9. a_van      -> -b
    // 10. b_van     -> a                      10. b_van     -> a
    // 11. fallback  -> a                      11. fallback  -> a

    wire sc_a_undef   = a_undef;
    wire sc_b_undef   = ~a_undef & b_undef;
    wire sc_tf_tf     = ~a_undef & ~b_undef & a_transf & b_transf;
    wire sc_van_van   = ~a_undef & ~b_undef & ~sc_tf_tf & a_vanished & b_vanished;
    wire sc_a_transf  = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van & a_transf;
    wire sc_b_transf  = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van & ~sc_a_transf & b_transf;
    wire sc_a_zero    = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & a_is_zero;
    wire sc_b_zero    = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_zero & b_is_zero;
    wire sc_a_van     = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_zero & ~sc_b_zero & a_vanished;
    wire sc_b_van     = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_a_van & b_vanished;
    wire sc_fallback  = any_non_normal & ~sc_a_undef & ~sc_b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_a_van & ~sc_b_van;

    wire shortcut = any_non_normal & (sc_a_undef | sc_b_undef | sc_tf_tf | sc_van_van |
                                      sc_a_transf | sc_b_transf | sc_a_zero | sc_b_zero |
                                      sc_a_van | sc_b_van | sc_fallback);

    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_tf_tf    ? (sub ? UNDEF_TF_M_TF    : UNDEF_TF_P_TF) :
        sc_van_van  ? (sub ? UNDEF_VAN_M_VAN  : UNDEF_VAN_P_VAN) :
        sc_a_transf ? (sub ? UNDEF_TF_M_FIN   : UNDEF_TF_P_FIN) :
        sc_b_transf ? (sub ? UNDEF_FIN_M_TF   : UNDEF_FIN_P_TF) :
        sc_a_zero   ? (sub ? neg_b_frac : b_frac) :
        sc_b_zero   ? a_frac :
        sc_a_van    ? (sub ? neg_b_frac : b_frac) :
        sc_b_van    ? a_frac :
                      a_frac;

    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
        sc_tf_tf    ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_van_van  ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_a_transf ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_b_transf ? AMBIG_EXP[EXP_BITS-1:0] :
        sc_a_zero   ? (sub ? neg_b_exp : b_exp) :
        sc_b_zero   ? a_exp :
        sc_a_van    ? (sub ? neg_b_exp : b_exp) :
        sc_b_van    ? a_exp :
                      a_exp;

    // ========== Normal arithmetic (used when !shortcut) ==========================

    // Exponent difference + big/small selection.
    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                       - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;
    wire signed [EXP_BITS-1:0]  small_exp  = a_is_big ? b_exp  : a_exp;

    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;
    // N0: shift cap is FRAC-1 (not FRAC as in N1) because inflated values reach
    // magnitude 2^FRAC, so shift by FRAC would need 2*FRAC+1 bits.
    wire negligible = (exp_diff >= FRAC_BITS - 1);

    wire negate_small = sub &  a_is_big;
    wire negate_big   = sub & !a_is_big;

    // Bypass: operand is negligible at the other's precision.
    wire big_is_pos_one = (big_frac == POS_ONE_NORMAL);
    wire big_is_neg_one = (big_frac == NEG_ONE_NORMAL);

    wire signed [EXP_BITS-1:0] big_exp_m1 = big_exp - 1'b1;
    wire signed [EXP_BITS-1:0] big_exp_p1 = big_exp + 1'b1;
    wire big_exp_m1_ambig = (big_exp_m1 == AMBIG_EXP[EXP_BITS-1:0]);
    wire big_exp_p1_ambig = (big_exp_p1 == AMBIG_EXP[EXP_BITS-1:0]);

    wire signed [FRAC_BITS-1:0] neg_big_frac =
        big_is_pos_one ? (big_exp_m1_ambig ? NEG_ONE_VANISHED : NEG_ONE_NORMAL) :
        big_is_neg_one ? (big_exp_p1_ambig ? POS_ONE_EXPLODED : POS_ONE_NORMAL) :
                         -big_frac;
    wire signed [EXP_BITS-1:0] neg_big_exp =
        big_is_pos_one ? (big_exp_m1_ambig ? AMBIG_EXP[EXP_BITS-1:0] : big_exp_m1) :
        big_is_neg_one ? (big_exp_p1_ambig ? AMBIG_EXP[EXP_BITS-1:0] : big_exp_p1) :
                         big_exp;

    wire bypass_add = negligible & !negate_big;
    wire bypass_sub = negligible &  negate_big;

    // N0 inflate: low FRAC bits = stored, high FRAC bits = ~stored[FRAC-1].
    wire signed [INT_BITS-1:0] big_inflated   = {{FRAC_BITS{~big_frac[FRAC_BITS-1]}},   big_frac};
    wire signed [INT_BITS-1:0] small_inflated = {{FRAC_BITS{~small_frac[FRAC_BITS-1]}}, small_frac};

    // Align big to small's exp, add/subtract.
    wire [SHIFT_BITS-1:0] shift_amt = exp_diff[SHIFT_BITS-1:0];
    wire signed [INT_BITS-1:0] big_shifted = big_inflated <<< shift_amt;

    wire signed [INT_BITS-1:0] sum = (big_shifted   ^ {INT_BITS{negate_big}})
                                   + (small_inflated ^ {INT_BITS{negate_small}})
                                   + {{(INT_BITS-1){1'b0}}, sub};
    wire is_zero = (sum == 0);

    // Leading-same count: count of bits from MSB matching the sign bit, up to INT_BITS-1.
    // Implemented via position of highest differing-from-MSB bit.
    wire [INT_BITS-2:0] xor_bits = sum[INT_BITS-1:1] ^ sum[INT_BITS-2:0];
    reg [SHIFT_BITS:0] leading;
    integer ci;
    always @(*) begin
        leading = INT_BITS;  // all bits same (shouldn't happen for nonzero sum)
        for (ci = 0; ci < INT_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) leading = INT_BITS - 1 - ci[SHIFT_BITS-1:0];
    end

    // Normalize: target leading_same = FRAC. Shift by (leading - FRAC).
    // Positive → left shift (sum too small); negative → right shift (sum too big).
    wire signed [SHIFT_BITS+1:0] shl_amount = $signed({1'b0, leading}) - FRAC_BITS;

    wire signed [INT_BITS-1:0] canonical =
        shl_amount >= 0 ? (sum <<< shl_amount[SHIFT_BITS-1:0])
                        : (sum >>> (-shl_amount[SHIFT_BITS:0]));

    // Normal output fraction = low FRAC bits of canonical.
    wire signed [FRAC_BITS-1:0] out_frac = canonical[FRAC_BITS-1:0];

    // Exponent: offset = small.exp + (FRAC - leading) = small.exp - shl_amount.
    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS){small_exp[EXP_BITS-1]}}, small_exp})
        - $signed({{(ECW-SHIFT_BITS-2){shl_amount[SHIFT_BITS+1]}}, shl_amount});

    wire signed [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    wire underflow = (exp_calc < MIN_EXP);
    wire overflow  = (exp_calc > MAX_EXP);

    // Vanished fraction (N-2 shape, input sign preserved).
    // Shift sum to N-2 position: shl_amount_van = leading - 2 - FRAC.
    wire signed [SHIFT_BITS+1:0] van_shl = $signed({1'b0, leading}) - 2 - FRAC_BITS;
    wire signed [INT_BITS-1:0] van_wide =
        van_shl >= 0 ? (sum <<< van_shl[SHIFT_BITS-1:0])
                     : (sum >>> (-van_shl[SHIFT_BITS:0]));
    wire signed [FRAC_BITS-1:0] vanished_frac = van_wide[FRAC_BITS-1:0];

    // Exploded fraction (N-1 shape, input sign preserved).
    wire signed [SHIFT_BITS+1:0] exp_shl = $signed({1'b0, leading}) - 1 - FRAC_BITS;
    wire signed [INT_BITS-1:0] exp_wide =
        exp_shl >= 0 ? (sum <<< exp_shl[SHIFT_BITS-1:0])
                     : (sum >>> (-exp_shl[SHIFT_BITS:0]));
    wire signed [FRAC_BITS-1:0] exploded_frac = exp_wide[FRAC_BITS-1:0];

    // ========== Output mux ==========================================================
    //
    // Priority: shortcut > bypass > zero > overflow > underflow > normal.

    assign result_frac = shortcut    ? sc_frac :
                         bypass_add  ? big_frac :
                         bypass_sub  ? neg_big_frac :
                         is_zero     ? {FRAC_BITS{1'b0}} :
                         overflow    ? exploded_frac :
                         underflow   ? vanished_frac :
                                       out_frac;

    assign result_exp  = shortcut              ? sc_exp :
                         bypass_add            ? big_exp :
                         bypass_sub            ? neg_big_exp :
                         (is_zero | overflow | underflow) ? AMBIG_EXP[EXP_BITS-1:0] :
                                                             out_exp;

endmodule
