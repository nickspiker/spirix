// spirix_multiply — Minimal combinational multiply for Spirix scalars
//
// Floor-only (no rounding), single-path design.
// Parameterized: FRAC_BITS in {2..256}, EXP_BITS in {2..256}.
//
// Architecture: sign-extend -> multiply -> CLZ -> normalize -> extract (floor).
//   Matches the Rust reference model exactly (2x intermediate width,
//   single floor point at extraction).
//   No rounding. No Karatsuba. Synthesizer infers DSP or LUT multiplier.
//
// N1 * N1 produces at most 1 bit of cancellation, so the normalize shift
// is bounded to {0, 1} for normal inputs. The CLZ handles the general case
// (including non-N1 inputs gracefully).
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
    // Product of two FRAC_BITS-bit signed values fits in 2*FRAC_BITS-1 bits,
    // but we use 2*FRAC_BITS to match Rust's full type width.
    localparam INT_BITS   = 2 * FRAC_BITS;
    localparam SHIFT_BITS = $clog2(INT_BITS);
    localparam AMBIG_EXP  = -(1 <<< (EXP_BITS - 1));

    // Exponent calc width: enough to hold a_exp + b_exp - expo_adjust
    // where expo_adjust can be up to INT_BITS-2.
    localparam ECW = (SHIFT_BITS + 1 > EXP_BITS + 1) ? SHIFT_BITS + 2 : EXP_BITS + 2;

    // == Step 1: Sign-extend and multiply =======================================

    wire signed [INT_BITS-1:0] a_ext = $signed(a_frac);
    wire signed [INT_BITS-1:0] b_ext = $signed(b_frac);
    wire signed [INT_BITS-1:0] product = a_ext * b_ext;

    wire is_zero = (product == 0);

    // == Step 2: CLZ (count leading sign bits - 1) ==============================
    //
    // Rust: leading = max(leading_ones, leading_zeros)
    //       expo_adjust = leading - 2
    //       shift = leading - 1
    //
    // Our CLZ gives leading_m1 = leading - 1 (= shift).
    // expo_adjust = leading_m1 - 1.

    wire [INT_BITS-2:0] xor_bits = product[INT_BITS-1:1] ^ product[INT_BITS-2:0];

    reg [SHIFT_BITS-1:0] leading_m1;
    integer ci;
    always @(*) begin
        leading_m1 = INT_BITS - 1;
        for (ci = 0; ci < INT_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) leading_m1 = (INT_BITS - 2) - ci[SHIFT_BITS-1:0];
    end

    // == Step 3: Normalize (left shift) + extract top FRAC_BITS (floor) =========

    wire signed [INT_BITS-1:0] normalized = product <<< leading_m1;
    wire signed [FRAC_BITS-1:0] out_frac = normalized[INT_BITS-1 -: FRAC_BITS];

    // == Step 4: Exponent =======================================================
    //
    // result_exp = a_exp + b_exp - expo_adjust
    //            = a_exp + b_exp - (leading_m1 - 1)
    //            = a_exp + b_exp - leading_m1 + 1

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

    // == Step 5: Output mux =====================================================

    assign result_frac = is_zero    ? {FRAC_BITS{1'b0}} :
                         overflow   ? out_frac :
                         underflow  ? vanished_frac :
                                      out_frac;

    assign result_exp  = is_zero              ? AMBIG_EXP[EXP_BITS-1:0] :
                         (overflow | underflow) ? AMBIG_EXP[EXP_BITS-1:0] :
                                                  out_exp;

endmodule
