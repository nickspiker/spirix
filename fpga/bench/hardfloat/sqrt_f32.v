// HardFloat IEEE binary32 iterative square root wrapper
//
// Wraps divSqrtRecFN_small(expWidth=8, sigWidth=24) with sqrtOp=1
// Sequential: ~25 cycles per sqrt. b input unused (tied to 0).

`include "HardFloat_consts.vi"
`include "HardFloat_specialize.vi"

module sqrt_f32 (
    input  wire        nReset,
    input  wire        clock,
    output wire        inReady,
    input  wire        inValid,
    input  wire [32:0] a,
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
        .sqrtOp        (1'b1),
        .a             (a),
        .b             (33'b0),
        .roundingMode  (roundingMode),
        .outValid      (outValid),
        .sqrtOpOut     (sqrtOpOut),
        .out           (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
