// spirix_alu_round_pipe — 2-stage pipelined floor/ceil ALU
//
// Ops (1-bit select):
//   0: FLOOR(a)  — largest integer ≤ a
//   1: CEIL(a)   — smallest integer ≥ a = -floor(-a)
//
// Architecture: shared barrel shift with pipeline register.
//   S1: neg(a) || edge detect → mux(a/-a) → barrel → floor logic → register
//   S2: conditional neg (ceil) → output
//
// Single barrel shift, one neg per stage. 2-cycle latency, 1 result/clock.
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64

module spirix_alu_round_pipe #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                       clk,
    input  wire                       ce,
    input  wire                       op,       // 0=FLOOR, 1=CEIL
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    output wire signed [MAX_FRAC-1:0] result_frac,
    output wire signed [MAX_EXP-1:0]  result_exp
);

    // ================================================================
    //  CONSTANTS
    // ================================================================
    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    wire [6:0] frac_bits = (frac_width == 2'd0) ? 7'd8  :
                            (frac_width == 2'd1) ? 7'd16 :
                            (frac_width == 2'd2) ? 7'd32 : 7'd64;
    wire [6:0] frac_bits_m1 = frac_bits - 7'd1;

    // ================================================================
    //  STAGE 1: neg(a) + mux + barrel shift + floor logic
    // ================================================================

    // Pre-negate a (runs in parallel with direct-path edge detect)
    wire signed [MAX_FRAC-1:0] neg_a_frac;
    wire signed [MAX_EXP-1:0]  neg_a_exp;

    spirix_neg #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_neg_in (
        .frac_width(frac_width), .exp_width(exp_width),
        .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(neg_a_frac), .result_exp(neg_a_exp)
    );

    // Input mux: floor sees a (floor op) or -a (ceil op)
    wire signed [MAX_FRAC-1:0] f_frac = op ? neg_a_frac : a_frac;
    wire signed [MAX_EXP-1:0]  f_exp  = op ? neg_a_exp  : a_exp;

    // State detection on muxed input
    wire f_is_ambig  = (f_exp == AMBIG_EXP);
    wire f_is_normal = ~f_is_ambig;
    wire f_neg       = f_frac[MAX_FRAC-1];
    wire f_n1        = (f_frac[MAX_FRAC-1] != f_frac[MAX_FRAC-2]);
    wire f_n2        = ~f_n1 & (f_frac[MAX_FRAC-1] != f_frac[MAX_FRAC-3]);
    wire f_vanished  = f_n2;

    // Single barrel shift for floor mask
    wire [6:0] f_exp_small = f_exp[6:0];
    wire [6:0] f_zero_count = 7'd63 - f_exp_small;
    wire [MAX_FRAC-1:0] f_ones = {MAX_FRAC{1'b1}};
    wire [MAX_FRAC-1:0] f_mask = f_ones << f_zero_count;

    wire f_exp_sub_one = f_is_normal & (f_exp <= $signed({{(MAX_EXP-1){1'b0}}, 1'b0}));
    wire f_exp_big     = f_is_normal & (f_exp >= $signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}));

    // Floor result (combinational)
    reg signed [MAX_FRAC-1:0] s1_fl_frac;
    reg signed [MAX_EXP-1:0]  s1_fl_exp;

    always @(*) begin
        if (~f_is_normal) begin
            if (f_vanished) begin
                if (f_neg) begin
                    s1_fl_frac = NEG_ONE;
                    s1_fl_exp  = {MAX_EXP{1'b0}};
                end else begin
                    s1_fl_frac = {MAX_FRAC{1'b0}};
                    s1_fl_exp  = AMBIG_EXP;
                end
            end else begin
                s1_fl_frac = f_frac;
                s1_fl_exp  = f_exp;
            end
        end else if (f_exp_sub_one) begin
            if (f_neg) begin
                s1_fl_frac = NEG_ONE;
                s1_fl_exp  = {MAX_EXP{1'b0}};
            end else begin
                s1_fl_frac = {MAX_FRAC{1'b0}};
                s1_fl_exp  = AMBIG_EXP;
            end
        end else if (f_exp_big) begin
            s1_fl_frac = f_frac;
            s1_fl_exp  = f_exp;
        end else begin
            s1_fl_frac = f_frac & f_mask;
            s1_fl_exp  = f_exp;
        end
    end

    // ================================================================
    //  PIPELINE REGISTER (S1 → S2)
    // ================================================================
    reg signed [MAX_FRAC-1:0] s2_frac;
    reg signed [MAX_EXP-1:0]  s2_exp;
    reg                        s2_op;
    reg [1:0]                  s2_frac_width, s2_exp_width;

    always @(posedge clk) if (ce) begin
        s2_frac       <= s1_fl_frac;
        s2_exp        <= s1_fl_exp;
        s2_op         <= op;
        s2_frac_width <= frac_width;
        s2_exp_width  <= exp_width;
    end

    // ================================================================
    //  STAGE 2: conditional negate (ceil) → output
    // ================================================================
    wire signed [MAX_FRAC-1:0] neg_s2_frac;
    wire signed [MAX_EXP-1:0]  neg_s2_exp;

    spirix_neg #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_neg_out (
        .frac_width(s2_frac_width), .exp_width(s2_exp_width),
        .a_frac(s2_frac), .a_exp(s2_exp),
        .result_frac(neg_s2_frac), .result_exp(neg_s2_exp)
    );

    // Output: ceil negates, floor passes through
    assign result_frac = s2_op ? neg_s2_frac : s2_frac;
    assign result_exp  = s2_op ? neg_s2_exp  : s2_exp;

endmodule
