// Spirix Subtraction: a - b — 4-stage pipelined
// Negate b (with N1 corner case handling), then feed pipelined adder.
//
// Negation is purely combinational (3 muxes + 1 exp adder), absorbed into
// stage 1 of the pipelined adder (runs in parallel with the exp subtract).
//
// Latency: 4 clock cycles. Throughput: 1 result per clock cycle.

module spirix_subtract_pipe4 #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
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
    wire signed [FRAC_BITS-1:0] neg_b_frac = b_is_pos_half ? NEG_ONE  :
                                              b_is_neg_one  ? POS_HALF :
                                              -b_frac;
    wire signed [EXP_BITS-1:0]  neg_b_exp  = b_is_pos_half ? (b_exp - 1) :
                                              b_is_neg_one  ? (b_exp + 1) :
                                              b_exp;

    // a - b = a + (-b), pipelined
    spirix_add_pipe4 #(
        .FRAC_BITS(FRAC_BITS),
        .EXP_BITS(EXP_BITS)
    ) add (
        .clk(clk),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(neg_b_frac), .b_exp(neg_b_exp),
        .result_frac(result_frac), .result_exp(result_exp)
    );

endmodule
