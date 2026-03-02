// Spirix Addition: a + b — 3-stage pipelined Close/Far split
// Parameterized pipelined adder for Spirix floating-point scalars.
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Pipeline stages:
//   Stage 1 (Swap): exp subtract → |diff| → mux big/small → pipeline regs.
//       Swap muxes absorbed into pipeline register input LUTs — the mux
//       function occupies the LUT4 that feeds each pipeline FF (same PFU
//       slice on ECP5), so the swap costs no additional routing or LUTs
//       beyond what the pipeline registers already require.
//   Stage 2 (Align+Add / CLZ): Close path does add + full CLZ on the sum.
//       Far path does alignment barrel shift + sticky + add. Both paths
//       complete their addition in this stage.
//   Stage 3 (Normalize/Round): Close: barrel normalize + exp compute.
//       Far: bounded normalize (0-2 shift) + banker's round + exp compute.
//       Output mux selects close/far/negligible/zero.
//
// Close/Far split:
//   Close path (exp_diff <= 1): 1-bit alignment (free wiring), full CLZ +
//       normalization barrel shifter. Handles massive cancellation.
//   Far path (exp_diff >= 2): alignment barrel shifter, add, normalization
//       bounded to {0, 1, 2} bit shift (no barrel shifter needed).
//
// Latency: 3 clock cycles (input to output register).
// Throughput: 1 result per clock cycle.

module spirix_add_pipe #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam INT_BITS = FRAC_BITS + 2; // sign + guard + fraction

    // =========================================================================
    // STAGE 1: Swap
    // Single subtraction gives swap direction (sign bit) and exp_diff (|diff|).
    // Swap muxes feed directly into S1→S2 pipeline registers — the mux LUT4
    // is in the same PFU slice as the pipeline FF, so it costs nothing beyond
    // what the registers already require. This eliminates the "swapzies tax"
    // that plagues the combinational design (~58 standalone LUT4s).
    // =========================================================================

    // Exponent difference + swap direction in one subtraction
    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                       - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS]; // non-negative → a_exp >= b_exp

    // |raw_diff| = exp_diff (conditional negate replaces second subtraction)
    wire signed [EXP_BITS:0] exp_diff_s1 = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;

    // Path control signals
    wire negligible_s1 = (exp_diff_s1 >= FRAC_BITS);
    wire is_close_s1   = (exp_diff_s1 <= 1);

    // Swap + pre-shift by 1 (guard bit, free wiring).
    // These are the "swapzies" — absorbed into pipeline reg input LUTs.
    wire signed [INT_BITS-1:0] big_ext_s1 = a_is_big
        ? ($signed(a_frac) <<< 1) : ($signed(b_frac) <<< 1);
    wire signed [INT_BITS-1:0] small_ext_s1 = a_is_big
        ? ($signed(b_frac) <<< 1) : ($signed(a_frac) <<< 1);
    wire signed [EXP_BITS-1:0] big_exp_s1 = a_is_big ? a_exp : b_exp;
    wire signed [FRAC_BITS-1:0] big_frac_s1 = a_is_big ? a_frac : b_frac;

    // --- S1→S2 pipeline registers ---
    reg signed [INT_BITS-1:0]  s2_big_ext, s2_small_ext;
    reg signed [EXP_BITS-1:0]  s2_big_exp;
    reg signed [FRAC_BITS-1:0] s2_big_frac;   // forwarded for negligible output
    reg signed [EXP_BITS:0]    s2_exp_diff;
    reg                         s2_negligible, s2_is_close;

    always @(posedge clk) begin
        s2_big_ext    <= big_ext_s1;
        s2_small_ext  <= small_ext_s1;
        s2_big_exp    <= big_exp_s1;
        s2_big_frac   <= big_frac_s1;
        s2_exp_diff   <= exp_diff_s1;
        s2_negligible <= negligible_s1;
        s2_is_close   <= is_close_s1;
    end

    // =========================================================================
    // STAGE 2: Align+Add / CLZ
    //   Close path: add + full compressBy2 CLZ on the sum.
    //   Far path: alignment barrel shift + sticky + add.
    //   Both paths complete their addition in this stage so we don't need to
    //   forward big_ext through to stage 3 (saves 27 pipeline register bits).
    // =========================================================================

    // --- Close path: alignment (free wiring) + add + CLZ ---

    // Close alignment: shift small right by exp_diff[0] (0 or 1 bit = free wiring)
    wire signed [INT_BITS-1:0] close_small = s2_exp_diff[0]
        ? (s2_small_ext >>> 1) : s2_small_ext;
    wire signed [INT_BITS-1:0] close_sum_s2 = s2_big_ext + close_small;
    wire close_is_zero_s2 = (close_sum_s2 == 0);

    // XOR-adjacent + compressBy2 + tree CLZ
    localparam DIFF_W = INT_BITS - 1;
    wire [DIFF_W-1:0] close_diff = close_sum_s2[INT_BITS-1:1] ^ close_sum_s2[INT_BITS-2:0];

    // CompressBy2: OR adjacent pairs FROM MSB to halve the CLZ width.
    // Pairs: (DIFF_W-1, DIFF_W-2), (DIFF_W-3, DIFF_W-4), ...
    // CLZ(compressed) gives coarse count within ±1 of actual.
    localparam COMP_W = (DIFF_W + 1) / 2; // ceil(DIFF_W/2)
    wire [COMP_W-1:0] compressed;
    genvar gi;
    generate
        for (gi = 0; gi < COMP_W; gi = gi + 1) begin : compress
            if (DIFF_W - 1 - 2*gi - 1 >= 0)
                assign compressed[COMP_W-1-gi] = close_diff[DIFF_W-1-2*gi]
                                                | close_diff[DIFF_W-1-2*gi-1];
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
    wire [$clog2(DIFF_W):0] coarse_pos = {comp_clz, 1'b0}; // 2 * comp_clz
    wire fixup_bit = close_diff[DIFF_W - 1 - coarse_pos];
    wire [$clog2(DIFF_W):0] actual_clz = fixup_bit ? coarse_pos : (coarse_pos + 1);
    wire [$clog2(INT_BITS):0] close_leading_s2 = actual_clz + 1;

    // --- Far path: alignment barrel shift + sticky + add ---

    // Alignment barrel shifter
    wire [$clog2(FRAC_BITS)-1:0] align_shift = s2_exp_diff[$clog2(FRAC_BITS)-1:0];
    wire signed [INT_BITS-1:0] small_aligned = s2_small_ext >>> align_shift;

    // Sticky bit (decoder mask approach): generate (1 << align_shift) - 1,
    // mask low bits of small_ext, OR-reduce to get sticky.
    wire [INT_BITS-1:0] sticky_one_hot = {{(INT_BITS-1){1'b0}}, 1'b1} << align_shift;
    wire [INT_BITS-1:0] sticky_mask = sticky_one_hot - 1;
    wire far_sticky_s2 = |($unsigned(s2_small_ext) & sticky_mask);

    // Far addition (in stage 2, so we don't forward big_ext to stage 3)
    wire signed [INT_BITS-1:0] far_sum_s2 = s2_big_ext + small_aligned;

    // --- S2→S3 pipeline registers ---
    reg signed [INT_BITS-1:0]   s3_close_sum;
    reg [$clog2(INT_BITS):0]    s3_close_leading;
    reg                          s3_close_is_zero;
    reg signed [INT_BITS-1:0]   s3_far_sum;
    reg                          s3_far_sticky;
    reg signed [EXP_BITS-1:0]   s3_big_exp;
    reg signed [FRAC_BITS-1:0]  s3_big_frac;   // forwarded for negligible output
    reg                          s3_negligible, s3_is_close;

    always @(posedge clk) begin
        // Close path results
        s3_close_sum     <= close_sum_s2;
        s3_close_leading <= close_leading_s2;
        s3_close_is_zero <= close_is_zero_s2;
        // Far path results
        s3_far_sum       <= far_sum_s2;
        s3_far_sticky    <= far_sticky_s2;
        // Forwarded through pipeline
        s3_big_exp       <= s2_big_exp;
        s3_big_frac      <= s2_big_frac;
        s3_negligible    <= s2_negligible;
        s3_is_close      <= s2_is_close;
    end

    // =========================================================================
    // STAGE 3: Normalize / Round / Output
    //   Close path: barrel normalize by close_leading, extract frac, compute exp.
    //   Far path: bounded normalize (0-2 shift), banker's round, compute exp.
    //   Output mux selects close/far/negligible/zero.
    // =========================================================================

    // --- Close path: barrel normalize ---
    wire [$clog2(INT_BITS):0] close_norm_shift = s3_close_leading - 1;
    wire [INT_BITS-1:0] close_normalized = $unsigned(s3_close_sum) << close_norm_shift;
    wire signed [FRAC_BITS-1:0] close_out_frac = close_normalized[INT_BITS-1 -: FRAC_BITS];

    // Close exponent: big_exp + 2 - leading
    wire signed [EXP_BITS:0] close_exp_wide = $signed({s3_big_exp[EXP_BITS-1], s3_big_exp})
                                              + 2 - $signed({{1'b0}, s3_close_leading});
    wire signed [EXP_BITS-1:0] close_out_exp = close_exp_wide[EXP_BITS-1:0];

    // Close underflow: exponent wrapped past AMBIGUOUS_EXP
    wire signed [EXP_BITS-1:0] close_offset = close_out_exp - 1;
    wire close_underflow = s3_big_exp[EXP_BITS-1] && !close_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] close_underflow_frac = {close_normalized[INT_BITS-1],
                                                         close_normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // --- Far path: bounded normalize + banker's round ---

    // Far normalization: leading is bounded to {1, 2, 3} for N1 inputs with
    // exp_diff >= 2. 3-way mux instead of barrel shifter.
    wire far_d0 = s3_far_sum[INT_BITS-1] ^ s3_far_sum[INT_BITS-2];
    wire far_d1 = s3_far_sum[INT_BITS-2] ^ s3_far_sum[INT_BITS-3];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 :
                                 far_d1 ? 2'd1 : 2'd2;
    wire [$clog2(INT_BITS):0] far_leading = {2'b0, far_norm_shift} + 1;
    wire [INT_BITS-1:0] far_normalized = $unsigned(s3_far_sum) << far_norm_shift;

    // Banker's rounding: round_up = guard && (sticky || lsb_odd).
    // In two's complement, +1 is +1 regardless of sign — no sign handling.
    wire far_guard = far_normalized[INT_BITS - 1 - FRAC_BITS]; // bit just below fraction
    wire far_lsb   = far_normalized[INT_BITS - FRAC_BITS];     // LSB of fraction
    wire far_round_up = far_guard & (s3_far_sticky | far_lsb);
    wire signed [FRAC_BITS-1:0] far_out_frac_raw = far_normalized[INT_BITS-1 -: FRAC_BITS];
    wire signed [FRAC_BITS-1:0] far_out_frac = far_out_frac_raw
                                              + {{(FRAC_BITS-1){1'b0}}, far_round_up};

    // Far exponent: big_exp + 2 - far_leading
    wire signed [EXP_BITS:0] far_exp_wide = $signed({s3_big_exp[EXP_BITS-1], s3_big_exp})
                                            + 2 - $signed({{1'b0}, far_leading});
    wire signed [EXP_BITS-1:0] far_out_exp = far_exp_wide[EXP_BITS-1:0];

    // Far underflow
    wire signed [EXP_BITS-1:0] far_offset = far_out_exp - 1;
    wire far_underflow = s3_big_exp[EXP_BITS-1] && !far_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] far_underflow_frac = {far_normalized[INT_BITS-1],
                                                       far_normalized[INT_BITS-1 -: FRAC_BITS-1]};
    wire far_is_zero = (s3_far_sum == 0);

    // --- Output mux: select close/far/negligible/zero ---
    wire use_close = s3_is_close && !s3_negligible;

    wire signed [FRAC_BITS-1:0] path_frac = use_close ? close_out_frac : far_out_frac;
    wire signed [EXP_BITS-1:0]  path_exp  = use_close ? close_out_exp  : far_out_exp;
    wire path_underflow = use_close ? close_underflow : far_underflow;
    wire signed [FRAC_BITS-1:0] path_uf_frac = use_close ? close_underflow_frac
                                                         : far_underflow_frac;
    wire path_is_zero = use_close ? s3_close_is_zero : far_is_zero;

    // Stage 3 output register
    always @(posedge clk) begin
        result_frac <= s3_negligible     ? s3_big_frac :
                       path_is_zero      ? {FRAC_BITS{1'b0}} :
                       path_underflow    ? path_uf_frac :
                                           path_frac;

        result_exp  <= s3_negligible     ? s3_big_exp :
                       path_is_zero      ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                       path_underflow    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                           path_exp;
    end

endmodule
