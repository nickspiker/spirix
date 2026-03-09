// spirix_alu_addbit — 2-stage pipelined add/sub + bitwise ALU
//
// Ops: ADD(0), SUB(1), AND(2), OR(3), XOR(4)
//
// Architecture: close/far split with shared barrel shifter.
// Working width = FRAC+1 (one carry bit at top, no guard bits at bottom).
// Floor-only — no rounding means no guard/round/sticky bits needed.
//
//   Stage 1 (Prepare + Barrel):
//     - Edge case detect + shortcut mux
//     - Swap: exp subtract, |diff|, mux big/small
//     - Close (|exp_diff|≤1): 1-bit align + op + CLZ
//     - Far (|exp_diff|>1): compute align shift
//     - Shared barrel: close normalizes (bit-reverse right shift),
//       far aligns (arithmetic right shift)
//
//   Stage 2 (Finish):
//     - Close: extract frac, exp calc
//     - Far: op (big OP aligned_small), bounded normalize (0-2 bits), exp
//     - Output mux: shortcut / close / far / negligible / zero / underflow
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer, ±1 arithmetic).
// Floor-only (no rounding). Full edge case handling.

module spirix_alu_addbit #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                       clk,
    input  wire                       ce,
    input  wire [2:0]                 op,       // 0=ADD 1=SUB 2=AND 3=OR 4=XOR
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output reg  signed [MAX_FRAC-1:0] result_frac,
    output reg  signed [MAX_EXP-1:0]  result_exp
);

    // ================================================================
    //  OP DECODE
    // ================================================================
    localparam OP_ADD = 3'd0, OP_SUB = 3'd1,
               OP_AND = 3'd2, OP_OR  = 3'd3, OP_XOR = 3'd4;

    wire is_add    = (op == OP_ADD);
    wire is_sub    = (op == OP_SUB);
    wire is_addsub = is_add | is_sub;
    wire is_and    = (op == OP_AND);
    wire is_or     = (op == OP_OR);
    wire is_xor    = (op == OP_XOR);
    wire is_bitwise = is_and | is_or | is_xor;

    // ================================================================
    //  WORKING WIDTH: FRAC + 1 (one carry bit at top, no guard bits)
    // ================================================================
    localparam INT_BITS    = MAX_FRAC + 1;  // 65 for MAX_FRAC=64
    localparam BARREL_BITS = $clog2(MAX_FRAC);  // 6 for MAX_FRAC=64 (64-bit barrel)
    localparam LEAD_BITS   = BARREL_BITS + 1;

    // ================================================================
    //  WIDTH DECODE + CONSTANTS
    // ================================================================
    wire [6:0] frac_bits = (frac_width == 2'd0) ? 7'd8  :
                           (frac_width == 2'd1) ? 7'd16 :
                           (frac_width == 2'd2) ? 7'd32 : 7'd64;

    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {2'b01, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_SMALL = {3'b001, {(MAX_FRAC-3){1'b0}}};
    localparam signed [MAX_FRAC-1:0] NEG_SMALL = {2'b11, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // frac_neg1_val: all-ones in active width, zeros below
    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                               {64'hFFFFFFFFFFFFFFFF};

    // Exponents are LSB-aligned (plain integers). a_exp, b_exp used directly.
    wire signed [MAX_EXP-1:0] a_ei = a_exp;
    wire signed [MAX_EXP-1:0] b_ei = b_exp;

    // Universal AMBIG: always -2^63 regardless of active width.
    // Caller maps width-specific AMBIG to this value at the interface.
    // AMBIG_EXP is already defined as a localparam above.

    // ================================================================
    //  STATE DETECTION
    // ================================================================
    wire a_n1   = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire b_n1   = (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-2]);
    wire a_n2   = ~a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire b_n2   = ~b_n1 & (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-3]);
    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire b_top3 = (b_frac[MAX_FRAC-1] == b_frac[MAX_FRAC-2]) &
                  (b_frac[MAX_FRAC-2] == b_frac[MAX_FRAC-3]);

    wire a_frac_zero = (a_frac == {MAX_FRAC{1'b0}});
    wire b_frac_zero = (b_frac == {MAX_FRAC{1'b0}});
    wire a_frac_neg1 = (a_frac == frac_neg1_val);
    wire b_frac_neg1 = (b_frac == frac_neg1_val);
    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;

    // AMBIG detection (constant comparison — universal AMBIG = -2^63)
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);

    wire a_is_zero   = a_is_ambig & a_frac_zero;
    wire a_is_inf    = a_is_ambig & a_frac_neg1;
    wire a_exploded  = a_is_ambig & a_n1;
    wire a_transf    = a_is_inf | a_exploded;
    wire a_vanished  = a_n2;
    wire a_undef     = ~a_n0 & a_top3;
    wire a_is_neg    = a_frac[MAX_FRAC-1];

    wire b_is_zero   = b_is_ambig & b_frac_zero;
    wire b_is_inf    = b_is_ambig & b_frac_neg1;
    wire b_exploded  = b_is_ambig & b_n1;
    wire b_transf    = b_is_inf | b_exploded;
    wire b_vanished  = b_n2;
    wire b_undef     = ~b_n0 & b_top3;
    wire b_is_neg    = b_frac[MAX_FRAC-1];

    // Rust is_normal() = exponent != AMBIG (no N1 check).
    wire a_is_normal  = ~a_is_ambig;
    wire b_is_normal  = ~b_is_ambig;
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    wire signed [MAX_FRAC-1:0] not_a_frac = a_frac ^ frac_neg1_val;
    wire signed [MAX_FRAC-1:0] not_b_frac = b_frac ^ frac_neg1_val;

    // ================================================================
    //  SPIRIX NEGATION OF B (for sub edge cases)
    // ================================================================
    wire b_is_pos_half  = (b_frac == POS_HALF);
    wire b_is_neg_one   = (b_frac == NEG_ONE);
    wire b_is_pos_small = (b_frac == POS_SMALL);
    wire b_is_neg_small = (b_frac == NEG_SMALL);

    wire signed [MAX_EXP-1:0] b_ei_m1 = b_ei - 1;
    // Width-specific AMBIG pattern on b_ei_m1 (bit-pattern detect, no 64-bit mux)
    // -128 sign-ext: bit[7]=1, bits[6:0]=0, bits[63:8]=all 1
    wire bm1_ambig_e3 = b_ei_m1[7]  & ~|b_ei_m1[6:0]   & &b_ei_m1[63:8];
    wire bm1_ambig_e4 = b_ei_m1[15] & ~|b_ei_m1[14:0]  & &b_ei_m1[63:16];
    wire bm1_ambig_e5 = b_ei_m1[31] & ~|b_ei_m1[30:0]  & &b_ei_m1[63:32];
    wire b_ei_m1_ambig = (exp_width == 2'd0) ? bm1_ambig_e3 :
                         (exp_width == 2'd1) ? bm1_ambig_e4 :
                         (exp_width == 2'd2) ? bm1_ambig_e5 :
                                               (b_ei_m1 == AMBIG_EXP);

    wire signed [MAX_FRAC-1:0] neg_b_frac_normal =
        b_is_pos_half ? (b_ei_m1_ambig ? NEG_SMALL : NEG_ONE) :
        b_is_neg_one  ? POS_HALF :
                        -b_frac;
    wire signed [MAX_EXP-1:0] b_ei_p1 = b_ei + 1;
    // b_ei_p1 out of range for active width? (NEG_ONE@max → overflow)
    wire bp1_fits_e3 = &(b_ei_p1[63:7]  ^~ {57{b_ei_p1[7]}});
    wire bp1_fits_e4 = &(b_ei_p1[63:15] ^~ {49{b_ei_p1[15]}});
    wire bp1_fits_e5 = &(b_ei_p1[63:31] ^~ {33{b_ei_p1[31]}});
    wire bp1_e3_ambig = b_ei_p1[7]  & ~|b_ei_p1[6:0]   & &b_ei_p1[63:8];
    wire bp1_e4_ambig = b_ei_p1[15] & ~|b_ei_p1[14:0]  & &b_ei_p1[63:16];
    wire bp1_e5_ambig = b_ei_p1[31] & ~|b_ei_p1[30:0]  & &b_ei_p1[63:32];
    wire b_ei_p1_bad = (exp_width == 2'd0) ? (!bp1_fits_e3 | bp1_e3_ambig) :
                       (exp_width == 2'd1) ? (!bp1_fits_e4 | bp1_e4_ambig) :
                       (exp_width == 2'd2) ? (!bp1_fits_e5 | bp1_e5_ambig) :
                                             (b_ei_p1 == AMBIG_EXP);
    wire signed [MAX_EXP-1:0] neg_b_ei_normal =
        b_is_pos_half ? (b_ei_m1_ambig ? AMBIG_EXP : b_ei_m1) :
        b_is_neg_one  ? (b_ei_p1_bad   ? AMBIG_EXP : b_ei_p1) :
                        b_ei;

    wire b_nonnorm_nochange = b_n0 | (~b_n0 & b_top3);
    wire signed [MAX_FRAC-1:0] neg_b_frac_nonnorm =
        b_nonnorm_nochange ? b_frac :
        b_is_pos_half  ? NEG_ONE :
        b_is_neg_one   ? POS_HALF :
        b_is_pos_small ? NEG_SMALL :
        b_is_neg_small ? POS_SMALL :
                         -b_frac;

    wire signed [MAX_FRAC-1:0] neg_b_frac = b_is_ambig ? neg_b_frac_nonnorm : neg_b_frac_normal;
    wire signed [MAX_EXP-1:0]  neg_b_ei   = b_is_ambig ? b_ei               : neg_b_ei_normal;

    // ================================================================
    //  EDGE CASE SHORTCUTS — ADD/SUB
    // ================================================================
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_P_TF   = {8'h1F, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_M_TF   = {8'hE0, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_VAN_P_VAN  = {8'h1E, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_VAN_M_VAN  = {8'hE1, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_P_FIN   = {8'h1C, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_M_FIN   = {8'hE3, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_FIN_P_TF   = {8'h18, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_FIN_M_TF   = {8'hE7, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_AND         = {8'h12, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_OR          = {8'hED, 56'b0};
    localparam signed [MAX_FRAC-1:0] UNDEF_XOR         = {8'h11, 56'b0};

    // --- Shared undef passthrough ---
    wire sc_a_undef = any_non_normal & a_undef;
    wire sc_b_undef = any_non_normal & ~a_undef & b_undef;

    // --- Add/sub shortcuts ---
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

    wire add_shortcut = sc_a_undef | sc_b_undef | sc_add_tf_tf | sc_add_van_van |
                        sc_add_a_transf | sc_add_b_transf | sc_add_a_van | sc_add_b_van |
                        sc_add_a_zero | sc_add_fallback;

    // ================================================================
    //  EDGE CASE SHORTCUTS — BITWISE (AND/OR/XOR)
    // ================================================================
    wire and_sc_esc_esc      = any_non_normal & ((a_exploded & b_exploded) | (a_vanished & b_vanished) | a_is_inf | b_is_inf);
    wire and_sc_any_zero     = any_non_normal & ~and_sc_esc_esc & (a_is_zero | b_is_zero);
    wire and_sc_a_van        = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & a_vanished;
    wire and_sc_b_van        = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & b_vanished;
    wire and_sc_a_norm_b_esc = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & ~and_sc_b_van & a_is_normal & ~b_is_normal;
    wire and_sc_b_norm_a_esc = any_non_normal & ~and_sc_esc_esc & ~and_sc_any_zero & ~and_sc_a_van & ~and_sc_b_van & ~and_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire or_sc_esc           = any_non_normal & (a_is_inf | b_is_inf | (a_exploded & b_exploded));
    wire or_sc_a_zero        = any_non_normal & ~or_sc_esc & a_is_zero;
    wire or_sc_b_zero        = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & b_is_zero;
    wire or_sc_a_exp         = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & a_exploded & ~b_exploded;
    wire or_sc_b_exp         = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & b_exploded;
    wire or_sc_a_norm_b_esc  = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & ~or_sc_b_exp & a_is_normal & ~b_is_normal;
    wire or_sc_b_norm_a_esc  = any_non_normal & ~or_sc_esc & ~or_sc_a_zero & ~or_sc_b_zero & ~or_sc_a_exp & ~or_sc_b_exp & ~or_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    wire xor_sc_esc          = any_non_normal & (a_is_inf | b_is_inf | (a_exploded & b_exploded) | (a_vanished & b_vanished));
    wire xor_sc_a_zero       = any_non_normal & ~xor_sc_esc & a_is_zero;
    wire xor_sc_b_zero       = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & b_is_zero;
    wire xor_sc_a_van        = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & a_vanished;
    wire xor_sc_b_van        = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & b_vanished;
    wire xor_sc_a_norm_b_esc = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & ~xor_sc_b_van & a_is_normal & ~b_is_normal;
    wire xor_sc_b_norm_a_esc = any_non_normal & ~xor_sc_esc & ~xor_sc_a_zero & ~xor_sc_b_zero & ~xor_sc_a_van & ~xor_sc_b_van & ~xor_sc_a_norm_b_esc & b_is_normal & ~a_is_normal;

    // Bitwise shortcut output mux (all exps LSB-aligned)
    reg sc_bitwise;
    reg signed [MAX_FRAC-1:0] sc_bw_frac;
    reg signed [MAX_EXP-1:0]  sc_bw_ei;

    always @(*) begin
        sc_bitwise = 1'b0;
        sc_bw_frac = {MAX_FRAC{1'b0}};
        sc_bw_ei   = AMBIG_EXP;

        if (is_bitwise && any_non_normal && a_undef) begin
            sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_ei = a_ei;
        end else if (is_bitwise && any_non_normal && b_undef) begin
            sc_bitwise = 1'b1; sc_bw_frac = b_frac; sc_bw_ei = b_ei;
        end else if (is_and) begin
            if (and_sc_esc_esc) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_AND; sc_bw_ei = AMBIG_EXP;
            end else if (and_sc_any_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_ei = AMBIG_EXP;
            end else if (and_sc_a_van) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_ei = AMBIG_EXP; end
            end else if (and_sc_b_van) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_ei = AMBIG_EXP; end
            end else if (and_sc_a_norm_b_esc) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_ei = AMBIG_EXP; end
            end else if (and_sc_b_norm_a_esc) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
                else          begin sc_bw_frac = {MAX_FRAC{1'b0}}; sc_bw_ei = AMBIG_EXP; end
            end else if (~a_is_normal | ~b_is_normal) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_AND; sc_bw_ei = AMBIG_EXP;
            end
        end else if (is_or) begin
            if (or_sc_esc) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_OR; sc_bw_ei = AMBIG_EXP;
            end else if (or_sc_a_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = b_frac; sc_bw_ei = b_ei;
            end else if (or_sc_b_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_ei = a_ei;
            end else if (or_sc_a_exp) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
                else          begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
            end else if (or_sc_b_exp) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
                else          begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
            end else if (or_sc_a_norm_b_esc) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
                else          begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
            end else if (or_sc_b_norm_a_esc) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
                else          begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
            end else if (~a_is_normal | ~b_is_normal) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_OR; sc_bw_ei = AMBIG_EXP;
            end
        end else if (is_xor) begin
            if (xor_sc_esc) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_XOR; sc_bw_ei = AMBIG_EXP;
            end else if (xor_sc_a_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = b_frac; sc_bw_ei = b_ei;
            end else if (xor_sc_b_zero) begin
                sc_bitwise = 1'b1; sc_bw_frac = a_frac; sc_bw_ei = a_ei;
            end else if (xor_sc_a_van) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = not_b_frac; sc_bw_ei = b_ei; end
                else          begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
            end else if (xor_sc_b_van) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = not_a_frac; sc_bw_ei = a_ei; end
                else          begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
            end else if (xor_sc_a_norm_b_esc) begin
                sc_bitwise = 1'b1;
                if (a_is_neg) begin sc_bw_frac = not_b_frac; sc_bw_ei = b_ei; end
                else          begin sc_bw_frac = b_frac; sc_bw_ei = b_ei; end
            end else if (xor_sc_b_norm_a_esc) begin
                sc_bitwise = 1'b1;
                if (b_is_neg) begin sc_bw_frac = not_a_frac; sc_bw_ei = a_ei; end
                else          begin sc_bw_frac = a_frac; sc_bw_ei = a_ei; end
            end else if (~a_is_normal | ~b_is_normal) begin
                sc_bitwise = 1'b1; sc_bw_frac = UNDEF_XOR; sc_bw_ei = AMBIG_EXP;
            end
        end
    end

    wire bw_shortcut = sc_bitwise;

    // --- Combined shortcut ---
    wire shortcut = is_addsub  ? add_shortcut :
                    is_bitwise ? bw_shortcut  :
                                 1'b0;

    // Shortcut output mux (exps LSB-aligned)
    wire signed [MAX_FRAC-1:0] sc_frac =
        sc_a_undef       ? a_frac :
        sc_b_undef       ? b_frac :
        sc_add_tf_tf     ? (is_sub ? UNDEF_TF_M_TF  : UNDEF_TF_P_TF) :
        sc_add_van_van   ? (is_sub ? UNDEF_VAN_M_VAN : UNDEF_VAN_P_VAN) :
        sc_add_a_transf  ? (is_sub ? UNDEF_TF_M_FIN  : UNDEF_TF_P_FIN) :
        sc_add_b_transf  ? (is_sub ? UNDEF_FIN_M_TF  : UNDEF_FIN_P_TF) :
        sc_add_a_van     ? (is_sub ? neg_b_frac : b_frac) :
        sc_add_b_van     ? a_frac :
        sc_add_a_zero    ? (is_sub ? neg_b_frac : b_frac) :
        sc_add_fallback  ? a_frac :
        sc_bitwise       ? sc_bw_frac :
                           {MAX_FRAC{1'b0}};

    wire signed [MAX_EXP-1:0] sc_ei =
        sc_a_undef      ? a_ei :
        sc_b_undef      ? b_ei :
        sc_add_a_van    ? (is_sub ? neg_b_ei : b_ei) :
        sc_add_b_van    ? a_ei :
        sc_add_a_zero   ? (is_sub ? neg_b_ei : b_ei) :
        sc_add_fallback ? a_ei :
        sc_bitwise      ? sc_bw_ei :
                          AMBIG_EXP;

    // ================================================================
    //  STAGE 1: SWAP + CLOSE OP + CLZ + SHARED BARREL
    // ================================================================

    // --- Exponent difference (LSB-aligned = already integer) ---
    wire signed [MAX_EXP:0] raw_diff = $signed({a_ei[MAX_EXP-1], a_ei})
                                      - $signed({b_ei[MAX_EXP-1], b_ei});
    wire a_is_big = !raw_diff[MAX_EXP];

    wire signed [MAX_FRAC-1:0] big_frac  = a_is_big ? a_frac : b_frac;
    wire signed [MAX_EXP-1:0]  big_ei    = a_is_big ? a_ei   : b_ei;
    wire signed [MAX_FRAC-1:0] small_frac = a_is_big ? b_frac : a_frac;
    wire signed [MAX_EXP-1:0]  small_ei   = a_is_big ? b_ei   : a_ei;

    // Integer exp difference — no de-scaling needed
    wire signed [MAX_EXP:0] exp_diff_int = raw_diff[MAX_EXP] ? -raw_diff : raw_diff;

    wire negligible = (exp_diff_int >= $signed({{(MAX_EXP+1-7){1'b0}}, frac_bits}));
    wire is_close   = (exp_diff_int <= 1) & ~negligible;

    // Subtraction negate flags
    wire negate_small = is_sub &  a_is_big;
    wire negate_big   = is_sub & !a_is_big;

    // --- Negligible bypass for add/sub ---
    wire big_is_pos_half = (big_frac == POS_HALF);
    wire big_is_neg_one  = (big_frac == NEG_ONE);
    wire signed [MAX_FRAC-1:0] neg_big_frac = big_is_pos_half ? NEG_ONE  :
                                              big_is_neg_one  ? POS_HALF :
                                              -big_frac;
    wire signed [MAX_EXP-1:0]  neg_big_ei   = big_is_pos_half ? (big_ei - 1) :
                                               big_is_neg_one  ? (big_ei + 1) :
                                               big_ei;

    wire bypass_add = is_addsub & negligible & !negate_big;
    wire bypass_sub = is_addsub & negligible &  negate_big;

    // --- Negligible bypass for bitwise ---
    wire small_is_neg  = small_frac[MAX_FRAC-1];
    wire bw_negligible = is_bitwise & negligible;
    wire big_undef     = a_is_big ? a_undef : b_undef;

    reg signed [MAX_FRAC-1:0] bw_negl_frac;
    reg signed [MAX_EXP-1:0]  bw_negl_ei;
    always @(*) begin
        bw_negl_frac = {MAX_FRAC{1'b0}};
        bw_negl_ei   = AMBIG_EXP;
        if (is_and) begin
            if (small_is_neg) begin bw_negl_frac = big_frac; bw_negl_ei = big_ei; end
            else              begin bw_negl_frac = {MAX_FRAC{1'b0}}; bw_negl_ei = AMBIG_EXP; end
        end else if (is_or) begin
            if (small_is_neg) begin bw_negl_frac = small_frac; bw_negl_ei = small_ei; end
            else              begin bw_negl_frac = big_frac; bw_negl_ei = big_ei; end
        end else begin // XOR
            if (small_is_neg & ~big_undef) begin bw_negl_frac = big_frac ^ frac_neg1_val; bw_negl_ei = big_ei; end
            else                           begin bw_negl_frac = big_frac; bw_negl_ei = big_ei; end
        end
    end

    // --- Extend to INT_BITS (65): one-bit sign extension for carry ---
    wire signed [INT_BITS-1:0] big_ext   = {big_frac[MAX_FRAC-1], big_frac};
    wire signed [INT_BITS-1:0] small_ext = {small_frac[MAX_FRAC-1], small_frac};

    // --- Close path: 1-bit align + op in 65-bit space ---
    wire signed [INT_BITS-1:0] close_small = exp_diff_int[0] ? (small_ext >>> 1) : small_ext;

    wire signed [INT_BITS-1:0] close_result =
        is_and ? (big_ext & close_small) :
        is_or  ? (big_ext | close_small) :
        is_xor ? (big_ext ^ close_small) :
                 (big_ext ^ {INT_BITS{negate_big}})
                 + (close_small ^ {INT_BITS{negate_small}})
                 + {{(INT_BITS-1){1'b0}}, is_sub};

    wire close_is_zero = (close_result == 0);

    // Overflow: top 2 bits of 65-bit result differ (carry beyond 64-bit range)
    wire close_ovf = close_result[INT_BITS-1] ^ close_result[INT_BITS-2];
    wire signed [MAX_FRAC-1:0] close_ovf_frac = close_result[INT_BITS-1:1];

    // CLZ on non-overflow 64-bit result (bits [MAX_FRAC-1:0])
    localparam DIFF_W = MAX_FRAC - 1;
    wire [DIFF_W-1:0] xor_diff = close_result[MAX_FRAC-1:1] ^ close_result[MAX_FRAC-2:0];

    reg [BARREL_BITS-1:0] close_norm_shift;
    integer ci;
    always @(*) begin
        close_norm_shift = DIFF_W[BARREL_BITS-1:0];
        for (ci = 0; ci < DIFF_W; ci = ci + 1)
            if (xor_diff[ci]) close_norm_shift = DIFF_W[BARREL_BITS-1:0] - 1 - ci[BARREL_BITS-1:0];
    end
    wire [LEAD_BITS-1:0] close_leading = close_norm_shift + 1;

    // --- Shared 64-bit barrel shifter ---
    // Close (non-overflow): bit-reverse -> right shift -> bit-reverse = left shift
    // Far: arithmetic right shift for alignment
    // Overflow bypasses barrel entirely (frac = result[64:1], exp + 1)

    wire [MAX_FRAC-1:0] close_result_lo = close_result[MAX_FRAC-1:0];
    wire [MAX_FRAC-1:0] close_result_rev;
    genvar gi;
    generate
        for (gi = 0; gi < MAX_FRAC; gi = gi + 1) begin : bitrev_in
            assign close_result_rev[gi] = close_result_lo[MAX_FRAC - 1 - gi];
        end
    endgenerate

    wire [BARREL_BITS-1:0] far_shift_raw = exp_diff_int[BARREL_BITS-1:0];
    wire [BARREL_BITS-1:0] far_shift = (exp_diff_int >= MAX_FRAC)
        ? {BARREL_BITS{1'b1}} : far_shift_raw;

    wire [MAX_FRAC-1:0] barrel_in = is_close ? close_result_rev : $unsigned(small_frac);
    wire [BARREL_BITS-1:0] barrel_shift = is_close ? close_norm_shift : far_shift;

    // Barrel stages (6 stages for 64 bits: 1,2,4,8,16,32)
    wire fill0 = !is_close & barrel_in[MAX_FRAC-1];
    wire [MAX_FRAC-1:0] b0_val = barrel_shift[0] ? {fill0, barrel_in[MAX_FRAC-1:1]} : barrel_in;

    wire fill1 = !is_close & b0_val[MAX_FRAC-1];
    wire [MAX_FRAC-1:0] b1_val = barrel_shift[1] ? {{2{fill1}}, b0_val[MAX_FRAC-1:2]} : b0_val;

    wire fill2 = !is_close & b1_val[MAX_FRAC-1];
    wire [MAX_FRAC-1:0] b2_val = barrel_shift[2] ? {{4{fill2}}, b1_val[MAX_FRAC-1:4]} : b1_val;

    wire fill3 = !is_close & b2_val[MAX_FRAC-1];
    wire [MAX_FRAC-1:0] b3_val = barrel_shift[3] ? {{8{fill3}}, b2_val[MAX_FRAC-1:8]} : b2_val;

    wire fill4 = !is_close & b3_val[MAX_FRAC-1];
    wire [MAX_FRAC-1:0] b4_val = barrel_shift[4] ? {{16{fill4}}, b3_val[MAX_FRAC-1:16]} : b3_val;

    wire fill5 = !is_close & b4_val[MAX_FRAC-1];
    wire [MAX_FRAC-1:0] b5_val = barrel_shift[5] ? {{32{fill5}}, b4_val[MAX_FRAC-1:32]} : b4_val;

    // Bit-reverse barrel output for close path (free wiring)
    wire [MAX_FRAC-1:0] close_normalized;
    generate
        for (gi = 0; gi < MAX_FRAC; gi = gi + 1) begin : bitrev_out
            assign close_normalized[gi] = b5_val[MAX_FRAC - 1 - gi];
        end
    endgenerate

    // ================================================================
    //  S1 → S2 PIPELINE REGISTER
    // ================================================================
    reg [MAX_FRAC-1:0]          s2_close_normalized;
    reg                         s2_close_ovf;
    reg signed [MAX_FRAC-1:0]   s2_close_ovf_frac;
    reg [LEAD_BITS-1:0]         s2_close_leading;
    reg                         s2_close_is_zero;
    reg signed [MAX_FRAC-1:0]   s2_far_aligned;
    reg signed [MAX_FRAC-1:0]   s2_big_frac;
    reg signed [MAX_EXP-1:0]    s2_big_ei;
    reg                         s2_is_close;
    reg                         s2_negligible;
    reg                         s2_negate_big;
    reg                         s2_negate_small;
    reg                         s2_sub;
    reg                         s2_shortcut;
    reg signed [MAX_FRAC-1:0]   s2_sc_frac;
    reg signed [MAX_EXP-1:0]    s2_sc_ei;
    reg                         s2_bypass_add;
    reg                         s2_bypass_sub;
    reg signed [MAX_FRAC-1:0]   s2_neg_big_frac;
    reg signed [MAX_EXP-1:0]    s2_neg_big_ei;
    reg                         s2_bw_negligible;
    reg signed [MAX_FRAC-1:0]   s2_bw_negl_frac;
    reg signed [MAX_EXP-1:0]    s2_bw_negl_ei;
    reg [2:0]                   s2_op;
    reg [1:0]                   s2_frac_width;
    reg [1:0]                   s2_exp_width;
    reg [6:0]                   s2_frac_bits;
    reg signed [MAX_EXP-1:0]    s2_small_ei;

    always @(posedge clk) if (ce) begin
        s2_close_normalized <= close_normalized;
        s2_close_ovf        <= close_ovf;
        s2_close_ovf_frac   <= close_ovf_frac;
        s2_close_leading    <= close_leading;
        s2_close_is_zero    <= close_is_zero;
        s2_far_aligned      <= $signed(b5_val);
        s2_big_frac         <= big_frac;
        s2_big_ei           <= big_ei;
        s2_is_close         <= is_close;
        s2_negligible       <= negligible;
        s2_negate_big       <= negate_big;
        s2_negate_small     <= negate_small;
        s2_sub              <= is_sub;
        s2_shortcut         <= shortcut;
        s2_sc_frac          <= sc_frac;
        s2_sc_ei            <= sc_ei;
        s2_bypass_add       <= bypass_add;
        s2_bypass_sub       <= bypass_sub;
        s2_neg_big_frac     <= neg_big_frac;
        s2_neg_big_ei       <= neg_big_ei;
        s2_bw_negligible    <= bw_negligible;
        s2_bw_negl_frac     <= bw_negl_frac;
        s2_bw_negl_ei       <= bw_negl_ei;
        s2_op               <= op;
        s2_frac_width       <= frac_width;
        s2_exp_width        <= exp_width;
        s2_frac_bits        <= frac_bits;
        s2_small_ei         <= small_ei;
    end

    // ================================================================
    //  STAGE 2: FINISH
    // ================================================================

    wire s2_is_addsub  = (s2_op == OP_ADD) | (s2_op == OP_SUB);
    wire s2_is_and     = (s2_op == OP_AND);
    wire s2_is_or      = (s2_op == OP_OR);
    wire s2_is_xor     = (s2_op == OP_XOR);
    wire s2_is_bitwise = s2_is_and | s2_is_or | s2_is_xor;

    // --- Close path: extract frac (floor) + exp ---
    // Overflow: take bits[64:1] (right-shift by 1), exp = big_ei + 1
    // Non-overflow: barrel-normalized 64-bit result, exp = big_ei - norm_shift
    wire signed [MAX_FRAC-1:0] close_out_frac = s2_close_ovf
        ? s2_close_ovf_frac : $signed(s2_close_normalized);

    // Exponent offset: overflow → +1, else → -(norm_shift) = 1 - leading
    wire signed [7:0] close_int_offset = s2_close_ovf
        ? 8'sd1 : (8'sd1 - $signed({1'b0, s2_close_leading}));

    // Direct integer add — no scaling needed
    wire signed [MAX_EXP:0] close_exp_wide = $signed({s2_big_ei[MAX_EXP-1], s2_big_ei})
                                            + $signed({{(MAX_EXP-7){close_int_offset[7]}}, close_int_offset});
    wire signed [MAX_EXP-1:0] close_out_ei = close_exp_wide[MAX_EXP-1:0];

    // Close: does 65-bit exp fit in active width? (parallel bit-pattern checks, 1-bit mux)
    wire close_fits_e3 = &(close_exp_wide[64:7]  ^~ {58{close_exp_wide[7]}});
    wire close_fits_e4 = &(close_exp_wide[64:15] ^~ {50{close_exp_wide[15]}});
    wire close_fits_e5 = &(close_exp_wide[64:31] ^~ {34{close_exp_wide[31]}});
    wire close_exp_fits = (s2_exp_width == 2'd0) ? close_fits_e3 :
                          (s2_exp_width == 2'd1) ? close_fits_e4 :
                          (s2_exp_width == 2'd2) ? close_fits_e5 : 1'b1;
    // Also check for width-specific AMBIG pattern (e.g. -128 for E3 fits in 8-bit but is reserved)
    wire close_e3_ambig = close_exp_wide[7]  & ~|close_exp_wide[6:0]   & &close_exp_wide[64:8];
    wire close_e4_ambig = close_exp_wide[15] & ~|close_exp_wide[14:0]  & &close_exp_wide[64:16];
    wire close_e5_ambig = close_exp_wide[31] & ~|close_exp_wide[30:0]  & &close_exp_wide[64:32];
    wire close_width_ambig = (s2_exp_width == 2'd0) ? close_e3_ambig :
                             (s2_exp_width == 2'd1) ? close_e4_ambig :
                             (s2_exp_width == 2'd2) ? close_e5_ambig : 1'b0;
    wire close_out_of_range = !close_exp_fits | close_width_ambig;
    wire close_underflow = close_out_of_range & close_exp_wide[MAX_EXP];
    wire close_overflow  = close_out_of_range & !close_exp_wide[MAX_EXP];
    wire signed [MAX_FRAC-1:0] close_uf_frac = s2_close_ovf
        ? {s2_close_ovf_frac[MAX_FRAC-1], s2_close_ovf_frac[MAX_FRAC-1:1]}
        : {s2_close_normalized[MAX_FRAC-1], s2_close_normalized[MAX_FRAC-1:1]};

    // --- Far path: op in 65-bit space + overflow detect + normalize ---
    wire signed [INT_BITS-1:0] big_ext_s2 = {s2_big_frac[MAX_FRAC-1], s2_big_frac};
    wire signed [INT_BITS-1:0] aligned_ext = {s2_far_aligned[MAX_FRAC-1], s2_far_aligned};

    wire signed [INT_BITS-1:0] far_result =
        s2_is_and ? (big_ext_s2 & aligned_ext) :
        s2_is_or  ? (big_ext_s2 | aligned_ext) :
        s2_is_xor ? (big_ext_s2 ^ aligned_ext) :
                    (big_ext_s2 ^ {INT_BITS{s2_negate_big}})
                    + (aligned_ext ^ {INT_BITS{s2_negate_small}})
                    + {{(INT_BITS-1){1'b0}}, s2_sub};

    wire far_is_zero = (far_result == 0);

    // Overflow: top 2 bits differ (carry beyond 64-bit range, add/sub only)
    wire far_ovf = far_result[INT_BITS-1] ^ far_result[INT_BITS-2];
    wire signed [MAX_FRAC-1:0] far_ovf_frac = far_result[INT_BITS-1:1];

    // Non-overflow result (lower 64 bits)
    wire [MAX_FRAC-1:0] far_nonovf = far_result[MAX_FRAC-1:0];

    // --- Add/sub: bounded normalize (0-2 bits, guaranteed by far-path constraint) ---
    wire far_d0 = far_nonovf[MAX_FRAC-1] ^ far_nonovf[MAX_FRAC-2];
    wire far_d1 = far_nonovf[MAX_FRAC-2] ^ far_nonovf[MAX_FRAC-3];
    wire [1:0] far_as_norm_shift = far_d0 ? 2'd0 : far_d1 ? 2'd1 : 2'd2;
    wire [LEAD_BITS-1:0] far_as_leading = {{(LEAD_BITS-2){1'b0}}, far_as_norm_shift} + 1;
    wire [MAX_FRAC-1:0] far_as_normalized = far_d0 ? far_nonovf :
                                            far_d1 ? (far_nonovf << 1) :
                                                     (far_nonovf << 2);

    // --- Bitwise: full CLZ + barrel normalize (can need up to frac_bits-1 shift) ---
    wire [DIFF_W-1:0] far_xor_diff = far_nonovf[MAX_FRAC-1:1] ^ far_nonovf[MAX_FRAC-2:0];
    reg [BARREL_BITS-1:0] far_bw_norm_shift;
    integer fi;
    always @(*) begin
        far_bw_norm_shift = DIFF_W[BARREL_BITS-1:0];
        for (fi = 0; fi < DIFF_W; fi = fi + 1)
            if (far_xor_diff[fi]) far_bw_norm_shift = DIFF_W[BARREL_BITS-1:0] - 1 - fi[BARREL_BITS-1:0];
    end
    wire [LEAD_BITS-1:0] far_bw_leading = far_bw_norm_shift + 1;

    // Bitwise normalize barrel (bit-reverse → right shift → bit-reverse = left shift)
    wire [MAX_FRAC-1:0] far_bw_rev;
    generate
        for (gi = 0; gi < MAX_FRAC; gi = gi + 1) begin : far_bitrev_in
            assign far_bw_rev[gi] = far_nonovf[MAX_FRAC - 1 - gi];
        end
    endgenerate

    wire [MAX_FRAC-1:0] fb0 = far_bw_norm_shift[0] ? {1'b0, far_bw_rev[MAX_FRAC-1:1]} : far_bw_rev;
    wire [MAX_FRAC-1:0] fb1 = far_bw_norm_shift[1] ? {{2{1'b0}}, fb0[MAX_FRAC-1:2]} : fb0;
    wire [MAX_FRAC-1:0] fb2 = far_bw_norm_shift[2] ? {{4{1'b0}}, fb1[MAX_FRAC-1:4]} : fb1;
    wire [MAX_FRAC-1:0] fb3 = far_bw_norm_shift[3] ? {{8{1'b0}}, fb2[MAX_FRAC-1:8]} : fb2;
    wire [MAX_FRAC-1:0] fb4 = far_bw_norm_shift[4] ? {{16{1'b0}}, fb3[MAX_FRAC-1:16]} : fb3;
    wire [MAX_FRAC-1:0] fb5 = far_bw_norm_shift[5] ? {{32{1'b0}}, fb4[MAX_FRAC-1:32]} : fb4;

    wire [MAX_FRAC-1:0] far_bw_normalized;
    generate
        for (gi = 0; gi < MAX_FRAC; gi = gi + 1) begin : far_bitrev_out
            assign far_bw_normalized[gi] = fb5[MAX_FRAC - 1 - gi];
        end
    endgenerate

    // --- Mux: add/sub uses bounded, bitwise uses full CLZ ---
    wire [MAX_FRAC-1:0] far_norm_result = s2_is_bitwise ? far_bw_normalized : far_as_normalized;
    wire [LEAD_BITS-1:0] far_leading = s2_is_bitwise ? far_bw_leading : far_as_leading;

    wire signed [MAX_FRAC-1:0] far_out_frac = far_ovf
        ? far_ovf_frac : $signed(far_norm_result);

    // Far exponent: overflow → +1, else → 1 - leading (direct integer add)
    wire signed [7:0] far_int_offset = far_ovf
        ? 8'sd1 : (8'sd1 - $signed({{(8-LEAD_BITS){1'b0}}, far_leading}));
    wire signed [MAX_EXP:0] far_exp_wide = $signed({s2_big_ei[MAX_EXP-1], s2_big_ei})
                                          + $signed({{(MAX_EXP-7){far_int_offset[7]}}, far_int_offset});
    wire signed [MAX_EXP-1:0] far_out_ei = far_exp_wide[MAX_EXP-1:0];

    // Far: does 65-bit exp fit in active width? (same pattern as close)
    wire far_fits_e3 = &(far_exp_wide[64:7]  ^~ {58{far_exp_wide[7]}});
    wire far_fits_e4 = &(far_exp_wide[64:15] ^~ {50{far_exp_wide[15]}});
    wire far_fits_e5 = &(far_exp_wide[64:31] ^~ {34{far_exp_wide[31]}});
    wire far_exp_fits = (s2_exp_width == 2'd0) ? far_fits_e3 :
                        (s2_exp_width == 2'd1) ? far_fits_e4 :
                        (s2_exp_width == 2'd2) ? far_fits_e5 : 1'b1;
    wire far_e3_ambig = far_exp_wide[7]  & ~|far_exp_wide[6:0]   & &far_exp_wide[64:8];
    wire far_e4_ambig = far_exp_wide[15] & ~|far_exp_wide[14:0]  & &far_exp_wide[64:16];
    wire far_e5_ambig = far_exp_wide[31] & ~|far_exp_wide[30:0]  & &far_exp_wide[64:32];
    wire far_width_ambig = (s2_exp_width == 2'd0) ? far_e3_ambig :
                           (s2_exp_width == 2'd1) ? far_e4_ambig :
                           (s2_exp_width == 2'd2) ? far_e5_ambig : 1'b0;
    wire far_out_of_range = !far_exp_fits | far_width_ambig;
    wire far_underflow = far_out_of_range & far_exp_wide[MAX_EXP];
    wire far_overflow  = far_out_of_range & !far_exp_wide[MAX_EXP];
    wire signed [MAX_FRAC-1:0] far_uf_frac = far_ovf
        ? {far_ovf_frac[MAX_FRAC-1], far_ovf_frac[MAX_FRAC-1:1]}
        : {far_norm_result[MAX_FRAC-1], far_norm_result[MAX_FRAC-1:1]};

    // --- Output mux ---
    wire use_close = s2_is_close & ~s2_negligible;

    wire signed [MAX_FRAC-1:0] path_frac = use_close ? close_out_frac : far_out_frac;
    wire signed [MAX_EXP-1:0]  path_ei   = use_close ? close_out_ei   : far_out_ei;
    wire path_underflow = use_close ? close_underflow : far_underflow;
    wire path_overflow  = use_close ? close_overflow  : far_overflow;
    wire signed [MAX_FRAC-1:0] path_uf_frac = use_close ? close_uf_frac : far_uf_frac;
    wire path_is_zero = use_close ? s2_close_is_zero : far_is_zero;

    // Sub-precision mask: zero out bits below active frac width
    wire [MAX_FRAC-1:0] subprec_mask =
        (s2_frac_width == 2'd0) ? {{8{1'b1}},  56'b0} :
        (s2_frac_width == 2'd1) ? {{16{1'b1}}, 48'b0} :
        (s2_frac_width == 2'd2) ? {{32{1'b1}}, 32'b0} :
                                  {MAX_FRAC{1'b1}};

    // --- Output clamp: bypass exp may exceed active width range ---
    // Bypass sub outputs neg_big_ei which can overflow (e.g. NEG_ONE@127 → exp 128 for E3)
    wire byp_fits_e3 = &(s2_neg_big_ei[63:7]  ^~ {57{s2_neg_big_ei[7]}});
    wire byp_fits_e4 = &(s2_neg_big_ei[63:15] ^~ {49{s2_neg_big_ei[15]}});
    wire byp_fits_e5 = &(s2_neg_big_ei[63:31] ^~ {33{s2_neg_big_ei[31]}});
    wire byp_exp_fits = (s2_exp_width == 2'd0) ? byp_fits_e3 :
                        (s2_exp_width == 2'd1) ? byp_fits_e4 :
                        (s2_exp_width == 2'd2) ? byp_fits_e5 : 1'b1;
    wire byp_e3_ambig = s2_neg_big_ei[7]  & ~|s2_neg_big_ei[6:0]   & &s2_neg_big_ei[63:8];
    wire byp_e4_ambig = s2_neg_big_ei[15] & ~|s2_neg_big_ei[14:0]  & &s2_neg_big_ei[63:16];
    wire byp_e5_ambig = s2_neg_big_ei[31] & ~|s2_neg_big_ei[30:0]  & &s2_neg_big_ei[63:32];
    wire byp_width_ambig = (s2_exp_width == 2'd0) ? byp_e3_ambig :
                           (s2_exp_width == 2'd1) ? byp_e4_ambig :
                           (s2_exp_width == 2'd2) ? byp_e5_ambig : 1'b0;
    wire bypass_sub_bad = s2_bypass_sub & (!byp_exp_fits | byp_width_ambig);

    // Result exp (LSB-aligned)
    wire signed [MAX_EXP-1:0] result_ei =
        s2_shortcut           ? s2_sc_ei :
        s2_bypass_add         ? s2_big_ei :
        s2_bypass_sub         ? (bypass_sub_bad ? AMBIG_EXP : s2_neg_big_ei) :
        s2_bw_negligible      ? s2_bw_negl_ei :
        (path_is_zero | path_underflow | path_overflow) ? AMBIG_EXP :
                                path_ei;

    always @(posedge clk) if (ce) begin
        result_frac <= s2_shortcut           ? s2_sc_frac :
                       s2_bypass_add         ? s2_big_frac :
                       s2_bypass_sub         ? s2_neg_big_frac :
                       s2_bw_negligible      ? s2_bw_negl_frac :
                       path_is_zero          ? {MAX_FRAC{1'b0}} :
                       path_underflow        ? (path_uf_frac & subprec_mask) :
                                               (path_frac & subprec_mask);

        result_exp  <= result_ei;
    end

endmodule