// HardFloat IEEE binary32 iterative divider wrapper
//
// Wraps divSqrtRecFN_small(expWidth=8, sigWidth=24) with sqrtOp=0
// Sequential: ~26 cycles per divide. Handshake: inReady/inValid/outValid.

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module div_f32 (
    input  wire        nReset,
    input  wire        clock,
    output wire        inReady,
    input  wire        inValid,
    input  wire [32:0] a,
    input  wire [32:0] b,
    input  wire [2:0]  roundingMode,
    output wire        outValid,
    output wire [32:0] out,
    output wire [4:0]  exceptionFlags
);
    wire sqrtOpOut;
    divSqrtRecFN_small #(.expWidth(8), .sigWidth(24)) ds (
        .nReset        (nReset),
        .clock         (clock),
        .control       (`flControl_tininessAfterRounding),
        .inReady       (inReady),
        .inValid       (inValid),
        .sqrtOp        (1'b0),
        .a             (a),
        .b             (b),
        .roundingMode  (roundingMode),
        .outValid      (outValid),
        .sqrtOpOut     (sqrtOpOut),
        .out           (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
