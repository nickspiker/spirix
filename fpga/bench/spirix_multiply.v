// spirix_multiply — Combinational multiply for Spirix scalars with edge cases
//
// Floor-only (no rounding), single-path design.
// Parameterized: FRAC_BITS in {8..256}, EXP_BITS in {2..256}.
//
// Architecture: state detect -> sign-extend -> multiply -> bounded normalize -> extract (floor).
//   Matches the Rust reference model (scalar_multiply_scalar) exactly.
//   Edge cases handled: undefined passthrough, Inf*Zero, Zero*anything,
//   exploded*vanished, and abnormal compute (n_level=-1/-2).
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction: signed two's complement, N1-normalized (top two bits differ).
// Exponent: signed two's complement. AMBIGUOUS_EXP = -2^(EXP_BITS-1)
// encodes zero/exploded/vanished/infinity/undefined.

module spirix_multiply #(
    parameter FRAC_BITS = 32,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    // 2x width intermediate — matches Rust i16/i32/i64 promotion.
    localparam INT_BITS   = 2 * FRAC_BITS;
    localparam AMBIG_EXP  = -(1 <<< (EXP_BITS - 1));

    // Exponent calc width (needs room for a_exp + b_exp + 1 - lm1)
    localparam ECW = EXP_BITS + 2;

    // Undefined prefix constants (top 8 bits of fraction, zero-padded)
    // UPAD avoids negative repeat count for FRAC < 8 (constants wrong but unused at small FRAC)
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_MUL_NEG = {8'hEF, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_NEG_MUL_TF = {8'h10, {UPAD{1'b0}}};

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

    // Note: Rust vanished()/is_undefined() check fraction only (no exp check).
    // exploded()/is_zero()/is_infinite()/is_transfinite() DO check AMBIG exp.
    wire a_is_zero  = a_is_ambig & a_frac_zero;
    wire a_is_inf   = a_is_ambig & a_frac_neg1;
    wire a_exploded = a_is_ambig & a_n1;
    wire a_vanished = a_n2;             // fraction-only check
    wire a_undef    = ~a_n0 & a_top3;   // fraction-only check

    wire b_is_zero  = b_is_ambig & b_frac_zero;
    wire b_is_inf   = b_is_ambig & b_frac_neg1;
    wire b_exploded = b_is_ambig & b_n1;
    wire b_vanished = b_n2;             // fraction-only check
    wire b_undef    = ~b_n0 & b_top3;   // fraction-only check

    wire a_is_normal  = ~a_is_ambig & a_n1;
    wire b_is_normal  = ~b_is_ambig & b_n1;
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // ========== Edge case priority (matches Rust scalar_multiply_scalar) ========
    //
    // 1. a undefined      -> passthrough a
    // 2. b undefined      -> passthrough b
    // 3. a=inf & b=zero   -> TRANSFINITE_MULTIPLY_NEGLIGIBLE
    // 4. a=zero & b=inf   -> NEGLIGIBLE_MULTIPLY_TRANSFINITE
    // 5. a=zero | b=zero  -> ZERO
    // 6. a=expl & b=van   -> TRANSFINITE_MULTIPLY_NEGLIGIBLE
    // 7. a=van  & b=expl  -> NEGLIGIBLE_MULTIPLY_TRANSFINITE
    // 8. remaining abnorm -> compute with n_level (-1 if exploded, -2 otherwise)
    // 9. both normal      -> normal multiply

    wire sc_a_undef  = a_undef;
    wire sc_b_undef  = ~a_undef & b_undef;
    wire sc_inf_zero = ~a_undef & ~b_undef & ((a_is_inf & b_is_zero) | (a_is_zero & b_is_inf));
    wire sc_any_inf  = ~a_undef & ~b_undef & ~sc_inf_zero & (a_is_inf | b_is_inf);
    wire sc_any_zero = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_inf & (a_is_zero | b_is_zero);
    wire sc_exp_van  = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_inf & ~sc_any_zero &
                       ((a_exploded & b_vanished) | (a_vanished & b_exploded));
    wire shortcut    = sc_a_undef | sc_b_undef | sc_inf_zero | sc_any_inf | sc_any_zero | sc_exp_van;

    // Abnormal compute path (case 8)
    wire abnormal_compute = any_non_normal & ~shortcut;
    // n_level=-1 when either input is exploded; -2 otherwise
    wire n_level_neg1 = a_exploded | b_exploded;

    // ========== Step 1: Sign-extend and multiply ================================

    wire signed [INT_BITS-1:0] a_ext = $signed(a_frac);
    wire signed [INT_BITS-1:0] b_ext = $signed(b_frac);
    wire signed [INT_BITS-1:0] product = a_ext * b_ext;

    wire prod_is_zero = (product == 0);

    // ========== Step 2: Bounded normalize (no CLZ, no barrel) =====================
    //
    // Normal (N1×N1): lm1 ∈ {0, 1, 2} — 3-way check on top 3 product bits.
    // Abnormal (inf shortcutted): lm1 ∈ {0..4} — 5-way check on top 5 product bits.
    //   exploded×normal or exploded×exploded: lm1 = 0..1
    //   vanished×normal: lm1 = 2..3
    //   vanished×vanished: lm1 = 3..4
    // n_level=-1 (exploded present): shift = lm1
    // n_level=-2 (vanished only):    shift = lm1 - 1

    // Normal path: 3-way lm1 for exponent calc and normalize
    wire [1:0] norm_lm1 =
        (product[INT_BITS-1] != product[INT_BITS-2]) ? 2'd0 :
        (product[INT_BITS-2] != product[INT_BITS-3]) ? 2'd1 : 2'd2;

    wire signed [FRAC_BITS-1:0] norm_frac =
        (norm_lm1 == 2'd0) ? product[INT_BITS-1 -: FRAC_BITS] :
        (norm_lm1 == 2'd1) ? product[INT_BITS-2 -: FRAC_BITS] :
                              product[INT_BITS-3 -: FRAC_BITS];

    // Abnormal path: 5-way lm1 with n_level adjustment
    wire [2:0] abn_lm1 =
        (product[INT_BITS-1] != product[INT_BITS-2]) ? 3'd0 :
        (product[INT_BITS-2] != product[INT_BITS-3]) ? 3'd1 :
        (product[INT_BITS-3] != product[INT_BITS-4]) ? 3'd2 :
        (product[INT_BITS-4] != product[INT_BITS-5]) ? 3'd3 : 3'd4;

    wire abnormal_n2 = abnormal_compute & ~n_level_neg1;
    wire [2:0] abn_shift = (abnormal_n2 & |abn_lm1) ? abn_lm1 - 3'd1 : abn_lm1;

    wire signed [FRAC_BITS-1:0] abn_frac =
        (abn_shift == 3'd0) ? product[INT_BITS-1 -: FRAC_BITS] :
        (abn_shift == 3'd1) ? product[INT_BITS-2 -: FRAC_BITS] :
        (abn_shift == 3'd2) ? product[INT_BITS-3 -: FRAC_BITS] :
        (abn_shift == 3'd3) ? product[INT_BITS-4 -: FRAC_BITS] :
                               product[INT_BITS-5 -: FRAC_BITS];

    // ========== Step 4: Exponent (normal path only) ==============================

    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS){a_exp[EXP_BITS-1]}}, a_exp})
        + $signed({{(ECW-EXP_BITS){b_exp[EXP_BITS-1]}}, b_exp})
        - $signed({{(ECW-2){1'b0}}, norm_lm1})
        + 1;

    wire signed [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    // Overflow: exp_calc > MAX_EXP (= -AMBIG_EXP - 1)
    // Underflow: exp_calc < MIN_EXP (= AMBIG_EXP + 1)
    localparam signed [ECW-1:0] MAX_EXP_W = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [ECW-1:0] MIN_EXP_W = -(1 <<< (EXP_BITS - 1)) + 1;

    wire overflow  = (exp_calc > MAX_EXP_W);
    wire underflow = (exp_calc < MIN_EXP_W);

    // Vanished fraction: duplicate sign bit (>> 1 arithmetic)
    wire signed [FRAC_BITS-1:0] vanished_frac = {norm_frac[FRAC_BITS-1],
                                                   norm_frac[FRAC_BITS-1:1]};

    // ========== Step 5: Output mux ==============================================
    //
    // Priority: shortcut > abnormal_compute > prod_zero > overflow > underflow > normal

    // Shortcut fraction
    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_inf_zero ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        sc_any_inf  ? {FRAC_BITS{1'b1}} :
        sc_any_zero ? {FRAC_BITS{1'b0}} :
                      ((a_exploded)             ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF);

    // Shortcut exponent
    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
                      AMBIG_EXP[EXP_BITS-1:0];

    assign result_frac = shortcut          ? sc_frac :
                         abnormal_compute  ? abn_frac :
                         prod_is_zero      ? {FRAC_BITS{1'b0}} :
                         overflow          ? norm_frac :
                         underflow         ? vanished_frac :
                                             norm_frac;

    assign result_exp  = shortcut                          ? sc_exp :
                         (abnormal_compute | prod_is_zero) ? AMBIG_EXP[EXP_BITS-1:0] :
                         (overflow | underflow)            ? AMBIG_EXP[EXP_BITS-1:0] :
                                                             out_exp;

endmodule
