// HardFloat IEEE binary32 adder wrapper
//
// Wraps addRecFN(expWidth=8, sigWidth=24) with subOp=0
// Input/output are in HardFloat's recoded format (33 bits for binary32)

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module add_f32 (
    input  wire [32:0] a,              // recoded float32 (33 bits)
    input  wire [32:0] b,              // recoded float32 (33 bits)
    input  wire [2:0]  roundingMode,
    output wire [32:0] out,            // recoded float32 result
    output wire [4:0]  exceptionFlags
);
    addRecFN #(.expWidth(8), .sigWidth(24)) adder (
        .control      (`flControl_tininessAfterRounding),
        .subOp        (1'b0),
        .a            (a),
        .b            (b),
        .roundingMode (roundingMode),
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
