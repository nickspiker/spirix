// spirix_alu_round_pipe — 2-stage pipelined floor/ceil ALU
//
// Ops (1-bit select):
//   0: FLOOR(a)  — largest integer ≤ a
//   1: CEIL(a)   — smallest integer ≥ a = -floor(-a)
//
// Architecture: shared barrel shift with pipeline register.
//   S1: neg(a) || edge detect → mux(a/-a) → barrel → floor logic → register
//   S2: simplified neg (ceil) → output
//
// S2 uses inline neg instead of full spirix_neg because floor output is
// constrained: never POS_SMALL/NEG_SMALL, and classification flags from S1
// eliminate 64-bit equality comparisons from the critical path.
//
// Single barrel shift, one neg per stage. 1-cycle latency, 1 result/clock.
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
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {2'b01, {(MAX_FRAC-2){1'b0}}};
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

    // Floor result + classification flags (combinational)
    reg signed [MAX_FRAC-1:0] s1_fl_frac;
    reg signed [MAX_EXP-1:0]  s1_fl_exp;
    reg        s1_fl_is_zero;       // result is ZERO @ AMBIG
    reg        s1_fl_is_ambig;      // result has AMBIG exp (non-normal)
    reg        s1_fl_is_pos_half;   // result frac == POS_HALF
    reg        s1_fl_is_neg_one;    // result frac == NEG_ONE

    always @(*) begin
        s1_fl_is_zero     = 0;
        s1_fl_is_ambig    = 0;
        s1_fl_is_pos_half = 0;
        s1_fl_is_neg_one  = 0;

        if (~f_is_normal) begin
            s1_fl_is_ambig = 1;
            if (f_vanished) begin
                if (f_neg) begin
                    s1_fl_frac = NEG_ONE;
                    s1_fl_exp  = {MAX_EXP{1'b0}};
                    s1_fl_is_ambig   = 0;  // exp=0, normal
                    s1_fl_is_neg_one = 1;
                end else begin
                    s1_fl_frac = {MAX_FRAC{1'b0}};
                    s1_fl_exp  = AMBIG_EXP;
                    s1_fl_is_zero = 1;
                end
            end else begin
                // Passthrough non-normal (zero/inf/exploded/undef)
                s1_fl_frac = f_frac;
                s1_fl_exp  = f_exp;
                s1_fl_is_pos_half = (f_frac == POS_HALF);
                s1_fl_is_neg_one  = (f_frac == NEG_ONE);
            end
        end else if (f_exp_sub_one) begin
            if (f_neg) begin
                s1_fl_frac       = NEG_ONE;
                s1_fl_exp        = {MAX_EXP{1'b0}};
                s1_fl_is_neg_one = 1;
            end else begin
                s1_fl_frac    = {MAX_FRAC{1'b0}};
                s1_fl_exp     = AMBIG_EXP;
                s1_fl_is_zero = 1;
                s1_fl_is_ambig = 1;
            end
        end else if (f_exp_big) begin
            // Passthrough normal integer
            s1_fl_frac = f_frac;
            s1_fl_exp  = f_exp;
            s1_fl_is_pos_half = (f_frac == POS_HALF);
            s1_fl_is_neg_one  = (f_frac == NEG_ONE);
        end else begin
            // Masked normal
            s1_fl_frac = f_frac & f_mask;
            s1_fl_exp  = f_exp;
            s1_fl_is_pos_half = ((f_frac & f_mask) == POS_HALF);
            s1_fl_is_neg_one  = ((f_frac & f_mask) == NEG_ONE);
        end
    end

    // ================================================================
    //  PIPELINE REGISTER (S1 → S2)
    // ================================================================
    reg signed [MAX_FRAC-1:0] s2_frac;
    reg signed [MAX_EXP-1:0]  s2_exp;
    reg                        s2_op;
    reg [1:0]                  s2_exp_width;
    reg                        s2_is_zero, s2_is_ambig;
    reg                        s2_is_pos_half, s2_is_neg_one;

    always @(posedge clk) if (ce) begin
        s2_frac        <= s1_fl_frac;
        s2_exp         <= s1_fl_exp;
        s2_op          <= op;
        s2_exp_width   <= exp_width;
        s2_is_zero     <= s1_fl_is_zero;
        s2_is_ambig    <= s1_fl_is_ambig;
        s2_is_pos_half <= s1_fl_is_pos_half;
        s2_is_neg_one  <= s1_fl_is_neg_one;
    end

    // ================================================================
    //  STAGE 2: simplified inline neg (ceil) → output
    // ================================================================
    // Floor never produces POS_SMALL/NEG_SMALL, so those cases are gone.
    // Classification flags are registered — no 64-bit equality in S2.
    //
    // Cases:
    //   zero       → zero (passthrough)
    //   non-normal POS_HALF → NEG_ONE, same exp
    //   non-normal NEG_ONE  → POS_HALF, same exp
    //   non-normal other (zero/inf/undef) → passthrough (neg of passthrough)
    //   normal POS_HALF → NEG_ONE, exp-1 (with AMBIG check)
    //   normal NEG_ONE  → POS_HALF, exp+1 (with overflow check)
    //   normal other    → -frac, same exp

    // EXP-1 AMBIG check (for normal POS_HALF → NEG_ONE)
    wire signed [MAX_EXP-1:0] s2_exp_m1 = s2_exp - 1;
    wire em1_ambig_e3 = s2_exp_m1[7]  & ~|s2_exp_m1[6:0]   & &s2_exp_m1[63:8];
    wire em1_ambig_e4 = s2_exp_m1[15] & ~|s2_exp_m1[14:0]  & &s2_exp_m1[63:16];
    wire em1_ambig_e5 = s2_exp_m1[31] & ~|s2_exp_m1[30:0]  & &s2_exp_m1[63:32];
    wire s2_exp_m1_ambig = (s2_exp_width == 2'd0) ? em1_ambig_e3 :
                           (s2_exp_width == 2'd1) ? em1_ambig_e4 :
                           (s2_exp_width == 2'd2) ? em1_ambig_e5 :
                                                    (s2_exp_m1 == AMBIG_EXP);

    // EXP+1 overflow check (for normal NEG_ONE → POS_HALF)
    wire signed [MAX_EXP-1:0] s2_exp_p1 = s2_exp + 1;
    wire ep1_fits_e3 = &(s2_exp_p1[63:7]  ^~ {57{s2_exp_p1[7]}});
    wire ep1_fits_e4 = &(s2_exp_p1[63:15] ^~ {49{s2_exp_p1[15]}});
    wire ep1_fits_e5 = &(s2_exp_p1[63:31] ^~ {33{s2_exp_p1[31]}});
    wire ep1_e3_ambig = s2_exp_p1[7]  & ~|s2_exp_p1[6:0]   & &s2_exp_p1[63:8];
    wire ep1_e4_ambig = s2_exp_p1[15] & ~|s2_exp_p1[14:0]  & &s2_exp_p1[63:16];
    wire ep1_e5_ambig = s2_exp_p1[31] & ~|s2_exp_p1[30:0]  & &s2_exp_p1[63:32];
    wire s2_exp_p1_bad = (s2_exp_width == 2'd0) ? (!ep1_fits_e3 | ep1_e3_ambig) :
                         (s2_exp_width == 2'd1) ? (!ep1_fits_e4 | ep1_e4_ambig) :
                         (s2_exp_width == 2'd2) ? (!ep1_fits_e5 | ep1_e5_ambig) :
                                                  (s2_exp_p1 == AMBIG_EXP);

    // Negate mux (no 64-bit comparisons — uses registered flags)
    reg signed [MAX_FRAC-1:0] neg_frac;
    reg signed [MAX_EXP-1:0]  neg_exp;

    // Top-3-bits passthrough: zero, infinity, undefined all have uniform top 3
    // bits. 2 XOR gates — replaces 64-bit equality comparisons.
    wire s2_top3_same = (s2_frac[MAX_FRAC-1] == s2_frac[MAX_FRAC-2]) &
                        (s2_frac[MAX_FRAC-2] == s2_frac[MAX_FRAC-3]);

    always @(*) begin
        if (s2_is_zero) begin
            // neg(zero) = zero
            neg_frac = s2_frac;
            neg_exp  = s2_exp;
        end else if (s2_is_ambig) begin
            // Non-normal path (no POS_SMALL/NEG_SMALL possible)
            if (s2_top3_same) begin
                // Zero, infinity, undefined → passthrough
                neg_frac = s2_frac;
                neg_exp  = s2_exp;
            end else if (s2_is_pos_half) begin
                neg_frac = NEG_ONE;
                neg_exp  = s2_exp;
            end else if (s2_is_neg_one) begin
                neg_frac = POS_HALF;
                neg_exp  = s2_exp;
            end else begin
                // Exploded N1 (not POS_HALF/NEG_ONE) → simple negate
                neg_frac = -s2_frac;
                neg_exp  = s2_exp;
            end
        end else begin
            // Normal path
            if (s2_is_pos_half) begin
                if (s2_exp_m1_ambig) begin
                    neg_frac = {2'b11, {(MAX_FRAC-2){1'b0}}};  // NEG_SMALL
                    neg_exp  = AMBIG_EXP;
                end else begin
                    neg_frac = NEG_ONE;
                    neg_exp  = s2_exp_m1;
                end
            end else if (s2_is_neg_one) begin
                if (s2_exp_p1_bad) begin
                    neg_frac = POS_HALF;
                    neg_exp  = AMBIG_EXP;
                end else begin
                    neg_frac = POS_HALF;
                    neg_exp  = s2_exp_p1;
                end
            end else begin
                neg_frac = -s2_frac;
                neg_exp  = s2_exp;
            end
        end
    end

    // Output: ceil negates, floor passes through
    assign result_frac = s2_op ? neg_frac : s2_frac;
    assign result_exp  = s2_op ? neg_exp  : s2_exp;

endmodule
