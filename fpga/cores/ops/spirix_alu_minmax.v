// spirix_alu_minmax — combinational MIN/MAX
//
// Ops (1-bit select):
//   0: MIN(a, b)   — minimum (with unordered detection)
//   1: MAX(a, b)   — maximum (with unordered detection)
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.

module spirix_alu_minmax #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                       op,       // 0=MIN, 1=MAX
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output wire signed [MAX_FRAC-1:0] result_frac,
    output wire signed [MAX_EXP-1:0]  result_exp
);

    // ================================================================
    //  CONSTANTS
    // ================================================================
    localparam signed [MAX_EXP-1:0] AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // Undefined prefix constants (MSB-aligned from 8-bit patterns)
    // MAX_UNORDERED = 0x19 = 0b00011001
    localparam signed [MAX_FRAC-1:0] UNDEF_MAX = {8'h19, {(MAX_FRAC-8){1'b0}}};
    // MIN_UNORDERED = 0xE6 = 0b11100110
    localparam signed [MAX_FRAC-1:0] UNDEF_MIN = {8'hE6, {(MAX_FRAC-8){1'b0}}};

    // ================================================================
    //  CMP — instantiate spirix_cmp
    // ================================================================
    wire cmp_lt, cmp_eq, cmp_gt, cmp_unord;

    spirix_cmp #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_cmp (
        .frac_width(frac_width), .exp_width(exp_width),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .cmp_lt(cmp_lt), .cmp_eq(cmp_eq),
        .cmp_gt(cmp_gt), .cmp_unord(cmp_unord)
    );

    // ================================================================
    //  EDGE CASE DETECTION
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);

    wire a_frac_zero = (a_frac == {MAX_FRAC{1'b0}});
    wire b_frac_zero = (b_frac == {MAX_FRAC{1'b0}});

    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                               {64'hFFFFFFFFFFFFFFFF};
    wire a_frac_neg1 = (a_frac == frac_neg1_val);
    wire b_frac_neg1 = (b_frac == frac_neg1_val);

    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;

    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire b_top3 = (b_frac[MAX_FRAC-1] == b_frac[MAX_FRAC-2]) &
                  (b_frac[MAX_FRAC-2] == b_frac[MAX_FRAC-3]);
    wire a_undef = ~a_n0 & a_top3;
    wire b_undef = ~b_n0 & b_top3;

    wire a_is_inf = a_is_ambig & a_frac_neg1;
    wire b_is_inf = b_is_ambig & b_frac_neg1;

    wire a_n1 = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire b_n1 = (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-2]);
    wire a_n2 = ~a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire b_n2 = ~b_n1 & (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-3]);
    wire a_exploded = a_is_ambig & a_n1;
    wire b_exploded = b_is_ambig & b_n1;
    wire a_vanished = a_n2;
    wire b_vanished = b_n2;

    wire a_neg = a_frac[MAX_FRAC-1];
    wire b_neg = b_frac[MAX_FRAC-1];
    wire same_sign = (a_neg == b_neg);

    // ================================================================
    //  UNORDERED DETECTION
    // ================================================================
    wire either_undef = (a_undef & a_is_ambig) | (b_undef & b_is_ambig);
    wire either_inf = a_is_inf | b_is_inf;
    wire same_sign_vanished = a_vanished & b_vanished & same_sign;
    wire same_sign_exploded = a_exploded & b_exploded & same_sign;
    wire minmax_unord = either_undef | either_inf | same_sign_vanished | same_sign_exploded;

    // For undefined passthrough: first undefined wins
    wire [MAX_FRAC-1:0] undef_passthru_frac = (a_undef & a_is_ambig) ? a_frac : b_frac;
    wire [MAX_EXP-1:0]  undef_passthru_exp  = (a_undef & a_is_ambig) ? a_exp  : b_exp;

    // ================================================================
    //  OUTPUT MUX
    // ================================================================
    reg signed [MAX_FRAC-1:0] r_frac;
    reg signed [MAX_EXP-1:0]  r_exp;

    always @(*) begin
        if (either_undef) begin
            r_frac = undef_passthru_frac;
            r_exp  = undef_passthru_exp;
        end else if (either_inf | same_sign_vanished | same_sign_exploded) begin
            r_frac = op ? UNDEF_MAX : UNDEF_MIN;
            r_exp  = AMBIG_EXP;
        end else if (op) begin
            // MAX: pick a if greater or equal, else b
            if (cmp_gt | cmp_eq) begin
                r_frac = a_frac;
                r_exp  = a_exp;
            end else begin
                r_frac = b_frac;
                r_exp  = b_exp;
            end
        end else begin
            // MIN: pick a if less or equal, else b
            if (cmp_lt | cmp_eq) begin
                r_frac = a_frac;
                r_exp  = a_exp;
            end else begin
                r_frac = b_frac;
                r_exp  = b_exp;
            end
        end
    end

    assign result_frac = r_frac;
    assign result_exp  = r_exp;

endmodule
