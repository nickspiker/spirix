// spirix_alu_unified — 2-stage pipelined unified ALU
//
// Merges add/sub + bitwise (AND/OR/XOR) into one shared barrel+CLZ datapath.
// Trivial ops (NOT/SHL/SHR/CMP/NEG/MAG/SIGN) bypass the barrel entirely.
// Multiply uses the same Stage 2 CLZ+normalize (product is 2×FRAC wide).
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Physical datapath is MAX_FRAC=64 bits. ALL inputs/outputs are MSB-aligned
// (Spirix sa() convention). ALL exponent arithmetic stays MSB-aligned
// throughout — no de-scaling or re-scaling shifts.
//
// Floor-only (no rounding). Full edge case handling.
//
// Stage 1: edge case detect + barrel align + op (add/sub/and/or/xor/mul)
// Stage 2: CLZ + normalize + exponent calc + output mux
//
// Early-tap barrel: F3E3 exits after 3 stages, F4 after 4, F5 after 5, F6 after 6.
// Early-tap CLZ: same — narrow widths resolve in fewer LUT levels.
// CLZ result is pre-set per width (clz_16 ∈ [0,15], clz_32 ∈ [0,31], etc.)
// so frac_bits - leading_m1 gives the correct exponent offset directly.
// No post-adjustment needed — the width is already baked into the CLZ tap.
//
// op[3:0]:
//   0=ADD  1=SUB  2=MUL  3=AND  4=OR  5=XOR
//   6=NOT  7=SHL  8=SHR  9=CMP  10=NEG  11=MAG  12=SIGN

module spirix_alu_unified #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                       clk,
    input  wire                       ce,
    input  wire [3:0]                 op,
    input  wire [1:0]                 frac_width, // 00=8 01=16 10=32 11=64
    input  wire [1:0]                 exp_width,  // 00=8 01=16 10=32 11=64

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

    // ========== Op decode ==========
    localparam OP_ADD  = 4'd0;
    localparam OP_SUB  = 4'd1;
    localparam OP_MUL  = 4'd2;
    localparam OP_AND  = 4'd3;
    localparam OP_OR   = 4'd4;
    localparam OP_XOR  = 4'd5;
    localparam OP_NOT  = 4'd6;
    localparam OP_SHL  = 4'd7;
    localparam OP_SHR  = 4'd8;
    localparam OP_CMP  = 4'd9;
    localparam OP_NEG  = 4'd10;
    localparam OP_MAG  = 4'd11;
    localparam OP_SIGN = 4'd12;

    localparam INT_BITS   = 2 * MAX_FRAC;  // 128: working width for barrel ops
    localparam SHIFT_BITS = $clog2(INT_BITS); // 7

    wire is_add    = (op == OP_ADD);
    wire is_sub    = (op == OP_SUB);
    wire is_mul    = (op == OP_MUL);
    wire is_addsub = is_add | is_sub;
    wire is_bitwise = (op == OP_AND) | (op == OP_OR) | (op == OP_XOR);
    wire is_barrel  = is_addsub | is_bitwise;  // ops that use barrel+CLZ
    wire is_trivial = (op >= OP_NOT);           // bypass barrel entirely

    // ========== Width decode ==========
    wire [6:0] frac_bits = (frac_width == 2'd0) ? 7'd8  :
                           (frac_width == 2'd1) ? 7'd16 :
                           (frac_width == 2'd2) ? 7'd32 : 7'd64;

    wire [6:0] exp_bits = (exp_width == 2'd0) ? 7'd8  :
                          (exp_width == 2'd1) ? 7'd16 :
                          (exp_width == 2'd2) ? 7'd32 : 7'd64;

    // ========== MSB-aligned constants ==========
    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {2'b01, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_SMALL = {3'b001, {(MAX_FRAC-3){1'b0}}};
    localparam signed [MAX_FRAC-1:0] NEG_SMALL = {2'b11, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // exp_one_msb: the value "1" in MSB-aligned exponent space
    wire signed [MAX_EXP-1:0] exp_one_msb =
        (exp_width == 2'd0) ? {8'd1, 56'b0} :
        (exp_width == 2'd1) ? {16'd1, 48'b0} :
        (exp_width == 2'd2) ? {32'd1, 32'b0} :
                              64'd1;

    // ========== State detection ==========
    // MSB-aligned: top 3 bits always at [63:61] regardless of width.
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
    wire a_transf   = a_is_inf | a_exploded;
    wire a_vanished = a_n2;
    wire a_undef    = ~a_n0 & a_top3;

    wire b_is_zero  = b_is_ambig & b_frac_zero;
    wire b_is_inf   = b_is_ambig & b_frac_neg1;
    wire b_exploded = b_is_ambig & b_n1;
    wire b_transf   = b_is_inf | b_exploded;
    wire b_vanished = b_n2;
    wire b_undef    = ~b_n0 & b_top3;

    // Rust is_normal() = exponent != AMBIG (no N1 check).
    // N2/non-N0 fractions at non-AMBIG exp go through normal arithmetic.
    wire a_is_normal = ~a_is_ambig;
    wire b_is_normal = ~b_is_ambig;
    wire a_is_neg    = a_frac[63];
    wire b_is_neg    = b_frac[63];
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    wire signed [MAX_FRAC-1:0] not_a_frac = a_frac ^ frac_neg1_val;
    wire signed [MAX_FRAC-1:0] not_b_frac = b_frac ^ frac_neg1_val;

    // ========== Undefined prefix constants ==========
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_P_TF   = {8'h1F, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_M_TF   = {8'hE0, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_VAN_P_VAN  = {8'h1E, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_VAN_M_VAN  = {8'hE1, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_P_FIN   = {8'h1C, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_M_FIN   = {8'hE3, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_FIN_P_TF   = {8'h18, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_FIN_M_TF   = {8'hE7, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_MUL_NEG = {8'hEF, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_NEG_MUL_TF = {8'h10, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_AND        = {8'h12, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_OR         = {8'hED, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_XOR        = {8'h11, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_SIGN       = {8'hE4, 56'b0};

    // ================================================================
    //  EDGE CASE SHORTCUTS — shared undef, then per-op chains
    // ================================================================

    // Shortcuts only fire when at least one input is non-normal (AMBIG exp).
    // Rust is_normal() = exp != AMBIG, so normal N2/undef fractions go thru arithmetic.
    // Inside the chain, vanished()/undef() check fraction only (matching Rust).
    wire sc_a_undef = any_non_normal & a_undef;
    wire sc_b_undef = any_non_normal & ~a_undef & b_undef;

    // --- Add/sub shortcuts (only when any_non_normal) ---
    // Inside the chain, vanished()/undef() check fraction only (matching Rust).
    // But a_undef only matters when a has AMBIG exp; at non-AMBIG, undef fracs
    // go through normal arithmetic even if the OTHER input triggered the chain.
    // However, Rust checks is_undefined() on the fraction alone in the chain.
    wire sc_add_tf_tf    = is_addsub & any_non_normal & ~a_undef & ~b_undef & a_transf & b_transf;
    wire sc_add_van_van  = is_addsub & any_non_normal & ~a_undef & ~b_undef & ~sc_add_tf_tf & a_vanished & b_vanished;
    wire sc_add_a_transf = is_addsub & any_non_normal & ~a_undef & ~b_undef & ~sc_add_tf_tf & ~sc_add_van_van & a_transf;
    wire sc_add_b_transf = is_addsub & any_non_normal & ~a_undef & ~b_undef & ~sc_add_tf_tf & ~sc_add_van_van & ~sc_add_a_transf & b_transf;
    wire sc_add_a_van    = is_addsub & any_non_normal & ~a_undef & ~b_undef & ~sc_add_tf_tf & ~sc_add_van_van &
                           ~sc_add_a_transf & ~sc_add_b_transf & a_vanished;
    wire sc_add_b_van    = is_addsub & any_non_normal & ~a_undef & ~b_undef & ~sc_add_tf_tf & ~sc_add_van_van &
                           ~sc_add_a_transf & ~sc_add_b_transf & ~sc_add_a_van & b_vanished;
    wire sc_add_a_zero   = is_addsub & any_non_normal & ~a_undef & ~b_undef & ~sc_add_tf_tf & ~sc_add_van_van &
                           ~sc_add_a_transf & ~sc_add_b_transf & ~sc_add_a_van & ~sc_add_b_van & a_is_zero;
    wire sc_add_fallback = is_addsub & any_non_normal & ~sc_a_undef & ~sc_b_undef & ~sc_add_tf_tf &
                           ~sc_add_van_van & ~sc_add_a_transf & ~sc_add_b_transf &
                           ~sc_add_a_van & ~sc_add_b_van & ~sc_add_a_zero;

    // --- Mul shortcuts (only when any_non_normal) ---
    wire sc_mul_inf_zero = is_mul & any_non_normal & ~a_undef & ~b_undef & ((a_is_inf & b_is_zero) | (a_is_zero & b_is_inf));
    wire sc_mul_any_zero = is_mul & any_non_normal & ~a_undef & ~b_undef & ~sc_mul_inf_zero & (a_is_zero | b_is_zero);
    wire sc_mul_exp_van  = is_mul & any_non_normal & ~a_undef & ~b_undef & ~sc_mul_inf_zero & ~sc_mul_any_zero &
                           ((a_exploded & b_vanished) | (a_vanished & b_exploded));

    wire mul_shortcut = sc_a_undef | sc_b_undef | sc_mul_inf_zero | sc_mul_any_zero | sc_mul_exp_van;
    wire add_shortcut = sc_a_undef | sc_b_undef | sc_add_tf_tf | sc_add_van_van |
                        sc_add_a_transf | sc_add_b_transf | sc_add_a_van | sc_add_b_van |
                        sc_add_a_zero | sc_add_fallback;

    wire mul_abnormal = is_mul & any_non_normal & ~mul_shortcut;
    wire mul_n_level_neg1 = a_exploded | b_exploded;

    // --- AND/OR/XOR edge case shortcuts (only when any_non_normal) ---
    // Rust bitwise: edge cases gate on !self.is_normal() || !other.is_normal()
    // Inside the chain, vanished()/exploded() check by fraction only in Rust.
    wire and_sc_esc_esc = any_non_normal & ((a_exploded & b_exploded) | (a_vanished & b_vanished) |
                          a_is_inf | b_is_inf);
    wire and_sc_any_zero = any_non_normal & ~and_sc_esc_esc & (a_is_zero | b_is_zero);
    wire and_sc_a_van = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & a_vanished;
    wire and_sc_b_van = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & b_vanished;
    wire and_sc_a_norm_b_esc = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & ~and_sc_b_van & a_is_normal & ~b_is_normal;
    wire and_sc_b_norm_a_esc = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & ~and_sc_b_van & ~and_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire or_sc_esc = any_non_normal & (a_is_inf | b_is_inf | (a_exploded & b_exploded));
    wire or_sc_a_zero = any_non_normal & ~or_sc_esc & a_is_zero;
    wire or_sc_b_zero = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & b_is_zero;
    wire or_sc_a_exp  = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & a_exploded & ~b_exploded;
    wire or_sc_b_exp  = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & b_exploded;
    wire or_sc_a_norm_b_esc = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & ~or_sc_b_exp & a_is_normal & ~b_is_normal;
    wire or_sc_b_norm_a_esc = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & ~or_sc_b_exp & ~or_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire xor_sc_esc = any_non_normal & (a_is_inf | b_is_inf | (a_exploded & b_exploded) | (a_vanished & b_vanished));
    wire xor_sc_a_zero = any_non_normal & ~xor_sc_esc & a_is_zero;
    wire xor_sc_b_zero = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & b_is_zero;
    wire xor_sc_a_van  = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & a_vanished;
    wire xor_sc_b_van  = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & b_vanished;
    wire xor_sc_a_norm_b_esc = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & ~xor_sc_b_van & a_is_normal & ~b_is_normal;
    wire xor_sc_b_norm_a_esc = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & ~xor_sc_b_van & ~xor_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    // Bitwise shortcut output mux
    reg sc_bitwise;
    reg signed [MAX_FRAC-1:0] sc_bw_frac;
    reg signed [MAX_EXP-1:0]  sc_bw_exp;

    always @(*) begin
        sc_bitwise = 1'b0;
        sc_bw_frac = {MAX_FRAC{1'b0}};
        sc_bw_exp  = AMBIG_EXP;

        if (is_bitwise && any_non_normal && a_undef) begin
            sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_exp = a_exp;
        end else if (is_bitwise && any_non_normal && b_undef) begin
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

    // --- Trivial op shortcuts ---
    // NOT/SHL/SHR/CMP/NEG/MAG/SIGN compute in Stage 1, bypass barrel entirely.

    wire bw_shortcut = sc_bitwise;

    wire shortcut = is_addsub  ? add_shortcut :
                    is_mul     ? mul_shortcut :
                    is_bitwise ? bw_shortcut :
                                 1'b0;  // trivial ops handle their own path

    // ================================================================
    //  NEGATION FOR SUB (MSB-aligned exp arithmetic)
    // ================================================================

    wire b_is_pos_half = (b_frac == POS_HALF);
    wire b_is_neg_one  = (b_frac == NEG_ONE);

    wire signed [MAX_EXP-1:0] b_exp_m1 = b_exp - exp_one_msb;
    wire b_exp_m1_ambig = (b_exp_m1 == AMBIG_EXP);

    wire b_is_pos_small = (b_frac == POS_SMALL);
    wire b_is_neg_small = (b_frac == NEG_SMALL);

    wire signed [MAX_FRAC-1:0] neg_b_frac_normal =
        b_is_pos_half ? (b_exp_m1_ambig ? NEG_SMALL : NEG_ONE) :
        b_is_neg_one  ? POS_HALF :
                        -b_frac;
    wire signed [MAX_EXP-1:0] neg_b_exp_normal =
        b_is_pos_half ? b_exp_m1 :
        b_is_neg_one  ? (b_exp + exp_one_msb) :
                        b_exp;

    wire b_nonnorm_nochange = b_n0 | (~b_n0 & b_top3);
    wire signed [MAX_FRAC-1:0] neg_b_frac_nonnorm =
        b_nonnorm_nochange ? b_frac :
        b_is_pos_half  ? NEG_ONE :
        b_is_neg_one   ? POS_HALF :
        b_is_pos_small ? NEG_SMALL :
        b_is_neg_small ? POS_SMALL :
                         -b_frac;

    wire signed [MAX_FRAC-1:0] neg_b_frac = b_is_ambig ? neg_b_frac_nonnorm : neg_b_frac_normal;
    wire signed [MAX_EXP-1:0]  neg_b_exp  = b_is_ambig ? b_exp              : neg_b_exp_normal;

    // ================================================================
    //  SHORTCUT OUTPUT MUX (all MSB-aligned, no re-scaling)
    // ================================================================

    wire signed [MAX_FRAC-1:0] sc_frac =
        sc_a_undef       ? a_frac :
        sc_b_undef       ? b_frac :
        // Add/sub shortcuts
        sc_add_tf_tf     ? (is_sub ? UNDEF_TF_M_TF  : UNDEF_TF_P_TF) :
        sc_add_van_van   ? (is_sub ? UNDEF_VAN_M_VAN : UNDEF_VAN_P_VAN) :
        sc_add_a_transf  ? (is_sub ? UNDEF_TF_M_FIN  : UNDEF_TF_P_FIN) :
        sc_add_b_transf  ? (is_sub ? UNDEF_FIN_M_TF  : UNDEF_FIN_P_TF) :
        sc_add_a_van     ? (is_sub ? neg_b_frac : b_frac) :
        sc_add_b_van     ? a_frac :
        sc_add_a_zero    ? (is_sub ? neg_b_frac : b_frac) :
        sc_add_fallback  ? a_frac :
        // Mul shortcuts
        sc_mul_inf_zero  ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        sc_mul_any_zero  ? {MAX_FRAC{1'b0}} :
        sc_mul_exp_van   ? (a_exploded ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        // Bitwise shortcuts
        sc_bitwise       ? sc_bw_frac :
                           {MAX_FRAC{1'b0}};

    wire signed [MAX_EXP-1:0] sc_exp =
        sc_a_undef      ? a_exp :
        sc_b_undef      ? b_exp :
        sc_add_a_van    ? (is_sub ? neg_b_exp : b_exp) :
        sc_add_b_van    ? a_exp :
        sc_add_a_zero   ? (is_sub ? neg_b_exp : b_exp) :
        sc_add_fallback ? a_exp :
        sc_bitwise      ? sc_bw_exp :
                          AMBIG_EXP;

    // ================================================================
    //  STAGE 1: SHARED BARREL ALIGN (add/sub/bitwise)
    // ================================================================

    // Exponent difference — stays MSB-aligned. De-scale only the shift amount.
    wire signed [MAX_EXP:0] raw_diff = $signed({a_exp[MAX_EXP-1], a_exp})
                                      - $signed({b_exp[MAX_EXP-1], b_exp});
    wire a_is_big = !raw_diff[MAX_EXP];

    wire signed [MAX_FRAC-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [MAX_EXP-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [MAX_FRAC-1:0] small_frac = a_is_big ? b_frac : a_frac;
    wire signed [MAX_EXP-1:0]  small_exp  = a_is_big ? b_exp  : a_exp;

    // De-scale ONLY the shift amount (MSB-aligned → integer)
    wire signed [MAX_EXP:0] abs_diff = raw_diff[MAX_EXP] ? -raw_diff : raw_diff;
    wire [6:0] exp_rshift = 7'd64 - exp_bits;
    wire signed [MAX_EXP:0] exp_diff_int = abs_diff >>> exp_rshift;
    wire [SHIFT_BITS-1:0] exp_diff = exp_diff_int[SHIFT_BITS-1:0];

    wire negligible = (exp_diff_int >= $signed({{(MAX_EXP+1-7){1'b0}}, frac_bits}));

    // Negate flags for subtraction
    wire negate_small = is_sub &  a_is_big;
    wire negate_big   = is_sub & !a_is_big;

    // Bypass paths (negligible exp_diff, no overlap)
    wire big_is_pos_half = (big_frac == POS_HALF);
    wire big_is_neg_one  = (big_frac == NEG_ONE);
    wire signed [MAX_FRAC-1:0] neg_big_frac = big_is_pos_half ? NEG_ONE  :
                                              big_is_neg_one  ? POS_HALF :
                                              -big_frac;
    wire signed [MAX_EXP-1:0]  neg_big_exp  = big_is_pos_half ? (big_exp - exp_one_msb) :
                                              big_is_neg_one  ? (big_exp + exp_one_msb) :
                                              big_exp;

    wire bypass_add = is_addsub & negligible & !negate_big;
    wire bypass_sub = is_addsub & negligible &  negate_big;

    // --- Bitwise negligible result mux ---
    wire small_is_neg = small_frac[MAX_FRAC-1];
    wire bw_negligible = is_bitwise & negligible;
    // big_undef: Rust !undef = undef (passthrough), so XOR negligible w/ neg small
    // must skip inversion when big is undefined
    wire big_undef = a_is_big ? a_undef : b_undef;

    reg signed [MAX_FRAC-1:0] bw_negl_frac;
    reg signed [MAX_EXP-1:0]  bw_negl_exp;
    always @(*) begin
        bw_negl_frac = {MAX_FRAC{1'b0}};
        bw_negl_exp  = AMBIG_EXP;
        if (op == OP_AND) begin
            if (small_is_neg) begin bw_negl_frac = big_frac; bw_negl_exp = big_exp; end
            else              begin bw_negl_frac = {MAX_FRAC{1'b0}}; bw_negl_exp = AMBIG_EXP; end
        end else if (op == OP_OR) begin
            if (small_is_neg) begin bw_negl_frac = small_frac; bw_negl_exp = small_exp; end
            else              begin bw_negl_frac = big_frac; bw_negl_exp = big_exp; end
        end else begin // XOR: negligible → !big, but !undef = undef
            if (small_is_neg & ~big_undef) begin bw_negl_frac = big_frac ^ frac_neg1_val; bw_negl_exp = big_exp; end
            else                           begin bw_negl_frac = big_frac; bw_negl_exp = big_exp; end
        end
    end

    // ================================================================
    //  SHARED BARREL SHIFTER — 7 stages with early taps
    // ================================================================

    // Pre-shift to place data at top of CLZ scan window.
    // Without this, 8-bit data at [63:56] sign-extends to [127:64]=sign, [63:0]=data.
    // CLZ for F3E3 scans [126:112] — far above the data. Pre-shift moves data up.
    // Multiply path doesn't use these (product naturally lands near [126:112]).
    wire signed [INT_BITS-1:0] big_ext =
        (frac_width == 2'd0) ? ($signed(big_frac) <<< 56) :
        (frac_width == 2'd1) ? ($signed(big_frac) <<< 48) :
        (frac_width == 2'd2) ? ($signed(big_frac) <<< 32) :
                                $signed(big_frac);
    wire signed [INT_BITS-1:0] small_ext =
        (frac_width == 2'd0) ? ($signed(small_frac) <<< 56) :
        (frac_width == 2'd1) ? ($signed(small_frac) <<< 48) :
        (frac_width == 2'd2) ? ($signed(small_frac) <<< 32) :
                                $signed(small_frac);

    wire [SHIFT_BITS-1:0] shift_amt = negligible ? (frac_bits[SHIFT_BITS-1:0] - 1'b1) : exp_diff;

    // Stage 0: shift by 1
    wire signed [INT_BITS-1:0] b0 = shift_amt[0] ? (big_ext <<< 1) : big_ext;
    // Stage 1: shift by 2
    wire signed [INT_BITS-1:0] b1 = shift_amt[1] ? (b0 <<< 2) : b0;
    // Stage 2: shift by 4 — F3E3 (8-bit frac, max shift 7) exits here
    wire signed [INT_BITS-1:0] b2 = shift_amt[2] ? (b1 <<< 4) : b1;
    // Stage 3: shift by 8 — F4E3 (16-bit frac, max shift 15) exits here
    wire signed [INT_BITS-1:0] b3 = shift_amt[3] ? (b2 <<< 8) : b2;
    // Stage 4: shift by 16 — F5E3 (32-bit frac, max shift 31) exits here
    wire signed [INT_BITS-1:0] b4 = shift_amt[4] ? (b3 <<< 16) : b3;
    // Stage 5: shift by 32 — F6E3 (64-bit frac, max shift 63) exits here
    wire signed [INT_BITS-1:0] b5 = shift_amt[5] ? (b4 <<< 32) : b4;
    // Stage 6: shift by 64 — full 128-bit (FMA alignment, future)
    wire signed [INT_BITS-1:0] b6 = shift_amt[6] ? (b5 <<< 64) : b5;

    // Early-tap mux: narrow widths exit earlier (fewer LUT levels on timing)
    wire signed [INT_BITS-1:0] barrel_out =
        (frac_width == 2'd0) ? b2 :  // 8-bit:  stages 0-2
        (frac_width == 2'd1) ? b3 :  // 16-bit: stages 0-3
        (frac_width == 2'd2) ? b4 :  // 32-bit: stages 0-4
                               b5;   // 64-bit: stages 0-5

    // ================================================================
    //  STAGE 1: OP SELECT — add/sub/and/or/xor in the same slot
    // ================================================================

    // Add/sub: shifted_big +/- small (two's complement trick)
    wire signed [INT_BITS-1:0] addsub_result =
        (barrel_out ^ {INT_BITS{negate_big}})
        + (small_ext ^ {INT_BITS{negate_small}})
        + {{(INT_BITS-1){1'b0}}, is_sub};

    // Bitwise: shifted_big OP small
    wire signed [INT_BITS-1:0] bitwise_result =
        (op == OP_AND) ? (barrel_out & small_ext) :
        (op == OP_OR)  ? (barrel_out | small_ext) :
                         (barrel_out ^ small_ext);  // XOR

    // Multiply: product of raw fractions (uses DSP, independent of barrel)
    wire signed [INT_BITS-1:0] mul_result = $signed(a_frac) * $signed(b_frac);

    // Select intermediate for Stage 2
    wire signed [INT_BITS-1:0] s1_intermediate =
        is_mul     ? mul_result :
        is_bitwise ? bitwise_result :
                     addsub_result;

    // Width-gated zero check: only test active 2×frac_bits window from MSB
    wire [INT_BITS-1:0] zero_mask =
        (frac_width == 2'd0) ? {{16{1'b1}}, {(INT_BITS-16){1'b0}}} :
        (frac_width == 2'd1) ? {{32{1'b1}}, {(INT_BITS-32){1'b0}}} :
        (frac_width == 2'd2) ? {{64{1'b1}}, {(INT_BITS-64){1'b0}}} :
                               {INT_BITS{1'b1}};
    wire s1_is_zero = ((s1_intermediate & $signed(zero_mask)) == 0);

    // ================================================================
    //  TRIVIAL OPS (bypass barrel, compute in Stage 1)
    // ================================================================

    // NOT
    wire signed [MAX_FRAC-1:0] not_frac = a_undef ? a_frac : not_a_frac;
    wire signed [MAX_EXP-1:0]  not_exp  = a_exp;

    // SHL/SHR (exponent arithmetic, MSB-aligned)
    wire signed [MAX_EXP-1:0] shl_new_exp = a_exp + b_exp;
    wire signed [MAX_EXP-1:0] shr_new_exp = a_exp - b_exp;

    // SHL/SHR overflow/underflow detection
    // Rust underflow: both-neg inputs + result wraps non-neg OR lands on AMBIG.
    // AMBIG case: (-128)-1=127 wrapping, which is non-negative in Rust's check.
    wire shl_ovf = ~b_exp[MAX_EXP-1] & ~a_exp[MAX_EXP-1] & shl_new_exp[MAX_EXP-1] & (shl_new_exp != AMBIG_EXP);
    wire shl_udf = b_exp[MAX_EXP-1] & a_exp[MAX_EXP-1] &
                   ((~shl_new_exp[MAX_EXP-1] & (shl_new_exp != {MAX_EXP{1'b0}})) | (shl_new_exp == AMBIG_EXP));
    wire shr_ovf = b_exp[MAX_EXP-1] & ~a_exp[MAX_EXP-1] & shr_new_exp[MAX_EXP-1] & (shr_new_exp != AMBIG_EXP);
    wire shr_udf = ~b_exp[MAX_EXP-1] & a_exp[MAX_EXP-1] &
                   ((~shr_new_exp[MAX_EXP-1] & (shr_new_exp != {MAX_EXP{1'b0}})) | (shr_new_exp == AMBIG_EXP));

    wire sc_shl = (op == OP_SHL);
    wire signed [MAX_FRAC-1:0] a_vanished_frac = {a_frac[MAX_FRAC-1], a_frac[MAX_FRAC-1:1]};

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

    // CMP — MSB-aligned signed comparison
    reg comb_cmp_lt, comb_cmp_eq, comb_cmp_gt, comb_cmp_unord;

    always @(*) begin
        comb_cmp_lt = 1'b0; comb_cmp_eq = 1'b0; comb_cmp_gt = 1'b0; comb_cmp_unord = 1'b0;

        if (a_is_normal & b_is_normal) begin
            if (a_is_neg == b_is_neg) begin
                if (a_exp != b_exp) begin
                    if (a_is_neg) begin
                        if ($signed(a_exp) > $signed(b_exp)) comb_cmp_lt = 1'b1;
                        else comb_cmp_gt = 1'b1;
                    end else begin
                        if ($signed(a_exp) > $signed(b_exp)) comb_cmp_gt = 1'b1;
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

    // NEG — full boundary handling
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

    // MAG — magnitude = |a|, reuse negate
    wire signed [MAX_FRAC-1:0] mag_frac = a_is_neg ? negate_frac : a_frac;
    wire signed [MAX_EXP-1:0]  mag_exp  = a_is_neg ? negate_exp  : a_exp;

    // SIGN — extract sign as ±0.5 or undef
    wire signed [MAX_FRAC-1:0] sign_frac =
        a_undef ? a_frac :
        a_n0    ? UNDEF_SIGN :
                  (a_is_neg ? NEG_ONE : POS_HALF);
    // sign(positive) = ONE = (POS_HALF, exp=1), sign(negative) = NEG_ONE = (NEG_ONE, exp=0)
    wire signed [MAX_EXP-1:0] sign_exp =
        a_undef ? a_exp :
        a_n0    ? AMBIG_EXP :
                  (a_is_neg ? {MAX_EXP{1'b0}} : exp_one_msb);

    // Trivial output mux
    wire signed [MAX_FRAC-1:0] trivial_frac =
        (op == OP_NOT)  ? not_frac :
        (op == OP_SHL || op == OP_SHR) ? shift_frac :
        (op == OP_NEG)  ? negate_frac :
        (op == OP_MAG)  ? mag_frac :
        (op == OP_SIGN) ? sign_frac :
                          {MAX_FRAC{1'b0}};  // CMP returns zero/ambig
    wire signed [MAX_EXP-1:0] trivial_exp =
        (op == OP_NOT)  ? not_exp :
        (op == OP_SHL || op == OP_SHR) ? shift_exp :
        (op == OP_NEG)  ? negate_exp :
        (op == OP_MAG)  ? mag_exp :
        (op == OP_SIGN) ? sign_exp :
                          AMBIG_EXP;  // CMP

    // ================================================================
    //  STAGE 1 → STAGE 2 PIPELINE REGISTER
    // ================================================================

    reg signed [INT_BITS-1:0] s2_intermediate;
    reg s2_is_zero;
    reg s2_shortcut;
    reg s2_is_trivial;
    reg signed [MAX_FRAC-1:0] s2_sc_frac;
    reg signed [MAX_EXP-1:0]  s2_sc_exp;
    reg signed [MAX_FRAC-1:0] s2_trivial_frac;
    reg signed [MAX_EXP-1:0]  s2_trivial_exp;
    reg [3:0]  s2_op;
    reg [6:0]  s2_frac_bits;
    reg [1:0]  s2_frac_width;
    reg [1:0]  s2_exp_width;
    reg signed [MAX_EXP-1:0]  s2_small_exp;    // MSB-aligned
    reg signed [MAX_EXP-1:0]  s2_a_exp;
    reg signed [MAX_EXP-1:0]  s2_b_exp;
    reg s2_bypass_add, s2_bypass_sub;
    reg signed [MAX_FRAC-1:0] s2_big_frac;
    reg signed [MAX_EXP-1:0]  s2_big_exp;
    reg signed [MAX_FRAC-1:0] s2_neg_big_frac;
    reg signed [MAX_EXP-1:0]  s2_neg_big_exp;
    reg s2_mul_abnormal;
    reg s2_mul_n_level_neg1;
    reg s2_bw_negligible;
    reg signed [MAX_FRAC-1:0] s2_bw_negl_frac;
    reg signed [MAX_EXP-1:0]  s2_bw_negl_exp;
    reg s2_cmp_lt, s2_cmp_eq, s2_cmp_gt, s2_cmp_unord;
    reg s2_big_exp_neg;

    always @(posedge clk) begin
        if (ce) begin
            s2_intermediate     <= s1_intermediate;
            s2_is_zero          <= s1_is_zero;
            s2_shortcut         <= shortcut;
            s2_is_trivial       <= is_trivial;
            s2_sc_frac          <= sc_frac;
            s2_sc_exp           <= sc_exp;
            s2_trivial_frac     <= trivial_frac;
            s2_trivial_exp      <= trivial_exp;
            s2_op               <= op;
            s2_frac_bits        <= frac_bits;
            s2_frac_width       <= frac_width;
            s2_exp_width        <= exp_width;
            s2_small_exp        <= small_exp;
            s2_a_exp            <= a_exp;
            s2_b_exp            <= b_exp;
            s2_bypass_add       <= bypass_add;
            s2_bypass_sub       <= bypass_sub;
            s2_big_frac         <= big_frac;
            s2_big_exp          <= big_exp;
            s2_neg_big_frac     <= neg_big_frac;
            s2_neg_big_exp      <= neg_big_exp;
            s2_mul_abnormal     <= mul_abnormal;
            s2_mul_n_level_neg1 <= mul_n_level_neg1;
            s2_bw_negligible    <= bw_negligible;
            s2_bw_negl_frac     <= bw_negl_frac;
            s2_bw_negl_exp      <= bw_negl_exp;
            s2_cmp_lt           <= comb_cmp_lt;
            s2_cmp_eq           <= comb_cmp_eq;
            s2_cmp_gt           <= comb_cmp_gt;
            s2_cmp_unord        <= comb_cmp_unord;
            s2_big_exp_neg      <= big_exp[MAX_EXP-1];
        end
    end

    // ================================================================
    //  STAGE 2: EARLY-TAP CLZ + NORMALIZE
    // ================================================================
    //
    // The CLZ scans for the first N1 transition (sign bit != next bit).
    // Each width tier scans only its active window. The CLZ result is
    // already "pre-set" for the width: clz_16 ∈ [0,15], clz_32 ∈ [0,31],
    // etc. So (frac_bits - leading_m1) gives the correct exponent offset
    // directly — no post-width-adjustment subtract needed.
    //
    // XOR adjacent bits to find transitions:
    wire [INT_BITS-2:0] xor_bits = s2_intermediate[INT_BITS-1:1]
                                 ^ s2_intermediate[INT_BITS-2:0];

    // 16-bit window CLZ (F3E3: 2×8 = 16 bits → 15 xor_bits → 4 LUT levels)
    localparam W16_TOP = INT_BITS - 2;   // xor_bits MSB index
    localparam W16_BOT = INT_BITS - 16;  // scan down to here
    reg [SHIFT_BITS-1:0] clz_16;
    integer ci16;
    always @(*) begin
        clz_16 = 7'd15;  // default: all 16 bits matched sign
        for (ci16 = W16_BOT; ci16 <= W16_TOP; ci16 = ci16 + 1)
            if (xor_bits[ci16]) clz_16 = (W16_TOP - ci16);
    end

    // 32-bit window CLZ (F4E3: 2×16 = 32 bits)
    localparam W32_BOT = INT_BITS - 32;
    reg [SHIFT_BITS-1:0] clz_32;
    integer ci32;
    always @(*) begin
        if (clz_16 < 7'd15) begin
            clz_32 = clz_16;  // found in upper 16, reuse
        end else begin
            clz_32 = 7'd31;
            for (ci32 = W32_BOT; ci32 < W16_BOT; ci32 = ci32 + 1)
                if (xor_bits[ci32]) clz_32 = (W16_TOP - ci32);
        end
    end

    // 64-bit window CLZ (F5E3: 2×32 = 64 bits)
    localparam W64_BOT = INT_BITS - 64;
    reg [SHIFT_BITS-1:0] clz_64;
    integer ci64;
    always @(*) begin
        if (clz_32 < 7'd31) begin
            clz_64 = clz_32;
        end else begin
            clz_64 = 7'd63;
            for (ci64 = W64_BOT; ci64 < W32_BOT; ci64 = ci64 + 1)
                if (xor_bits[ci64]) clz_64 = (W16_TOP - ci64);
        end
    end

    // 128-bit window CLZ (F6E3: 2×64 = 128 bits — full width)
    reg [SHIFT_BITS-1:0] clz_128;
    integer ci128;
    always @(*) begin
        if (clz_64 < 7'd63) begin
            clz_128 = clz_64;
        end else begin
            clz_128 = INT_BITS - 1;
            for (ci128 = 0; ci128 < W64_BOT; ci128 = ci128 + 1)
                if (xor_bits[ci128]) clz_128 = (W16_TOP - ci128);
        end
    end

    // Early-tap mux: select CLZ result for active width.
    // The CLZ value is already bounded to [0, 2×frac_bits-1] by construction,
    // so (frac_bits - leading_m1) is the correct exponent adjustment with
    // zero additional width correction.
    wire [SHIFT_BITS-1:0] leading_m1 =
        (s2_frac_width == 2'd0) ? clz_16  :
        (s2_frac_width == 2'd1) ? clz_32  :
        (s2_frac_width == 2'd2) ? clz_64  :
                                  clz_128;

    // Multiply abnormal: shift one less for n_level=-2
    wire s2_is_mul = (s2_op == OP_MUL);
    wire s2_is_addsub = (s2_op == OP_ADD) | (s2_op == OP_SUB);
    wire s2_is_bitwise = (s2_op == OP_AND) | (s2_op == OP_OR) | (s2_op == OP_XOR);
    wire s2_abnormal_n2 = s2_mul_abnormal & ~s2_mul_n_level_neg1;
    wire [SHIFT_BITS-1:0] norm_shift = (s2_abnormal_n2 & |leading_m1) ?
                                        leading_m1 - 1'b1 : leading_m1;

    wire signed [INT_BITS-1:0] normalized = s2_intermediate <<< norm_shift;

    wire signed [MAX_FRAC-1:0] out_frac = normalized[INT_BITS-1 -: MAX_FRAC];

    // ================================================================
    //  STAGE 2: EXPONENT CALCULATION (all MSB-aligned, no re-scaling)
    // ================================================================
    //
    // The CLZ early-tap gives leading_m1 already bounded per width.
    // frac_bits - leading_m1 is the exponent offset in integer space.
    // We scale it to MSB-aligned exponent space with a left shift.
    // No width correction needed — the tap did it for us.

    localparam ECW = MAX_EXP + 2;

    // exp_one_msb for stage 2 (re-derive from s2_exp_width)
    wire signed [MAX_EXP-1:0] s2_exp_one_msb =
        (s2_exp_width == 2'd0) ? {8'd1, 56'b0} :
        (s2_exp_width == 2'd1) ? {16'd1, 48'b0} :
        (s2_exp_width == 2'd2) ? {32'd1, 32'b0} :
                                 64'd1;

    // Add/sub/bitwise: out_exp = small_exp + (frac_bits - norm_shift) * exp_one_msb
    // Signed: frac_minus_lead can be negative when CLZ > frac_bits
    // (pre-shift adds frac_bits-1 sign-extension bits above data in CLZ window)
    wire signed [SHIFT_BITS:0] frac_minus_lead = $signed({1'b0, s2_frac_bits})
                                                - $signed({1'b0, norm_shift});
    wire [6:0] s2_exp_bits = (s2_exp_width == 2'd0) ? 7'd8  :
                             (s2_exp_width == 2'd1) ? 7'd16 :
                             (s2_exp_width == 2'd2) ? 7'd32 : 7'd64;
    wire [6:0] s2_exp_lshift = 7'd64 - s2_exp_bits;
    wire signed [ECW-1:0] exp_adjust = $signed(frac_minus_lead) <<< s2_exp_lshift;
    wire signed [ECW-1:0] s2_small_exp_w = $signed({{(ECW-MAX_EXP){s2_small_exp[MAX_EXP-1]}}, s2_small_exp});

    wire signed [ECW-1:0] exp_calc_add = s2_small_exp_w + exp_adjust;

    // Multiply: out_exp = a_exp + b_exp + (1 - norm_shift) * exp_one_msb
    wire signed [ECW-1:0] s2_a_exp_w = $signed({{(ECW-MAX_EXP){s2_a_exp[MAX_EXP-1]}}, s2_a_exp});
    wire signed [ECW-1:0] s2_b_exp_w = $signed({{(ECW-MAX_EXP){s2_b_exp[MAX_EXP-1]}}, s2_b_exp});
    // Signed: one_minus_lead can be negative when leading_m1 > 1 (common for products)
    wire signed [SHIFT_BITS:0] one_minus_lead = 8'sd1 - $signed({1'b0, leading_m1});
    wire signed [ECW-1:0] mul_exp_adjust = $signed(one_minus_lead) <<< s2_exp_lshift;
    wire signed [ECW-1:0] exp_calc_mul = s2_a_exp_w + s2_b_exp_w + mul_exp_adjust;

    wire signed [ECW-1:0] exp_calc = s2_is_mul ? exp_calc_mul : exp_calc_add;
    wire signed [MAX_EXP-1:0] out_exp = exp_calc[MAX_EXP-1:0];

    // Overflow/underflow in MSB-aligned space
    wire signed [ECW-1:0] max_exp_msb = $signed({{(ECW-MAX_EXP){1'b0}}, AMBIG_EXP - s2_exp_one_msb});
    wire signed [ECW-1:0] min_exp_msb = $signed({{(ECW-MAX_EXP){1'b1}}, AMBIG_EXP + s2_exp_one_msb});

    wire overflow  = (exp_calc > max_exp_msb);
    wire underflow = (exp_calc < min_exp_msb);

    wire signed [MAX_FRAC-1:0] vanished_frac = {out_frac[MAX_FRAC-1], out_frac[MAX_FRAC-1:1]};

    // Bitwise-specific: detect exponent overflow from negative big_exp crossing zero
    wire bw_exp_overflow = s2_big_exp_neg & ~out_exp[MAX_EXP-1] & (out_exp != 0);

    // ================================================================
    //  STAGE 2: OUTPUT MUX
    // ================================================================

    always @(*) begin
        result_frac = {MAX_FRAC{1'b0}};
        result_exp  = AMBIG_EXP;
        cmp_lt      = s2_cmp_lt;
        cmp_eq      = s2_cmp_eq;
        cmp_gt      = s2_cmp_gt;
        cmp_unord   = s2_cmp_unord;

        if (s2_is_trivial) begin
            result_frac = s2_trivial_frac;
            result_exp  = s2_trivial_exp;
        end else if (s2_shortcut) begin
            result_frac = s2_sc_frac;
            result_exp  = s2_sc_exp;
        end else if (s2_is_addsub && s2_bypass_add) begin
            result_frac = s2_big_frac;
            result_exp  = s2_big_exp;
        end else if (s2_is_addsub && s2_bypass_sub) begin
            result_frac = s2_neg_big_frac;
            result_exp  = s2_neg_big_exp;
        end else if (s2_bw_negligible) begin
            result_frac = s2_bw_negl_frac;
            result_exp  = s2_bw_negl_exp;
        end else if (s2_is_zero) begin
            result_frac = {MAX_FRAC{1'b0}};
            result_exp  = AMBIG_EXP;
        end else if (s2_mul_abnormal) begin
            result_frac = out_frac;
            result_exp  = AMBIG_EXP;
        end else if (overflow | underflow | (s2_is_bitwise & bw_exp_overflow)) begin
            result_frac = underflow ? vanished_frac : out_frac;
            result_exp  = AMBIG_EXP;
        end else begin
            result_frac = out_frac;
            result_exp  = out_exp;
        end

        // Mask sub-precision bits: the 128-bit intermediate has 2×frac_bits of
        // data, but the output should only have frac_bits of precision.
        // Applied last so vanished_frac, out_frac, etc. all get cleaned.
        case (s2_frac_width)
            2'd0: result_frac = result_frac & {{8{1'b1}},  56'b0};
            2'd1: result_frac = result_frac & {{16{1'b1}}, 48'b0};
            2'd2: result_frac = result_frac & {{32{1'b1}}, 32'b0};
            default: ;  // 64-bit: no masking needed
        endcase
    end

endmodule
