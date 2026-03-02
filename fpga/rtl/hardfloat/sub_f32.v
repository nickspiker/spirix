// HardFloat IEEE binary32 subtractor wrapper
//
// Wraps addRecFN(expWidth=8, sigWidth=24) with subOp=1
// Input/output are in HardFloat's recoded format (33 bits for binary32)
// Use fNToRecFN and recFNToFN for conversion to/from standard IEEE format
//
// control=0 : tininess detected after rounding
// roundingMode=0 : round to nearest even

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module sub_f32 (
    input  wire [32:0] a,              // recoded float32 (33 bits)
    input  wire [32:0] b,              // recoded float32 (33 bits)
    input  wire [2:0]  roundingMode,
    output wire [32:0] out,            // recoded float32 result
    output wire [4:0]  exceptionFlags
);
    addRecFN #(.expWidth(8), .sigWidth(24)) adder (
        .control      (`flControl_tininessAfterRounding),
        .subOp        (1'b1),
        .a            (a),
        .b            (b),
        .roundingMode (roundingMode),
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
