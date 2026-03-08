// spirix_alu_bitwise — single-stage combinational bitwise/comparison ALU
//
// Supports AND, OR, XOR, NOT, SHL, SHR, CMP, NEGATE, MAGNITUDE, SIGN, FLOOR
// at runtime-selectable precision:
//   frac_width[1:0]: 00=8, 01=16, 10=32, 11=64
//   exp_width[1:0]:  00=8, 01=16, 10=32, 11=64
//
// Physical datapath is MAX_FRAC=64 bits. ALL inputs/outputs are MSB-aligned
// (Spirix sa() convention — left-handed cast).
//
// Single-stage: all logic combinational, output registered on clk+ce.
// No multiplier → barrel+CLZ is the critical path (same depth as addsub).
//
// op[3:0]: 0=AND 1=OR 2=XOR 3=NOT 4=SHL 5=SHR 6=CMP 7=NEGATE 8=MAGNITUDE 9=SIGN 10=FLOOR

module spirix_alu_bitwise #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                       clk,
    input  wire                       ce,
    input  wire [3:0]                 op,
    input  wire [1:0]                 frac_width, // 00=8 01=16 10=32 11=64
    input  wire [1:0]                 exp_width,  // 00=8 01=16 10=32 11=64

    // ALL inputs MSB-aligned (sa() convention)
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,

    output reg  signed [MAX_FRAC-1:0] result_frac,
    output reg  signed [MAX_EXP-1:0]  result_exp,

    // CMP flags (valid when op==CMP)
    output reg                        cmp_lt,
    output reg                        cmp_eq,
    output reg                        cmp_gt,
    output reg                        cmp_unord
);

    localparam OP_AND  = 4'd0;
    localparam OP_OR   = 4'd1;
    localparam OP_XOR  = 4'd2;
    localparam OP_NOT  = 4'd3;
    localparam OP_SHL  = 4'd4;
    localparam OP_SHR  = 4'd5;
    localparam OP_CMP  = 4'd6;
    localparam OP_NEG  = 4'd7;
    localparam OP_MAG  = 4'd8;
    localparam OP_SIGN = 4'd9;
    localparam OP_FLOOR = 4'd10;

    localparam INT_BITS   = 2 * MAX_FRAC;  // 128
    localparam SHIFT_BITS = $clog2(INT_BITS); // 7

    wire is_bitwise = (op <= OP_XOR);

    // Width decode
    wire [6:0] frac_bits = (frac_width == 2'd0) ? 7'd8  :
                           (frac_width == 2'd1) ? 7'd16 :
                           (frac_width == 2'd2) ? 7'd32 : 7'd64;

    // ========== MSB-aligned constants ==========
    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {2'b01, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_SMALL = {3'b001, {(MAX_FRAC-3){1'b0}}};
    localparam signed [MAX_FRAC-1:0] NEG_SMALL = {2'b11, {(MAX_FRAC-2){1'b0}}};

    localparam signed [MAX_EXP-1:0] AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    localparam signed [MAX_FRAC-1:0] UNDEF_AND  = {8'h12, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_OR   = {8'hED, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_XOR  = {8'h11, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_SIGN = {8'hE4, 56'b0};

    // ========== State detection ==========
    wire a_n1   = (a_frac[63] != a_frac[62]);
    wire b_n1   = (b_frac[63] != b_frac[62]);
    wire a_n2   = ~a_n1 & (a_frac[63] != a_frac[61]);
    wire b_n2   = ~b_n1 & (b_frac[63] != b_frac[61]);
    wire a_top3 = (a_frac[63] == a_frac[62]) & (a_frac[62] == a_frac[61]);
    wire b_top3 = (b_frac[63] == b_frac[62]) & (b_frac[62] == b_frac[61]);

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

    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);

    wire a_is_zero  = a_is_ambig & a_frac_zero;
    wire a_is_inf   = a_is_ambig & a_frac_neg1;
    wire a_exploded = a_is_ambig & a_n1;
    wire a_vanished = a_n2;
    wire a_undef    = ~a_n0 & a_top3;

    wire b_is_zero  = b_is_ambig & b_frac_zero;
    wire b_is_inf   = b_is_ambig & b_frac_neg1;
    wire b_exploded = b_is_ambig & b_n1;
    wire b_vanished = b_n2;
    wire b_undef    = ~b_n0 & b_top3;

    wire a_is_normal = ~a_is_ambig & a_n1;
    wire b_is_normal = ~b_is_ambig & b_n1;
    wire a_is_neg = a_frac[63];
    wire b_is_neg = b_frac[63];

    // ========== Exponent de-scaling ==========
    wire signed [MAX_EXP-1:0] a_exp_int =
        (exp_width == 2'd0) ? {{56{a_exp[63]}}, a_exp[63:56]} :
        (exp_width == 2'd1) ? {{48{a_exp[63]}}, a_exp[63:48]} :
        (exp_width == 2'd2) ? {{32{a_exp[63]}}, a_exp[63:32]} :
                              a_exp;

    wire signed [MAX_EXP-1:0] b_exp_int =
        (exp_width == 2'd0) ? {{56{b_exp[63]}}, b_exp[63:56]} :
        (exp_width == 2'd1) ? {{48{b_exp[63]}}, b_exp[63:48]} :
        (exp_width == 2'd2) ? {{32{b_exp[63]}}, b_exp[63:32]} :
                              b_exp;

    wire signed [MAX_EXP-1:0] exp_one_msb =
        (exp_width == 2'd0) ? {8'd1, 56'b0} :
        (exp_width == 2'd1) ? {16'd1, 48'b0} :
        (exp_width == 2'd2) ? {32'd1, 32'b0} :
                              64'd1;

    // ========== AND/OR/XOR edge case shortcut ==========
    wire and_sc_esc_esc = (a_exploded & b_exploded) | (a_vanished & b_vanished) |
                          a_is_inf | b_is_inf;
    wire and_sc_any_zero = ~and_sc_esc_esc & (a_is_zero | b_is_zero);
    wire and_sc_a_van = ~and_sc_esc_esc & ~and_sc_any_zero & a_vanished;
    wire and_sc_b_van = ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & b_vanished;
    wire and_sc_a_norm_b_esc = ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & ~and_sc_b_van & a_is_normal & ~b_is_normal;
    wire and_sc_b_norm_a_esc = ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & ~and_sc_b_van & ~and_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire or_sc_esc = a_is_inf | b_is_inf | (a_exploded & b_exploded);
    wire or_sc_a_zero = ~or_sc_esc & a_is_zero;
    wire or_sc_b_zero = ~or_sc_esc & ~or_sc_a_zero & b_is_zero;
    wire or_sc_a_exp  = ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & a_exploded & ~b_exploded;
    wire or_sc_b_exp  = ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & b_exploded;
    wire or_sc_a_norm_b_esc = ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & ~or_sc_b_exp & a_is_normal & ~b_is_normal;
    wire or_sc_b_norm_a_esc = ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & ~or_sc_b_exp & ~or_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire xor_sc_esc = a_is_inf | b_is_inf | (a_exploded & b_exploded) | (a_vanished & b_vanished);
    wire xor_sc_a_zero = ~xor_sc_esc & a_is_zero;
    wire xor_sc_b_zero = ~xor_sc_esc & ~xor_sc_a_zero & b_is_zero;
    wire xor_sc_a_van  = ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & a_vanished;
    wire xor_sc_b_van  = ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & b_vanished;
    wire xor_sc_a_norm_b_esc = ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & ~xor_sc_b_van & a_is_normal & ~b_is_normal;
    wire xor_sc_b_norm_a_esc = ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & ~xor_sc_b_van & ~xor_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire signed [MAX_FRAC-1:0] not_a_frac = a_frac ^ frac_neg1_val;
    wire signed [MAX_FRAC-1:0] not_b_frac = b_frac ^ frac_neg1_val;

    // Bitwise shortcut output mux
    reg sc_bitwise;
    reg signed [MAX_FRAC-1:0] sc_bw_frac;
    reg signed [MAX_EXP-1:0]  sc_bw_exp;

    always @(*) begin
        sc_bitwise = 1'b0;
        sc_bw_frac = {MAX_FRAC{1'b0}};
        sc_bw_exp  = AMBIG_EXP;

        if (a_undef) begin
            sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_exp = a_exp;
        end else if (b_undef) begin
            sc_bitwise = 1'b1; sc_bw_frac = b_frac; sc_bw_exp = b_exp;
        end else if (op == OP_AND) begin
            if (and_sc_esc_esc) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_AND; sc_bw_exp = AMBIG_EXP;
            end else if (and_sc_any_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_exp = AMBIG_EXP;
            end else if (and_sc_a_van) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_exp = AMBIG_EXP; end
            end else if (and_sc_b_van) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_exp = AMBIG_EXP; end
            end else if (and_sc_a_norm_b_esc) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_exp = AMBIG_EXP; end
            end else if (and_sc_b_norm_a_esc) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_exp = AMBIG_EXP; end
            end else if (~a_is_normal | ~b_is_normal) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_AND; sc_bw_exp = AMBIG_EXP;
            end
        end else if (op == OP_OR) begin
            if (or_sc_esc) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_OR; sc_bw_exp = AMBIG_EXP;
            end else if (or_sc_a_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = b_frac; sc_bw_exp = b_exp;
            end else if (or_sc_b_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_exp = a_exp;
            end else if (or_sc_a_exp) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
                else          begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
            end else if (or_sc_b_exp) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
                else          begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
            end else if (or_sc_a_norm_b_esc) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
                else          begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
            end else if (or_sc_b_norm_a_esc) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
                else          begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
            end else if (~a_is_normal | ~b_is_normal) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_OR; sc_bw_exp = AMBIG_EXP;
            end
        end else if (op == OP_XOR) begin
            if (xor_sc_esc) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_XOR; sc_bw_exp = AMBIG_EXP;
            end else if (xor_sc_a_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = b_frac; sc_bw_exp = b_exp;
            end else if (xor_sc_b_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_exp = a_exp;
            end else if (xor_sc_a_van) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = not_b_frac; sc_bw_exp = b_exp; end
                else          begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
            end else if (xor_sc_b_van) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = not_a_frac; sc_bw_exp = a_exp; end
                else          begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
            end else if (xor_sc_a_norm_b_esc) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = not_b_frac; sc_bw_exp = b_exp; end
                else          begin sc_bw_frac = b_frac; sc_bw_exp = b_exp; end
            end else if (xor_sc_b_norm_a_esc) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = not_a_frac; sc_bw_exp = a_exp; end
                else          begin sc_bw_frac = a_frac; sc_bw_exp = a_exp; end
            end else if (~a_is_normal | ~b_is_normal) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_XOR; sc_bw_exp = AMBIG_EXP;
            end
        end
    end

    // ========== NOT ==========
    wire signed [MAX_FRAC-1:0] not_frac = a_undef ? a_frac : not_a_frac;
    wire signed [MAX_EXP-1:0]  not_exp  = a_exp;

    // ========== SHL/SHR ==========
    wire signed [MAX_EXP-1:0] shl_new_exp = a_exp + b_exp;
    wire signed [MAX_EXP-1:0] shr_new_exp = a_exp - b_exp;

    wire shl_ovf = ~b_exp[63] & ~a_exp[63] & shl_new_exp[63] & (shl_new_exp != AMBIG_EXP);
    wire shl_udf = b_exp[63] & a_exp[63] & ~(shl_new_exp[63]) & ~(shl_new_exp == {MAX_EXP{1'b0}});
    wire shr_ovf = b_exp[63] & ~a_exp[63] & shr_new_exp[63] & (shr_new_exp != AMBIG_EXP);
    wire shr_udf = ~b_exp[63] & a_exp[63] & ~(shr_new_exp[63]) & ~(shr_new_exp == {MAX_EXP{1'b0}});

    wire signed [MAX_FRAC-1:0] a_vanished_frac = {a_frac[MAX_FRAC-1], a_frac[MAX_FRAC-1:1]};

    wire sc_shl = (op == OP_SHL);

    wire signed [MAX_FRAC-1:0] shift_frac =
        ~a_is_normal ? a_frac :
        (sc_shl & shl_ovf) ? a_frac :
        (sc_shl & shl_udf) ? a_vanished_frac :
        (~sc_shl & shr_ovf) ? a_frac :
        (~sc_shl & shr_udf) ? a_vanished_frac :
                               a_frac;

    wire signed [MAX_EXP-1:0] shift_exp =
        ~a_is_normal ? a_exp :
        (sc_shl & shl_ovf) ? AMBIG_EXP :
        (sc_shl & shl_udf) ? AMBIG_EXP :
        (~sc_shl & shr_ovf) ? AMBIG_EXP :
        (~sc_shl & shr_udf) ? AMBIG_EXP :
        sc_shl ? shl_new_exp :
                 shr_new_exp;

    // ========== NEGATE ==========
    wire a_is_pos_half  = (a_frac == POS_HALF);
    wire a_is_neg_one   = (a_frac == NEG_ONE);
    wire a_is_pos_small = (a_frac == POS_SMALL);
    wire a_is_neg_small = (a_frac == NEG_SMALL);

    wire signed [MAX_EXP-1:0] neg_exp_m1 = a_exp - exp_one_msb;
    wire neg_exp_m1_ambig = (neg_exp_m1 == AMBIG_EXP);

    wire a_nonnorm_nochange = a_n0 | (~a_n0 & a_top3);

    wire signed [MAX_FRAC-1:0] neg_frac_nonnorm =
        a_nonnorm_nochange ? a_frac :
        a_is_pos_half  ? NEG_ONE :
        a_is_neg_one   ? POS_HALF :
        a_is_pos_small ? NEG_SMALL :
        a_is_neg_small ? POS_SMALL :
                         -a_frac;

    wire signed [MAX_FRAC-1:0] neg_frac_normal =
        a_is_pos_half ? (neg_exp_m1_ambig ? NEG_SMALL : NEG_ONE) :
        a_is_neg_one  ? POS_HALF :
                        -a_frac;

    wire signed [MAX_EXP-1:0] neg_exp_normal =
        a_is_pos_half ? neg_exp_m1 :
        a_is_neg_one  ? (a_exp + exp_one_msb) :
                        a_exp;

    wire signed [MAX_FRAC-1:0] negate_frac = ~a_is_normal ? neg_frac_nonnorm : neg_frac_normal;
    wire signed [MAX_EXP-1:0]  negate_exp  = ~a_is_normal ? a_exp            : neg_exp_normal;

    // ========== MAGNITUDE ==========
    wire signed [MAX_FRAC-1:0] mag_frac = a_is_neg ? negate_frac : a_frac;
    wire signed [MAX_EXP-1:0]  mag_exp  = a_is_neg ? negate_exp  : a_exp;

    // ========== SIGN ==========
    wire signed [MAX_FRAC-1:0] sign_frac =
        a_undef ? a_frac :
        a_n0    ? UNDEF_SIGN :
                  (a_is_neg ? NEG_ONE : POS_HALF);
    wire signed [MAX_EXP-1:0] sign_exp =
        a_undef ? a_exp :
        a_n0    ? AMBIG_EXP :
                  exp_one_msb;

    // ========== FLOOR ==========
    wire signed [MAX_EXP-1:0] floor_exp_int = a_exp_int;
    wire floor_exp_le_zero = floor_exp_int[MAX_EXP-1] | (floor_exp_int == 0);
    wire [6:0] frac_bits_m1 = frac_bits - 1'b1;
    wire floor_exp_ge_width = ($signed(floor_exp_int) >= $signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}));

    wire [6:0] actual_mask_shift = 7'd63 - floor_exp_int[6:0];
    wire signed [MAX_FRAC-1:0] floor_mask = ({MAX_FRAC{1'b1}} << actual_mask_shift);
    wire signed [MAX_FRAC-1:0] floor_masked = a_frac & floor_mask;

    wire signed [MAX_FRAC-1:0] floor_frac =
        (~a_is_normal & a_vanished) ? (a_is_neg ? NEG_ONE : {MAX_FRAC{1'b0}}) :
        ~a_is_normal                ? a_frac :
        floor_exp_le_zero           ? (a_is_neg ? NEG_ONE : {MAX_FRAC{1'b0}}) :
        floor_exp_ge_width          ? a_frac :
                                      floor_masked;

    wire signed [MAX_EXP-1:0] floor_exp =
        (~a_is_normal & a_vanished) ? (a_is_neg ? exp_one_msb : AMBIG_EXP) :
        ~a_is_normal                ? a_exp :
        floor_exp_le_zero           ? (a_is_neg ? exp_one_msb : AMBIG_EXP) :
        floor_exp_ge_width          ? a_exp :
                                      a_exp;

    // ========== CMP ==========
    reg comb_cmp_lt, comb_cmp_eq, comb_cmp_gt, comb_cmp_unord;

    always @(*) begin
        comb_cmp_lt = 1'b0; comb_cmp_eq = 1'b0; comb_cmp_gt = 1'b0; comb_cmp_unord = 1'b0;

        if (a_is_normal & b_is_normal) begin
            if (a_is_neg == b_is_neg) begin
                if (a_exp != b_exp) begin
                    if (a_is_neg) begin
                        if ($signed(a_exp_int) > $signed(b_exp_int)) comb_cmp_lt = 1'b1;
                        else comb_cmp_gt = 1'b1;
                    end else begin
                        if ($signed(a_exp_int) > $signed(b_exp_int)) comb_cmp_gt = 1'b1;
                        else comb_cmp_lt = 1'b1;
                    end
                end else begin
                    if ($signed(a_frac) > $signed(b_frac)) comb_cmp_gt = 1'b1;
                    else if ($signed(a_frac) < $signed(b_frac)) comb_cmp_lt = 1'b1;
                    else comb_cmp_eq = 1'b1;
                end
            end else begin
                if (a_is_neg) comb_cmp_lt = 1'b1;
                else comb_cmp_gt = 1'b1;
            end
        end else if (a_undef | a_is_inf | b_undef | b_is_inf) begin
            comb_cmp_unord = 1'b1;
        end else if (a_is_zero & b_is_zero) begin
            comb_cmp_eq = 1'b1;
        end else if (a_is_zero) begin
            if (b_is_neg) comb_cmp_gt = 1'b1;
            else comb_cmp_lt = 1'b1;
        end else if (b_is_zero) begin
            if (a_is_neg) comb_cmp_lt = 1'b1;
            else comb_cmp_gt = 1'b1;
        end else if ((a_exploded & b_exploded) | (a_vanished & b_vanished)) begin
            if (a_is_neg == b_is_neg) comb_cmp_unord = 1'b1;
            else if (a_is_neg) comb_cmp_lt = 1'b1;
            else comb_cmp_gt = 1'b1;
        end else if (a_vanished) begin
            if (b_is_neg) comb_cmp_gt = 1'b1;
            else comb_cmp_lt = 1'b1;
        end else if (b_vanished) begin
            if (a_is_neg) comb_cmp_lt = 1'b1;
            else comb_cmp_gt = 1'b1;
        end else if (a_exploded) begin
            if (a_is_neg) comb_cmp_lt = 1'b1;
            else comb_cmp_gt = 1'b1;
        end else begin
            if (b_is_neg) comb_cmp_gt = 1'b1;
            else comb_cmp_lt = 1'b1;
        end
    end

    // ========== AND/OR/XOR datapath ==========
    wire signed [MAX_EXP:0] raw_diff = $signed({a_exp_int[MAX_EXP-1], a_exp_int})
                                      - $signed({b_exp_int[MAX_EXP-1], b_exp_int});
    wire a_is_big = !raw_diff[MAX_EXP];

    wire signed [MAX_FRAC-1:0] big_frac     = a_is_big ? a_frac : b_frac;
    wire signed [MAX_EXP-1:0]  big_exp_int  = a_is_big ? a_exp_int : b_exp_int;
    wire signed [MAX_FRAC-1:0] small_frac   = a_is_big ? b_frac : a_frac;
    wire signed [MAX_EXP-1:0]  small_exp_int = a_is_big ? b_exp_int : a_exp_int;

    wire signed [MAX_EXP:0] exp_diff_signed = raw_diff[MAX_EXP] ? -raw_diff : raw_diff;
    wire [SHIFT_BITS-1:0] exp_diff = exp_diff_signed[SHIFT_BITS-1:0];

    wire negligible = (exp_diff_signed >= $signed({1'b0, frac_bits}));
    wire small_is_neg = small_frac[MAX_FRAC-1];

    wire bw_negligible_shortcut = is_bitwise & negligible;
    reg signed [MAX_FRAC-1:0] bw_negl_frac;
    reg signed [MAX_EXP-1:0]  bw_negl_exp;
    always @(*) begin
        bw_negl_frac = {MAX_FRAC{1'b0}};
        bw_negl_exp  = AMBIG_EXP;
        if (op == OP_AND) begin
            if (small_is_neg) begin bw_negl_frac = big_frac; bw_negl_exp = a_is_big ? a_exp : b_exp; end
            else              begin bw_negl_frac = {MAX_FRAC{1'b0}}; bw_negl_exp = AMBIG_EXP; end
        end else if (op == OP_OR) begin
            if (small_is_neg) begin bw_negl_frac = small_frac; bw_negl_exp = a_is_big ? b_exp : a_exp; end
            else              begin bw_negl_frac = big_frac; bw_negl_exp = a_is_big ? a_exp : b_exp; end
        end else begin
            if (small_is_neg) begin
                bw_negl_frac = big_frac ^ frac_neg1_val;
                bw_negl_exp = a_is_big ? a_exp : b_exp;
            end else begin
                bw_negl_frac = big_frac; bw_negl_exp = a_is_big ? a_exp : b_exp;
            end
        end
    end

    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac);
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac);

    wire [SHIFT_BITS-1:0] shift_amt = negligible ? (frac_bits[SHIFT_BITS-1:0] - 1'b1) : exp_diff;
    wire signed [INT_BITS-1:0] big_shifted = big_ext <<< shift_amt;

    wire signed [INT_BITS-1:0] bw_result =
        (op == OP_AND) ? (big_shifted & small_ext) :
        (op == OP_OR)  ? (big_shifted | small_ext) :
                         (big_shifted ^ small_ext);
    wire bw_is_zero = (bw_result == 0);

    wire big_exp_neg = big_exp_int[MAX_EXP-1];

    // ========== CLZ + Normalize ==========
    wire [INT_BITS-2:0] xor_bits = bw_result[INT_BITS-1:1] ^ bw_result[INT_BITS-2:0];

    reg [SHIFT_BITS-1:0] leading_m1;
    integer ci;
    always @(*) begin
        leading_m1 = INT_BITS - 1;
        for (ci = 0; ci < INT_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) leading_m1 = (INT_BITS - 2) - ci[SHIFT_BITS-1:0];
    end

    wire signed [INT_BITS-1:0] normalized = bw_result <<< leading_m1;
    wire signed [MAX_FRAC-1:0] out_frac = normalized[INT_BITS-1 -: MAX_FRAC];

    localparam ECW = MAX_EXP + 2;
    wire signed [ECW-1:0] small_exp_w = $signed({{(ECW-MAX_EXP){small_exp_int[MAX_EXP-1]}}, small_exp_int});

    wire signed [ECW-1:0] exp_calc =
        small_exp_w
        + MAX_FRAC
        - $signed({{(ECW-SHIFT_BITS){1'b0}}, leading_m1});

    wire signed [MAX_EXP-1:0] out_exp_int = exp_calc[MAX_EXP-1:0];

    wire signed [MAX_EXP-1:0] out_exp_msb =
        (exp_width == 2'd0) ? out_exp_int <<< 56 :
        (exp_width == 2'd1) ? out_exp_int <<< 48 :
        (exp_width == 2'd2) ? out_exp_int <<< 32 :
                              out_exp_int;

    wire signed [ECW-1:0] max_exp_val =
        (exp_width == 2'd0) ? 127 :
        (exp_width == 2'd1) ? 32767 :
        (exp_width == 2'd2) ? 2147483647 :
                              {{(ECW-MAX_EXP){1'b0}}, {(MAX_EXP-1){1'b1}}};

    wire signed [ECW-1:0] min_exp_val =
        (exp_width == 2'd0) ? -127 :
        (exp_width == 2'd1) ? -32767 :
        (exp_width == 2'd2) ? -2147483647 :
                              {{(ECW-MAX_EXP+1){1'b1}}, {(MAX_EXP-2){1'b0}}, 1'b1};

    wire overflow  = (exp_calc > max_exp_val);
    wire underflow = (exp_calc < min_exp_val);

    wire signed [MAX_FRAC-1:0] vanished_frac = {out_frac[MAX_FRAC-1], out_frac[MAX_FRAC-1:1]};
    wire bw_exp_overflow = big_exp_neg & ~out_exp_int[MAX_EXP-1] & (out_exp_int != 0);

    // ========== Combinational output mux ==========
    reg signed [MAX_FRAC-1:0] comb_frac;
    reg signed [MAX_EXP-1:0]  comb_exp;

    always @(*) begin
        comb_frac = {MAX_FRAC{1'b0}};
        comb_exp  = AMBIG_EXP;

        case (op)
            OP_AND, OP_OR, OP_XOR: begin
                if (sc_bitwise) begin
                    comb_frac = sc_bw_frac;
                    comb_exp  = sc_bw_exp;
                end else if (bw_negligible_shortcut) begin
                    comb_frac = bw_negl_frac;
                    comb_exp  = bw_negl_exp;
                end else if (bw_is_zero) begin
                    comb_frac = {MAX_FRAC{1'b0}};
                    comb_exp  = AMBIG_EXP;
                end else if (overflow | bw_exp_overflow) begin
                    comb_frac = out_frac;
                    comb_exp  = AMBIG_EXP;
                end else if (underflow) begin
                    comb_frac = vanished_frac;
                    comb_exp  = AMBIG_EXP;
                end else begin
                    comb_frac = out_frac;
                    comb_exp  = out_exp_msb;
                end
            end
            OP_NOT: begin
                comb_frac = not_frac;
                comb_exp  = not_exp;
            end
            OP_SHL, OP_SHR: begin
                comb_frac = shift_frac;
                comb_exp  = shift_exp;
            end
            OP_CMP: begin
                comb_frac = {MAX_FRAC{1'b0}};
                comb_exp  = AMBIG_EXP;
            end
            OP_NEG: begin
                comb_frac = negate_frac;
                comb_exp  = negate_exp;
            end
            OP_MAG: begin
                comb_frac = mag_frac;
                comb_exp  = mag_exp;
            end
            OP_SIGN: begin
                comb_frac = sign_frac;
                comb_exp  = sign_exp;
            end
            OP_FLOOR: begin
                comb_frac = floor_frac;
                comb_exp  = floor_exp;
            end
            default: begin
                comb_frac = a_frac;
                comb_exp  = a_exp;
            end
        endcase
    end

    // ========== Output register ==========
    always @(posedge clk) begin
        if (ce) begin
            result_frac <= comb_frac;
            result_exp  <= comb_exp;
            cmp_lt      <= comb_cmp_lt;
            cmp_eq      <= comb_cmp_eq;
            cmp_gt      <= comb_cmp_gt;
            cmp_unord   <= comb_cmp_unord;
        end
    end

endmodule
