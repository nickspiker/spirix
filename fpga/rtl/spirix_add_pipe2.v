// Spirix Addition: a ± b — 2-stage pipelined Close/Far split
// Shared barrel shifter, banker's rounding, early rovf detection.
// sub=0: add (a + b).  sub=1: subtract (a - b).
// Subtraction integrated via XOR + carry-in on existing adders — zero extra LUTs.
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Pipeline stages:
//   Stage 1 (Prepare + Barrel):
//       Swap: exp subtract → |diff| → mux big/small.
//       Close: 1-bit align + add + CLZ → compute norm shift.
//       Far: compute align shift.
//       Shared barrel: close normalizes (bit-reverse right shift),
//           far aligns (arithmetic right shift + sticky accumulation).
//       Only one path's barrel result is meaningful per cycle.
//   Stage 2 (Finish):
//       Close: extract frac, banker's round, early rovf, exp compute.
//       Far: add big_ext + aligned, bounded normalize (0-2 bit shift),
//           banker's round, early rovf, exp compute.
//       big_ext reconstructed from big_frac (free wiring: frac <<< 2).
//       Output mux.
//
// Valid parameter range: FRAC_BITS 4..29, EXP_BITS 4..16.
// (Barrel is 5 stages; FRAC_BITS > 29 → INT_BITS > 32 needs a 6th stage.)
//
// Latency: 2 clock cycles. Throughput: 1 result per clock cycle.

module spirix_add_pipe2 #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    input  wire                         sub,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam INT_BITS = FRAC_BITS + 3; // frac + G + R + S
    localparam BARREL_BITS = $clog2(INT_BITS);  // shift amount width
    localparam LEAD_BITS   = BARREL_BITS + 1;   // leading count width
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};

    // =========================================================================
    // STAGE 1: Swap + Close Add + CLZ + Shared Barrel
    //
    // The full close path through the barrel completes here. The far path
    // uses the barrel for alignment. Both paths produce a barrel_out that
    // feeds the stage 1→2 register.
    //
    // Critical path: exp_sub → swap_mux → close_add → CLZ → barrel_mux
    //     → barrel stages (5 levels).
    // =========================================================================

    // --- Swap ---
    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                       - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;

    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;
    wire negligible = (exp_diff >= FRAC_BITS);
    wire is_close   = (exp_diff <= 1) && !negligible;

    // Subtraction: negate whichever operand holds b.
    wire negate_small = sub & a_is_big;
    wire negate_big   = sub & !a_is_big;

    // --- Extend ---
    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac) <<< 2;
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac) <<< 2;

    // --- Close path: 1-bit align + add + CLZ ---
    wire signed [INT_BITS-1:0] close_small = exp_diff[0] ? (small_ext >>> 1) : small_ext;
    wire close_align_sticky = exp_diff[0] & small_ext[0];
    wire signed [INT_BITS-1:0] close_sum = (big_ext ^ {INT_BITS{negate_big}})
                                        + (close_small ^ {INT_BITS{negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, sub};
    wire close_is_zero = (close_sum == 0);

    // CLZ via XOR-adjacent + compress-by-2
    localparam DIFF_W = INT_BITS - 1;
    wire [DIFF_W-1:0] xor_diff = close_sum[INT_BITS-1:1] ^ close_sum[INT_BITS-2:0];

    localparam COMP_W = (DIFF_W + 1) / 2;
    wire [COMP_W-1:0] compressed;
    genvar gi;
    generate
        for (gi = 0; gi < COMP_W; gi = gi + 1) begin : compress
            if (DIFF_W - 1 - 2*gi - 1 >= 0)
                assign compressed[COMP_W-1-gi] = xor_diff[DIFF_W-1-2*gi] | xor_diff[DIFF_W-1-2*gi-1];
            else
                assign compressed[0] = xor_diff[0];
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
    wire fixup_bit = xor_diff[DIFF_W - 1 - coarse_pos];
    wire [$clog2(DIFF_W):0] actual_clz = fixup_bit ? coarse_pos : (coarse_pos + 1);
    wire [LEAD_BITS-1:0] close_leading = actual_clz + 1;
    wire [BARREL_BITS-1:0] close_norm_shift = close_leading - 1;

    // --- Shared barrel shifter ---
    // Close: bit-reverse(close_sum) → logical right shift → bit-reverse
    // Far:   small_ext → arithmetic right shift (+ sticky accumulation)

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
    wire [INT_BITS-1:0] b3_val;
    wire b3_sticky;
    generate if (INT_BITS > 8) begin : gen_b3
        wire fill3 = !is_close & b2_val[INT_BITS-1];
        assign b3_val = barrel_shift[3] ? {{8{fill3}}, b2_val[INT_BITS-1:8]} : b2_val;
        assign b3_sticky = b2_sticky | (barrel_shift[3] & |b2_val[7:0]);
    end else begin : gen_b3
        wire fill3 = !is_close & b2_val[INT_BITS-1];
        assign b3_val = barrel_shift[3] ? {INT_BITS{fill3}} : b2_val;
        assign b3_sticky = b2_sticky | (barrel_shift[3] & |b2_val);
    end endgenerate

    // Stage 4: shift by 16
    wire [INT_BITS-1:0] b4_val;
    wire b4_sticky;
    generate if (INT_BITS > 16) begin : gen_b4
        wire fill4 = !is_close & b3_val[INT_BITS-1];
        assign b4_val = barrel_shift[4] ? {{16{fill4}}, b3_val[INT_BITS-1:16]} : b3_val;
        assign b4_sticky = b3_sticky | (barrel_shift[4] & |b3_val[15:0]);
    end else begin : gen_b4
        wire fill4 = !is_close & b3_val[INT_BITS-1];
        assign b4_val = barrel_shift[4] ? {INT_BITS{fill4}} : b3_val;
        assign b4_sticky = b3_sticky | (barrel_shift[4] & |b3_val);
    end endgenerate

    // Bit-reverse barrel output for close path (free wiring)
    wire [INT_BITS-1:0] close_normalized;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_out
            assign close_normalized[bi] = b4_val[INT_BITS - 1 - bi];
        end
    endgenerate

    // =========================================================================
    // S1→S2 pipeline registers
    //
    // Close: close_normalized (barrel output, bit-reversed), close_leading,
    //        close_is_zero, close_align_sticky.
    // Far:   barrel_out (aligned small), barrel_sticky.
    // Shared: big_frac (reconstruct big_ext in S2), big_exp, is_close,
    //         negligible.
    // =========================================================================

    reg [INT_BITS-1:0]          s2_close_normalized;
    reg [LEAD_BITS-1:0]         s2_close_leading;
    reg                          s2_close_is_zero;
    reg                          s2_close_align_sticky;
    reg signed [INT_BITS-1:0]   s2_far_aligned;
    reg                          s2_far_sticky;
    reg signed [FRAC_BITS-1:0]  s2_big_frac;
    reg signed [EXP_BITS-1:0]   s2_big_exp;
    reg                          s2_is_close;
    reg                          s2_negligible;
    reg                          s2_negate_big;
    reg                          s2_negate_small;
    reg                          s2_sub;

    always @(posedge clk) begin
        s2_close_normalized  <= close_normalized;
        s2_close_leading     <= close_leading;
        s2_close_is_zero     <= close_is_zero;
        s2_close_align_sticky<= close_align_sticky;
        s2_far_aligned       <= $signed(b4_val);
        s2_far_sticky        <= b4_sticky;
        s2_big_frac          <= big_frac;
        s2_big_exp           <= big_exp;
        s2_is_close          <= is_close;
        s2_negligible        <= negligible;
        s2_negate_big        <= negate_big;
        s2_negate_small      <= negate_small;
        s2_sub               <= sub;
    end

    // =========================================================================
    // STAGE 2: Finish
    //
    // Close: extract frac + GRS bits, banker's round, early rovf, exp.
    // Far: reconstruct big_ext, add + bounded normalize, round, early rovf, exp.
    // Output mux selects close/far/negligible/zero.
    //
    // Critical path (far): big_ext_reconstruct(free) → far_add(carry chain)
    //     → bounded normalize(mux) → round → rovf → exp → output mux.
    // =========================================================================

    // --- Close path: extract + round ---

    wire signed [FRAC_BITS-1:0] close_frac_raw = s2_close_normalized[INT_BITS-1 -: FRAC_BITS];
    wire close_guard     = s2_close_normalized[INT_BITS - 1 - FRAC_BITS];
    wire close_lsb       = s2_close_normalized[INT_BITS - FRAC_BITS];
    wire close_round_bit = s2_close_normalized[INT_BITS - 2 - FRAC_BITS];
    wire close_ext_sticky = |s2_close_normalized[INT_BITS - 3 - FRAC_BITS:0];
    wire close_sticky     = close_ext_sticky | s2_close_align_sticky;
    wire close_round_up   = close_guard & (close_round_bit | close_sticky | close_lsb);

    wire signed [FRAC_BITS-1:0] close_frac_rounded = close_frac_raw
                                                    + {{(FRAC_BITS-1){1'b0}}, close_round_up};

    // Early rovf detection (pre-round signals)
    wire close_rovf_pos = !close_frac_raw[FRAC_BITS-1]
                        & (&close_frac_raw[FRAC_BITS-2:0])
                        & close_round_up;
    wire close_rovf_neg = close_frac_raw[FRAC_BITS-1]
                        & !close_frac_raw[FRAC_BITS-2]
                        & (&close_frac_raw[FRAC_BITS-3:0])
                        & close_round_up;

    wire signed [FRAC_BITS-1:0] close_out_frac = close_rovf_pos ? POS_HALF :
                                                   close_rovf_neg ? NEG_ONE  :
                                                   close_frac_rounded;

    wire signed [EXP_BITS:0] close_exp_wide = $signed({s2_big_exp[EXP_BITS-1], s2_big_exp})
                                              + 2 - $signed({{1'b0}, s2_close_leading})
                                              + {{EXP_BITS{1'b0}}, close_rovf_pos}
                                              - {{EXP_BITS{1'b0}}, close_rovf_neg};
    wire signed [EXP_BITS-1:0] close_out_exp = close_exp_wide[EXP_BITS-1:0];

    // Close underflow
    wire signed [EXP_BITS-1:0] close_offset = close_out_exp - 1;
    wire close_underflow = s2_big_exp[EXP_BITS-1] && !close_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] close_uf_frac = {s2_close_normalized[INT_BITS-1],
                                                   s2_close_normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // --- Far path: add + bounded normalize + round ---

    // Reconstruct big_ext from big_frac (free wiring: sign-extend + shift left 2)
    wire signed [INT_BITS-1:0] big_ext_s2 = $signed(s2_big_frac) <<< 2;
    wire signed [INT_BITS-1:0] far_sum = (big_ext_s2 ^ {INT_BITS{s2_negate_big}})
                                        + (s2_far_aligned ^ {INT_BITS{s2_negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, s2_sub};  // s2_sub forwarded directly
    wire far_is_zero = (far_sum == 0);

    // Bounded normalize: leading is 1, 2, or 3
    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2];
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 : far_d1 ? 2'd1 : 2'd2;
    wire [LEAD_BITS-1:0] far_leading = {{(LEAD_BITS-2){1'b0}}, far_norm_shift} + 1;

    wire [INT_BITS-1:0] far_normalized = far_d0 ? $unsigned(far_sum) :
                                          far_d1 ? ($unsigned(far_sum) << 1) :
                                                   ($unsigned(far_sum) << 2);

    // Far rounding
    wire signed [FRAC_BITS-1:0] far_frac_raw = far_normalized[INT_BITS-1 -: FRAC_BITS];
    wire far_guard     = far_normalized[INT_BITS - 1 - FRAC_BITS];
    wire far_lsb       = far_normalized[INT_BITS - FRAC_BITS];
    wire far_round_bit = far_normalized[INT_BITS - 2 - FRAC_BITS];
    wire far_ext_sticky = |far_normalized[INT_BITS - 3 - FRAC_BITS:0];
    wire far_sticky     = far_ext_sticky | s2_far_sticky;
    wire far_round_up   = far_guard & (far_round_bit | far_sticky | far_lsb);

    wire signed [FRAC_BITS-1:0] far_frac_rounded = far_frac_raw
                                                  + {{(FRAC_BITS-1){1'b0}}, far_round_up};

    // Early rovf detection
    wire far_rovf_pos = !far_frac_raw[FRAC_BITS-1]
                      & (&far_frac_raw[FRAC_BITS-2:0])
                      & far_round_up;
    wire far_rovf_neg = far_frac_raw[FRAC_BITS-1]
                      & !far_frac_raw[FRAC_BITS-2]
                      & (&far_frac_raw[FRAC_BITS-3:0])
                      & far_round_up;

    wire signed [FRAC_BITS-1:0] far_out_frac = far_rovf_pos ? POS_HALF :
                                                 far_rovf_neg ? NEG_ONE  :
                                                 far_frac_rounded;

    wire signed [EXP_BITS:0] far_exp_wide = $signed({s2_big_exp[EXP_BITS-1], s2_big_exp})
                                            + 2 - $signed({{1'b0}, far_leading})
                                            + {{EXP_BITS{1'b0}}, far_rovf_pos}
                                            - {{EXP_BITS{1'b0}}, far_rovf_neg};
    wire signed [EXP_BITS-1:0] far_out_exp = far_exp_wide[EXP_BITS-1:0];

    // Far underflow
    wire signed [EXP_BITS-1:0] far_offset = far_out_exp - 1;
    wire far_underflow = s2_big_exp[EXP_BITS-1] && !far_offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] far_uf_frac = {far_normalized[INT_BITS-1],
                                                 far_normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // --- Output mux ---
    wire use_close = s2_is_close && !s2_negligible;

    wire signed [FRAC_BITS-1:0] path_frac = use_close ? close_out_frac : far_out_frac;
    wire signed [EXP_BITS-1:0]  path_exp  = use_close ? close_out_exp  : far_out_exp;
    wire path_underflow = use_close ? close_underflow : far_underflow;
    wire signed [FRAC_BITS-1:0] path_uf_frac = use_close ? close_uf_frac : far_uf_frac;
    wire path_is_zero = use_close ? s2_close_is_zero : far_is_zero;

    // Negligible bypass only when b is small; when b is big and subtracted,
    // -big_frac may not be N1 — let the far path normalize it.
    wire s2_use_negligible = s2_negligible & !s2_negate_big;

    // Stage 2 output register
    always @(posedge clk) begin
        result_frac <= s2_use_negligible ? s2_big_frac :
                       path_is_zero      ? {FRAC_BITS{1'b0}} :
                       path_underflow    ? path_uf_frac :
                                           path_frac;

        result_exp  <= s2_use_negligible ? s2_big_exp :
                       path_is_zero      ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                       path_underflow    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                           path_exp;
    end

endmodule
