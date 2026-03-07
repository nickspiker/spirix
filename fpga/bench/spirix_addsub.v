// spirix_addsub — Combinational add/subtract for Spirix scalars
//
// Computes a + b (sub=0) or a - b (sub=1) on N1-normalized signed fractions
// with signed exponents. Fully parameterized, purely combinational.
//
// Architecture: edge case detection + close/far split with one shared
//   barrel shifter. Edge cases match the Rust reference model exactly.
//   Close (|exp_diff| <= 1): 0-1 bit align, add, full CLZ, barrel normalize.
//   Far   (|exp_diff| >= 2): barrel align, add, bounded 0-2 bit normalize.
//   One physical barrel: close bit-reverses through it (normalize as right
//   shift), far uses it directly (arithmetic right shift + sticky).
//   Shared rounding: banker's round (RNE) with early rounding-overflow
//   detection (rovf from pre-round signals, no post-round compare).
//
// Subtraction: XOR + carry-in on existing adders. After the exponent-based
// swap, b may land in either the big or small position. negate_small and
// negate_big track which operand holds b and apply one's-complement
// inversion; the single carry-in (+sub) completes two's complement.
// On ECP5, the XOR folds into the LUT4 already feeding the CCU2C carry
// chain — zero extra LUTs in the ideal case.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/underflow.
//
// Valid parameter range: FRAC_BITS 4..29, EXP_BITS 4..16.
// (Barrel is 5 stages; FRAC_BITS > 29 needs a 6th stage.)

module spirix_addsub #(
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
    localparam INT_BITS = FRAC_BITS + 3; // frac + guard + round + sticky
    localparam BARREL_BITS = $clog2(INT_BITS);  // shift amount width
    localparam LEAD_BITS   = BARREL_BITS + 1;   // leading count width
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_SMALL = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_SMALL = {2'b11, {(FRAC_BITS-2){1'b0}}};

    // Undefined prefix constants (top 8 bits, zero-padded)
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_TF  = {8'h1F, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_TF  = {8'hE0, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_P_VAN = {8'h1E, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_M_VAN = {8'hE1, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_FIN  = {8'h1C, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_FIN  = {8'hE3, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_P_TF  = {8'h18, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_M_TF  = {8'hE7, {UPAD{1'b0}}};

    // =========================================================================
    // Step 0: Edge case detection (matches Rust scalar_add/subtract_scalar)
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

    wire a_is_zero   = a_is_ambig & a_frac_zero;
    wire a_is_inf    = a_is_ambig & a_frac_neg1;
    wire a_exploded  = a_is_ambig & a_n1;
    wire a_transf    = a_is_inf | a_exploded;
    wire a_vanished  = a_n2;
    wire a_undef     = ~a_n0 & a_top3;

    wire b_is_zero   = b_is_ambig & b_frac_zero;
    wire b_is_inf    = b_is_ambig & b_frac_neg1;
    wire b_exploded  = b_is_ambig & b_n1;
    wire b_transf    = b_is_inf | b_exploded;
    wire b_vanished  = b_n2;
    wire b_undef     = ~b_n0 & b_top3;

    wire a_is_normal  = ~a_is_ambig & a_n1;
    wire b_is_normal  = ~b_is_ambig & b_n1;
    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // Spirix negation of b (for sub edge cases)
    wire b_is_pos_half  = (b_frac == POS_HALF);
    wire b_is_neg_one_f = (b_frac == NEG_ONE);
    wire b_is_pos_small = (b_frac == POS_SMALL);
    wire b_is_neg_small = (b_frac == NEG_SMALL);

    // Normal negation (b has non-AMBIG exp)
    wire signed [EXP_BITS-1:0] b_exp_m1 = b_exp - 1'b1;
    wire b_exp_m1_ambig = (b_exp_m1 == AMBIGUOUS_EXP[EXP_BITS-1:0]);

    wire signed [FRAC_BITS-1:0] neg_b_frac_normal =
        b_is_pos_half  ? (b_exp_m1_ambig ? NEG_SMALL : NEG_ONE) :
        b_is_neg_one_f ? POS_HALF :
                         -b_frac;
    wire signed [EXP_BITS-1:0] neg_b_exp_normal =
        b_is_pos_half  ? b_exp_m1 :
        b_is_neg_one_f ? (b_exp + 1'b1) :
                         b_exp;

    // Non-normal negation (b has AMBIG exp)
    wire b_top3_same = (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                       (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);
    wire b_nonnorm_nochange = (b_frac_zero | b_frac_neg1) | (~(b_frac_zero | b_frac_neg1) & b_top3_same);

    wire signed [FRAC_BITS-1:0] neg_b_frac_nonnorm =
        b_nonnorm_nochange ? b_frac :
        b_is_pos_half      ? NEG_ONE :
        b_is_neg_one_f     ? POS_HALF :
        b_is_pos_small     ? NEG_SMALL :
        b_is_neg_small     ? POS_SMALL :
                             -b_frac;

    wire signed [FRAC_BITS-1:0] neg_b_frac = b_is_ambig ? neg_b_frac_nonnorm : neg_b_frac_normal;
    wire signed [EXP_BITS-1:0]  neg_b_exp  = b_is_ambig ? b_exp               : neg_b_exp_normal;

    // Edge case priority chain
    wire sc_a_undef   = a_undef;
    wire sc_b_undef   = ~a_undef & b_undef;
    wire sc_tf_tf     = ~a_undef & ~b_undef & a_transf & b_transf;
    wire sc_van_van   = ~a_undef & ~b_undef & ~sc_tf_tf & a_vanished & b_vanished;
    wire sc_a_transf  = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van & a_transf;
    wire sc_b_transf  = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van & ~sc_a_transf & b_transf;
    wire sc_a_van     = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & a_vanished;
    wire sc_b_van     = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_van & b_vanished;
    wire sc_a_zero    = ~a_undef & ~b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_van & ~sc_b_van & a_is_zero;
    wire sc_fallback  = any_non_normal & ~sc_a_undef & ~sc_b_undef & ~sc_tf_tf & ~sc_van_van &
                        ~sc_a_transf & ~sc_b_transf & ~sc_a_van & ~sc_b_van & ~sc_a_zero;

    wire shortcut = any_non_normal & (sc_a_undef | sc_b_undef | sc_tf_tf | sc_van_van |
                                      sc_a_transf | sc_b_transf | sc_a_van | sc_b_van |
                                      sc_a_zero | sc_fallback);

    // Shortcut fraction (add vs sub selects different undefined prefixes)
    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_tf_tf    ? (sub ? UNDEF_TF_M_TF  : UNDEF_TF_P_TF) :
        sc_van_van  ? (sub ? UNDEF_VAN_M_VAN : UNDEF_VAN_P_VAN) :
        sc_a_transf ? (sub ? UNDEF_TF_M_FIN  : UNDEF_TF_P_FIN) :
        sc_b_transf ? (sub ? UNDEF_FIN_M_TF  : UNDEF_FIN_P_TF) :
        sc_a_van    ? (sub ? neg_b_frac : b_frac) :
        sc_b_van    ? a_frac :
        sc_a_zero   ? (sub ? neg_b_frac : b_frac) :
                      a_frac;

    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
        sc_tf_tf    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        sc_van_van  ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        sc_a_transf ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        sc_b_transf ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
        sc_a_van    ? (sub ? neg_b_exp : b_exp) :
        sc_b_van    ? a_exp :
        sc_a_zero   ? (sub ? neg_b_exp : b_exp) :
                      a_exp;

    // =========================================================================
    // Step 1: Exponent difference + swap
    //
    // Sign-extend both exponents by 1 bit to subtract without overflow.
    // |raw_diff| gives the unsigned distance; its sign picks big vs small.
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

    // Subtraction: negate whichever operand holds b after swap.
    wire negate_small = sub & a_is_big;
    wire negate_big   = sub & !a_is_big;

    // =========================================================================
    // Step 2: Extend to internal width (frac << 2 makes room for G, R, S)
    // =========================================================================
    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac) <<< 2;
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac) <<< 2;

    // =========================================================================
    // Step 3: Close path — 0-1 bit align + add + CLZ
    //
    // When exponents differ by at most 1, massive cancellation is possible,
    // so a full count-leading-zeros finds the normalization shift.
    // The close add and CLZ run in parallel with the far barrel.
    // =========================================================================
    wire signed [INT_BITS-1:0] close_small = exp_diff[0] ? (small_ext >>> 1) : small_ext;
    wire close_align_sticky = exp_diff[0] & small_ext[0];
    wire signed [INT_BITS-1:0] close_sum = (big_ext ^ {INT_BITS{negate_big}})
                                        + (close_small ^ {INT_BITS{negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, sub};
    wire close_is_zero = (close_sum == 0);

    // CLZ: XOR adjacent bits to find first sign-bit boundary, then priority
    // encode to get the leading count. Simple tree — ~3-4 LUT levels for 27 bits.
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
    // Step 4: Shared barrel shifter
    //
    // Close: bit-reverse → logical right shift → bit-reverse = left shift.
    //        Fill bits are zero (logical shift via is_close masking).
    // Far:   arithmetic right shift with sticky-bit accumulation.
    //        Fill bits replicate the sign (arithmetic shift).
    // One physical barrel; is_close selects input, shift amount, and fill.
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

    // Barrel stage 0: shift by 1
    wire fill0 = !is_close & barrel_in[INT_BITS-1];
    wire [INT_BITS-1:0] b0_val = barrel_shift[0] ? {fill0, barrel_in[INT_BITS-1:1]} : barrel_in;
    wire b0_sticky = barrel_shift[0] & barrel_in[0];

    // Barrel stage 1: shift by 2
    wire fill1 = !is_close & b0_val[INT_BITS-1];
    wire [INT_BITS-1:0] b1_val = barrel_shift[1] ? {{2{fill1}}, b0_val[INT_BITS-1:2]} : b0_val;
    wire b1_sticky = b0_sticky | (barrel_shift[1] & |b0_val[1:0]);

    // Barrel stage 2: shift by 4
    wire fill2 = !is_close & b1_val[INT_BITS-1];
    wire [INT_BITS-1:0] b2_val = barrel_shift[2] ? {{4{fill2}}, b1_val[INT_BITS-1:4]} : b1_val;
    wire b2_sticky = b1_sticky | (barrel_shift[2] & |b1_val[3:0]);

    // Barrel stage 3: shift by 8
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

    // Barrel stage 4: shift by 16
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

    // Bit-reverse barrel output for close path (free wiring, no logic)
    wire [INT_BITS-1:0] close_normalized;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_out
            assign close_normalized[bi] = barrel_out[INT_BITS - 1 - bi];
        end
    endgenerate

    // =========================================================================
    // Step 5: Far path — add + bounded normalize
    //
    // The far adder combines big_ext with the barrel-aligned small operand.
    // Subtraction uses the same XOR + carry-in as the close path.
    // The result needs at most a 0-2 bit left shift to reach N1 form.
    // =========================================================================
    wire signed [INT_BITS-1:0] far_aligned = $signed(barrel_out);
    wire signed [INT_BITS-1:0] far_sum = (big_ext ^ {INT_BITS{negate_big}})
                                        + (far_aligned ^ {INT_BITS{negate_small}})
                                        + {{(INT_BITS-1){1'b0}}, sub};
    wire far_is_zero = (far_sum == 0);

    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2];
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 : far_d1 ? 2'd1 : 2'd2;
    wire [LEAD_BITS-1:0] far_leading = {{(LEAD_BITS-2){1'b0}}, far_norm_shift} + 1;

    wire [INT_BITS-1:0] far_normalized = far_d0 ? $unsigned(far_sum) :
                                          far_d1 ? ($unsigned(far_sum) << 1) :
                                                   ($unsigned(far_sum) << 2);

    // =========================================================================
    // Step 6: Shared rounding (banker's round / RNE)
    //
    // Both paths produce a normalized INT_BITS-wide value. Extract the
    // FRAC_BITS fraction and the guard/round/sticky bits below it.
    // Round up when guard=1 AND (round | sticky | lsb).
    //
    // Rounding overflow (rovf): the fraction is at its maximum representable
    // value and the +1 from rounding pushes it out of N1 range. Detected
    // from pre-round signals and corrected to POS_HALF/NEG_ONE with an
    // exponent bump.
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
    // Step 7: Exponent
    //
    // big_exp + 2 accounts for the <<< 2 extension. Subtract leading count
    // for normalization. Adjust +1/-1 for rounding overflow.
    // =========================================================================
    wire signed [EXP_BITS:0] exp_wide = $signed({big_exp[EXP_BITS-1], big_exp})
                                        + 2 - $signed({{1'b0}, leading})
                                        + {{EXP_BITS{1'b0}}, rovf_pos}
                                        - {{EXP_BITS{1'b0}}, rovf_neg};
    wire signed [EXP_BITS-1:0] out_exp = exp_wide[EXP_BITS-1:0];

    // Underflow: result exponent wrapped past the minimum representable value.
    wire signed [EXP_BITS-1:0] offset = out_exp - 1;
    wire underflow = big_exp[EXP_BITS-1] && !offset[EXP_BITS-1];
    wire signed [FRAC_BITS-1:0] underflow_frac = {normalized[INT_BITS-1],
                                                     normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // =========================================================================
    // Step 8: Output mux
    //
    // Priority: shortcut > negligible bypass > zero > underflow > normal.
    // Negligible bypass: when |exp_diff| >= FRAC_BITS, the small operand
    // vanishes. Disabled when subtracting and b is big, because -big_frac
    // may not be N1 (let the far path normalize it).
    // =========================================================================
    wire use_negligible = negligible & !negate_big;

    assign result_frac = shortcut        ? sc_frac :
                         use_negligible  ? big_frac :
                         path_is_zero    ? {FRAC_BITS{1'b0}} :
                         underflow       ? underflow_frac :
                                           out_frac;

    assign result_exp  = shortcut        ? sc_exp :
                         use_negligible  ? big_exp :
                         path_is_zero    ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                         underflow       ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                           out_exp;

endmodule
