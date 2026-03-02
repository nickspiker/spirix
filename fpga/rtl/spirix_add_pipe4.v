// Spirix Addition: a + b — 4-stage pipelined Close/Far split
// Shared barrel shifter version.
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Pipeline stages:
//   Stage 1 (Swap + Close Add):
//       Swap: exp subtract → |diff| → mux big/small.
//       Close: 1-bit alignment + carry-propagate add. Produces close_sum
//           and close_is_zero. Both run within the swap stage because the
//           close add only depends on the swap mux outputs (big_ext, small_ext)
//           and exp_diff[0], all of which are available after the exp subtract.
//       Swap muxes absorbed into pipeline register input LUTs.
//   Stage 2 (CLZ / Sticky):
//       Close: full compressBy2 CLZ on close_sum. No adder — just CLZ.
//       Far: sticky bit computation (decoder mask on pre-shift data).
//   Stage 3 (Shared Barrel):
//       Single barrel shifter, muxed by is_close.
//       Close: left-shift normalize (via bit-reverse → right-shift → bit-reverse).
//       Far: right-shift alignment.
//       Both directions map to right shift: close data is bit-reversed
//       on input and output (free wiring), so one right-shift barrel handles both.
//   Stage 4 (Finish):
//       Close: extract frac + exp compute.
//       Far: add big_ext + aligned, bounded normalize, banker's round, exp.
//       big_ext is reconstructed from big_frac (free wiring: frac <<< 1),
//       eliminating 54 FFs of pipeline forwarding.
//       Output mux.
//
// Close/Far split:
//   Close path (exp_diff <= 1): 1-bit alignment (free wiring), full CLZ +
//       normalization barrel shifter. Handles massive cancellation.
//   Far path (exp_diff >= 2): alignment barrel shifter, add, normalization
//       bounded to {0, 1, 2} bit shift (no barrel shifter needed).
//
// Latency: 4 clock cycles. Throughput: 1 result per clock cycle.

module spirix_add_pipe4 #(
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
    // STAGE 1: Swap + Close Add
    //
    // Single subtraction gives swap direction (sign bit) and exp_diff (|diff|).
    // Close path add runs here: big_ext + close_small. The close add depends
    // only on the swap mux outputs and exp_diff[0], all ready after the exp
    // subtract + conditional negate. ECP5 carry chains keep this fast:
    //   exp subtract (~1.4 ns) → swap mux (~0.5 ns) → close_small mux (~0.5 ns)
    //   → close add (~4 ns) = ~6.4 ns total. Fits in period at 68+ MHz.
    //
    // big_ext is NOT registered — it's consumed by the close add here and
    // reconstructed from big_frac in stage 4 (free wiring: frac <<< 1).
    // This eliminates 27 LUT4s (swap mux) and 54 FFs of forwarding.
    // =========================================================================

    // --- Swap ---

    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                       - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];
    wire signed [EXP_BITS:0] exp_diff_s1 = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;

    wire negligible_s1 = (exp_diff_s1 >= FRAC_BITS);
    wire is_close_s1   = (exp_diff_s1 <= 1);

    // Swap + pre-shift by 1 (guard bit)
    // big_ext and small_ext are combinational — NOT registered directly.
    wire signed [INT_BITS-1:0] big_ext_s1 = a_is_big
        ? ($signed(a_frac) <<< 1) : ($signed(b_frac) <<< 1);
    wire signed [INT_BITS-1:0] small_ext_s1 = a_is_big
        ? ($signed(b_frac) <<< 1) : ($signed(a_frac) <<< 1);
    wire signed [EXP_BITS-1:0] big_exp_s1 = a_is_big ? a_exp : b_exp;
    wire signed [FRAC_BITS-1:0] big_frac_s1 = a_is_big ? a_frac : b_frac;

    // --- Close path: alignment + add (in stage 1, parallel with swap) ---

    wire signed [INT_BITS-1:0] close_small_s1 = exp_diff_s1[0]
        ? (small_ext_s1 >>> 1) : small_ext_s1;
    wire signed [INT_BITS-1:0] close_sum_s1 = big_ext_s1 + close_small_s1;
    wire close_is_zero_s1 = (close_sum_s1 == 0);

    // --- Far path: align shift amount (5 bits, from exp_diff) ---
    wire [$clog2(FRAC_BITS)-1:0] align_shift_s1 = exp_diff_s1[$clog2(FRAC_BITS)-1:0];

    // --- S1→S2 pipeline registers ---
    // close_sum replaces big_ext (same width, saves big_ext swap mux LUT4s).
    // align_shift (5 bits) replaces full exp_diff (9 bits).
    reg signed [INT_BITS-1:0]    s2_close_sum;
    reg                           s2_close_is_zero;
    reg signed [INT_BITS-1:0]    s2_small_ext;
    reg [$clog2(FRAC_BITS)-1:0]  s2_align_shift;
    reg signed [EXP_BITS-1:0]    s2_big_exp;
    reg signed [FRAC_BITS-1:0]   s2_big_frac;
    reg                           s2_negligible, s2_is_close;

    always @(posedge clk) begin
        s2_close_sum    <= close_sum_s1;
        s2_close_is_zero<= close_is_zero_s1;
        s2_small_ext    <= small_ext_s1;
        s2_align_shift  <= align_shift_s1;
        s2_big_exp      <= big_exp_s1;
        s2_big_frac     <= big_frac_s1;
        s2_negligible   <= negligible_s1;
        s2_is_close     <= is_close_s1;
    end

    // =========================================================================
    // STAGE 2: CLZ / Sticky
    //   Close path: full compressBy2 CLZ on close_sum. No adder here — the add
    //       moved to stage 1, so this stage is just the CLZ (~5 LUT4 levels).
    //   Far path: sticky bit computation on pre-shift data (decoder mask).
    //   Both paths prepare their barrel shifter inputs for stage 3.
    // =========================================================================

    // --- Close path: CLZ on close_sum ---

    // XOR-adjacent + compressBy2 + tree CLZ
    localparam DIFF_W = INT_BITS - 1;
    wire [DIFF_W-1:0] close_diff = s2_close_sum[INT_BITS-1:1] ^ s2_close_sum[INT_BITS-2:0];

    localparam COMP_W = (DIFF_W + 1) / 2;
    wire [COMP_W-1:0] compressed;
    genvar gi;
    generate
        for (gi = 0; gi < COMP_W; gi = gi + 1) begin : compress
            if (DIFF_W - 1 - 2*gi - 1 >= 0)
                assign compressed[COMP_W-1-gi] = close_diff[DIFF_W-1-2*gi]
                                                | close_diff[DIFF_W-1-2*gi-1];
            else
                assign compressed[0] = close_diff[0];
        end
    endgenerate

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

    wire [$clog2(DIFF_W):0] coarse_pos = {comp_clz, 1'b0};
    wire fixup_bit = close_diff[DIFF_W - 1 - coarse_pos];
    wire [$clog2(DIFF_W):0] actual_clz = fixup_bit ? coarse_pos : (coarse_pos + 1);
    wire [$clog2(INT_BITS):0] close_leading_s2 = actual_clz + 1;

    // Close barrel shift amount: normalize by (leading - 1)
    wire [$clog2(INT_BITS):0] close_norm_shift_s2 = close_leading_s2 - 1;

    // --- Far path: sticky bit (decoder mask on pre-shift data) ---

    wire [INT_BITS-1:0] sticky_one_hot = {{(INT_BITS-1){1'b0}}, 1'b1} << s2_align_shift;
    wire [INT_BITS-1:0] sticky_mask = sticky_one_hot - 1;
    wire far_sticky_s2 = |($unsigned(s2_small_ext) & sticky_mask);

    // --- S2→S3 pipeline registers ---
    // Both paths feed the shared barrel in stage 3.
    // Close: barrel input = close_sum, shift = close_norm_shift, direction = LEFT
    // Far: barrel input = small_ext, shift = align_shift, direction = RIGHT
    // No big_ext forwarding — reconstructed from big_frac in stage 4.
    reg signed [INT_BITS-1:0]    s3_close_sum;
    reg [$clog2(INT_BITS):0]     s3_close_leading;
    reg [$clog2(INT_BITS):0]     s3_close_norm_shift;
    reg                           s3_close_is_zero;
    reg signed [INT_BITS-1:0]    s3_small_ext;      // far barrel input
    reg [$clog2(FRAC_BITS)-1:0]  s3_align_shift;    // far barrel shift amount
    reg                           s3_far_sticky;
    reg signed [EXP_BITS-1:0]    s3_big_exp;
    reg signed [FRAC_BITS-1:0]   s3_big_frac;
    reg                           s3_negligible, s3_is_close;

    always @(posedge clk) begin
        // Close path
        s3_close_sum        <= s2_close_sum;
        s3_close_leading    <= close_leading_s2;
        s3_close_norm_shift <= close_norm_shift_s2;
        s3_close_is_zero    <= s2_close_is_zero;
        // Far path
        s3_small_ext        <= s2_small_ext;
        s3_align_shift      <= s2_align_shift;
        s3_far_sticky       <= far_sticky_s2;
        // Forwarded (no big_ext — reconstructed from big_frac in stage 4)
        s3_big_exp          <= s2_big_exp;
        s3_big_frac         <= s2_big_frac;
        s3_negligible       <= s2_negligible;
        s3_is_close         <= s2_is_close;
    end

    // =========================================================================
    // STAGE 3: Shared Barrel Shifter
    // One barrel shifter, always right shift. For close path (left shift),
    // bit-reverse input and output — free wiring on FPGA.
    //
    // Close: bit_rev(close_sum) >>> close_norm_shift → bit_rev(result)
    //        Equivalent to: close_sum << close_norm_shift
    // Far:   small_ext >>> align_shift (standard alignment right shift)
    // =========================================================================

    // Bit-reverse function for close path (free wiring, no LUTs)
    wire [INT_BITS-1:0] close_sum_rev;
    genvar bri;
    generate
        for (bri = 0; bri < INT_BITS; bri = bri + 1) begin : bitrev_in
            assign close_sum_rev[bri] = s3_close_sum[INT_BITS - 1 - bri];
        end
    endgenerate

    // Mux barrel inputs: close (bit-reversed, unsigned) or far (signed)
    localparam BARREL_BITS = $clog2(INT_BITS);

    wire [INT_BITS-1:0] barrel_in = s3_is_close
        ? close_sum_rev
        : $unsigned(s3_small_ext);
    wire barrel_signed = !s3_is_close; // far path needs sign extension
    wire [BARREL_BITS-1:0] barrel_shift = s3_is_close
        ? s3_close_norm_shift[BARREL_BITS-1:0]
        : {{(BARREL_BITS - $clog2(FRAC_BITS)){1'b0}}, s3_align_shift};

    // Fill bit: signed extension for far, zero-fill for close
    wire fill_bit = barrel_signed & barrel_in[INT_BITS-1];
    wire [INT_BITS-1:0] barrel_out;

    // Manual barrel shifter stages for clean synthesis
    wire [INT_BITS-1:0] bs0, bs1, bs2, bs3, bs4;
    assign bs0 = barrel_in;
    assign bs1 = barrel_shift[0] ? {{1{fill_bit}}, bs0[INT_BITS-1:1]} : bs0;
    assign bs2 = barrel_shift[1] ? {{2{fill_bit}}, bs1[INT_BITS-1:2]} : bs1;
    assign bs3 = barrel_shift[2] ? {{4{fill_bit}}, bs2[INT_BITS-1:4]} : bs2;
    assign bs4 = barrel_shift[3] ? {{8{fill_bit}}, bs3[INT_BITS-1:8]} : bs3;
    assign barrel_out = barrel_shift[4] ? {{16{fill_bit}}, bs4[INT_BITS-1:16]} : bs4;

    // Bit-reverse barrel output for close path (free wiring)
    wire [INT_BITS-1:0] barrel_out_rev;
    generate
        for (bri = 0; bri < INT_BITS; bri = bri + 1) begin : bitrev_out
            assign barrel_out_rev[bri] = barrel_out[INT_BITS - 1 - bri];
        end
    endgenerate

    // --- S3→S4 pipeline registers ---
    // No big_ext — reconstructed from big_frac in stage 4 (saves 27 FFs).
    reg [INT_BITS-1:0]           s4_close_normalized;
    reg [$clog2(INT_BITS):0]     s4_close_leading;
    reg                           s4_close_is_zero;
    reg signed [INT_BITS-1:0]    s4_far_aligned;
    reg                           s4_far_sticky;
    reg signed [EXP_BITS-1:0]    s4_big_exp;
    reg signed [FRAC_BITS-1:0]   s4_big_frac;
    reg                           s4_negligible, s4_is_close;

    always @(posedge clk) begin
        s4_close_normalized <= barrel_out_rev;
        s4_close_leading    <= s3_close_leading;
        s4_close_is_zero    <= s3_close_is_zero;
        s4_far_aligned      <= barrel_out;
        s4_far_sticky       <= s3_far_sticky;
        s4_big_exp          <= s3_big_exp;
        s4_big_frac         <= s3_big_frac;
        s4_negligible       <= s3_negligible;
        s4_is_close         <= s3_is_close;
    end

    // =========================================================================
    // STAGE 4: Finish
    //   Close: extract frac from normalized, compute exponent.
    //   Far: add big_ext + aligned, bounded normalize, banker's round, exp.
    //       big_ext reconstructed from big_frac: $signed(frac) <<< 1 = free wiring.
    //   Output mux.
    // =========================================================================

    // --- Rounding overflow constants ---
    localparam signed [FRAC_BITS-1:0] POS_HALF_S4   = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_S4    = {1'b1, {(FRAC_BITS-1){1'b0}}};

    // --- Close path: extract + round + exp ---
    wire signed [FRAC_BITS-1:0] close_out_frac_raw = s4_close_normalized[INT_BITS-1 -: FRAC_BITS];

    // Close-path banker's rounding on the 2 bits below FRAC extraction
    wire close_guard  = s4_close_normalized[INT_BITS - 1 - FRAC_BITS];
    wire close_clsb   = s4_close_normalized[INT_BITS - FRAC_BITS];
    wire close_sticky = (INT_BITS - 2 - FRAC_BITS >= 0) ?
                        |s4_close_normalized[INT_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire close_round_up = close_guard & (close_sticky | close_clsb);
    wire signed [FRAC_BITS-1:0] close_out_frac_rnd = close_out_frac_raw
                                                     + {{(FRAC_BITS-1){1'b0}}, close_round_up};

    // Rounding overflow: two cases
    //   Positive wrap: 01..1 + 1 → 10..0 (sign flips 0→1). Fix: POS_HALF, exp+1
    //   Negative exit: 10..1 + 1 → 11..0 (top bits same). Fix: NEG_ONE, exp-1
    wire close_rovf_pos = !close_out_frac_raw[FRAC_BITS-1] & close_out_frac_rnd[FRAC_BITS-1];
    wire close_rovf_neg = close_out_frac_rnd[FRAC_BITS-1] & close_out_frac_rnd[FRAC_BITS-2];
    wire signed [FRAC_BITS-1:0] close_out_frac = close_rovf_pos ? POS_HALF_S4 :
                                                   close_rovf_neg ? NEG_ONE_S4  :
                                                   close_out_frac_rnd;

    wire signed [EXP_BITS:0] close_exp_wide = $signed({s4_big_exp[EXP_BITS-1], s4_big_exp})
                                              + 2 - $signed({{1'b0}, s4_close_leading});
    wire signed [EXP_BITS-1:0] close_out_exp_base = close_exp_wide[EXP_BITS-1:0];
    wire signed [EXP_BITS-1:0] close_out_exp = close_rovf_pos ? (close_out_exp_base + 1) :
                                                close_rovf_neg ? (close_out_exp_base - 1) :
                                                close_out_exp_base;

    wire signed [EXP_BITS-1:0] close_offset = close_out_exp - 1;
    wire close_underflow = s4_big_exp[EXP_BITS-1] && !close_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] close_underflow_frac = {s4_close_normalized[INT_BITS-1],
                                                         s4_close_normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // --- Far path: add + bounded normalize + round ---

    // Reconstruct big_ext from big_frac (free wiring: sign-extend + shift left 1)
    wire signed [INT_BITS-1:0] big_ext_s4 = $signed(s4_big_frac) <<< 1;
    wire signed [INT_BITS-1:0] far_sum = big_ext_s4 + $signed(s4_far_aligned);

    // Bounded normalization (leading in {1, 2, 3} → shift in {0, 1, 2})
    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2];
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 :
                                 far_d1 ? 2'd1 : 2'd2;
    wire [$clog2(INT_BITS):0] far_leading = {2'b0, far_norm_shift} + 1;
    wire [INT_BITS-1:0] far_normalized = $unsigned(far_sum) << far_norm_shift;

    // Banker's rounding (alignment sticky + extraction sticky)
    wire far_guard = far_normalized[INT_BITS - 1 - FRAC_BITS];
    wire far_lsb   = far_normalized[INT_BITS - FRAC_BITS];
    wire far_ext_sticky = (INT_BITS - 2 - FRAC_BITS >= 0) ?
                          |far_normalized[INT_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire far_round_up = far_guard & ((s4_far_sticky | far_ext_sticky) | far_lsb);
    wire signed [FRAC_BITS-1:0] far_out_frac_raw = far_normalized[INT_BITS-1 -: FRAC_BITS];
    wire signed [FRAC_BITS-1:0] far_out_frac_rounded = far_out_frac_raw
                                              + {{(FRAC_BITS-1){1'b0}}, far_round_up};

    // Rounding overflow: two cases
    //   Positive wrap: 01..1 + 1 → 10..0 (sign flips 0→1). Fix: POS_HALF, exp+1
    //   Negative exit: 10..1 + 1 → 11..0 (top bits same). Fix: NEG_ONE, exp-1
    wire far_rovf_pos = !far_out_frac_raw[FRAC_BITS-1] & far_out_frac_rounded[FRAC_BITS-1];
    wire far_rovf_neg = far_out_frac_rounded[FRAC_BITS-1] & far_out_frac_rounded[FRAC_BITS-2];

    wire signed [FRAC_BITS-1:0] far_out_frac = far_rovf_pos ? POS_HALF_S4 :
                                                 far_rovf_neg ? NEG_ONE_S4  :
                                                 far_out_frac_rounded;

    // Far exponent, with rounding overflow correction
    wire signed [EXP_BITS:0] far_exp_wide = $signed({s4_big_exp[EXP_BITS-1], s4_big_exp})
                                            + 2 - $signed({{1'b0}, far_leading});
    wire signed [EXP_BITS-1:0] far_out_exp_base = far_exp_wide[EXP_BITS-1:0];
    wire signed [EXP_BITS-1:0] far_out_exp = far_rovf_pos ? (far_out_exp_base + 1) :
                                              far_rovf_neg ? (far_out_exp_base - 1) :
                                              far_out_exp_base;

    // Far underflow
    wire signed [EXP_BITS-1:0] far_offset = far_out_exp - 1;
    wire far_underflow = s4_big_exp[EXP_BITS-1] && !far_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] far_underflow_frac = {far_normalized[INT_BITS-1],
                                                       far_normalized[INT_BITS-1 -: FRAC_BITS-1]};
    wire far_is_zero = (far_sum == 0);

    // --- Output mux ---
    wire use_close = s4_is_close && !s4_negligible;

    wire signed [FRAC_BITS-1:0] path_frac = use_close ? close_out_frac : far_out_frac;
    wire signed [EXP_BITS-1:0]  path_exp  = use_close ? close_out_exp  : far_out_exp;
    wire path_underflow = use_close ? close_underflow : far_underflow;
    wire signed [FRAC_BITS-1:0] path_uf_frac = use_close ? close_underflow_frac
                                                         : far_underflow_frac;
    wire path_is_zero = use_close ? s4_close_is_zero : far_is_zero;

    // Stage 4 output register
    always @(posedge clk) begin
        result_frac <= s4_negligible     ? s4_big_frac :
                       path_is_zero      ? {FRAC_BITS{1'b0}} :
                       path_underflow    ? path_uf_frac :
                                           path_frac;

        result_exp  <= s4_negligible     ? s4_big_exp :
                       path_is_zero      ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                       path_underflow    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                           path_exp;
    end

endmodule
