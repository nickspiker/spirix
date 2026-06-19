// spirix_neg — combinational negate
//
// Spirix negation with edge case handling.
// POS_HALF ↔ NEG_ONE with exponent adjust.
// POS_SMALL ↔ NEG_SMALL for non-normal.
// Zero, infinity, undefined pass thru unchanged.
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.

module spirix_neg #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
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
    localparam signed [MAX_FRAC-1:0] POS_SMALL = {3'b001, {(MAX_FRAC-3){1'b0}}};
    localparam signed [MAX_FRAC-1:0] NEG_SMALL = {2'b11, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // ================================================================
    //  STATE DETECTION
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire a_is_normal = ~a_is_ambig;

    wire a_frac_zero = (a_frac == {MAX_FRAC{1'b0}});

    // frac_neg1_val: all-ones in active width
    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                               {64'hFFFFFFFFFFFFFFFF};
    wire a_frac_neg1 = (a_frac == frac_neg1_val);
    wire a_n0 = a_frac_zero | a_frac_neg1;

    // Top 3 bits uniform = zero/inf/undef (passthrough targets)
    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire a_passthrough = a_n0 | (~a_n0 & a_top3);  // zero, inf, or undef

    // Special fractions
    wire a_is_pos_half  = (a_frac == POS_HALF);
    wire a_is_neg_one   = (a_frac == NEG_ONE);
    wire a_is_pos_small = (a_frac == POS_SMALL);
    wire a_is_neg_small = (a_frac == NEG_SMALL);

    // ================================================================
    //  EXP-1 AMBIG CHECK (for normal POS_HALF negate)
    // ================================================================
    wire signed [MAX_EXP-1:0] a_exp_m1 = a_exp - 1;

    // Width-specific AMBIG pattern on a_exp_m1
    wire em1_ambig_e3 = a_exp_m1[7]  & ~|a_exp_m1[6:0]   & &a_exp_m1[63:8];
    wire em1_ambig_e4 = a_exp_m1[15] & ~|a_exp_m1[14:0]  & &a_exp_m1[63:16];
    wire em1_ambig_e5 = a_exp_m1[31] & ~|a_exp_m1[30:0]  & &a_exp_m1[63:32];
    wire a_exp_m1_ambig = (exp_width == 2'd0) ? em1_ambig_e3 :
                          (exp_width == 2'd1) ? em1_ambig_e4 :
                          (exp_width == 2'd2) ? em1_ambig_e5 :
                                                (a_exp_m1 == AMBIG_EXP);

    // ================================================================
    //  EXP+1 BAD CHECK (for normal NEG_ONE negate)
    // ================================================================
    wire signed [MAX_EXP-1:0] a_exp_p1 = a_exp + 1;

    wire ep1_fits_e3 = &(a_exp_p1[63:7]  ^~ {57{a_exp_p1[7]}});
    wire ep1_fits_e4 = &(a_exp_p1[63:15] ^~ {49{a_exp_p1[15]}});
    wire ep1_fits_e5 = &(a_exp_p1[63:31] ^~ {33{a_exp_p1[31]}});
    wire ep1_e3_ambig = a_exp_p1[7]  & ~|a_exp_p1[6:0]   & &a_exp_p1[63:8];
    wire ep1_e4_ambig = a_exp_p1[15] & ~|a_exp_p1[14:0]  & &a_exp_p1[63:16];
    wire ep1_e5_ambig = a_exp_p1[31] & ~|a_exp_p1[30:0]  & &a_exp_p1[63:32];
    wire a_exp_p1_bad = (exp_width == 2'd0) ? (!ep1_fits_e3 | ep1_e3_ambig) :
                        (exp_width == 2'd1) ? (!ep1_fits_e4 | ep1_e4_ambig) :
                        (exp_width == 2'd2) ? (!ep1_fits_e5 | ep1_e5_ambig) :
                                              (a_exp_p1 == AMBIG_EXP);

    // ================================================================
    //  NEGATE LOGIC
    // ================================================================
    reg signed [MAX_FRAC-1:0] r_frac;
    reg signed [MAX_EXP-1:0]  r_exp;

    always @(*) begin
        if (~a_is_normal) begin
            // Non-normal path
            if (a_passthrough) begin
                // Zero, infinity, undefined → unchanged
                r_frac = a_frac;
                r_exp  = a_exp;
            end else if (a_is_pos_half) begin
                r_frac = NEG_ONE;
                r_exp  = a_exp;
            end else if (a_is_neg_one) begin
                r_frac = POS_HALF;
                r_exp  = a_exp;
            end else if (a_is_pos_small) begin
                r_frac = NEG_SMALL;
                r_exp  = a_exp;
            end else if (a_is_neg_small) begin
                r_frac = POS_SMALL;
                r_exp  = a_exp;
            end else begin
                r_frac = -a_frac;
                r_exp  = a_exp;
            end
        end else begin
            // Normal path
            if (a_is_pos_half) begin
                // POS_HALF → NEG_ONE, exp-1
                // If exp-1 hits AMBIG → NEG_SMALL (vanish boundary)
                if (a_exp_m1_ambig) begin
                    r_frac = NEG_SMALL;
                    r_exp  = AMBIG_EXP;
                end else begin
                    r_frac = NEG_ONE;
                    r_exp  = a_exp_m1;
                end
            end else if (a_is_neg_one) begin
                // NEG_ONE → POS_HALF, exp+1
                // If exp+1 overflows active width, Rust wraps to AMBIG
                // Result: POS_HALF @ AMBIG (exploded positive)
                if (a_exp_p1_bad) begin
                    r_frac = POS_HALF;
                    r_exp  = AMBIG_EXP;
                end else begin
                    r_frac = POS_HALF;
                    r_exp  = a_exp_p1;
                end
            end else begin
                r_frac = -a_frac;
                r_exp  = a_exp;
            end
        end
    end

    assign result_frac = r_frac;
    assign result_exp  = r_exp;

endmodule
