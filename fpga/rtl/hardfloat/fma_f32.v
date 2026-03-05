// HardFloat IEEE binary32 fused multiply-add wrapper
//
// Wraps mulAddRecFN(expWidth=8, sigWidth=24)
// Input/output in HardFloat's recoded format (33 bits for binary32)
// op[1:0]: 00 = a*b+c, 01 = a*b-c, 10 = -(a*b)+c, 11 = -(a*b)-c

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module fma_f32 (
    input  wire [32:0] a,
    input  wire [32:0] b,
    input  wire [32:0] c,
    input  wire [1:0]  op,
    input  wire [2:0]  roundingMode,
    output wire [32:0] out,
    output wire [4:0]  exceptionFlags
);
    mulAddRecFN #(.expWidth(8), .sigWidth(24)) fma (
        .control      (`flControl_tininessAfterRounding),
        .op           (op),
        .a            (a),
        .b            (b),
        .c            (c),
        .roundingMode (roundingMode),
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
