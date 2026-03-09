// spirix_alu_round — combinational floor/ceil ALU
//
// Ops (1-bit select):
//   0: FLOOR(a)  — largest integer ≤ a
//   1: CEIL(a)   — smallest integer ≥ a = -floor(-a)
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.
//
// Floor logic inlined (no external spirix_floor dependency).
// Ceil = -floor(-a) via spirix_neg. Parallel floor paths for timing.

module spirix_alu_round #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
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
    //  FLOOR path — direct floor(a)
    // ================================================================
    wire fl_is_ambig  = (a_exp == AMBIG_EXP);
    wire fl_is_normal = ~fl_is_ambig;
    wire fl_neg       = a_frac[MAX_FRAC-1];
    wire fl_n1        = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire fl_n2        = ~fl_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire fl_vanished  = fl_n2;

    // Floor mask: zero out fractional bits
    wire [6:0] fl_exp_small = a_exp[6:0];
    wire [6:0] fl_zero_count = 7'd63 - fl_exp_small;
    wire [MAX_FRAC-1:0] fl_ones = {MAX_FRAC{1'b1}};
    wire [MAX_FRAC-1:0] fl_mask = fl_ones << fl_zero_count;

    wire fl_exp_sub_one = fl_is_normal & (a_exp <= $signed({{(MAX_EXP-1){1'b0}}, 1'b0}));
    wire fl_exp_big     = fl_is_normal & (a_exp >= $signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}));

    reg signed [MAX_FRAC-1:0] floor_frac;
    reg signed [MAX_EXP-1:0]  floor_exp;

    always @(*) begin
        if (~fl_is_normal) begin
            if (fl_vanished) begin
                if (fl_neg) begin
                    floor_frac = NEG_ONE;
                    floor_exp  = {MAX_EXP{1'b0}};
                end else begin
                    floor_frac = {MAX_FRAC{1'b0}};
                    floor_exp  = AMBIG_EXP;
                end
            end else begin
                floor_frac = a_frac;
                floor_exp  = a_exp;
            end
        end else if (fl_exp_sub_one) begin
            if (fl_neg) begin
                floor_frac = NEG_ONE;
                floor_exp  = {MAX_EXP{1'b0}};
            end else begin
                floor_frac = {MAX_FRAC{1'b0}};
                floor_exp  = AMBIG_EXP;
            end
        end else if (fl_exp_big) begin
            floor_frac = a_frac;
            floor_exp  = a_exp;
        end else begin
            floor_frac = a_frac & fl_mask;
            floor_exp  = a_exp;
        end
    end

    // ================================================================
    //  CEIL path — ceil(a) = -floor(-a)
    //  Step 1: negate a
    //  Step 2: floor the negated value (parallel with floor path)
    //  Step 3: negate the result
    // ================================================================

    // Step 1: negate input
    wire signed [MAX_FRAC-1:0] neg_a_frac;
    wire signed [MAX_EXP-1:0]  neg_a_exp;

    spirix_neg #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_neg_in (
        .frac_width(frac_width), .exp_width(exp_width),
        .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(neg_a_frac), .result_exp(neg_a_exp)
    );

    // Step 2: floor(-a) — parallel floor on negated input
    wire cl_is_ambig  = (neg_a_exp == AMBIG_EXP);
    wire cl_is_normal = ~cl_is_ambig;
    wire cl_neg       = neg_a_frac[MAX_FRAC-1];
    wire cl_n1        = (neg_a_frac[MAX_FRAC-1] != neg_a_frac[MAX_FRAC-2]);
    wire cl_n2        = ~cl_n1 & (neg_a_frac[MAX_FRAC-1] != neg_a_frac[MAX_FRAC-3]);
    wire cl_vanished  = cl_n2;

    wire [6:0] cl_exp_small = neg_a_exp[6:0];
    wire [6:0] cl_zero_count = 7'd63 - cl_exp_small;
    wire [MAX_FRAC-1:0] cl_mask = fl_ones << cl_zero_count;

    wire cl_exp_sub_one = cl_is_normal & (neg_a_exp <= $signed({{(MAX_EXP-1){1'b0}}, 1'b0}));
    wire cl_exp_big     = cl_is_normal & (neg_a_exp >= $signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}));

    reg signed [MAX_FRAC-1:0] floor_neg_frac;
    reg signed [MAX_EXP-1:0]  floor_neg_exp;

    always @(*) begin
        if (~cl_is_normal) begin
            if (cl_vanished) begin
                if (cl_neg) begin
                    floor_neg_frac = NEG_ONE;
                    floor_neg_exp  = {MAX_EXP{1'b0}};
                end else begin
                    floor_neg_frac = {MAX_FRAC{1'b0}};
                    floor_neg_exp  = AMBIG_EXP;
                end
            end else begin
                floor_neg_frac = neg_a_frac;
                floor_neg_exp  = neg_a_exp;
            end
        end else if (cl_exp_sub_one) begin
            if (cl_neg) begin
                floor_neg_frac = NEG_ONE;
                floor_neg_exp  = {MAX_EXP{1'b0}};
            end else begin
                floor_neg_frac = {MAX_FRAC{1'b0}};
                floor_neg_exp  = AMBIG_EXP;
            end
        end else if (cl_exp_big) begin
            floor_neg_frac = neg_a_frac;
            floor_neg_exp  = neg_a_exp;
        end else begin
            floor_neg_frac = neg_a_frac & cl_mask;
            floor_neg_exp  = neg_a_exp;
        end
    end

    // Step 3: negate floor(-a)
    wire signed [MAX_FRAC-1:0] ceil_frac;
    wire signed [MAX_EXP-1:0]  ceil_exp;

    spirix_neg #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_neg_out (
        .frac_width(frac_width), .exp_width(exp_width),
        .a_frac(floor_neg_frac), .a_exp(floor_neg_exp),
        .result_frac(ceil_frac), .result_exp(ceil_exp)
    );

    // ================================================================
    //  OUTPUT MUX
    // ================================================================
    assign result_frac = op ? ceil_frac : floor_frac;
    assign result_exp  = op ? ceil_exp  : floor_exp;

endmodule
