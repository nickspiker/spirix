// spirix_multiply — Combinational multiply for Spirix scalars with edge cases
//
// Floor-only (no rounding), single-path design.
// Parameterized: FRAC_BITS in {8..256}, EXP_BITS in {2..256}.
//
// Architecture: state detect -> sign-extend -> multiply -> CLZ -> normalize -> extract (floor).
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
    localparam SHIFT_BITS = $clog2(INT_BITS);
    localparam AMBIG_EXP  = -(1 <<< (EXP_BITS - 1));

    // Exponent calc width
    localparam ECW = (SHIFT_BITS + 1 > EXP_BITS + 1) ? SHIFT_BITS + 2 : EXP_BITS + 2;

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
    wire sc_any_zero = ~a_undef & ~b_undef & ~sc_inf_zero & (a_is_zero | b_is_zero);
    wire sc_exp_van  = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_zero &
                       ((a_exploded & b_vanished) | (a_vanished & b_exploded));
    wire shortcut    = sc_a_undef | sc_b_undef | sc_inf_zero | sc_any_zero | sc_exp_van;

    // Abnormal compute path (case 8)
    wire abnormal_compute = any_non_normal & ~shortcut;
    // n_level=-1 when either input is exploded; -2 otherwise
    wire n_level_neg1 = a_exploded | b_exploded;

    // ========== Step 1: Sign-extend and multiply ================================

    wire signed [INT_BITS-1:0] a_ext = $signed(a_frac);
    wire signed [INT_BITS-1:0] b_ext = $signed(b_frac);
    wire signed [INT_BITS-1:0] product = a_ext * b_ext;

    wire prod_is_zero = (product == 0);

    // ========== Step 2: CLZ (count leading sign bits - 1) ========================

    wire [INT_BITS-2:0] xor_bits = product[INT_BITS-1:1] ^ product[INT_BITS-2:0];

    reg [SHIFT_BITS-1:0] leading_m1;
    integer ci;
    always @(*) begin
        leading_m1 = INT_BITS - 1;
        for (ci = 0; ci < INT_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) leading_m1 = (INT_BITS - 2) - ci[SHIFT_BITS-1:0];
    end

    // ========== Step 3: Normalize (left shift) + extract top FRAC_BITS ==========
    //
    // Normal: shift = leading_m1 (= leading - 1), produces N1 result
    // Abnormal n_level=-1: same shift as normal
    // Abnormal n_level=-2: shift = leading_m1 - 1 (one less, produces N2 result)

    wire abnormal_n2 = abnormal_compute & ~n_level_neg1;
    wire [SHIFT_BITS-1:0] shift_amt = (abnormal_n2 & |leading_m1) ?
                                       leading_m1 - 1'b1 : leading_m1;

    wire signed [INT_BITS-1:0] normalized = product <<< shift_amt;
    wire signed [FRAC_BITS-1:0] out_frac = normalized[INT_BITS-1 -: FRAC_BITS];

    // ========== Step 4: Exponent (normal path only) ==============================

    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS){a_exp[EXP_BITS-1]}}, a_exp})
        + $signed({{(ECW-EXP_BITS){b_exp[EXP_BITS-1]}}, b_exp})
        - $signed({{(ECW-SHIFT_BITS){1'b0}}, leading_m1})
        + 1;

    wire signed [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    // Overflow: exp_calc > MAX_EXP (= -AMBIG_EXP - 1)
    // Underflow: exp_calc < MIN_EXP (= AMBIG_EXP + 1)
    localparam signed [ECW-1:0] MAX_EXP_W = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [ECW-1:0] MIN_EXP_W = -(1 <<< (EXP_BITS - 1)) + 1;

    wire overflow  = (exp_calc > MAX_EXP_W);
    wire underflow = (exp_calc < MIN_EXP_W);

    // Vanished fraction: duplicate sign bit (>> 1 arithmetic)
    wire signed [FRAC_BITS-1:0] vanished_frac = {out_frac[FRAC_BITS-1],
                                                   out_frac[FRAC_BITS-1:1]};

    // ========== Step 5: Output mux ==============================================
    //
    // Priority: shortcut > abnormal_compute > prod_zero > overflow > underflow > normal

    // Shortcut fraction
    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_inf_zero ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        sc_any_zero ? {FRAC_BITS{1'b0}} :
                      ((a_exploded)             ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF);

    // Shortcut exponent
    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
                      AMBIG_EXP[EXP_BITS-1:0];

    assign result_frac = shortcut          ? sc_frac :
                         abnormal_compute  ? out_frac :
                         prod_is_zero      ? {FRAC_BITS{1'b0}} :
                         overflow          ? out_frac :
                         underflow         ? vanished_frac :
                                             out_frac;

    assign result_exp  = shortcut                          ? sc_exp :
                         (abnormal_compute | prod_is_zero) ? AMBIG_EXP[EXP_BITS-1:0] :
                         (overflow | underflow)            ? AMBIG_EXP[EXP_BITS-1:0] :
                                                             out_exp;

endmodule
