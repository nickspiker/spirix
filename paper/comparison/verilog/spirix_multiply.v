// spirix_multiply — Banker's-rounded N0 multiply for paper comparison.
//
// Specialty module for the paper/comparison harness. Adds banker's RNE to
// the production floor-only multiply, ports to N0 storage at FRAC=24
// (binary32-equivalent precision; 1:1 with IEEE 754 binary32's 23+1 bits).
//
// Architecture:
//   1. State detect (zero, infinity, exploded, vanished, undefined)
//   2. Edge-case shortcut (truth-table dispatch)
//   3. N0 inflate to compute Q (FRAC+1 bits, plain two's complement)
//   4. Signed multiply: 2*COMPUTE_FRAC bit product
//   5. Bounded normalize (0-2 bits, no CLZ needed: canonical N1×N1 ⇒
//      product magnitude in [2^(2F-2), 2^(2F)), so leading zero count
//      is ≤ 2)
//   6. Banker's RNE on the truncated top COMPUTE_FRAC bits + GRS below
//   7. ROVF detection / renormalization
//   8. Deflate compute Q result to N0 storage (drop the redundant top bit)

module spirix_multiply #(
    parameter FRAC_BITS = 24,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);
    localparam COMPUTE_FRAC = FRAC_BITS + 1;          // N0 inflate width
    localparam INT_BITS     = 2 * COMPUTE_FRAC;       // product width = 50 @ FRAC=24
    localparam EW           = EXP_BITS + 2;

    localparam signed [EXP_BITS-1:0] AMBIG_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [EXP_BITS-1:0] MAX_EXP   = {1'b0, {(EXP_BITS-1){1'b1}}};
    localparam signed [EXP_BITS-1:0] MIN_EXP   = -((1 <<< (EXP_BITS-1)) - 1);

    // N0 storage boundary patterns.
    localparam signed [FRAC_BITS-1:0] POS_ONE_NORMAL   = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_NORMAL   = {FRAC_BITS{1'b0}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_EXPLODED = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_EXPLODED = {1'b1, 1'b0, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_VANISHED = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_VANISHED = {2'b11, {(FRAC_BITS-2){1'b0}}};

    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_MUL_NEG = {8'hE6, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_NEG_MUL_TF = {8'h19, {UPAD{1'b0}}};

    // ───── State detection ─────────────────────────────────────────────────
    wire a_is_ambig  = (a_exp == AMBIG_EXP);
    wire b_is_ambig  = (b_exp == AMBIG_EXP);
    wire a_frac_zero = (a_frac == {FRAC_BITS{1'b0}});
    wire a_frac_neg1 = &a_frac;
    wire b_frac_zero = (b_frac == {FRAC_BITS{1'b0}});
    wire b_frac_neg1 = &b_frac;
    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;
    wire a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire b_n1 = (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-2]);
    wire a_n2 = ~a_n1 & ~a_n0 & (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-3]);
    wire b_n2 = ~b_n1 & ~b_n0 & (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-3]);
    wire a_top3 = ~a_n0 & (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire b_top3 = ~b_n0 & (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                  (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);

    wire a_is_zero  = a_is_ambig & a_frac_zero;
    wire a_is_inf   = a_is_ambig & a_frac_neg1;
    wire a_exploded = a_is_ambig & a_n1;
    wire a_vanished = a_is_ambig & a_n2;
    wire a_undef    = a_is_ambig & a_top3;
    wire b_is_zero  = b_is_ambig & b_frac_zero;
    wire b_is_inf   = b_is_ambig & b_frac_neg1;
    wire b_exploded = b_is_ambig & b_n1;
    wire b_vanished = b_is_ambig & b_n2;
    wire b_undef    = b_is_ambig & b_top3;

    wire any_non_normal = a_is_ambig | b_is_ambig;

    // ───── Edge-case shortcut (multiply truth table) ─────────────────────
    //   1. a_undef    → a passthrough
    //   2. b_undef    → b passthrough
    //   3. inf × zero (or zero × inf) → UNDEF_TF_MUL_NEG / UNDEF_NEG_MUL_TF
    //   4. any × inf  → infinity
    //   5. any × zero → zero
    //   6. exploded × vanished → UNDEF_TF_MUL_NEG / UNDEF_NEG_MUL_TF
    wire sc_a_undef  = a_undef;
    wire sc_b_undef  = ~a_undef & b_undef;
    wire sc_inf_zero = ~a_undef & ~b_undef &
                       ((a_is_inf & b_is_zero) | (a_is_zero & b_is_inf));
    wire sc_any_inf  = ~a_undef & ~b_undef & ~sc_inf_zero & (a_is_inf | b_is_inf);
    wire sc_any_zero = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_inf &
                       (a_is_zero | b_is_zero);
    wire sc_exp_van  = ~a_undef & ~b_undef & ~sc_inf_zero & ~sc_any_inf & ~sc_any_zero &
                       ((a_exploded & b_vanished) | (a_vanished & b_exploded));

    // Remaining non-normal cases (e.g. exploded × normal, vanished × normal,
    // inf × normal): these still fall through to "any_inf" or get treated as
    // abnormal compute. For the IEEE comparison we don't deeply distinguish
    // these — Spirix's truth table will produce a sensible state and our
    // gold model maps to either Exploded, Vanished, or Undefined accordingly.

    wire shortcut = sc_a_undef | sc_b_undef | sc_inf_zero | sc_any_inf | sc_any_zero | sc_exp_van;
    wire abnormal_compute = any_non_normal & ~shortcut;
    wire n_level_neg1 = a_exploded | b_exploded;

    // ───── N0 inflate to compute Q ───────────────────────────────────────
    wire signed [COMPUTE_FRAC-1:0] a_q = {~a_frac[FRAC_BITS-1], a_frac};
    wire signed [COMPUTE_FRAC-1:0] b_q = {~b_frac[FRAC_BITS-1], b_frac};

    // Sign-extended multiply: COMPUTE_FRAC × COMPUTE_FRAC = 2*COMPUTE_FRAC bit product.
    wire signed [INT_BITS-1:0] a_ext = $signed(a_q);
    wire signed [INT_BITS-1:0] b_ext = $signed(b_q);
    wire signed [INT_BITS-1:0] product = a_ext * b_ext;

    wire prod_is_zero = (product == {INT_BITS{1'b0}});

    // ───── Bounded normalize (canonical: 0/1/2 bits) ─────────────────────
    // For canonical compute Q operands (top 2 bits differ), product magnitude
    // is in [2^(2F-2), 2^(2F-1)) — top 3 bits cover the 0/1/2 normalize cases.
    // For abnormal (exploded/vanished operands), magnitude can be smaller; we
    // need up to 4-bit normalize.
    //
    // Production multiply uses the same logic with N1 storage; we use it on
    // the inflated compute Q form, which behaves identically to N1 internally.

    wire [1:0] norm_lm1 =
        (product[INT_BITS-1] != product[INT_BITS-2]) ? 2'd0 :
        (product[INT_BITS-2] != product[INT_BITS-3]) ? 2'd1 : 2'd2;

    wire [2:0] abn_lm1 =
        (product[INT_BITS-1] != product[INT_BITS-2]) ? 3'd0 :
        (product[INT_BITS-2] != product[INT_BITS-3]) ? 3'd1 :
        (product[INT_BITS-3] != product[INT_BITS-4]) ? 3'd2 :
        (product[INT_BITS-4] != product[INT_BITS-5]) ? 3'd3 : 3'd4;

    wire abnormal_n2 = abnormal_compute & ~n_level_neg1;
    wire [2:0] abn_shift = (abnormal_n2 & |abn_lm1) ? abn_lm1 - 3'd1 : abn_lm1;

    // ───── Banker's RNE: extract top COMPUTE_FRAC bits + GRS ─────────────
    // After normalize-by-norm_lm1, the canonical compute Q result occupies
    // positions [INT_BITS-1-norm_lm1 : INT_BITS-COMPUTE_FRAC-norm_lm1].
    // The bits below are the discarded precision: top of those is guard,
    // next is round, OR of remaining is sticky.
    //
    // We don't physically shift; we slice the appropriate window.

    function automatic [COMPUTE_FRAC+2:0] extract_round;
        input [INT_BITS-1:0] p;
        input [1:0] shift;
        reg [INT_BITS-1:0] aligned;
        begin
            aligned = p << shift;
            // Top COMPUTE_FRAC bits + 1 guard + 1 round + 1 sticky-or
            extract_round = {aligned[INT_BITS-1 -: COMPUTE_FRAC],
                             aligned[INT_BITS-COMPUTE_FRAC-1],
                             aligned[INT_BITS-COMPUTE_FRAC-2],
                             |aligned[INT_BITS-COMPUTE_FRAC-3:0]};
        end
    endfunction

    wire [COMPUTE_FRAC+2:0] norm_extract = extract_round(product, {1'b0, norm_lm1});
    wire signed [COMPUTE_FRAC-1:0] out_q_raw = norm_extract[COMPUTE_FRAC+2:3];
    wire guard_bit  = norm_extract[2];
    wire round_bit  = norm_extract[1];
    wire sticky_bit = norm_extract[0];
    wire lsb_bit    = out_q_raw[0];
    wire round_up   = guard_bit & (round_bit | sticky_bit | lsb_bit);

    // ROVF detection on out_q_raw (compute Q form, before deflate).
    // For multiply, rovf can fire when round_up pushes the magnitude from
    // canonical max to overflow.
    //   pos: 0_111...1 → 1_000...0 (compute Q wraps positive→negative)
    //        → renorm: +0.5·2^(exp+1) = POS_ONE_NORMAL @ exp+1
    //   neg: 1_0_111..1 → 1_1_000..0 (non-canonical neg)
    //        → renorm: -1.0·2^(exp-1) = NEG_ONE_NORMAL @ exp-1
    wire rovf_pos = ~out_q_raw[COMPUTE_FRAC-1] & (&out_q_raw[COMPUTE_FRAC-2:0]) & round_up;
    wire rovf_neg = out_q_raw[COMPUTE_FRAC-1] & ~out_q_raw[COMPUTE_FRAC-2] &
                   (&out_q_raw[COMPUTE_FRAC-3:0]) & round_up;

    wire signed [COMPUTE_FRAC-1:0] out_q_rounded =
        out_q_raw + {{(COMPUTE_FRAC-1){1'b0}}, round_up};

    // N0 storage = compute_q[FRAC-1:0]; rovf overrides with renormalized targets.
    wire signed [FRAC_BITS-1:0] out_frac_normal =
        rovf_pos ? POS_ONE_NORMAL :
        rovf_neg ? NEG_ONE_NORMAL :
                   out_q_rounded[FRAC_BITS-1:0];

    // For the abnormal path: just use the slice without rounding (matches
    // production multiply's floor; abnormal values are sentinels anyway).
    wire signed [FRAC_BITS-1:0] abn_frac =
        (abn_shift == 3'd0) ? product[INT_BITS-1 -: FRAC_BITS] :
        (abn_shift == 3'd1) ? product[INT_BITS-2 -: FRAC_BITS] :
        (abn_shift == 3'd2) ? product[INT_BITS-3 -: FRAC_BITS] :
        (abn_shift == 3'd3) ? product[INT_BITS-4 -: FRAC_BITS] :
                               product[INT_BITS-5 -: FRAC_BITS];

    // ───── Exponent ───────────────────────────────────────────────────────
    // result_exp = a_exp + b_exp + 1 - norm_lm1 + rovf_adj
    //   The +1 accounts for compute Q being in [2^(F-1), 2^F) (canonical
    //   normalized has magnitude bit at FRAC-1, so the product magnitude bit
    //   at INT_BITS-3 corresponds to the result's compute Q bit FRAC-1; the
    //   shift by norm_lm1 brings the magnitude bit to INT_BITS-2; the
    //   +1 normalizes the exponent).
    wire signed [EW-1:0] a_exp_ext = $signed({{(EW-EXP_BITS){a_exp[EXP_BITS-1]}}, a_exp});
    wire signed [EW-1:0] b_exp_ext = $signed({{(EW-EXP_BITS){b_exp[EXP_BITS-1]}}, b_exp});
    wire signed [EW-1:0] norm_lm1_ext = $signed({{(EW-2){1'b0}}, norm_lm1});
    wire signed [EW-1:0] rovf_adj =
        rovf_pos ? $signed({{(EW-1){1'b0}}, 1'b1}) :
        rovf_neg ? -$signed({{(EW-1){1'b0}}, 1'b1}) :
                   {EW{1'b0}};
    wire signed [EW-1:0] exp_calc =
        a_exp_ext + b_exp_ext + 1 - norm_lm1_ext + rovf_adj;

    wire signed [EXP_BITS-1:0] out_exp_normal = exp_calc[EXP_BITS-1:0];

    wire underflow = (exp_calc < $signed({{(EW-EXP_BITS){MIN_EXP[EXP_BITS-1]}}, MIN_EXP}));
    wire overflow  = (exp_calc > $signed({{(EW-EXP_BITS){MAX_EXP[EXP_BITS-1]}}, MAX_EXP}));

    // Saturation outputs (sign-preserving). After normalize, sign of result =
    // top bit of out_q_rounded (compute Q sign).
    wire result_is_pos = ~out_q_rounded[COMPUTE_FRAC-1];
    wire signed [FRAC_BITS-1:0] sat_exploded =
        result_is_pos ? POS_ONE_EXPLODED : NEG_ONE_EXPLODED;
    wire signed [FRAC_BITS-1:0] sat_vanished =
        result_is_pos ? POS_ONE_VANISHED : NEG_ONE_VANISHED;

    // ───── Shortcut output ───────────────────────────────────────────────
    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef  ? a_frac :
        sc_b_undef  ? b_frac :
        sc_inf_zero ? ((a_is_inf | a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF) :
        sc_any_inf  ? {FRAC_BITS{1'b1}} :
        sc_any_zero ? {FRAC_BITS{1'b0}} :
                      ((a_exploded) ? UNDEF_TF_MUL_NEG : UNDEF_NEG_MUL_TF);
    wire signed [EXP_BITS-1:0] sc_exp =
        sc_a_undef  ? a_exp :
        sc_b_undef  ? b_exp :
                      AMBIG_EXP;

    // ───── Output mux ────────────────────────────────────────────────────
    assign result_frac =
        shortcut         ? sc_frac :
        abnormal_compute ? abn_frac :
        prod_is_zero     ? {FRAC_BITS{1'b0}} :
        overflow         ? sat_exploded :
        underflow        ? sat_vanished :
                           out_frac_normal;
    assign result_exp =
        shortcut                          ? sc_exp :
        (abnormal_compute | prod_is_zero) ? AMBIG_EXP :
        (overflow | underflow)            ? AMBIG_EXP :
                                            out_exp_normal;

endmodule
