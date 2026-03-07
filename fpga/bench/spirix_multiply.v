// spirix_multiply — Combinational multiply for Spirix scalars
//
// Computes a * b (or -(a*b) when negate=1) on N1-normalized signed fractions.
// Fully parameterized, purely combinational.
//
// Algorithm:
//   0. Edge case detection and shortcutting (matches Rust spec).
//   1. Signed multiply: product = a_frac * b_frac (2*FRAC_BITS-1 bits).
//   2. Bounded normalization: 0 or 1 bit left shift (no barrel needed).
//      N1 * N1 always produces a leading count of 1 or 2.
//   3. Extract top FRAC_BITS, banker's round (RNE) on guard + sticky.
//   4. Early rounding-overflow detection (from pre-round signals).
//   5. Exponent: a_exp + b_exp - norm_shift +/- rovf adjustment.
//   6. Overflow/underflow clamp to AMBIGUOUS_EXP.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_multiply #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8,
    parameter USE_KARATSUBA = 0  // 1 = Karatsuba (saves LUT4 no-DSP), 0 = naive (faster with DSP)
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    input  wire                        negate,  // 1 = compute -(a*b)
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam PROD_BITS = 2 * FRAC_BITS - 1; // signed product width
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    // Undefined prefix constants (top 8 bits of fraction, zero-padded)
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_MUL_NEG = {8'hEF, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_NEG_MUL_TF = {8'h10, {UPAD{1'b0}}};

    // =========================================================================
    // Step 0a: Edge case detection (matches Rust scalar_multiply_scalar)
    // =========================================================================

    wire a_is_ambig = (a_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]);
    wire b_is_ambig = (b_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]);

    wire a_frac_zero = (a_frac == {FRAC_BITS{1'b0}});
    wire a_frac_neg1 = &a_frac;
    wire b_frac_zero = (b_frac == {FRAC_BITS{1'b0}});
    wire b_frac_neg1 = &b_frac;

    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;
    wire a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire b_n1 = (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-2]);
    wire a_n2 = ~a_n1 & (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-3]);
    wire b_n2 = ~b_n1 & (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-3]);
    wire a_top3 = (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire b_top3 = (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                  (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);

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

    wire a_is_normal  = ~a_is_ambig & a_n1;
    wire b_is_normal  = ~b_is_ambig & b_n1;
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // Edge case priority chain
    wire sc_a_undef  = a_undef;
    wire sc_b_undef  = ~a_undef & b_undef;
    wire sc_inf_zero = ~a_undef & ~b_undef & ((a_is_inf & b_is_zero) | (a_is_zero & b_is_inf));
    wire sc_any_zero = ~a_undef & ~b_undef & ~sc_inf_zero & (a_is_zero | b_is_zero);
    wire sc_exp_van  = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_zero &
                       ((a_exploded & b_vanished) | (a_vanished & b_exploded));
    wire shortcut    = sc_a_undef | sc_b_undef | sc_inf_zero | sc_any_zero | sc_exp_van;

    // Abnormal compute: run multiply normally but force exp to AMBIG
    wire abnormal_compute = any_non_normal & ~shortcut;
    wire n_level_neg1 = a_exploded | b_exploded;

    // Shortcut fraction
    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_inf_zero ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        sc_any_zero ? {FRAC_BITS{1'b0}} :
                      ((a_exploded)             ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF);

    // Shortcut exponent
    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
                      AMBIGUOUS_EXP[EXP_BITS-1:0];

    // =========================================================================
    // Step 0b: Optional negate — flip sign of a_frac before multiply
    //
    // -(a*b) = (-a)*b. Two's complement negation works for all N1 values
    // except NEG_ONE (10...0), which wraps to itself. Fix: use POS_HALF
    // (01...0) with exp+1, since -(-1.0 * 2^e) = +0.5 * 2^(e+1).
    // When negate=0, synthesis optimizes this away completely.
    // =========================================================================
    wire a_is_neg_one = negate & (a_frac == NEG_ONE);
    wire signed [FRAC_BITS-1:0] a_frac_eff = negate ? (a_is_neg_one ? POS_HALF : -a_frac)
                                                     : a_frac;
    wire signed [EXP_BITS-1:0]  a_exp_eff  = a_is_neg_one ? (a_exp + {{(EXP_BITS-1){1'b0}}, 1'b1})
                                                           : a_exp;

    // =========================================================================
    // Step 1: Signed multiply (2*FRAC_BITS-1 bits)
    //
    // USE_KARATSUBA=0: naive a*b, best with DSP (4 MULT18X18D, minimal LUT).
    // USE_KARATSUBA=1: Karatsuba decomposition, 3 sub-multiplies,
    //   ~15% LUT4 savings no-DSP but adds reconstruction overhead with DSP.
    // =========================================================================
    wire signed [PROD_BITS-1:0] product;

    generate if (USE_KARATSUBA) begin : gen_karatsuba
        localparam K = (FRAC_BITS + 1) / 2;
        localparam H = FRAC_BITS - K;

        wire signed [H-1:0] aH = a_frac_eff[FRAC_BITS-1 : K];
        wire        [K-1:0] aL = a_frac_eff[K-1 : 0];
        wire signed [H-1:0] bH = b_frac[FRAC_BITS-1 : K];
        wire        [K-1:0] bL = b_frac[K-1 : 0];

        wire signed [2*H-1:0]  phh = aH * bH;
        wire        [2*K-1:0]  pll = aL * bL;

        // aM, bM need K+2 bits: aH(H-bit signed) + aL(K-bit unsigned) can reach
        // H_max + K_max = (2^(H-1)-1) + (2^K-1) which exceeds K+1 signed range.
        wire signed [K+1:0] aM = $signed({{(K-H+2){aH[H-1]}}, aH}) + $signed({2'b0, aL});
        wire signed [K+1:0] bM = $signed({{(K-H+2){bH[H-1]}}, bH}) + $signed({2'b0, bL});
        wire signed [2*K+3:0] pmm = aM * bM;

        wire signed [2*K+3:0] cross = pmm
                                    - {{(2*K+4-2*H){phh[2*H-1]}}, phh}
                                    - {{2{1'b0}}, pll};

        wire signed [PROD_BITS:0] product_wide =
            ($signed({{(PROD_BITS+1-2*H){phh[2*H-1]}}, phh}) <<< (2*K))
          + ($signed({{(PROD_BITS-2*K-1){cross[2*K+3]}}, cross}) <<< K)
          + $signed({{(PROD_BITS+1-2*K){1'b0}}, pll});

        assign product = product_wide[PROD_BITS-1:0];
    end else begin : gen_naive
        wire signed [PROD_BITS:0] product_wide = a_frac_eff * b_frac;
        assign product = product_wide[PROD_BITS-1:0];
    end endgenerate

    // =========================================================================
    // Step 2: Bounded normalization (0 or 1 bit shift)
    //
    // N1 inputs guarantee the product's leading redundant sign bits are 1 or 2.
    // If top 2 bits differ: already N1 (shift 0).
    // If top 2 bits same: shift left 1.
    // =========================================================================
    wire is_n1 = (product[PROD_BITS-1] != product[PROD_BITS-2]);
    wire norm_shift = !is_n1;
    wire signed [PROD_BITS-1:0] normalized = norm_shift ? (product <<< 1) : product;

    // =========================================================================
    // Step 3: Extract top FRAC_BITS + banker's rounding (RNE)
    //
    // Guard bit is the first bit below the fraction. Sticky is the OR of
    // all remaining bits (combines IEEE round + sticky into one term).
    // Round up when guard=1 AND (sticky | lsb).
    // =========================================================================
    wire signed [FRAC_BITS-1:0] frac_raw = normalized[PROD_BITS-1 -: FRAC_BITS];

    wire guard  = normalized[PROD_BITS - 1 - FRAC_BITS];
    wire sticky = (PROD_BITS - 2 - FRAC_BITS >= 0) ?
                  |normalized[PROD_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire lsb    = frac_raw[0];
    wire round_up = guard & (sticky | lsb);

    wire signed [FRAC_BITS-1:0] frac_rounded = frac_raw + {{(FRAC_BITS-1){1'b0}}, round_up};

    // =========================================================================
    // Step 4: Rounding overflow — early detection from pre-round signals
    //
    // rovf_pos: 0_111...1 + round → sign flips. Fix: POS_HALF, exp+1.
    // rovf_neg: 10_111...1 + round → exits N1. Fix: NEG_ONE, exp-1.
    // =========================================================================
    wire rovf_pos = !frac_raw[FRAC_BITS-1]
                  & (&frac_raw[FRAC_BITS-2:0])
                  & round_up;
    wire rovf_neg = frac_raw[FRAC_BITS-1]
                  & !frac_raw[FRAC_BITS-2]
                  & (&frac_raw[FRAC_BITS-3:0])
                  & round_up;

    wire signed [FRAC_BITS-1:0] out_frac = rovf_pos ? POS_HALF :
                                             rovf_neg ? NEG_ONE  :
                                             frac_rounded;

    // =========================================================================
    // Step 5: Exponent
    // =========================================================================
    wire signed [EXP_BITS:0] exp_wide = $signed({a_exp_eff[EXP_BITS-1], a_exp_eff})
                                       + $signed({b_exp[EXP_BITS-1], b_exp})
                                       - {{EXP_BITS{1'b0}}, norm_shift}
                                       + {{EXP_BITS{1'b0}}, rovf_pos}
                                       - {{EXP_BITS{1'b0}}, rovf_neg};

    wire exp_too_big   = (exp_wide > MAX_EXP);
    wire exp_too_small = (exp_wide < MIN_EXP);
    wire signed [EXP_BITS-1:0] out_exp = exp_wide[EXP_BITS-1:0];

    // =========================================================================
    // Step 6: Output with edge cases, overflow/underflow clamping
    //
    // Priority: shortcut > abnormal_compute > overflow > underflow > normal.
    // Abnormal compute: result frac from multiply, exp forced to AMBIG.
    //   n_level=-2 (neither exploded): right-shift result to N2.
    // =========================================================================
    wire abnormal_n2 = abnormal_compute & ~n_level_neg1;
    wire signed [FRAC_BITS-1:0] abnormal_frac = abnormal_n2
        ? {out_frac[FRAC_BITS-1], out_frac[FRAC_BITS-1:1]} : out_frac;

    assign result_frac = shortcut          ? sc_frac :
                         abnormal_compute  ? abnormal_frac :
                         exp_too_big       ? out_frac :
                         exp_too_small     ? {out_frac[FRAC_BITS-1],
                                              out_frac[FRAC_BITS-1:1]} :
                                             out_frac;

    assign result_exp  = shortcut                           ? sc_exp :
                         (abnormal_compute |
                          exp_too_big | exp_too_small)      ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                                              out_exp;

endmodule
