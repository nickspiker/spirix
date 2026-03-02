// Registered wrapper for 4-stage pipelined Spirix add.

module bench_spirix_add_pipe4 (
    input  wire clk,
    input  wire signed [24:0] a_frac_in,
    input  wire signed [7:0]  a_exp_in,
    input  wire signed [24:0] b_frac_in,
    input  wire signed [7:0]  b_exp_in,
    output wire signed [24:0] result_frac_out,
    output wire signed [7:0]  result_exp_out
);

    // Input registers (I/O pad capture)
    reg signed [24:0] a_frac, b_frac;
    reg signed [7:0]  a_exp,  b_exp;
    always @(posedge clk) begin
        a_frac <= a_frac_in;
        a_exp  <= a_exp_in;
        b_frac <= b_frac_in;
        b_exp  <= b_exp_in;
    end

    // 4-stage pipelined add (output is registered)
    spirix_add_pipe4 #(.FRAC_BITS(25), .EXP_BITS(8)) add (
        .clk(clk),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(result_frac_out), .result_exp(result_exp_out)
    );

endmodule
