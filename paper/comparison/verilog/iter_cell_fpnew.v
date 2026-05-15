// iter_cell_fpnew — plain-Verilog translation of FPnew's iteration cell.
//
// Original: ETH Zürich / University of Bologna, FPnew div_sqrt_mvp.
// File: fpnew/src/fpu_div_sqrt_mvp/hdl/iteration_div_sqrt_mvp.sv
// Engineer: Lei Li (lile@iis.ee.ethz.ch)
// Copyright 2018 ETH Zurich and University of Bologna.
// Solderpad Hardware License v0.51 (http://solderpad.org/licenses/SHL-0.51).
// Distributed AS IS, see License for terms.
//
// Translated to plain Verilog (no SystemVerilog) for use in Spirix's restoring/
// non-restoring chain experiments. Logic is byte-for-byte identical to the
// original; only port type syntax (logic → wire) and a `assign`-style sum was
// changed to satisfy iverilog/yosys default frontend.

module iter_cell_fpnew #(
    parameter WIDTH = 26
)(
    input  wire [WIDTH-1:0] A_DI,
    input  wire [WIDTH-1:0] B_DI,
    input  wire             Div_enable_SI,
    input  wire             Div_start_dly_SI,
    input  wire             Sqrt_enable_SI,
    input  wire [1:0]       D_DI,
    output wire [1:0]       D_DO,
    output wire [WIDTH-1:0] Sum_DO,
    output wire             Carry_out_DO
);
    wire D_carry_D  = D_DI[1] | D_DI[0];
    wire Sqrt_cin_D = Sqrt_enable_SI & D_carry_D;
    wire Cin_D      = Div_enable_SI ? 1'b0 : Sqrt_cin_D;

    assign D_DO[0] = ~D_DI[0];
    assign D_DO[1] = ~(D_DI[1] ^ D_DI[0]);
    assign {Carry_out_DO, Sum_DO} = A_DI + B_DI + Cin_D;
endmodule
