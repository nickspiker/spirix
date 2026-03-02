// Spirix Addition: a + b (Close/Far split)
// Parameterized combinational adder for Spirix floating-point scalars.
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Close/Far split design:
//   Close path (exp_diff <= 1): 1-bit alignment (free wiring), full CLZ +
//       normalization barrel shifter. Handles massive cancellation.
//   Far path (exp_diff >= 2): alignment barrel shifter, add, normalization
//       bounded to {0, 1, 2} bit shift (no barrel shifter needed).

module spirix_add #(
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
    localparam INT_BITS = FRAC_BITS + 2; // sign + guard + fraction

    // =========================================================================
    // Step 1: Exponent difference + swap in one subtraction
    // Sign bit of raw_diff tells us which exponent is bigger (free wire).
    // |raw_diff| gives exp_diff without a second subtraction.
    // =========================================================================
    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                      - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS]; // non-negative → a_exp >= b_exp

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;

    // exp_diff = |raw_diff| (conditional negate replaces second subtraction)
    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;

    // If exponent difference >= FRAC_BITS, small contributes nothing
    wire negligible = (exp_diff >= FRAC_BITS);

    // Close/far selection: close handles exp_diff in {0, 1}
    wire is_close = (exp_diff <= 1);

    // =========================================================================
    // Pre-shift both by 1 (guard bit, free wiring in both paths)
    // =========================================================================
    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac) <<< 1;
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac) <<< 1;

    // =========================================================================
    // CLOSE PATH: exp_diff in {0, 1}
    // Alignment is 0 or 1 bit (free wiring). Full CLZ + barrel normalize.
    // =========================================================================

    // Close alignment: shift small right by exp_diff[0] (0 or 1 bit = free wiring)
    wire signed [INT_BITS-1:0] close_small = exp_diff[0] ? (small_ext >>> 1) : small_ext;
    wire signed [INT_BITS-1:0] close_sum = big_ext + close_small;

    // XOR-adjacent + compressBy2 + tree CLZ for close path
    localparam DIFF_W = INT_BITS - 1;
    wire [DIFF_W-1:0] close_diff = close_sum[INT_BITS-1:1] ^ close_sum[INT_BITS-2:0];

    // CompressBy2: OR adjacent pairs FROM MSB to halve the CLZ width.
    // Pairs: (DIFF_W-1, DIFF_W-2), (DIFF_W-3, DIFF_W-4), ...
    // CLZ(compressed) gives coarse count within ±1 of actual.
    localparam COMP_W = (DIFF_W + 1) / 2; // ceil(DIFF_W/2)
    wire [COMP_W-1:0] compressed;
    genvar gi;
    generate
        for (gi = 0; gi < COMP_W; gi = gi + 1) begin : compress
            if (DIFF_W - 1 - 2*gi - 1 >= 0)
                assign compressed[COMP_W-1-gi] = close_diff[DIFF_W-1-2*gi] | close_diff[DIFF_W-1-2*gi-1];
            else
                assign compressed[0] = close_diff[0]; // odd last bit at LSB
        end
    endgenerate

    // Tree CLZ on compressed (half width)
    localparam COMP_CLZ_BITS = $clog2(COMP_W + 1);

    wire [COMP_W-1:0] comp_rev;
    genvar ri;
    generate
        for (ri = 0; ri < COMP_W; ri = ri + 1) begin : comp_rev_gen
            assign comp_rev[ri] = compressed[COMP_W - 1 - ri];
        end
    endgenerate

    wire [COMP_W:0] comp_rev_ext = {1'b1, comp_rev};
    wire [COMP_W:0] comp_one_hot = comp_rev_ext & (~comp_rev_ext + 1);

    reg [COMP_CLZ_BITS-1:0] comp_clz;
    integer ci;
    always @(*) begin
        comp_clz = 0;
        for (ci = 0; ci <= COMP_W; ci = ci + 1)
            if (comp_one_hot[ci]) comp_clz = comp_clz | ci[COMP_CLZ_BITS-1:0];
    end

    // Coarse leading = 2 * comp_clz. Fixup: check if the actual first set
    // bit is in the even or odd position of the pair.
    // If close_diff[DIFF_W - 1 - 2*comp_clz] is set, actual CLZ = 2*comp_clz.
    // Otherwise, actual CLZ = 2*comp_clz + 1.
    wire [$clog2(DIFF_W):0] coarse_pos = {comp_clz, 1'b0}; // 2 * comp_clz
    // Index into close_diff to check the even bit of the pair
    wire fixup_bit = close_diff[DIFF_W - 1 - coarse_pos];
    wire [$clog2(DIFF_W):0] actual_clz = fixup_bit ? coarse_pos : (coarse_pos + 1);

    wire [$clog2(INT_BITS):0] close_leading = actual_clz + 1;

    // Close normalization barrel shifter
    wire [$clog2(INT_BITS):0] close_norm_shift = close_leading - 1;
    wire [INT_BITS-1:0] close_normalized = $unsigned(close_sum) << close_norm_shift;
    wire signed [FRAC_BITS-1:0] close_out_frac_raw = close_normalized[INT_BITS-1 -: FRAC_BITS];

    // Rounding overflow constants
    localparam signed [FRAC_BITS-1:0] POS_HALF_C = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}}; // 01..0
    localparam signed [FRAC_BITS-1:0] NEG_ONE_C  = {1'b1, {(FRAC_BITS-1){1'b0}}};        // 10..0

    // Close-path banker's rounding on the 2 bits below FRAC extraction
    wire close_guard  = close_normalized[INT_BITS - 1 - FRAC_BITS];
    wire close_clsb   = close_normalized[INT_BITS - FRAC_BITS]; // LSB of extracted frac
    wire close_sticky = (INT_BITS - 2 - FRAC_BITS >= 0) ?
                        |close_normalized[INT_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire close_round_up = close_guard & (close_sticky | close_clsb);
    wire signed [FRAC_BITS-1:0] close_out_frac_rnd = close_out_frac_raw
                                                     + {{(FRAC_BITS-1){1'b0}}, close_round_up};

    // Rounding overflow: two cases
    //   Positive wrap: 01..1 + 1 → 10..0 (sign flips 0→1). Fix: POS_HALF, exp+1
    //   Negative exit: 10..1 + 1 → 11..0 (top bits same). Fix: NEG_ONE, exp-1
    wire close_rovf_pos = !close_out_frac_raw[FRAC_BITS-1] & close_out_frac_rnd[FRAC_BITS-1];
    wire close_rovf_neg = close_out_frac_rnd[FRAC_BITS-1] & close_out_frac_rnd[FRAC_BITS-2];
    wire signed [FRAC_BITS-1:0] close_out_frac = close_rovf_pos ? POS_HALF_C :
                                                   close_rovf_neg ? NEG_ONE_C  :
                                                   close_out_frac_rnd;

    // Close exponent: big_exp + 2 - leading, with rounding overflow correction
    wire signed [EXP_BITS:0] close_exp_wide = $signed({big_exp[EXP_BITS-1], big_exp})
                                             + 2 - $signed({{1'b0}, close_leading});
    wire signed [EXP_BITS-1:0] close_out_exp_base = close_exp_wide[EXP_BITS-1:0];
    wire signed [EXP_BITS-1:0] close_out_exp = close_rovf_pos ? (close_out_exp_base + 1) :
                                                close_rovf_neg ? (close_out_exp_base - 1) :
                                                close_out_exp_base;

    // Close underflow
    wire signed [EXP_BITS-1:0] close_offset = close_out_exp - 1;
    wire close_underflow = big_exp[EXP_BITS-1] && !close_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] close_underflow_frac = {close_normalized[INT_BITS-1],
                                                          close_normalized[INT_BITS-1 -: FRAC_BITS-1]};

    wire close_is_zero = (close_sum == 0);

    // =========================================================================
    // FAR PATH: exp_diff >= 2
    // Alignment barrel shifter. Normalization bounded to 0-2 bit shift.
    // =========================================================================

    // Far alignment barrel shifter (starts at bit 1 since exp_diff >= 2)
    wire [$clog2(FRAC_BITS)-1:0] align_shift = exp_diff[$clog2(FRAC_BITS)-1:0];
    wire signed [INT_BITS-1:0] small_aligned = small_ext >>> align_shift;
    wire signed [INT_BITS-1:0] far_sum = big_ext + small_aligned;

    // Sticky bit: OR of bits shifted out during alignment.
    // Decoder mask approach: generate (1 << align_shift) - 1 to mask low bits,
    // then OR-reduce the masked value.
    wire [INT_BITS-1:0] sticky_one_hot = {{(INT_BITS-1){1'b0}}, 1'b1} << align_shift;
    wire [INT_BITS-1:0] sticky_mask = sticky_one_hot - 1;
    wire far_sticky = |($unsigned(small_ext) & sticky_mask);

    // Far normalization: leading is bounded to {1, 2, 3} for N1 inputs with
    // exp_diff >= 2. Instead of a full barrel shifter, use a 3-way mux:
    //   leading=1 (norm_shift=0): take sum as-is, shift right 1 relative to normal
    //   leading=2 (norm_shift=1): normal case, shift left 1
    //   leading=3 (norm_shift=2): shift left 2
    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2]; // diff MSB
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3]; // diff MSB-1

    // leading=1 when far_d0=1, leading=2 when far_d0=0,far_d1=1, leading=3 when both=0
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 :
                                 far_d1 ? 2'd1 : 2'd2;
    wire [$clog2(INT_BITS):0] far_leading = {2'b0, far_norm_shift} + 1;

    wire [INT_BITS-1:0] far_normalized = $unsigned(far_sum) << far_norm_shift;

    // Banker's rounding: guard bit is just below the fraction, after normalization.
    // round_up = guard && (sticky || lsb_odd). Sticky includes alignment losses
    // and any extraction bits below guard.
    wire far_guard = far_normalized[INT_BITS - 1 - FRAC_BITS]; // bit just below fraction
    wire far_lsb   = far_normalized[INT_BITS - FRAC_BITS];     // LSB of fraction
    wire far_ext_sticky = (INT_BITS - 2 - FRAC_BITS >= 0) ?
                          |far_normalized[INT_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire far_round_up = far_guard & ((far_sticky | far_ext_sticky) | far_lsb);
    wire signed [FRAC_BITS-1:0] far_out_frac_raw = far_normalized[INT_BITS-1 -: FRAC_BITS];
    wire signed [FRAC_BITS-1:0] far_out_frac_rounded = far_out_frac_raw + {{(FRAC_BITS-1){1'b0}}, far_round_up};

    // Rounding overflow: two cases
    //   Positive wrap: 01..1 + 1 → 10..0 (sign flips 0→1). Fix: POS_HALF, exp+1
    //   Negative exit: 10..1 + 1 → 11..0 (top bits same). Fix: NEG_ONE, exp-1
    wire far_rovf_pos = !far_out_frac_raw[FRAC_BITS-1] & far_out_frac_rounded[FRAC_BITS-1];
    wire far_rovf_neg = far_out_frac_rounded[FRAC_BITS-1] & far_out_frac_rounded[FRAC_BITS-2];

    wire signed [FRAC_BITS-1:0] far_out_frac = far_rovf_pos ? POS_HALF_C :
                                                 far_rovf_neg ? NEG_ONE_C  :
                                                 far_out_frac_rounded;

    // Far exponent: big_exp + 2 - far_leading, with rounding overflow correction
    wire signed [EXP_BITS:0] far_exp_wide = $signed({big_exp[EXP_BITS-1], big_exp})
                                           + 2 - $signed({{1'b0}, far_leading});
    wire signed [EXP_BITS-1:0] far_out_exp_base = far_exp_wide[EXP_BITS-1:0];
    wire signed [EXP_BITS-1:0] far_out_exp = far_rovf_pos ? (far_out_exp_base + 1) :
                                              far_rovf_neg ? (far_out_exp_base - 1) :
                                              far_out_exp_base;

    // Far underflow
    wire signed [EXP_BITS-1:0] far_offset = far_out_exp - 1;
    wire far_underflow = big_exp[EXP_BITS-1] && !far_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] far_underflow_frac = {far_normalized[INT_BITS-1],
                                                        far_normalized[INT_BITS-1 -: FRAC_BITS-1]};

    wire far_is_zero = (far_sum == 0);

    // =========================================================================
    // Output: select between close and far paths
    // =========================================================================
    wire use_close = is_close && !negligible;

    wire signed [FRAC_BITS-1:0] path_frac = use_close ? close_out_frac : far_out_frac;
    wire signed [EXP_BITS-1:0]  path_exp  = use_close ? close_out_exp  : far_out_exp;
    wire path_underflow = use_close ? close_underflow : far_underflow;
    wire signed [FRAC_BITS-1:0] path_uf_frac = use_close ? close_underflow_frac : far_underflow_frac;
    wire path_is_zero = use_close ? close_is_zero : far_is_zero;

    assign result_frac = negligible      ? big_frac :
                         path_is_zero    ? {FRAC_BITS{1'b0}} :
                         path_underflow  ? path_uf_frac :
                                           path_frac;

    assign result_exp  = negligible      ? big_exp :
                         path_is_zero    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                         path_underflow  ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                           path_exp;

endmodule
