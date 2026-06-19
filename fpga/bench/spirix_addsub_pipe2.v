// spirix_addsub_pipe2 — 2-stage pipelined add/subtract for Spirix scalars
//
// Computes a + b (sub=0) or a - b (sub=1) on N1-normalized signed fractions with signed exponents. Fully parameterized. Latency: 2 cycles.
// Throughput: 1 result per clock.
//
// Architecture: close/far split with one shared barrel shifter, split across two pipeline stages.
//
//   Stage 1 (Prepare + Barrel):
//       Swap: exp subtract, |diff|, mux big/small.
//       Close: 1-bit align + add + CLZ, compute norm shift.
//       Far: compute align shift.
//       Shared barrel: close normalizes (bit-reverse right shift),
//           far aligns (arithmetic right shift + sticky accumulation).
//       Only one path's barrel result is meaningful per cycle.
//
//   Stage 2 (Finish):
//       Close: extract frac + GRS, banker's round, early rovf, exp.
//       Far: add big_ext + aligned (XOR + carry-in for sub),
//           bounded normalize (0-2 bit shift), banker's round,
//           early rovf, exp.
//       big_ext reconstructed from big_frac (free wiring: frac <<< 2).
//       Output mux: close / far / negligible / zero / underflow.
//
// Subtraction: XOR + carry-in on existing adders. After the exponent-based
// swap, b may land in either the big or small position. negate_small and
// negate_big track which operand holds b. The close path adder in stage 1 and the far path adder in stage 2 both use the same pattern:
//   (A ^ {N{negate_A}}) + (B ^ {N{negate_B}}) + sub
// On ECP5, the XOR folds into the LUT4 feeding the CCU2C carry chain.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/underflow.
//
// Valid parameter range: FRAC_BITS 4..29, EXP_BITS 4..16.
// (Barrel is 5 stages; FRAC_BITS > 29 needs a 6th stage.)

module spirix_addsub_pipe2 #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire ce,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    input  wire                         sub,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam INT_BITS = FRAC_BITS + 3; // frac + guard + round + sticky
    localparam BARREL_BITS = $clog2(INT_BITS);  // shift amount width
    localparam LEAD_BITS   = BARREL_BITS + 1;   // leading count width
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_SMALL = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_SMALL = {2'b11, {(FRAC_BITS-2){1'b0}}};

    // Undefined prefix constants
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
    // Edge case detection (combinational, before stage 1)
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

    wire signed [EXP_BITS-1:0] ec_b_exp_m1 = b_exp - 1'b1;
    wire ec_b_exp_m1_ambig = (ec_b_exp_m1 == AMBIGUOUS_EXP[EXP_BITS-1:0]);

    wire signed [FRAC_BITS-1:0] neg_b_frac_normal =
        b_is_pos_half  ? (ec_b_exp_m1_ambig ? NEG_SMALL : NEG_ONE) :
        b_is_neg_one_f ? POS_HALF :
                         -b_frac;
    wire signed [EXP_BITS-1:0] neg_b_exp_normal =
        b_is_pos_half  ? ec_b_exp_m1 :
        b_is_neg_one_f ? (b_exp + 1'b1) :
                         b_exp;

    wire b_top3_same = (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                       (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);
    wire b_nonnorm_nochange = (b_frac_zero | b_frac_neg1) |
                              (~(b_frac_zero | b_frac_neg1) & b_top3_same);

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

    wire ec_shortcut = any_non_normal & (sc_a_undef | sc_b_undef | sc_tf_tf | sc_van_van |
                                         sc_a_transf | sc_b_transf | sc_a_van | sc_b_van |
                                         sc_a_zero | sc_fallback);

    wire signed [FRAC_BITS-1:0] ec_sc_frac =
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

    wire signed [EXP_BITS-1:0] ec_sc_exp =
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
    // STAGE 1: Swap + Close Add + CLZ + Shared Barrel
    //
    // The full close path thru the barrel completes here. The far path
    // uses the barrel for alignment. Both produce a barrel output that
    // feeds the S1-S2 register.
    //
    // Critical path: exp_sub -> swap_mux -> close_add -> CLZ -> barrel_mux
    //     -> barrel stages (5 levels).
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

    // Subtraction: negate whichever operand holds b after swap.
    wire negate_small = sub & a_is_big;
    wire negate_big   = sub & !a_is_big;

    // --- Extend to internal width ---
    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac) <<< 2;
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac) <<< 2;

    // --- Close path: 1-bit align + add + CLZ ---
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

    // --- Shared barrel shifter ---
    // Close: bit-reverse -> logical right shift -> bit-reverse = left shift.
    // Far:   arithmetic right shift with sticky accumulation.

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

    // Bit-reverse barrel output for close path (free wiring, no logic)
    wire [INT_BITS-1:0] close_normalized;
    generate
        for (bi = 0; bi < INT_BITS; bi = bi + 1) begin : bitrev_out
            assign close_normalized[bi] = b4_val[INT_BITS - 1 - bi];
        end
    endgenerate

    // =========================================================================
    // S1-S2 pipeline registers
    //
    // Close path: close_normalized, close_leading, close_is_zero,
    //     close_align_sticky.
    // Far path: barrel output (aligned small), barrel sticky.
    // Shared: big_frac (reconstruct big_ext in S2 via free wiring),
    //     big_exp, is_close, negligible.
    // Subtraction: negate_big, negate_small, sub (forwarded directly).
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
    reg                          s2_shortcut;
    reg signed [FRAC_BITS-1:0]  s2_sc_frac;
    reg signed [EXP_BITS-1:0]   s2_sc_exp;

    always @(posedge clk) if (ce) begin
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
        s2_shortcut          <= ec_shortcut;
        s2_sc_frac           <= ec_sc_frac;
        s2_sc_exp            <= ec_sc_exp;
    end

    // =========================================================================
    // STAGE 2: Finish
    //
    // Close: extract frac + GRS bits, banker's round, early rovf, exp.
    // Far: reconstruct big_ext, add + bounded normalize, round, rovf, exp.
    // Output mux selects close / far / negligible / zero / underflow.
    //
    // Critical path (far): big_ext_reconstruct(free) -> far_add(carry chain)
    //     -> bounded normalize(mux) -> round -> rovf -> exp -> output mux.
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

    // Early rovf detection (from pre-round signals, no post-round compare)
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
                                        + {{(INT_BITS-1){1'b0}}, s2_sub};
    wire far_is_zero = (far_sum == 0);

    // Bounded normalize: result needs at most a 0-2 bit left shift.
    wire far_d0 = far_sum[INT_BITS-1] ^ far_sum[INT_BITS-2];
    wire far_d1 = far_sum[INT_BITS-2] ^ far_sum[INT_BITS-3];
    wire [1:0] far_norm_shift = far_d0 ? 2'd0 : far_d1 ? 2'd1 : 2'd2;
    wire [LEAD_BITS-1:0] far_leading = {{(LEAD_BITS-2){1'b0}}, far_norm_shift} + 1;

    wire [INT_BITS-1:0] far_normalized = far_d0 ? $unsigned(far_sum) :
                                          far_d1 ? ($unsigned(far_sum) << 1) :
                                                   ($unsigned(far_sum) << 2);

    // Far rounding (banker's round / RNE)
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

    // Negligible bypass: disabled when subtracting and b is big, because
    // -big_frac may not be N1 (let the far path normalize it).
    wire s2_use_negligible = s2_negligible & !s2_negate_big;

    // Stage 2 output register
    always @(posedge clk) if (ce) begin
        result_frac <= s2_shortcut        ? s2_sc_frac :
                       s2_use_negligible  ? s2_big_frac :
                       path_is_zero       ? {FRAC_BITS{1'b0}} :
                       path_underflow     ? path_uf_frac :
                                            path_frac;

        result_exp  <= s2_shortcut        ? s2_sc_exp :
                       s2_use_negligible  ? s2_big_exp :
                       path_is_zero       ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                       path_underflow     ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                            path_exp;
    end

endmodule
