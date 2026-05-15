// FP32-only wrapper around div_sqrt_mvp_wrapper.
// Pins Format_sel = FP32, RM = RNE, Precision_ctl = 0, upper 32 bits of operands = 0.
// Yosys will const-prop these and remove the FP64/FP16/FP16ALT datapaths,
// giving an FP32-only LUT4 count comparable to Spirix's spirix_divide / spirix_sqrt.
module fpnew_divsqrt_fp32_wrap (
    input  wire        Clk_CI,
    input  wire        Rst_RBI,
    input  wire        Div_start_SI,
    input  wire        Sqrt_start_SI,
    input  wire [31:0] op_a,
    input  wire [31:0] op_b,
    input  wire        Kill_SI,
    output wire [31:0] result,
    output wire [4:0]  Fflags_SO,
    output wire        Ready_SO,
    output wire        Done_SO
);
    wire [63:0] result64;
    div_sqrt_mvp_wrapper #(.PrePipeline_depth_S(0), .PostPipeline_depth_S(0)) dut (
        .Clk_CI          (Clk_CI),
        .Rst_RBI         (Rst_RBI),
        .Div_start_SI    (Div_start_SI),
        .Sqrt_start_SI   (Sqrt_start_SI),
        .Operand_a_DI    ({32'b0, op_a}),
        .Operand_b_DI    ({32'b0, op_b}),
        .RM_SI           (3'b000),       // RNE
        .Precision_ctl_SI(6'b0),
        .Format_sel_SI   (2'b00),        // FP32
        .Kill_SI         (Kill_SI),
        .Result_DO       (result64),
        .Fflags_SO       (Fflags_SO),
        .Ready_SO        (Ready_SO),
        .Done_SO         (Done_SO)
    );
    assign result = result64[31:0];
endmodule
