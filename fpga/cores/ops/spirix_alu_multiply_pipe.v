// spirix_alu_multiply_pipe — 2-stage pipelined multiply for multi-width Spirix scalars
//
// Floor-only (no rounding), matches Rust scalar_multiply_scalar exactly.
//
// S1 (Multiply + Detect):
//   - State classification (undef/zero/inf/exploded/vanished)
//   - Edge case shortcut chain
//   - 4 Karatsuba sub-multiplies (P_HH, cross_a, cross_b, P_LL)
//   - Pre-sum exponents: a_exp + b_exp + 1
//   - Register raw products + edge case signals
//
// S2 (Reconstruct + Normalize + Output):
//   - Karatsuba reconstruction (cross_sum → lo_add → prod_hi/lo)
//   - Normal path: 1-bit normalize (no CLZ)
//   - Abnormal path: 5-way normalize
//   - Exponent finish: pre_exp - lm1
//   - Output mux
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (sa() convention). Exponents LSB-aligned.
// Full edge case handling: undef, inf×zero, zero, exploded×vanished, abnormal.

module spirix_alu_multiply_pipe #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                       clk,
    input  wire                       ce,
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output reg  signed [MAX_FRAC-1:0] result_frac,
    output reg  signed [MAX_EXP-1:0]  result_exp
);

    localparam PROD_BITS  = 2 * MAX_FRAC;  // 128
    localparam signed [MAX_EXP-1:0] AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // Undefined prefix constants (top 8 bits, zero-padded to MAX_FRAC)
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_MUL_NEG = {8'hEF, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_NEG_MUL_TF = {8'h10, {(MAX_FRAC-8){1'b0}}};

    // ================================================================
    //  WIDTH-DEPENDENT CONSTANTS
    // ================================================================
    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                                {64'hFFFFFFFFFFFFFFFF};

    wire signed [MAX_EXP:0] exp_max =
        (exp_width == 2'd0) ? 8'sd127 :
        (exp_width == 2'd1) ? 16'sd32767 :
        (exp_width == 2'd2) ? 32'sd2147483647 :
                               {1'b0, {(MAX_EXP-1){1'b1}}};
    wire signed [MAX_EXP:0] exp_min = -exp_max;

    wire [MAX_FRAC-1:0] frac_mask =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                                {64'hFFFFFFFFFFFFFFFF};

    // ================================================================
    //  STATE DETECTION (S1 combinational)
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);

    wire a_frac_zero = (a_frac == {MAX_FRAC{1'b0}});
    wire a_frac_neg1 = (a_frac == frac_neg1_val);
    wire b_frac_zero = (b_frac == {MAX_FRAC{1'b0}});
    wire b_frac_neg1 = (b_frac == frac_neg1_val);

    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;
    wire a_n1 = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire b_n1 = (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-2]);
    wire a_n2 = ~a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire b_n2 = ~b_n1 & (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-3]);
    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire b_top3 = (b_frac[MAX_FRAC-1] == b_frac[MAX_FRAC-2]) &
                  (b_frac[MAX_FRAC-2] == b_frac[MAX_FRAC-3]);

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
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // ================================================================
    //  EDGE CASE PRIORITY (matches Rust scalar_multiply_scalar)
    // ================================================================
    wire sc_a_undef  = a_undef;
    wire sc_b_undef  = ~a_undef & b_undef;
    wire sc_inf_zero = ~a_undef & ~b_undef & ((a_is_inf & b_is_zero) | (a_is_zero & b_is_inf));
    wire sc_any_inf  = ~a_undef & ~b_undef & ~sc_inf_zero & (a_is_inf | b_is_inf);
    wire sc_any_zero = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_inf & (a_is_zero | b_is_zero);
    wire sc_exp_van  = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_inf & ~sc_any_zero &
                       ((a_exploded & b_vanished) | (a_vanished & b_exploded));
    wire shortcut    = sc_a_undef | sc_b_undef | sc_inf_zero | sc_any_inf | sc_any_zero | sc_exp_van;

    wire abnormal_compute = any_non_normal & ~shortcut;
    wire n_level_neg1 = a_exploded | b_exploded;

    // Shortcut output (S1 combinational)
    wire signed [MAX_FRAC-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_inf_zero ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        sc_any_inf  ? frac_neg1_val :
        sc_any_zero ? {MAX_FRAC{1'b0}} :
                      ((a_exploded)             ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF);

    wire signed [MAX_EXP-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
                      AMBIG_EXP;

    // ================================================================
    //  MULTIPLY: 64s × 64s → 128-bit (Karatsuba split at 32)
    // ================================================================
    localparam HALF = MAX_FRAC / 2;  // 32

    wire signed [HALF-1:0] aH = a_frac[MAX_FRAC-1:HALF];
    wire        [HALF-1:0] aL = a_frac[HALF-1:0];
    wire signed [HALF-1:0] bH = b_frac[MAX_FRAC-1:HALF];
    wire        [HALF-1:0] bL = b_frac[HALF-1:0];

    // P_HH: signed 32×32 → 64
    wire signed [MAX_FRAC-1:0] P_HH = $signed({{HALF{aH[HALF-1]}}, aH})
                                     * $signed({{HALF{bH[HALF-1]}}, bH});

    // Cross terms: 33s × 33s → 66-bit
    wire signed [HALF:0] aH_s = {aH[HALF-1], aH};
    wire signed [HALF:0] bH_s = {bH[HALF-1], bH};
    wire signed [HALF:0] aL_s = {1'b0, aL};
    wire signed [HALF:0] bL_s = {1'b0, bL};

    wire signed [2*HALF+1:0] cross_a_raw = aH_s * bL_s;
    wire signed [MAX_FRAC-1:0] cross_a = cross_a_raw[MAX_FRAC-1:0];

    wire signed [2*HALF+1:0] cross_b_raw = aL_s * bH_s;
    wire signed [MAX_FRAC-1:0] cross_b = cross_b_raw[MAX_FRAC-1:0];

    // P_LL: 32u × 32u → 64u
    wire [2*HALF-1:0] P_LL = aL * bL;

    // Pre-sum exponents: a_exp + b_exp + 1 (S2 only subtracts lm1)
    wire signed [MAX_EXP:0] exp_pre = {a_exp[MAX_EXP-1], a_exp}
                                     + {b_exp[MAX_EXP-1], b_exp}
                                     + {{MAX_EXP{1'b0}}, 1'b1};

    // ================================================================
    //  S1 PIPELINE REGISTER
    // ================================================================
    reg signed [MAX_FRAC-1:0] s1_P_HH;
    reg signed [MAX_FRAC-1:0] s1_cross_a, s1_cross_b;
    reg        [2*HALF-1:0]   s1_P_LL;

    reg s1_shortcut, s1_abnormal_compute, s1_n_level_neg1;
    reg signed [MAX_FRAC-1:0] s1_sc_frac;
    reg signed [MAX_EXP-1:0]  s1_sc_exp;

    reg signed [MAX_EXP:0]    s1_exp_pre;
    reg [MAX_FRAC-1:0]        s1_frac_mask;
    reg signed [MAX_EXP:0]    s1_exp_max, s1_exp_min;

    always @(posedge clk) if (ce) begin
        s1_P_HH    <= P_HH;
        s1_cross_a <= cross_a;
        s1_cross_b <= cross_b;
        s1_P_LL    <= P_LL;

        s1_shortcut         <= shortcut;
        s1_abnormal_compute <= abnormal_compute;
        s1_n_level_neg1     <= n_level_neg1;
        s1_sc_frac          <= sc_frac;
        s1_sc_exp           <= sc_exp;

        s1_exp_pre      <= exp_pre;
        s1_frac_mask    <= frac_mask;
        s1_exp_max      <= exp_max;
        s1_exp_min      <= exp_min;
    end

    // ================================================================
    //  S2: KARATSUBA RECONSTRUCTION (from registered products)
    // ================================================================
    wire signed [MAX_FRAC:0] s2_cross_sum = {s1_cross_a[MAX_FRAC-1], s1_cross_a}
                                           + {s1_cross_b[MAX_FRAC-1], s1_cross_b};

    wire [HALF:0] s2_lo_add = {1'b0, s2_cross_sum[HALF-1:0]} + {1'b0, s1_P_LL[MAX_FRAC-1:HALF]};

    wire signed [MAX_FRAC-1:0] s2_prod_hi = s1_P_HH
        + {{(HALF-1){s2_cross_sum[MAX_FRAC]}}, s2_cross_sum[MAX_FRAC:HALF]}
        + {{(MAX_FRAC-1){1'b0}}, s2_lo_add[HALF]};

    wire [MAX_FRAC-1:0] s2_prod_lo = {s2_lo_add[HALF-1:0], s1_P_LL[HALF-1:0]};

    // ================================================================
    //  S2: NORMAL PATH — 1-bit normalize (no CLZ)
    // ================================================================
    // N1×N1 product can never be zero (magnitudes in (0.25,1)),
    // so no prod_is_zero check needed — shortcut catches actual zeros.
    wire [1:0] s2_norm_lm1 = (s2_prod_hi[MAX_FRAC-1] != s2_prod_hi[MAX_FRAC-2]) ? 2'd0 :
                              (s2_prod_hi[MAX_FRAC-2] != s2_prod_hi[MAX_FRAC-3]) ? 2'd1 : 2'd2;

    wire signed [MAX_FRAC-1:0] s2_norm_frac_raw =
        (s2_norm_lm1 == 2'd0) ? s2_prod_hi :
        (s2_norm_lm1 == 2'd1) ? {s2_prod_hi[MAX_FRAC-2:0], s2_prod_lo[MAX_FRAC-1]} :
                                 {s2_prod_hi[MAX_FRAC-3:0], s2_prod_lo[MAX_FRAC-1:MAX_FRAC-2]};
    wire signed [MAX_FRAC-1:0] s2_norm_frac = s2_norm_frac_raw & s1_frac_mask;

    // Normal exponent: pre_exp - lm1 (pre_exp = a_exp + b_exp + 1, computed in S1)
    wire signed [MAX_EXP:0] s2_exp_wide = s1_exp_pre
        - {{(MAX_EXP-1){1'b0}}, s2_norm_lm1};

    wire signed [MAX_EXP-1:0] s2_norm_exp = s2_exp_wide[MAX_EXP-1:0];
    wire s2_overflow  = (s2_exp_wide > $signed(s1_exp_max));
    wire s2_underflow = (s2_exp_wide < $signed(s1_exp_min));

    // Underflow: arithmetic right shift by 1 (vanish the fraction)
    wire signed [MAX_FRAC-1:0] s2_vanished_frac = {s2_norm_frac[MAX_FRAC-1],
                                                     s2_norm_frac[MAX_FRAC-1:1]} & s1_frac_mask;

    // ================================================================
    //  S2: ABNORMAL PATH — 5-way normalize (no CLZ, no barrel)
    // ================================================================
    wire [2:0] s2_abn_lm1 =
        (s2_prod_hi[MAX_FRAC-1] != s2_prod_hi[MAX_FRAC-2]) ? 3'd0 :
        (s2_prod_hi[MAX_FRAC-2] != s2_prod_hi[MAX_FRAC-3]) ? 3'd1 :
        (s2_prod_hi[MAX_FRAC-3] != s2_prod_hi[MAX_FRAC-4]) ? 3'd2 :
        (s2_prod_hi[MAX_FRAC-4] != s2_prod_hi[MAX_FRAC-5]) ? 3'd3 : 3'd4;

    wire s2_abnormal_n2 = s1_abnormal_compute & ~s1_n_level_neg1;
    wire [2:0] s2_abn_shift = (s2_abnormal_n2 & |s2_abn_lm1) ? s2_abn_lm1 - 3'd1 : s2_abn_lm1;

    wire signed [MAX_FRAC-1:0] s2_abn_frac_raw =
        (s2_abn_shift == 3'd0) ? s2_prod_hi :
        (s2_abn_shift == 3'd1) ? {s2_prod_hi[MAX_FRAC-2:0], s2_prod_lo[MAX_FRAC-1]} :
        (s2_abn_shift == 3'd2) ? {s2_prod_hi[MAX_FRAC-3:0], s2_prod_lo[MAX_FRAC-1:MAX_FRAC-2]} :
        (s2_abn_shift == 3'd3) ? {s2_prod_hi[MAX_FRAC-4:0], s2_prod_lo[MAX_FRAC-1:MAX_FRAC-3]} :
                                  {s2_prod_hi[MAX_FRAC-5:0], s2_prod_lo[MAX_FRAC-1:MAX_FRAC-4]};
    wire signed [MAX_FRAC-1:0] s2_abn_frac = s2_abn_frac_raw & s1_frac_mask;

    // ================================================================
    //  S2: OUTPUT MUX + REGISTER
    // ================================================================
    wire signed [MAX_FRAC-1:0] s2_out_frac =
        s1_shortcut         ? s1_sc_frac :
        s1_abnormal_compute ? s2_abn_frac :
        s2_overflow         ? s2_norm_frac :
        s2_underflow        ? s2_vanished_frac :
                              s2_norm_frac;

    wire signed [MAX_EXP-1:0] s2_out_exp =
        s1_shortcut         ? s1_sc_exp :
        s1_abnormal_compute ? AMBIG_EXP :
        (s2_overflow | s2_underflow) ? AMBIG_EXP :
                              s2_norm_exp;

    always @(posedge clk) if (ce) begin
        result_frac <= s2_out_frac;
        result_exp  <= s2_out_exp;
    end

endmodule
