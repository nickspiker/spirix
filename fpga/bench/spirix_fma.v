// spirix_fma — Combinational fused multiply-add for Spirix scalars
//
// Computes a * b + c (sub=0) or a * b - c (sub=1) with single rounding.
// Purely combinational. Uses DSP for the multiply.
//
// Architecture: pre-aligned FMA. The exponent difference and C alignment
// are computed from the raw product exponent (a_exp + b_exp) IN PARALLEL with the DSP multiply. The DSP result feeds directly into the close/far
// addsub without intermediate normalization.
//
// The product is N1 or N2 (N1*N1 produces at most 1 redundant sign bit).
// The close-path threshold is widened to |raw_diff| <= 2 (was 1) to cover the 1-bit exponent uncertainty. The far-path bounded normalize is
// widened to 0-3 bits (was 0-2) to absorb the extra redundant bit.
//
// Critical path improvement:
//   Old: DSP -> normalize -> exp_diff -> barrel -> add -> round
//   New: DSP -> add -> normalize(0-3) -> round  (barrel runs || with DSP)
//
// Single rounding at the output. No intermediate normalize+round.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. Minimum exponent = AMBIGUOUS_EXP.
//
// Valid parameter range: FRAC_BITS 4..29, EXP_BITS 4..16.

module spirix_fma #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    input  wire signed [FRAC_BITS-1:0] c_frac,
    input  wire signed [EXP_BITS-1:0]  c_exp,
    input  wire                         sub,      // 1 = a*b - c
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam PROD_BITS = 2 * FRAC_BITS - 1;
    localparam INT_BITS = PROD_BITS + 3;
    localparam BARREL_BITS = $clog2(INT_BITS);
    localparam LEAD_BITS   = BARREL_BITS + 1;
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_SMALL = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_SMALL = {2'b11, {(FRAC_BITS-2){1'b0}}};

    // Undefined prefix constants — match src/core/undefined.rs.
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    // Multiply prefixes
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_MUL_NEG = {8'hE6, {UPAD{1'b0}}}; // -0x1A
    localparam signed [FRAC_BITS-1:0] UNDEF_NEG_MUL_TF = {8'h19, {UPAD{1'b0}}}; //  0x19
    // Add prefixes
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_TF   = {8'h1A, {UPAD{1'b0}}}; //  0x1A TRANSFINITE_PLUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_TF   = {8'hE5, {UPAD{1'b0}}}; // -0x1B TRANSFINITE_MINUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_P_VAN = {8'h1D, {UPAD{1'b0}}}; //  0x1D VANISHED_PLUS_VANISHED
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_M_VAN = {8'hE2, {UPAD{1'b0}}}; // -0x1E VANISHED_MINUS_VANISHED
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_FIN  = {8'h1C, {UPAD{1'b0}}}; //  0x1C TRANSFINITE_PLUS_FINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_FIN  = {8'hE3, {UPAD{1'b0}}}; // -0x1D TRANSFINITE_MINUS_FINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_P_TF  = {8'h1B, {UPAD{1'b0}}}; //  0x1B FINITE_PLUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_M_TF  = {8'hE4, {UPAD{1'b0}}}; // -0x1C FINITE_MINUS_TRANSFINITE

    // =========================================================================
    // Edge case detection — Multiply (a*b) then Add (product + c)
    //
    // FMA decomposes into multiply edge cases for a*b, producing an effective
    // product, then add edge cases between the effective product and c.
    // The fused datapath only runs when all three inputs are normal.
    // =========================================================================

    // --- Multiply input classification ---
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
    wire mul_any_non_normal = ~a_is_normal | ~b_is_normal;

    // Multiply edge case priority chain
    wire mul_sc_a_undef  = a_undef;
    wire mul_sc_b_undef  = ~a_undef & b_undef;
    wire mul_sc_inf_zero = ~a_undef & ~b_undef &
                           ((a_is_inf & b_is_zero) | (a_is_zero & b_is_inf));
    wire mul_sc_any_zero = ~a_undef & ~b_undef & ~mul_sc_inf_zero &
                           (a_is_zero | b_is_zero);
    wire mul_sc_exp_van  = ~a_undef & ~b_undef & ~mul_sc_inf_zero & ~mul_sc_any_zero &
                           ((a_exploded & b_vanished) | (a_vanished & b_exploded));
    wire mul_shortcut = mul_sc_a_undef | mul_sc_b_undef | mul_sc_inf_zero |
                        mul_sc_any_zero | mul_sc_exp_van;

    wire mul_abnormal = mul_any_non_normal & ~mul_shortcut;
    wire mul_n_level_neg1 = a_exploded | b_exploded;

    // Effective product classification (for add edge case chain)
    wire eff_prod_undef = mul_sc_a_undef | mul_sc_b_undef | mul_sc_inf_zero | mul_sc_exp_van;
    wire eff_prod_zero  = mul_sc_any_zero;
    wire eff_prod_transf = mul_abnormal & mul_n_level_neg1;  // AMBIG + N1 = exploded
    wire eff_prod_vanished = mul_abnormal & ~mul_n_level_neg1; // AMBIG + N2
    wire eff_prod_normal = a_is_normal & b_is_normal;

    // Effective product fraction/exponent (for passthrough)
    wire signed [FRAC_BITS-1:0] mul_sc_frac =
        mul_sc_a_undef  ? a_frac :
        mul_sc_b_undef  ? b_frac :
        mul_sc_inf_zero ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        mul_sc_any_zero ? {FRAC_BITS{1'b0}} :
                          ((a_exploded)             ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF);

    wire signed [EXP_BITS-1:0] mul_sc_exp =
        mul_sc_a_undef  ? a_exp :
        mul_sc_b_undef  ? b_exp :
                          AMBIGUOUS_EXP[EXP_BITS-1:0];

    // --- C classification ---
    wire c_is_ambig = (c_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]);

    wire c_frac_zero = (c_frac == {FRAC_BITS{1'b0}});
    wire c_frac_neg1 = &c_frac;
    wire c_n0 = c_frac_zero | c_frac_neg1;
    wire c_n1 = (c_frac[FRAC_BITS-1] != c_frac[FRAC_BITS-2]);
    wire c_n2 = ~c_n1 & (c_frac[FRAC_BITS-1] != c_frac[FRAC_BITS-3]);
    wire c_top3 = (c_frac[FRAC_BITS-1] == c_frac[FRAC_BITS-2]) &
                  (c_frac[FRAC_BITS-2] == c_frac[FRAC_BITS-3]);

    wire c_is_zero   = c_is_ambig & c_frac_zero;
    wire c_is_inf    = c_is_ambig & c_frac_neg1;
    wire c_exploded  = c_is_ambig & c_n1;
    wire c_transf    = c_is_inf | c_exploded;
    wire c_vanished  = c_n2;
    wire c_undef     = ~c_n0 & c_top3;

    // Spirix negation of c (for sub edge cases)
    wire c_is_pos_half  = (c_frac == POS_HALF);
    wire c_is_neg_one_f = (c_frac == NEG_ONE);
    wire c_is_pos_small = (c_frac == POS_SMALL);
    wire c_is_neg_small = (c_frac == NEG_SMALL);

    wire signed [EXP_BITS-1:0] c_exp_m1 = c_exp - 1'b1;
    wire c_exp_m1_ambig = (c_exp_m1 == AMBIGUOUS_EXP[EXP_BITS-1:0]);

    wire signed [FRAC_BITS-1:0] neg_c_frac_normal =
        c_is_pos_half  ? (c_exp_m1_ambig ? NEG_SMALL : NEG_ONE) :
        c_is_neg_one_f ? POS_HALF :
                         -c_frac;
    wire signed [EXP_BITS-1:0] neg_c_exp_normal =
        c_is_pos_half  ? c_exp_m1 :
        c_is_neg_one_f ? (c_exp + 1'b1) :
                         c_exp;

    wire c_top3_same = (c_frac[FRAC_BITS-1] == c_frac[FRAC_BITS-2]) &
                       (c_frac[FRAC_BITS-2] == c_frac[FRAC_BITS-3]);
    wire c_nonnorm_nochange = (c_frac_zero | c_frac_neg1) |
                              (~(c_frac_zero | c_frac_neg1) & c_top3_same);

    wire signed [FRAC_BITS-1:0] neg_c_frac_nonnorm =
        c_nonnorm_nochange ? c_frac :
        c_is_pos_half      ? NEG_ONE :
        c_is_neg_one_f     ? POS_HALF :
        c_is_pos_small     ? NEG_SMALL :
        c_is_neg_small     ? POS_SMALL :
                             -c_frac;

    wire signed [FRAC_BITS-1:0] neg_c_frac = c_is_ambig ? neg_c_frac_nonnorm : neg_c_frac_normal;
    wire signed [EXP_BITS-1:0]  neg_c_exp  = c_is_ambig ? c_exp               : neg_c_exp_normal;

    // --- Add edge case chain (effective product + c) ---
    // "p" = effective product (a*b), "c" = addend
    // For sub=1: product - c, so c is the one being subtracted (negated)

    // p_undef check: product is undefined → passthrough product
    wire add_sc_p_undef = eff_prod_undef;

    // c_undef check: c is undefined → passthrough c
    wire add_sc_c_undef = ~eff_prod_undef & c_undef;

    // transfinite+transfinite
    wire p_transf = eff_prod_transf;
    wire add_sc_tf_tf = ~eff_prod_undef & ~c_undef & p_transf & c_transf;

    // vanished+vanished
    wire p_vanished = eff_prod_vanished;
    wire add_sc_van_van = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf &
                          p_vanished & c_vanished;

    // product transfinite (only)
    wire add_sc_p_transf = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                           p_transf;

    // c transfinite (only)
    wire add_sc_c_transf = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                           ~add_sc_p_transf & c_transf;

    // product vanished → return c (or -c)
    wire add_sc_p_van = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                        ~add_sc_p_transf & ~add_sc_c_transf & p_vanished;

    // c vanished → return product (only when product is non-normal;
    // normal product + vanished c is handled by the FMA datapath)
    wire add_sc_c_van = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                        ~add_sc_p_transf & ~add_sc_c_transf & ~add_sc_p_van &
                        ~eff_prod_normal & c_vanished;

    // product zero → return c (or -c)
    wire add_sc_p_zero = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                         ~add_sc_p_transf & ~add_sc_c_transf & ~add_sc_p_van & ~add_sc_c_van &
                         eff_prod_zero;

    // c zero → return product (only when product is non-normal;
    // normal product + zero c is handled by the FMA datapath)
    wire add_sc_c_zero = ~eff_prod_undef & ~c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                         ~add_sc_p_transf & ~add_sc_c_transf & ~add_sc_p_van & ~add_sc_c_van &
                         ~add_sc_p_zero & ~eff_prod_normal & c_is_zero;

    wire c_is_normal  = ~c_is_ambig & c_n1;

    // Fallback: any remaining non-normal → return product
    // Only fires when product is non-normal; normal product with weird c
    // goes through the FMA datapath (c contributes ~0 or the math works out).
    wire add_sc_fallback = ~eff_prod_normal & (mul_any_non_normal | ~c_is_normal) &
                           ~add_sc_p_undef & ~add_sc_c_undef & ~add_sc_tf_tf & ~add_sc_van_van &
                           ~add_sc_p_transf & ~add_sc_c_transf & ~add_sc_p_van & ~add_sc_c_van &
                           ~add_sc_p_zero & ~add_sc_c_zero;

    wire fma_shortcut = add_sc_p_undef | add_sc_c_undef | add_sc_tf_tf | add_sc_van_van |
                        add_sc_p_transf | add_sc_c_transf | add_sc_p_van | add_sc_c_van |
                        add_sc_p_zero | add_sc_c_zero | add_sc_fallback;

    // FMA shortcut fraction
    wire signed [FRAC_BITS-1:0] fma_sc_frac =
        add_sc_p_undef  ? mul_sc_frac :
        add_sc_c_undef  ? c_frac :
        add_sc_tf_tf    ? (sub ? UNDEF_TF_M_TF  : UNDEF_TF_P_TF) :
        add_sc_van_van  ? (sub ? UNDEF_VAN_M_VAN : UNDEF_VAN_P_VAN) :
        add_sc_p_transf ? (sub ? UNDEF_TF_M_FIN  : UNDEF_TF_P_FIN) :
        add_sc_c_transf ? (sub ? UNDEF_FIN_M_TF  : UNDEF_FIN_P_TF) :
        add_sc_p_van    ? (sub ? neg_c_frac : c_frac) :
        add_sc_c_van    ? mul_sc_frac :  // for abnormal product
        add_sc_p_zero   ? (sub ? neg_c_frac : c_frac) :
        add_sc_c_zero   ? mul_sc_frac :  // for abnormal product
                          mul_sc_frac;   // fallback → product

    // FMA shortcut exponent
    wire signed [EXP_BITS-1:0] fma_sc_exp =
        add_sc_p_undef  ? mul_sc_exp :
        add_sc_c_undef  ? c_exp :
        add_sc_tf_tf    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        add_sc_van_van  ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        add_sc_p_transf ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        add_sc_c_transf ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        add_sc_p_van    ? (sub ? neg_c_exp : c_exp) :
        add_sc_c_van    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        add_sc_p_zero   ? (sub ? neg_c_exp : c_exp) :
        add_sc_c_zero   ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                          AMBIGUOUS_EXP[EXP_BITS-1:0];

    // =========================================================================
    // Multiply (raw product, NOT normalized)
    // N1*N1 produces N1 or N2 — at most 1 redundant sign bit.
    //
    // Karatsuba decomposition: 3 sub-multiplies instead of 1 large multiply.
    // Saves ~250 LUT4 no-DSP; with DSP, use USE_KARATSUBA=0 for naive path.
    // =========================================================================
    localparam K = (FRAC_BITS + 1) / 2;
    localparam H = FRAC_BITS - K;

    wire signed [H-1:0] aH = a_frac[FRAC_BITS-1 : K];
    wire        [K-1:0] aL = a_frac[K-1 : 0];
    wire signed [H-1:0] bH = b_frac[FRAC_BITS-1 : K];
    wire        [K-1:0] bL = b_frac[K-1 : 0];

    wire signed [2*H-1:0]  phh = aH * bH;
    wire        [2*K-1:0]  pll = aL * bL;

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

    wire signed [PROD_BITS-1:0] product = product_wide[PROD_BITS-1:0];

    // Zero product when either input is ambiguous (zero/underflow)
    wire prod_is_zero = (a_exp == AMBIGUOUS_EXP[EXP_BITS-1:0])
                      | (b_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]);
    wire signed [PROD_BITS-1:0] prod_safe = prod_is_zero ? {PROD_BITS{1'b0}} : product;

    // =========================================================================
    // Raw product exponent (parallel with DSP — settles first)
    //
    // a_exp + b_exp WITHOUT subtracting norm_shift. The un-normalized
    // product's extra redundant bit is absorbed by the wider far-path
    // normalize and the wider close-path threshold. The exponent formula
    // (big_exp + 2 - leading) is unchanged because 'leading' is
    // correspondingly larger when the product is N2.
    // =========================================================================
    wire signed [EXP_BITS:0] prod_exp_raw = $signed({a_exp[EXP_BITS-1], a_exp})
                                           + $signed({b_exp[EXP_BITS-1], b_exp});
    wire signed [EXP_BITS:0] prod_exp_safe = prod_is_zero
                                           ? {1'b1, {EXP_BITS{1'b0}}}  // AMBIGUOUS_EXP sign-extended
                                           : prod_exp_raw;

    // =========================================================================
    // Widen c to PROD_BITS: c_frac in upper bits, zeros below
    // =========================================================================
    wire signed [PROD_BITS-1:0] c_wide = {c_frac, {(FRAC_BITS-1){1'b0}}};

    // =========================================================================
    // Exponent difference + swap (parallel with DSP)
    //
    // Uses raw product exponent. Swap, is_close, and barrel alignment all
    // settle before the DSP finishes (~1ns vs ~4ns).
    // =========================================================================
    wire signed [EXP_BITS+1:0] raw_diff = {prod_exp_safe[EXP_BITS], prod_exp_safe}
                                         - {{2{c_exp[EXP_BITS-1]}}, c_exp};
    wire prod_is_big = !raw_diff[EXP_BITS+1];

    wire signed [PROD_BITS-1:0] big_frac   = prod_is_big ? prod_safe : c_wide;
    wire signed [EXP_BITS:0]    big_exp    = prod_is_big ? prod_exp_safe
                                                         : $signed({c_exp[EXP_BITS-1], c_exp});
    wire signed [PROD_BITS-1:0] small_frac = prod_is_big ? c_wide : prod_safe;

    wire signed [EXP_BITS+1:0] exp_diff = raw_diff[EXP_BITS+1] ? -raw_diff : raw_diff;
    wire negligible = (exp_diff >= INT_BITS);
    // Close threshold widened to 2: raw exp_diff could be off by 1 due to
    // un-normalized product, so raw_diff=2 can mean true_diff=1.
    wire is_close = (exp_diff <= 2) && !negligible;

    // Subtraction: negate c operand
    wire negate_small = sub & prod_is_big;
    wire negate_big   = sub & !prod_is_big;

    // =========================================================================
    // Extend to internal width (frac << 2 makes room for G, R, S)
    // =========================================================================
    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac) <<< 2;
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac) <<< 2;

    // =========================================================================
    // Close path — 0-2 bit align + add + CLZ
    //
    // Widened from 0-1 to 0-2 bit alignment to match the close threshold.
    // =========================================================================
    wire [1:0] close_align_amt = exp_diff[1:0];
    wire signed [INT_BITS-1:0] close_small = (close_align_amt == 2'd2) ? (small_ext >>> 2) :
                                              (close_align_amt == 2'd1) ? (small_ext >>> 1) :
                                                                           small_ext;
    wire close_align_sticky = (close_align_amt[0] & small_ext[0])
                            | (close_align_amt[1] & |small_ext[1:0]);
    wire signed [INT_BITS-1:0] close_sum = (big_ext ^ {INT_BITS{negate_big}})
                                        + (close_small ^ {INT_BITS{negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, sub};
    wire close_is_zero = (close_sum == 0);

    // CLZ: priority encoder on XOR-adjacent-bits
    localparam DIFF_W = INT_BITS - 1;
    wire [DIFF_W-1:0] xor_diff = close_sum[INT_BITS-1:1] ^ close_sum[INT_BITS-2:0];

    reg [BARREL_BITS-1:0] close_norm_shift;
    integer ci;
    always @(*) begin
        close_norm_shift = DIFF_W[BARREL_BITS-1:0];
        for (ci = 0; ci < DIFF_W; ci = ci + 1)
            if (xor_diff[ci]) close_norm_shift = DIFF_W[BARREL_BITS-1:0] - 1 - ci[BARREL_BITS-1:0];
    end
    wire [LEAD_BITS-1:0] close_leading = close_norm_shift + 1;

    // =========================================================================
    // Shared barrel shifter — 6 stages for INT_BITS=52
    // =========================================================================
    wire [INT_BITS-1:0] close_sum_rev;
    genvar bi;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_in
            assign close_sum_rev[bi] = close_sum[INT_BITS - 1 - bi];
        end
    endgenerate

    wire [BARREL_BITS-1:0] far_shift = (exp_diff >= INT_BITS)
        ? INT_BITS[BARREL_BITS-1:0] - 1 : exp_diff[BARREL_BITS-1:0];

    wire [INT_BITS-1:0] barrel_in = is_close ? close_sum_rev : $unsigned(small_ext);
    wire [BARREL_BITS-1:0] barrel_shift = is_close ? close_norm_shift : far_shift;

    // Stage 0: shift by 1
    wire fill0 = !is_close & barrel_in[INT_BITS-1];
    wire [INT_BITS-1:0] b0_val = barrel_shift[0] ? {fill0, barrel_in[INT_BITS-1:1]} : barrel_in;
    wire b0_sticky = barrel_shift[0] & barrel_in[0];

    // Stage 1: shift by 2
    wire fill1 = !is_close & b0_val[INT_BITS-1];
    wire [INT_BITS-1:0] b1_val = barrel_shift[1] ? {{2{fill1}}, b0_val[INT_BITS-1:2]} : b0_val;
    wire b1_sticky = b0_sticky | (barrel_shift[1] & |b0_val[1:0]);

    // Stage 2: shift by 4
    wire fill2 = !is_close & b1_val[INT_BITS-1];
    wire [INT_BITS-1:0] b2_val = barrel_shift[2] ? {{4{fill2}}, b1_val[INT_BITS-1:4]} : b1_val;
    wire b2_sticky = b1_sticky | (barrel_shift[2] & |b1_val[3:0]);

    // Stage 3: shift by 8
    wire fill3 = !is_close & b2_val[INT_BITS-1];
    wire [INT_BITS-1:0] b3_val = barrel_shift[3] ? {{8{fill3}}, b2_val[INT_BITS-1:8]} : b2_val;
    wire b3_sticky = b2_sticky | (barrel_shift[3] & |b2_val[7:0]);

    // Stage 4: shift by 16
    wire fill4 = !is_close & b3_val[INT_BITS-1];
    wire [INT_BITS-1:0] b4_val = barrel_shift[4] ? {{16{fill4}}, b3_val[INT_BITS-1:16]} : b3_val;
    wire b4_sticky = b3_sticky | (barrel_shift[4] & |b3_val[15:0]);

    // Stage 5: shift by 32
    wire fill5 = !is_close & b4_val[INT_BITS-1];
    wire [INT_BITS-1:0] b5_val = barrel_shift[5] ? {{32{fill5}}, b4_val[INT_BITS-1:32]} : b4_val;
    wire b5_sticky = b4_sticky | (barrel_shift[5] & |b4_val[31:0]);

    wire [INT_BITS-1:0] barrel_out = b5_val;
    wire barrel_sticky = b5_sticky;

    // Bit-reverse barrel output for close path
    wire [INT_BITS-1:0] close_normalized;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_out
            assign close_normalized[bi] = barrel_out[INT_BITS - 1 - bi];
        end
    endgenerate

    // =========================================================================
    // Far path — add + bounded normalize (0-3 bit shift)
    //
    // Widened from 0-2 to 0-3: the un-normalized product (possibly N2) adds
    // 1 extra bit of possible leading redundancy after subtraction.
    // =========================================================================
    wire signed [INT_BITS-1:0] far_aligned = $signed(barrel_out);
    wire signed [INT_BITS-1:0] far_sum = (big_ext ^ {INT_BITS{negate_big}})
                                        + (far_aligned ^ {INT_BITS{negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, sub};
    wire far_is_zero = (far_sum == 0);

    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2];
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3];
    wire far_d2 = far_sum[INT_BITS-3] ^ far_sum[INT_BITS-4];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 : far_d1 ? 2'd1 : far_d2 ? 2'd2 : 2'd3;
    wire [LEAD_BITS-1:0] far_leading = {{(LEAD_BITS-2){1'b0}}, far_norm_shift} + 1;

    wire [INT_BITS-1:0] far_normalized = far_d0 ? $unsigned(far_sum) :
                                          far_d1 ? ($unsigned(far_sum) << 1) :
                                          far_d2 ? ($unsigned(far_sum) << 2) :
                                                   ($unsigned(far_sum) << 3);

    // =========================================================================
    // Shared rounding (banker's round / RNE)
    // =========================================================================
    wire [INT_BITS-1:0] normalized = is_close ? close_normalized : far_normalized;
    wire [LEAD_BITS-1:0] leading = is_close ? close_leading : far_leading;
    wire path_is_zero = is_close ? close_is_zero : far_is_zero;

    wire signed [FRAC_BITS-1:0] out_frac_raw = normalized[INT_BITS-1 -: FRAC_BITS];

    wire guard     = normalized[INT_BITS - 1 - FRAC_BITS];
    wire lsb       = normalized[INT_BITS - FRAC_BITS];
    wire round_bit = normalized[INT_BITS - 2 - FRAC_BITS];
    wire ext_sticky = |normalized[INT_BITS - 3 - FRAC_BITS:0];
    wire align_sticky = is_close ? close_align_sticky : barrel_sticky;
    wire sticky    = ext_sticky | align_sticky;
    wire round_up  = guard & (round_bit | sticky | lsb);

    wire signed [FRAC_BITS-1:0] out_frac_rounded = out_frac_raw
                                                     + {{(FRAC_BITS-1){1'b0}}, round_up};

    wire rovf_pos = !out_frac_raw[FRAC_BITS-1]
                  & (&out_frac_raw[FRAC_BITS-2:0])
                  & round_up;
    wire rovf_neg = out_frac_raw[FRAC_BITS-1]
                  & !out_frac_raw[FRAC_BITS-2]
                  & (&out_frac_raw[FRAC_BITS-3:0])
                  & round_up;

    wire signed [FRAC_BITS-1:0] out_frac = rovf_pos ? POS_HALF :
                                             rovf_neg ? NEG_ONE  :
                                             out_frac_rounded;

    // =========================================================================
    // Exponent
    //
    // big_exp + 2 - leading: the +2 comes from the <<< 2 extension to
    // INT_BITS. When big_exp is the raw product exponent (without norm_shift
    // subtracted), 'leading' is correspondingly larger — the norm_shift
    // cancels out in the subtraction, giving the correct result.
    // =========================================================================
    wire signed [EXP_BITS+1:0] exp_wide = {big_exp[EXP_BITS], big_exp}
                                         + 2
                                         - {{(EXP_BITS+2-LEAD_BITS){1'b0}}, leading}
                                         + {{(EXP_BITS+1){1'b0}}, rovf_pos}
                                         - {{(EXP_BITS+1){1'b0}}, rovf_neg};
    wire signed [EXP_BITS-1:0] out_exp = exp_wide[EXP_BITS-1:0];

    // Underflow detection
    wire signed [EXP_BITS-1:0] offset = out_exp - 1;
    wire underflow = big_exp[EXP_BITS] && !offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] underflow_frac = {normalized[INT_BITS-1],
                                                     normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // =========================================================================
    // Output mux
    // =========================================================================
    // Negligible bypass only when c is big (already FRAC_BITS wide).
    // When product is big, it needs rounding from PROD_BITS to FRAC_BITS,
    // so we let the far path handle it.
    wire use_negligible = negligible & !negate_big & !prod_is_big;

    assign result_frac = fma_shortcut    ? fma_sc_frac :
                         use_negligible  ? c_frac :
                         path_is_zero    ? {FRAC_BITS{1'b0}} :
                         underflow       ? underflow_frac :
                                           out_frac;

    assign result_exp  = fma_shortcut    ? fma_sc_exp :
                         use_negligible  ? c_exp :
                         path_is_zero    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                         underflow       ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                           out_exp;

endmodule
