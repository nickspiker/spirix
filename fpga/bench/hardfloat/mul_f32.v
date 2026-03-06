// HardFloat IEEE binary32 multiplier wrapper
//
// Wraps mulRecFN(expWidth=8, sigWidth=24)
// Input/output in HardFloat's recoded format (33 bits for binary32)

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module mul_f32 (
    input  wire [32:0] a,
    input  wire [32:0] b,
    input  wire [2:0]  roundingMode,
    output wire [32:0] out,
    output wire [4:0]  exceptionFlags
);
    mulRecFN #(.expWidth(8), .sigWidth(24)) mul (
        .control      (`flControl_tininessAfterRounding),
        .a            (a),
        .b            (b),
        .roundingMode (roundingMode),
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
