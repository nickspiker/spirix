// spirix_alu_multiply — combinational multiply for multi-width Spirix scalars
//
// Floor-only (no rounding), matches Rust scalar_multiply_scalar exactly.
// Architecture: state detect → multiply → 1-bit normalize (normal) or
//               64-bit CLZ+barrel (abnormal) → extract (floor).
//
// Key optimization: MSB-aligned N1 fractions always produce products with
// leading_m1 ∈ {1, 2} on the 128-bit result. Normal path uses a single-bit
// check + 2:1 mux (no CLZ, no barrel). Full 64-bit CLZ only on abnormal path.
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (sa() convention). Exponents LSB-aligned.
// Full edge case handling: undef, inf×zero, zero, exploded×vanished, abnormal.

module spirix_alu_multiply #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output wire signed [MAX_FRAC-1:0] result_frac,
    output wire signed [MAX_EXP-1:0]  result_exp
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

    // Mask to active width (MSB-aligned: keep top frac_bits, zero the rest)
    wire [MAX_FRAC-1:0] frac_mask =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                                {64'hFFFFFFFFFFFFFFFF};

    // ================================================================
    //  STATE DETECTION
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

    // ================================================================
    //  MULTIPLY: 64s × 64s → 128-bit (Karatsuba split at 32)
    // ================================================================
    // For F≤32 (frac_width<3), aL=bL=0 so only P_HH contributes.
    // For F=64, cross terms and P_LL reconstruct the full product_hi/lo.
    // Yosys maps all multiplies to 16 MULT18X18D (4 per 32×32 sub-product).
    localparam HALF = MAX_FRAC / 2;  // 32

    wire signed [HALF-1:0] aH = a_frac[MAX_FRAC-1:HALF];
    wire        [HALF-1:0] aL = a_frac[HALF-1:0];
    wire signed [HALF-1:0] bH = b_frac[MAX_FRAC-1:HALF];
    wire        [HALF-1:0] bL = b_frac[HALF-1:0];

    // P_HH: signed 32×32 → 64 (4 DSP18)
    wire signed [MAX_FRAC-1:0] P_HH = $signed({{HALF{aH[HALF-1]}}, aH})
                                     * $signed({{HALF{bH[HALF-1]}}, bH});

    // Cross terms: 33s × 33s → 66-bit (4 DSP18 each)
    wire signed [HALF:0] aH_s = {aH[HALF-1], aH};
    wire signed [HALF:0] bH_s = {bH[HALF-1], bH};
    wire signed [HALF:0] aL_s = {1'b0, aL};
    wire signed [HALF:0] bL_s = {1'b0, bL};

    wire signed [2*HALF+1:0] cross_a_raw = aH_s * bL_s;
    wire signed [MAX_FRAC-1:0] cross_a = cross_a_raw[MAX_FRAC-1:0];

    wire signed [2*HALF+1:0] cross_b_raw = aL_s * bH_s;
    wire signed [MAX_FRAC-1:0] cross_b = cross_b_raw[MAX_FRAC-1:0];

    // P_LL: 32u × 32u → 64u (4 DSP18)
    wire [2*HALF-1:0] P_LL = aL * bL;

    // Reconstruct: product = P_HH×2^64 + (cross_a + cross_b)×2^32 + P_LL
    wire signed [MAX_FRAC:0] cross_sum = {cross_a[MAX_FRAC-1], cross_a}
                                        + {cross_b[MAX_FRAC-1], cross_b};

    wire [HALF:0] lo_add = {1'b0, cross_sum[HALF-1:0]} + {1'b0, P_LL[MAX_FRAC-1:HALF]};

    wire signed [MAX_FRAC-1:0] prod_hi = P_HH
        + {{(HALF-1){cross_sum[MAX_FRAC]}}, cross_sum[MAX_FRAC:HALF]}
        + {{(MAX_FRAC-1){1'b0}}, lo_add[HALF]};

    wire [MAX_FRAC-1:0] prod_lo = {lo_add[HALF-1:0], P_LL[HALF-1:0]};

    wire prod_is_zero = (prod_hi == {MAX_FRAC{1'b0}}) & (prod_lo == {MAX_FRAC{1'b0}});

    // ================================================================
    //  NORMAL PATH: 1-bit normalize (no CLZ)
    // ================================================================
    // For N1×N1 MSB-aligned, leading_m1 on 128-bit product is 0, 1, or 2.
    //   lm1=0: prod_hi[63]!=prod_hi[62] (product magnitude ≥ 2^126)
    //   lm1=1: [63]==[62], [62]!=[61] (product in [2^125, 2^126))
    //   lm1=2: [63]==[62]==[61], [61]!=[60] (product in [2^124, 2^125))
    wire [1:0] norm_lm1 = (prod_hi[MAX_FRAC-1] != prod_hi[MAX_FRAC-2]) ? 2'd0 :
                           (prod_hi[MAX_FRAC-2] != prod_hi[MAX_FRAC-3]) ? 2'd1 : 2'd2;

    wire signed [MAX_FRAC-1:0] norm_frac_raw =
        (norm_lm1 == 2'd0) ? prod_hi :
        (norm_lm1 == 2'd1) ? {prod_hi[MAX_FRAC-2:0], prod_lo[MAX_FRAC-1]} :
                              {prod_hi[MAX_FRAC-3:0], prod_lo[MAX_FRAC-1:MAX_FRAC-2]};
    wire signed [MAX_FRAC-1:0] norm_frac = norm_frac_raw & frac_mask;

    // Normal exponent: exp = a_exp + b_exp - lm1 + 1
    wire signed [MAX_EXP:0] exp_wide =
        {a_exp[MAX_EXP-1], a_exp}
        + {b_exp[MAX_EXP-1], b_exp}
        + {{MAX_EXP{1'b0}}, 1'b1}
        - {{(MAX_EXP-1){1'b0}}, norm_lm1};

    wire signed [MAX_EXP-1:0] norm_exp = exp_wide[MAX_EXP-1:0];
    wire overflow  = (exp_wide > $signed(exp_max));
    wire underflow = (exp_wide < $signed(exp_min));

    // Underflow: arithmetic right shift by 1 (vanish the fraction)
    wire signed [MAX_FRAC-1:0] vanished_frac = {norm_frac[MAX_FRAC-1],
                                                  norm_frac[MAX_FRAC-1:1]} & frac_mask;

    // ================================================================
    //  ABNORMAL PATH: 5-way normalize (no CLZ, no barrel)
    // ================================================================
    // All abnormal results get AMBIG exponent — only fraction matters.
    // With infinity shortcut, remaining abnormal cases have bounded lm1:
    //   exploded×normal or exploded×exploded: lm1 = 0..1  (N1×N1)
    //   vanished×normal:  lm1 = 2..3  (N2×N1)
    //   vanished×vanished: lm1 = 3..4  (N2×N2)
    // n_level adjustment: n_level=-1 (exploded present) subtracts 0,
    //                     n_level=-2 (vanished only) subtracts 1 from lm1.
    // Net shift range: 0..4 bits on prod_hi, with fill from prod_lo MSBs.

    wire [2:0] abn_lm1 =
        (prod_hi[MAX_FRAC-1] != prod_hi[MAX_FRAC-2]) ? 3'd0 :
        (prod_hi[MAX_FRAC-2] != prod_hi[MAX_FRAC-3]) ? 3'd1 :
        (prod_hi[MAX_FRAC-3] != prod_hi[MAX_FRAC-4]) ? 3'd2 :
        (prod_hi[MAX_FRAC-4] != prod_hi[MAX_FRAC-5]) ? 3'd3 : 3'd4;

    // n_level=-1 (exploded present): shift = lm1
    // n_level=-2 (vanished only):    shift = lm1 - 1 (but min 0 when lm1=0)
    wire abnormal_n2 = abnormal_compute & ~n_level_neg1;
    wire [2:0] abn_shift = (abnormal_n2 & |abn_lm1) ? abn_lm1 - 3'd1 : abn_lm1;

    // 5-position mux (0..4 bit left shift on {prod_hi, prod_lo[63:60]})
    wire signed [MAX_FRAC-1:0] abn_frac_raw =
        (abn_shift == 3'd0) ? prod_hi :
        (abn_shift == 3'd1) ? {prod_hi[MAX_FRAC-2:0], prod_lo[MAX_FRAC-1]} :
        (abn_shift == 3'd2) ? {prod_hi[MAX_FRAC-3:0], prod_lo[MAX_FRAC-1:MAX_FRAC-2]} :
        (abn_shift == 3'd3) ? {prod_hi[MAX_FRAC-4:0], prod_lo[MAX_FRAC-1:MAX_FRAC-3]} :
                               {prod_hi[MAX_FRAC-5:0], prod_lo[MAX_FRAC-1:MAX_FRAC-4]};
    wire signed [MAX_FRAC-1:0] abn_frac = abn_frac_raw & frac_mask;

    // ================================================================
    //  SHORTCUT OUTPUT
    // ================================================================
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
    //  OUTPUT MUX
    // ================================================================
    assign result_frac = shortcut          ? sc_frac :
                         abnormal_compute  ? abn_frac :
                         prod_is_zero      ? {MAX_FRAC{1'b0}} :
                         overflow          ? norm_frac :
                         underflow         ? vanished_frac :
                                             norm_frac;

    assign result_exp  = shortcut                          ? sc_exp :
                         (abnormal_compute | prod_is_zero) ? AMBIG_EXP :
                         (overflow | underflow)            ? AMBIG_EXP :
                                                             norm_exp;

endmodule
