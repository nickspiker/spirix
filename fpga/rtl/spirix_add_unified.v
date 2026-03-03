// Spirix Addition: a ± b (close/far split, shared barrel, shared rounding)
//
// Parameterized combinational adder/subtractor for Spirix floating-point scalars.
// sub=0: add (a + b).  sub=1: subtract (a - b).
// Subtraction integrated via XOR + carry-in on existing adders — zero extra LUTs.
// Close path (exp_diff <= 1): free align, add, full CLZ, barrel normalize.
// Far path (exp_diff >= 2): barrel align, add, bounded 0-2 bit normalize.
// ONE physical barrel shared: close uses it for normalize (bit-reverse +
// logical right shift), far uses it for alignment (arithmetic right shift).
// Single shared rounding stage at output. Banker's round (RNE) with early
// rounding overflow detection (rovf computed from pre-round signals).
//
// Inputs are N1-normalized signed fractions with signed exponents.
// value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
//
// Valid parameter range: FRAC_BITS 4..29, EXP_BITS 4..16.
// (Barrel is 5 stages; FRAC_BITS > 29 → INT_BITS > 32 needs a 6th stage.)

module spirix_add_unified #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    input  wire                         sub,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam INT_BITS = FRAC_BITS + 3; // frac + G + R + S
    localparam BARREL_BITS = $clog2(INT_BITS);  // shift amount width
    localparam LEAD_BITS   = BARREL_BITS + 1;   // leading count width
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};

    // =========================================================================
    // Step 1: Exponent difference + swap
    // =========================================================================
    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                      - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;

    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;
    wire negligible = (exp_diff >= FRAC_BITS);
    wire is_close = (exp_diff <= 1) && !negligible;

    // Subtraction: negate whichever operand holds b.
    // XOR + carry-in absorbed into existing adder LUT4s (zero extra cost).
    wire negate_small = sub & a_is_big;
    wire negate_big   = sub & !a_is_big;

    // =========================================================================
    // Step 2: Extend
    // =========================================================================
    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac) <<< 2;
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac) <<< 2;

    // =========================================================================
    // Step 3: Close path — 0-1 bit align + add + CLZ
    //   Runs in parallel with far barrel. Only meaningful when is_close.
    // =========================================================================
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

    // =========================================================================
    // Step 4: Shared barrel shifter
    //   Close: bit-reverse(close_sum) → logical right shift → bit-reverse
    //   Far:   small_ext → arithmetic right shift (+ sticky accumulation)
    //   Fill = is_close ? 0 : sign_bit  (mask in zero for logical shift)
    // =========================================================================

    // Bit-reverse close_sum for normalize-via-right-shift (free wiring)
    wire [INT_BITS-1:0] close_sum_rev;
    genvar bi;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_in
            assign close_sum_rev[bi] = close_sum[INT_BITS - 1 - bi];
        end
    endgenerate

    // Far shift amount (clamped to INT_BITS-1)
    wire [BARREL_BITS-1:0] far_shift = (exp_diff >= INT_BITS)
        ? INT_BITS[BARREL_BITS-1:0] - 1 : exp_diff[BARREL_BITS-1:0];

    // Barrel input + shift mux
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

    // Stage 3: shift by 8 (generate guard for small INT_BITS)
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

    // Stage 4: shift by 16 (generate guard for small INT_BITS)
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

    wire [INT_BITS-1:0] barrel_out = b4_val;
    wire barrel_sticky = b4_sticky;

    // Bit-reverse barrel output for close path (free wiring)
    wire [INT_BITS-1:0] close_normalized;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_out
            assign close_normalized[bi] = barrel_out[INT_BITS - 1 - bi];
        end
    endgenerate

    // =========================================================================
    // Step 5: Far path — add + bounded normalize
    //   Far sum can shift by 0, 1, or 2. Three-way mux, no barrel needed.
    // =========================================================================
    wire signed [INT_BITS-1:0] far_aligned = $signed(barrel_out);
    wire signed [INT_BITS-1:0] far_sum = (big_ext ^ {INT_BITS{negate_big}})
                                        + (far_aligned ^ {INT_BITS{negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, sub};
    wire far_is_zero = (far_sum == 0);

    // Bounded normalize: leading is 1, 2, or 3 for N1 inputs with exp_diff >= 2
    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2];
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 : far_d1 ? 2'd1 : 2'd2;
    wire [LEAD_BITS-1:0] far_leading = {{(LEAD_BITS-2){1'b0}}, far_norm_shift} + 1;

    wire [INT_BITS-1:0] far_normalized = far_d0 ? $unsigned(far_sum) :
                                          far_d1 ? ($unsigned(far_sum) << 1) :
                                                   ($unsigned(far_sum) << 2);

    // =========================================================================
    // Step 6: Select + shared rounding
    //   Both paths produce a normalized INT_BITS value. Extract frac + G/R/S,
    //   apply single banker's round, compute exponent.
    // =========================================================================
    wire [INT_BITS-1:0] normalized = is_close ? close_normalized : far_normalized;
    wire [LEAD_BITS-1:0] leading = is_close ? close_leading : far_leading;
    wire path_is_zero = is_close ? close_is_zero : far_is_zero;

    wire signed [FRAC_BITS-1:0] out_frac_raw = normalized[INT_BITS-1 -: FRAC_BITS];

    wire guard     = normalized[INT_BITS - 1 - FRAC_BITS];
    wire lsb       = normalized[INT_BITS - FRAC_BITS];
    wire round_bit = normalized[INT_BITS - 2 - FRAC_BITS];
    // Sticky: barrel sticky only meaningful for far path; close uses align sticky
    wire ext_sticky = |normalized[INT_BITS - 3 - FRAC_BITS:0];
    wire align_sticky = is_close ? close_align_sticky : barrel_sticky;
    wire sticky    = ext_sticky | align_sticky;
    wire round_up  = guard & (round_bit | sticky | lsb);

    wire signed [FRAC_BITS-1:0] out_frac_rounded = out_frac_raw
                                                     + {{(FRAC_BITS-1){1'b0}}, round_up};

    // Rounding overflow — early detection from pre-round signals only.
    // rovf_pos: positive frac at max (01111...1), round carry flips sign bit
    // rovf_neg: negative N1 frac at max magnitude (10111...1), round breaks N1
    // Both computable immediately after normalization, before rounding adder.
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
    // Step 7: Exponent
    // =========================================================================
    wire signed [EXP_BITS:0] exp_wide = $signed({big_exp[EXP_BITS-1], big_exp})
                                        + 2 - $signed({{1'b0}, leading})
                                        + {{EXP_BITS{1'b0}}, rovf_pos}
                                        - {{EXP_BITS{1'b0}}, rovf_neg};
    wire signed [EXP_BITS-1:0] out_exp = exp_wide[EXP_BITS-1:0];

    // Underflow detection
    wire signed [EXP_BITS-1:0] offset = out_exp - 1;
    wire underflow = big_exp[EXP_BITS-1] && !offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] underflow_frac = {normalized[INT_BITS-1],
                                                     normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // =========================================================================
    // Step 8: Output
    //   Negligible bypass only when b is small (negate_small side); when b is
    //   big and subtracted, -big_frac may not be N1, so let the far path
    //   normalize it properly.
    // =========================================================================
    wire use_negligible = negligible & !negate_big;

    assign result_frac = use_negligible ? big_frac :
                         path_is_zero   ? {FRAC_BITS{1'b0}} :
                         underflow      ? underflow_frac :
                                          out_frac;

    assign result_exp  = use_negligible ? big_exp :
                         path_is_zero   ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                         underflow      ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                          out_exp;

endmodule
