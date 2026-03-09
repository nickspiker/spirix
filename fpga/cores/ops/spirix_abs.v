// spirix_abs — combinational magnitude (absolute value)
//
// If negative, negate. Otherwise passthrough.
// Uses spirix_neg for the negation path.
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.

module spirix_abs #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    output wire signed [MAX_FRAC-1:0] result_frac,
    output wire signed [MAX_EXP-1:0]  result_exp
);

    // Negate path
    wire signed [MAX_FRAC-1:0] neg_frac;
    wire signed [MAX_EXP-1:0]  neg_exp;

    spirix_neg #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_neg (
        .frac_width(frac_width), .exp_width(exp_width),
        .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(neg_frac), .result_exp(neg_exp)
    );

    // If MSB=1 (negative), use negated result; else passthrough
    assign result_frac = a_frac[MAX_FRAC-1] ? neg_frac : a_frac;
    assign result_exp  = a_frac[MAX_FRAC-1] ? neg_exp  : a_exp;

endmodule
