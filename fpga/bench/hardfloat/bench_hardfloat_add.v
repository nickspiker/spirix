// Registered wrapper for HardFloat add — gives nextpnr a real clock path so it can report actual Fmax through the combinational logic.

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module bench_hardfloat_add (
    input  wire        clk,
    input  wire [32:0] a_in,
    input  wire [32:0] b_in,
    output reg  [32:0] out_out,
    output reg  [4:0]  flags_out
);

    // Input registers
    reg [32:0] a, b;
    always @(posedge clk) begin
        a <= a_in;
        b <= b_in;
    end

    // Combinational add
    wire [32:0] r_out;
    wire [4:0]  r_flags;
    add_f32 add (
        .a(a), .b(b),
        .roundingMode(3'b000),
        .out(r_out),
        .exceptionFlags(r_flags)
    );

    // Output registers
    always @(posedge clk) begin
        out_out   <= r_out;
        flags_out <= r_flags;
    end

endmodule
