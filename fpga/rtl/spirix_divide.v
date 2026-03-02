// Spirix Division: a / b (combinational)
// Parameterized combinational divider for Spirix floating-point scalars.
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Algorithm:
//   1. Sign = a_sign XOR b_sign. Convert to unsigned absolute values.
//   2. Fixed-point division: q = (abs_a << FRAC_BITS) / abs_b
//      For N1 inputs, q ∈ [2^(FRAC-1), 2^(FRAC+1)).
//   3. Bounded normalization (0 or 1 bit shift — no barrel shifter).
//   4. Banker's rounding on guard + sticky bits.
//   5. Apply sign (with N1 corner case handling).
//   6. Exponent: a_exp - b_exp + overflow + round_ovf.

module spirix_divide #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    // =========================================================================
    // Step 1: Sign and absolute values
    // =========================================================================
    wire result_sign = a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];

    // NEG_ONE has |value| = 2^(FRAC-1), which overflows FRAC-1 unsigned bits.
    // Represent as POS_HALF magnitude with exponent+1.
    wire a_is_neg_one = (a_frac == NEG_ONE);
    wire b_is_neg_one = (b_frac == NEG_ONE);

    wire [FRAC_BITS-2:0] abs_a = a_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                  (a_frac[FRAC_BITS-1] ? (~a_frac[FRAC_BITS-2:0] + 1'b1) :
                                                          a_frac[FRAC_BITS-2:0]);
    wire [FRAC_BITS-2:0] abs_b = b_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                  (b_frac[FRAC_BITS-1] ? (~b_frac[FRAC_BITS-2:0] + 1'b1) :
                                                          b_frac[FRAC_BITS-2:0]);

    wire signed [EXP_BITS:0] a_exp_adj = $signed({a_exp[EXP_BITS-1], a_exp})
                                         + {{EXP_BITS{1'b0}}, a_is_neg_one};
    wire signed [EXP_BITS:0] b_exp_adj = $signed({b_exp[EXP_BITS-1], b_exp})
                                         + {{EXP_BITS{1'b0}}, b_is_neg_one};

    // =========================================================================
    // Step 2: Unsigned fixed-point division
    //   quotient = (abs_a << FRAC_BITS) / abs_b
    //   For N1 inputs: q ∈ [2^(FRAC-1), 2^(FRAC+1))
    // =========================================================================
    localparam WIDE = 2 * FRAC_BITS;
    wire [WIDE-1:0] wide_a = {{(FRAC_BITS+1){1'b0}}, abs_a} << FRAC_BITS;
    wire [WIDE-1:0] wide_b = {{(FRAC_BITS+1){1'b0}}, abs_b};
    wire [WIDE-1:0] quotient_full = wide_a / wide_b;
    wire [WIDE-1:0] remainder     = wide_a - quotient_full * wide_b;

    wire [FRAC_BITS:0] q = quotient_full[FRAC_BITS:0]; // FRAC+1 significant bits

    // =========================================================================
    // Step 3: Bounded normalization (0 or 1 bit shift)
    //   q[FRAC] set → value ≥ 1.0, shift right 1 (exp +1)
    //   q[FRAC] clear → value < 1.0, no shift
    // =========================================================================
    wire overflow = q[FRAC_BITS];
    wire [FRAC_BITS:0] q_norm = overflow ? (q >> 1) : q;
    // q_norm always in [2^(FRAC-1), 2^FRAC), bits [FRAC]=0, [FRAC-1]=1

    // Extract positive N1 fraction: q_norm[FRAC:1] = 01xxx (FRAC bits)
    wire [FRAC_BITS-1:0] frac_pos_raw = q_norm[FRAC_BITS:1];

    // =========================================================================
    // Step 4: Banker's rounding
    // =========================================================================
    wire guard = q_norm[0];
    wire ovf_sticky = overflow & q[0]; // bit shifted out during normalization
    wire rem_sticky = |remainder;
    wire sticky = ovf_sticky | rem_sticky;
    wire lsb = frac_pos_raw[0];
    wire round_up = guard & (sticky | lsb);

    wire [FRAC_BITS-1:0] frac_rounded = frac_pos_raw + {{(FRAC_BITS-1){1'b0}}, round_up};

    // Rounding overflow: 01111...1 + 1 → 10000...0 (sign flips)
    wire round_ovf = !frac_pos_raw[FRAC_BITS-1] & frac_rounded[FRAC_BITS-1];
    wire [FRAC_BITS-1:0] pos_frac = round_ovf ? POS_HALF : frac_rounded;

    // =========================================================================
    // Step 5: Exponent
    //   result_exp = a_exp_adj - b_exp_adj + overflow + round_ovf
    // =========================================================================
    wire signed [EXP_BITS:0] exp_wide = a_exp_adj - b_exp_adj
                                       + {{EXP_BITS{1'b0}}, overflow}
                                       + {{EXP_BITS{1'b0}}, round_ovf};

    wire exp_too_big   = (exp_wide > MAX_EXP);
    wire exp_too_small = (exp_wide < MIN_EXP);
    wire signed [EXP_BITS-1:0] pos_exp = exp_wide[EXP_BITS-1:0];

    // =========================================================================
    // Step 6: Apply sign
    //   Negative: negate fraction. Corner case: POS_HALF → NEG_ONE, exp-1
    // =========================================================================
    wire neg_is_pos_half = (pos_frac == POS_HALF);

    wire signed [FRAC_BITS-1:0] neg_frac = neg_is_pos_half ? NEG_ONE :
                                            (~pos_frac + 1'b1);
    wire signed [EXP_BITS-1:0]  neg_exp  = neg_is_pos_half ? (pos_exp - 1) : pos_exp;

    wire signed [FRAC_BITS-1:0] final_frac = result_sign ? neg_frac : $signed(pos_frac);
    wire signed [EXP_BITS-1:0]  final_exp  = result_sign ? neg_exp  : pos_exp;

    // =========================================================================
    // Step 7: Output with overflow/underflow clamping
    // =========================================================================
    assign result_frac = exp_too_big   ? final_frac :
                         exp_too_small ? {final_frac[FRAC_BITS-1],
                                          final_frac[FRAC_BITS-1:1]} :
                                         final_frac;

    assign result_exp  = (exp_too_big | exp_too_small) ?
                          AMBIGUOUS_EXP[EXP_BITS-1:0] : final_exp;

endmodule
