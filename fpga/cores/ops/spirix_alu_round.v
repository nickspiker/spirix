// spirix_alu_round — combinational floor/ceil/round ALU
//
// Ops (2-bit select):
//   00: FLOOR(a)  — largest integer ≤ a        (zero right half)
//   01: CEIL(a)   — smallest integer ≥ a        (floor + conditional +1)
//   10: ROUND(a)  — banker's rounding            (floor + GRS-based +1)
//
// FRAC moved to spirix_alu_addbit_pipe (shares CLZ+normalize barrel, 2-stage).
//
// Direct ceil/round: mask → floor, then add 1_ulp if fractional bits warrant it.
// No spirix_neg dependency.
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64

module spirix_alu_round #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire [1:0]                 op,       // 00=FLOOR, 01=CEIL, 10=ROUND
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

    wire [MAX_FRAC-1:0] f_ones = {MAX_FRAC{1'b1}};

    // ================================================================
    //  BARREL SHIFT + INPUT CLASSIFICATION
    // ================================================================
    wire [6:0] a_exp_small = a_exp[6:0];
    wire [6:0] a_zero_count = 7'd63 - a_exp_small;
    wire [MAX_FRAC-1:0] f_mask = f_ones << a_zero_count;
    wire [MAX_FRAC-1:0] mask_lsb = f_mask ^ {f_mask[MAX_FRAC-2:0], 1'b0};

    wire a_is_ambig  = (a_exp == AMBIG_EXP);
    wire a_is_normal = ~a_is_ambig;
    wire a_neg       = a_frac[MAX_FRAC-1];
    wire a_n1        = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire a_n2        = ~a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire a_vanished  = a_n2;

    wire a_exp_sub_one = a_is_normal & (a_exp <= $signed({{(MAX_EXP-1){1'b0}}, 1'b0}));
    wire a_exp_big     = a_is_normal & (a_exp >= $signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}));
    wire a_is_mid_range = a_is_normal & ~a_exp_sub_one & ~a_exp_big;

    // ================================================================
    //  FLOOR RESULT (all cases)
    // ================================================================
    reg signed [MAX_FRAC-1:0] floor_frac;
    reg signed [MAX_EXP-1:0]  floor_exp;

    always @(*) begin
        if (~a_is_normal) begin
            if (a_vanished) begin
                if (a_neg) begin
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
        end else if (a_exp_sub_one) begin
            if (a_neg) begin
                floor_frac = NEG_ONE;
                floor_exp  = {MAX_EXP{1'b0}};
            end else begin
                floor_frac = {MAX_FRAC{1'b0}};
                floor_exp  = AMBIG_EXP;
            end
        end else if (a_exp_big) begin
            floor_frac = a_frac;
            floor_exp  = a_exp;
        end else begin
            floor_frac = a_frac & f_mask;
            floor_exp  = a_exp;
        end
    end

    // ================================================================
    //  EDGE CASE RESULT (ceil/round, non-mid-range)
    // ================================================================
    wire had_frac_sub = (a_exp != {MAX_EXP{1'b0}}) | (|a_frac[MAX_FRAC-2:0]);

    wire signed [MAX_EXP-1:0] exp_p1 = a_exp + 1;
    wire signed [MAX_EXP-1:0] exp_m1 = a_exp - 1;

    wire ep1_ambig_e3 = exp_p1[7]  & ~|exp_p1[6:0];
    wire ep1_ambig_e4 = exp_p1[15] & ~|exp_p1[14:0];
    wire ep1_ambig_e5 = exp_p1[31] & ~|exp_p1[30:0];
    wire exp_p1_bad = (exp_width == 2'd0) ? ep1_ambig_e3 :
                      (exp_width == 2'd1) ? ep1_ambig_e4 :
                      (exp_width == 2'd2) ? ep1_ambig_e5 :
                                            (exp_p1 == AMBIG_EXP);

    wire a_is_neg_one = (a_frac == NEG_ONE);

    localparam signed [MAX_FRAC-1:0] NEG_HALF = {2'b11, {(MAX_FRAC-2){1'b0}}};
    wire a_gt_pos_half = $signed(a_frac) > $signed(POS_HALF);
    wire a_lt_neg_half = $signed(a_frac) < $signed(NEG_HALF);

    wire is_ceil  = (op == 2'd1);
    wire is_round = (op == 2'd2);

    reg signed [MAX_FRAC-1:0] edge_frac;
    reg signed [MAX_EXP-1:0]  edge_exp;

    always @(*) begin
        if (~a_is_normal) begin
            if (a_vanished) begin
                if (is_ceil & ~a_neg) begin
                    edge_frac = POS_HALF;
                    edge_exp  = {{(MAX_EXP-1){1'b0}}, 1'b1};
                end else begin
                    edge_frac = {MAX_FRAC{1'b0}};
                    edge_exp  = AMBIG_EXP;
                end
            end else begin
                edge_frac = a_frac;
                edge_exp  = a_exp;
            end
        end else if (a_exp_sub_one) begin
            if (is_round) begin
                if (a_exp != {MAX_EXP{1'b0}}) begin
                    edge_frac = {MAX_FRAC{1'b0}};
                    edge_exp  = AMBIG_EXP;
                end else if (a_gt_pos_half) begin
                    edge_frac = POS_HALF;
                    edge_exp  = {{(MAX_EXP-1){1'b0}}, 1'b1};
                end else if (a_lt_neg_half) begin
                    edge_frac = NEG_ONE;
                    edge_exp  = {MAX_EXP{1'b0}};
                end else begin
                    edge_frac = {MAX_FRAC{1'b0}};
                    edge_exp  = AMBIG_EXP;
                end
            end else begin
                if (~a_neg) begin
                    edge_frac = POS_HALF;
                    edge_exp  = {{(MAX_EXP-1){1'b0}}, 1'b1};
                end else if (~had_frac_sub) begin
                    edge_frac = NEG_ONE;
                    edge_exp  = {MAX_EXP{1'b0}};
                end else begin
                    edge_frac = {MAX_FRAC{1'b0}};
                    edge_exp  = AMBIG_EXP;
                end
            end
        end else begin
            if (is_ceil & a_is_neg_one & exp_p1_bad) begin
                edge_frac = NEG_ONE;
                edge_exp  = AMBIG_EXP;
            end else begin
                edge_frac = a_frac;
                edge_exp  = a_exp;
            end
        end
    end

    // ================================================================
    //  MID-RANGE: CEIL/ROUND ADDITION
    // ================================================================
    wire [MAX_FRAC-1:0] masked = a_frac & f_mask;
    wire had_frac = |(a_frac & ~f_mask);

    // GRS bits for banker's rounding
    wire [MAX_FRAC-1:0] guard_pos = {1'b0, mask_lsb[MAX_FRAC-1:1]};
    wire guard     = |(a_frac & guard_pos);
    wire sticky    = |(a_frac & (guard_pos - {{(MAX_FRAC-1){1'b0}}, 1'b1}));
    wire floor_lsb = |(masked & mask_lsb);

    // Ceil: add if fractional bits cut. Round: add if guard & (sticky | odd)
    wire round_up = guard & (sticky | floor_lsb);
    wire do_add = (op == 2'd1) ? had_frac : round_up;

    wire [MAX_FRAC-1:0] add_addend = do_add ? mask_lsb : {MAX_FRAC{1'b0}};
    wire signed [MAX_FRAC-1:0] add_raw = masked + add_addend;

    wire add_overflow = (a_neg != add_raw[MAX_FRAC-1]);
    wire add_n2 = a_n1 & ~add_overflow & add_raw[MAX_FRAC-1] & add_raw[MAX_FRAC-2];

    reg signed [MAX_FRAC-1:0] add_mid_frac;
    reg signed [MAX_EXP-1:0]  add_mid_exp;

    always @(*) begin
        if (~do_add) begin
            add_mid_frac = floor_frac;
            add_mid_exp  = floor_exp;
        end else if (add_overflow & a_neg) begin
            add_mid_frac = {MAX_FRAC{1'b0}};
            add_mid_exp  = a_exp;
        end else if (add_overflow & ~a_neg) begin
            add_mid_frac = POS_HALF;
            add_mid_exp  = exp_p1_bad ? AMBIG_EXP : exp_p1;
        end else if (add_n2) begin
            add_mid_frac = {add_raw[MAX_FRAC-2:0], 1'b0};
            add_mid_exp  = exp_m1;
        end else begin
            add_mid_frac = add_raw;
            add_mid_exp  = a_exp;
        end
    end

    // ================================================================
    //  OUTPUT MUX
    // ================================================================
    reg signed [MAX_FRAC-1:0] out_frac;
    reg signed [MAX_EXP-1:0]  out_exp;

    always @(*) begin
        if (op == 2'd0) begin
            out_frac = floor_frac;
            out_exp  = floor_exp;
        end else if (~a_is_mid_range) begin
            out_frac = edge_frac;
            out_exp  = edge_exp;
        end else begin
            out_frac = add_mid_frac;
            out_exp  = add_mid_exp;
        end
    end

    assign result_frac = out_frac;
    assign result_exp  = out_exp;

endmodule
