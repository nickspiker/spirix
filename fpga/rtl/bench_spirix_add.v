// Registered wrapper for Spirix add — gives nextpnr a real clock path
// so it can report actual Fmax through the combinational logic.

module bench_spirix_add (
    input  wire clk,
    input  wire signed [24:0] a_frac_in,
    input  wire signed [7:0]  a_exp_in,
    input  wire signed [24:0] b_frac_in,
    input  wire signed [7:0]  b_exp_in,
    output reg  signed [24:0] result_frac_out,
    output reg  signed [7:0]  result_exp_out
);

    // Input registers
    reg signed [24:0] a_frac, b_frac;
    reg signed [7:0]  a_exp,  b_exp;
    always @(posedge clk) begin
        a_frac <= a_frac_in;
        a_exp  <= a_exp_in;
        b_frac <= b_frac_in;
        b_exp  <= b_exp_in;
    end

    // Combinational add
    wire signed [24:0] r_frac;
    wire signed [7:0]  r_exp;
    spirix_add #(.FRAC_BITS(25), .EXP_BITS(8)) add (
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(r_frac), .result_exp(r_exp)
    );

    // Output registers
    always @(posedge clk) begin
        result_frac_out <= r_frac;
        result_exp_out  <= r_exp;
    end

endmodule
