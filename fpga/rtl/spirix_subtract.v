// Spirix Subtraction: a - b (negate b + add)
// Parameterized combinational subtractor for Spirix floating-point scalars.
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Implementation: negate b_frac (two's complement), then add.
// Two N1 corner cases where negation produces invalid N1:
//   1. b_frac = 01000...0 (+0.5): -0.5 → frac=10000...0, exp=b_exp-1
//   2. b_frac = 10000...0 (-1.0): +1.0 → frac=01000...0, exp=b_exp+1
// Both are single-value edge cases handled with equality checks.

module spirix_subtract #(
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

    // N1 corner case patterns
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}}; // 01000...0
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};        // 10000...0

    // Detect the two corner cases
    wire b_is_pos_half = (b_frac == POS_HALF);
    wire b_is_neg_one  = (b_frac == NEG_ONE);

    // Negate b_frac with N1-safe corner case handling
    //   Normal:    -b_frac (two's complement, always N1 for other N1 inputs)
    //   +0.5 case: represent -0.5 as -1.0 * 2^(exp-1)
    //   -1.0 case: represent +1.0 as +0.5 * 2^(exp+1)
    wire signed [FRAC_BITS-1:0] neg_b_frac = b_is_pos_half ? NEG_ONE  :
                                              b_is_neg_one  ? POS_HALF :
                                              -b_frac;
    wire signed [EXP_BITS-1:0]  neg_b_exp  = b_is_pos_half ? (b_exp - 1) :
                                              b_is_neg_one  ? (b_exp + 1) :
                                              b_exp;

    // a - b = a + (-b)
    spirix_add #(
        .FRAC_BITS(FRAC_BITS),
        .EXP_BITS(EXP_BITS)
    ) add (
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(neg_b_frac), .b_exp(neg_b_exp),
        .result_frac(result_frac), .result_exp(result_exp)
    );

endmodule
