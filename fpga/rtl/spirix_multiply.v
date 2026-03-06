// spirix_multiply — Combinational multiply for Spirix scalars
//
// Computes a * b (or -(a*b) when negate=1) on N1-normalized signed fractions.
// Fully parameterized, purely combinational.
//
// Algorithm:
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
    parameter EXP_BITS  = 8
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

    // =========================================================================
    // Step 0: Optional negate — flip sign of a_frac before multiply
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
    // Step 1: Karatsuba signed multiply (2*FRAC_BITS-1 bits)
    //
    // Split inputs at K = ceil(FRAC_BITS/2):
    //   a = aH * 2^K + aL,  b = bH * 2^K + bL
    //   product = P_HH * 2^(2K) + (P_MM - P_HH - P_LL) * 2^K + P_LL
    // Three sub-multiplies instead of one: ~15% LUT4 savings no-DSP.
    // With DSP: maps to 3 MULT18X18D (was 4). Bit-exact.
    // =========================================================================
    localparam K = (FRAC_BITS + 1) / 2;   // 13 for FRAC_BITS=25
    localparam H = FRAC_BITS - K;          // 12 for FRAC_BITS=25

    wire signed [H-1:0] aH = a_frac_eff[FRAC_BITS-1 : K];
    wire        [K-1:0] aL = a_frac_eff[K-1 : 0];
    wire signed [H-1:0] bH = b_frac[FRAC_BITS-1 : K];
    wire        [K-1:0] bL = b_frac[K-1 : 0];

    wire signed [2*H-1:0]  phh = aH * bH;          // H×H signed
    wire        [2*K-1:0]  pll = aL * bL;           // K×K unsigned

    wire signed [K:0] aM = $signed({{(K-H+1){aH[H-1]}}, aH}) + $signed({1'b0, aL});
    wire signed [K:0] bM = $signed({{(K-H+1){bH[H-1]}}, bH}) + $signed({1'b0, bL});
    wire signed [2*K+1:0] pmm = aM * bM;            // (K+1)×(K+1) signed

    wire signed [2*K+1:0] cross = pmm
                                - {{(2*K+2-2*H){phh[2*H-1]}}, phh}
                                - {2'b0, pll};

    wire signed [PROD_BITS:0] product_wide =
        ($signed({{(PROD_BITS+1-2*H){phh[2*H-1]}}, phh}) <<< (2*K))
      + ($signed({{(PROD_BITS-2*K-1){cross[2*K+1]}}, cross}) <<< K)
      + $signed({{(PROD_BITS+1-2*K){1'b0}}, pll});

    wire signed [PROD_BITS-1:0] product = product_wide[PROD_BITS-1:0];

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
    // Step 6: Output with overflow/underflow clamping
    //
    // Overflow: preserve fraction (sign indicates direction), AMBIGUOUS_EXP.
    // Underflow: truncate fraction (sign + MSBs), AMBIGUOUS_EXP.
    // =========================================================================
    assign result_frac = exp_too_big   ? out_frac :
                         exp_too_small ? {out_frac[FRAC_BITS-1],
                                          out_frac[FRAC_BITS-1:1]} :
                                         out_frac;

    assign result_exp  = (exp_too_big | exp_too_small) ?
                          AMBIGUOUS_EXP[EXP_BITS-1:0] : out_exp;

endmodule
