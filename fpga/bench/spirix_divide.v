// spirix_divide — Combinational divide for Spirix scalars
//
// Computes a / b on N1-normalized signed fractions with signed exponents.
// Fully parameterized, purely combinational.
//
// Algorithm:
//   1. Extract sign (XOR of input signs). Convert to unsigned magnitudes.
//      NEG_ONE (the only N1 value whose magnitude overflows FRAC_BITS-1
//      unsigned bits) is represented as POS_HALF with exponent+1.
//   2. Unsigned fixed-point division: q = (|a| << FRAC_BITS) / |b|.
//      For N1 inputs, q in [2^(FRAC-1), 2^(FRAC+1)), giving FRAC+1 bits.
//   3. Bounded normalization: 0 or 1 bit right shift (no barrel needed).
//   4. Banker's rounding (RNE). Guard from quotient, sticky from shifted-
//      out bits and the exact remainder.
//   5. Apply sign: negate fraction for negative results. POS_HALF maps
//      to NEG_ONE with exp-1 (since -0.5 is not N1).
//   6. Exponent: a_exp - b_exp + norm_shift + round_ovf, adjusted for
//      sign (POS_HALF -> NEG_ONE costs 1 exponent). Overflow/underflow
//      checked after sign adjustment to catch the edge case.
//   7. Output with overflow/underflow/div-by-zero clamping.
//
// Why sign extraction is required: two's complement multiplication has a
// modular identity that makes it sign-blind (low 2N bits are the same for
// signed and unsigned). Division has no such identity — the quotient bits
// depend fundamentally on the sign interpretation.
//
// Note: uses Verilog '/' operator — suitable for simulation/verification.
// For synthesis, use the iterative variant (spirix_divide_iter).
//
// Division by zero returns (0, AMBIGUOUS_EXP).
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_divide #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    // =========================================================================
    // Step 1: Sign and absolute values
    //
    // Result sign = XOR of input signs. Convert fractions to unsigned
    // magnitudes. NEG_ONE (10...0) has magnitude 2^(FRAC-1) which needs
    // FRAC unsigned bits — too wide for our FRAC-1 bit magnitude path.
    // Represent it as POS_HALF (magnitude 2^(FRAC-2)) with exponent+1:
    //   -1.0 * 2^E = -0.5 * 2^(E+1).
    // =========================================================================
    wire result_sign = a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
    wire b_is_zero = (b_frac == 0);

    wire a_is_neg_one = (a_frac == NEG_ONE);
    wire b_is_neg_one = (b_frac == NEG_ONE);

    wire [FRAC_BITS-2:0] abs_a = a_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                  (a_frac[FRAC_BITS-1] ? (~a_frac[FRAC_BITS-2:0] + 1'b1) :
                                                          a_frac[FRAC_BITS-2:0]);
    wire [FRAC_BITS-2:0] abs_b = b_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                  (b_frac[FRAC_BITS-1] ? (~b_frac[FRAC_BITS-2:0] + 1'b1) :
                                                          b_frac[FRAC_BITS-2:0]);

    wire signed [EXP_BITS:0] a_exp_adj = $signed({a_exp[EXP_BITS-1], a_exp})
                                         + {{EXP_BITS{1'b0}}, a_is_neg_one};
    wire signed [EXP_BITS:0] b_exp_adj = $signed({b_exp[EXP_BITS-1], b_exp})
                                         + {{EXP_BITS{1'b0}}, b_is_neg_one};

    // =========================================================================
    // Step 2: Unsigned fixed-point division
    //
    // Shift numerator left by FRAC_BITS to get FRAC_BITS+1 quotient bits.
    // For N1 magnitudes in [2^(FRAC-2), 2^(FRAC-1)):
    //   quotient in [2^(FRAC-1), 2^(FRAC+1))
    // The remainder provides exact sticky information for rounding.
    // =========================================================================
    localparam WIDE = 2 * FRAC_BITS;
    wire [WIDE-1:0] wide_a = {{(FRAC_BITS+1){1'b0}}, abs_a} << FRAC_BITS;
    wire [WIDE-1:0] wide_b = {{(FRAC_BITS+1){1'b0}}, abs_b};
    wire [WIDE-1:0] quotient_full = wide_a / wide_b;
    wire [WIDE-1:0] remainder     = wide_a - quotient_full * wide_b;

    wire [FRAC_BITS:0] q = quotient_full[FRAC_BITS:0];

    // =========================================================================
    // Step 3: Bounded normalization (0 or 1 bit right shift)
    //
    // q[FRAC] set means |a/b| >= 1.0. Shift right 1 to place the leading
    // 1 at bit FRAC-1 (positive N1 position). The shifted-out bit feeds
    // into sticky for rounding.
    // =========================================================================
    wire norm_shift = q[FRAC_BITS];
    wire [FRAC_BITS:0] q_norm = norm_shift ? (q >> 1) : q;

    // Extract positive N1 fraction: q_norm[FRAC:1] = 01xxx (FRAC bits)
    wire [FRAC_BITS-1:0] frac_pos_raw = q_norm[FRAC_BITS:1];

    // =========================================================================
    // Step 4: Banker's rounding (RNE)
    //
    // Guard: first bit below the extracted fraction.
    // Sticky: bit shifted out by normalization OR any nonzero remainder.
    // Round up when guard=1 AND (sticky | lsb).
    //
    // Early rounding overflow detection: frac_pos_raw is always positive
    // (01xxx), so only positive rovf applies. Max positive N1 (0_111...1)
    // + round_up overflows to 1_000...0, exiting N1.
    // =========================================================================
    wire guard      = q_norm[0];
    wire norm_sticky = norm_shift & q[0];
    wire rem_sticky  = |remainder;
    wire sticky = norm_sticky | rem_sticky;
    wire lsb    = frac_pos_raw[0];
    wire round_up = guard & (sticky | lsb);

    wire [FRAC_BITS-1:0] frac_rounded = frac_pos_raw + {{(FRAC_BITS-1){1'b0}}, round_up};

    wire round_ovf = (&frac_pos_raw[FRAC_BITS-2:0]) & round_up;
    wire [FRAC_BITS-1:0] pos_frac = round_ovf ? POS_HALF : frac_rounded;

    // =========================================================================
    // Step 5: Apply sign
    //
    // For negative results, negate the positive fraction. Two's complement
    // negation of an N1 value always produces an N1 value except for
    // POS_HALF: -(01_0...0) = 11_0...0 where top two bits match (not N1).
    // Fix: represent -0.5 as NEG_ONE (-1.0) with exponent-1.
    // =========================================================================
    wire neg_is_pos_half = (pos_frac == POS_HALF);

    wire signed [FRAC_BITS-1:0] neg_frac = neg_is_pos_half ? NEG_ONE :
                                            (~pos_frac + 1'b1);

    wire signed [FRAC_BITS-1:0] final_frac = result_sign ? neg_frac : $signed(pos_frac);

    // =========================================================================
    // Step 6: Exponent with sign adjustment
    //
    // Base: a_exp_adj - b_exp_adj + norm_shift + round_ovf.
    // Sign adjustment: POS_HALF -> NEG_ONE costs 1 exponent.
    // Overflow/underflow checked on the final (sign-adjusted) exponent
    // to catch the edge case where the -1 pushes past MIN_EXP.
    // =========================================================================
    wire signed [EXP_BITS:0] exp_base = a_exp_adj - b_exp_adj
                                       + {{EXP_BITS{1'b0}}, norm_shift}
                                       + {{EXP_BITS{1'b0}}, round_ovf};

    wire signed [EXP_BITS:0] exp_final = (result_sign & neg_is_pos_half) ?
                                          (exp_base - 1) : exp_base;

    wire exp_too_big   = (exp_final > MAX_EXP);
    wire exp_too_small = (exp_final < MIN_EXP);
    wire signed [EXP_BITS-1:0] final_exp = exp_final[EXP_BITS-1:0];

    // =========================================================================
    // Step 7: Output with overflow/underflow/div-by-zero clamping
    //
    // Division by zero: return (0, AMBIGUOUS_EXP).
    // Overflow: preserve fraction sign, AMBIGUOUS_EXP.
    // Underflow: truncate fraction (sign + MSBs), AMBIGUOUS_EXP.
    // =========================================================================
    assign result_frac = b_is_zero     ? {FRAC_BITS{1'b0}} :
                         exp_too_big   ? final_frac :
                         exp_too_small ? {final_frac[FRAC_BITS-1],
                                          final_frac[FRAC_BITS-1:1]} :
                                         final_frac;

    assign result_exp  = (b_is_zero | exp_too_big | exp_too_small) ?
                          AMBIGUOUS_EXP[EXP_BITS-1:0] : final_exp;

endmodule
