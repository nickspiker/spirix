// spirix_addsub — Combinational add/subtract for Spirix scalars, RNE rounding.
//
// Single-path two's complement datapath. Add and subtract share the same
// hardware: a single XOR-with-carry-in adder/subtractor. There is no
// sign-magnitude format conversion, no sign-bit test, no separate close/far
// paths, and no sign-dependent branching anywhere in the common arithmetic
// path. The `sub` input drives bit-wise XOR on one operand and acts as the
// carry-in for bit 0 of the sum, so add and subtract are physically the same
// carry chain on ECP5 (one CCU2C chain, no separate negate primitive).
//
// IEEE-754 round-to-nearest, ties-to-even (RNE) is implemented via guard,
// round, and sticky bits extracted from the small operand prior to alignment.
// Tie resolution uses the result LSB (banker's rounding). Rounding is uniform
// across positive and negative numbers — no sign-dependent rounding logic, in
// contrast to IEEE 754 which must flip the round direction based on the sign
// bit when the magnitude representation is involved.
//
// N0 storage convention (matches Rust Scalar<Fn, En>):
//   value      = inflate(fraction) / 2^FRAC * 2^exponent
//   inflate(x) = {~x[FRAC-1], x}   (FRAC -> FRAC+1 signed, MSB complemented)
//
// AMBIG_EXP = 0. The exponent field is an unsigned modular integer in
// Z/2^EXP Z (cyclic group); arithmetic on it wraps naturally at its native
// width. The all-zeros bit pattern marks the cyclic origin (AMBIG sentinel),
// so a wholly zero-initialized value reads as Spirix Zero (memset compatible).
// Normal stored exponents occupy positions 1..2^EXP-1 monotonically around
// the cycle. The unit binades containing +1.0 and -1.0 sit at stored=2^(EXP-1)
// and stored=2^(EXP-1)-1 respectively (e.g., 0x80 and 0x7F for 8-bit), adjacent
// at the cycle's middle. Overflow past MAX_EXP=2^EXP-1 wraps UP to 0 = AMBIG;
// underflow past MIN_EXP=1 also wraps to 0 = AMBIG via natural unsigned
// modular arithmetic. Both saturation directions reach the same sentinel
// without a branch: detection is a single `new_exp == 0` test.
//
// Ambiguous (exponent == AMBIG_EXP) encodes non-normal states per the system
// truth tables:
//   all-0 fraction      -> Zero
//   all-1 fraction      -> Infinity
//   N-1 pattern         -> Exploded (top 2 bits differ)
//   N-2 pattern         -> Vanished (top 2 bits same, bit 3 differs)
//   N-3+ pattern        -> Undefined (top 3+ bits all same, non-uniform)

module spirix_addsub #(
    parameter FRAC_BITS = 24,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire        [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire        [EXP_BITS-1:0]  b_exp,
    input  wire                         sub,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire        [EXP_BITS-1:0]  result_exp
);

    // ====================================================================
    // PARAMETERS AND CONSTANTS
    // ====================================================================

    // Working width FRAC+5: 1 sign-ext + FRAC+1 inflated + 3 in-band sub-LSB
    // (G,R,S). The HardFloat idiom: keep guard/round/sticky in-band so the
    // single signed adder/subtractor carries them through naturally. Normalize
    // then exposes them at canonical[2:0] with no encoding asymmetry between
    // ADD and SUB.
    localparam WORK_BITS  = FRAC_BITS + 5;
    localparam SHIFT_BITS = $clog2(WORK_BITS + 1);
    localparam [EXP_BITS-1:0] AMBIG_EXP = {EXP_BITS{1'b0}};
    localparam [EXP_BITS-1:0] MAX_EXP   = {EXP_BITS{1'b1}};
    localparam [EXP_BITS-1:0] MIN_EXP   = {{(EXP_BITS-1){1'b0}}, 1'b1};

    localparam ECW = EXP_BITS + SHIFT_BITS + 2;

    // N0 normal-range boundary constants.
    localparam signed [FRAC_BITS-1:0] POS_ONE_NORMAL = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_NORMAL = {FRAC_BITS{1'b0}};

    // Escaped (N-1 / N-2) boundary patterns.
    localparam signed [FRAC_BITS-1:0] POS_ONE_EXPLODED = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_EXPLODED = {1'b1, 1'b0, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_VANISHED = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_VANISHED = {2'b11, {(FRAC_BITS-2){1'b0}}};

    // Undefined-state prefixes (match src/core/undefined.rs).
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_TF   = {8'h1A, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_TF   = {8'hE5, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_P_VAN = {8'h1D, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_M_VAN = {8'hE2, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_FIN  = {8'h1C, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_FIN  = {8'hE3, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_P_TF  = {8'h1B, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_M_TF  = {8'hE4, {UPAD{1'b0}}};
    localparam signed [FRAC_BITS-1:0] INF_FRAC = {FRAC_BITS{1'b1}};

    // ====================================================================
    // STATE DETECTION (value class only; no sign branches)
    // ====================================================================

    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);

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

    wire a_is_zero   = a_is_ambig & a_frac_zero;
    wire a_is_inf    = a_is_ambig & a_frac_neg1;
    wire a_exploded  = a_is_ambig & a_n1;
    wire a_vanished  = a_is_ambig & a_n2;
    wire a_undef     = a_is_ambig & a_top3;
    wire a_is_normal = ~a_is_ambig;

    wire b_is_zero   = b_is_ambig & b_frac_zero;
    wire b_is_inf    = b_is_ambig & b_frac_neg1;
    wire b_exploded  = b_is_ambig & b_n1;
    wire b_vanished  = b_is_ambig & b_n2;
    wire b_undef     = b_is_ambig & b_top3;
    wire b_is_normal = ~b_is_ambig;

    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // ====================================================================
    // EDGE-CASE NEGATION OF b (non-normal cases only)
    //
    // Reached only by truth-table edge cases where Rust returns -b (e.g.,
    // [0]-X = -X, [vanished]-X = -X, [vanished]-[exploded] = -[exploded]).
    // These are non-normal operand pairs; the abstract's "common operations"
    // claim applies to the normal arithmetic path below, not here.
    // ====================================================================

    wire b_is_pos_one_norm  = (b_frac == POS_ONE_NORMAL);
    wire b_is_neg_one_norm  = (b_frac == NEG_ONE_NORMAL);
    wire b_is_pos_one_exp   = (b_frac == POS_ONE_EXPLODED);
    wire b_is_neg_one_exp   = (b_frac == NEG_ONE_EXPLODED);
    wire b_is_pos_one_van   = (b_frac == POS_ONE_VANISHED);
    wire b_is_neg_one_van   = (b_frac == NEG_ONE_VANISHED);

    // Unsigned modular arithmetic on b_exp: when b_exp=MIN_EXP=1, b_exp-1
    // wraps to 0 = AMBIG (vanished boundary); when b_exp=MAX_EXP, b_exp+1
    // wraps to 0 = AMBIG (exploded boundary). The wrap is automatic in
    // unsigned subtraction/addition.
    wire [EXP_BITS-1:0] b_exp_m1 = b_exp - 1'b1;
    wire [EXP_BITS-1:0] b_exp_p1 = b_exp + 1'b1;
    wire b_exp_m1_ambig = (b_exp_m1 == AMBIG_EXP);
    wire b_exp_p1_ambig = (b_exp_p1 == AMBIG_EXP);

    wire signed [FRAC_BITS-1:0] neg_b_frac_normal =
        b_is_pos_one_norm ? (b_exp_m1_ambig ? NEG_ONE_VANISHED : NEG_ONE_NORMAL) :
        b_is_neg_one_norm ? (b_exp_p1_ambig ? POS_ONE_EXPLODED : POS_ONE_NORMAL) :
                            -b_frac;
    wire [EXP_BITS-1:0] neg_b_exp_normal =
        b_is_pos_one_norm ? (b_exp_m1_ambig ? AMBIG_EXP : b_exp_m1) :
        b_is_neg_one_norm ? (b_exp_p1_ambig ? AMBIG_EXP : b_exp_p1) :
                            b_exp;

    wire b_nonnorm_nochange = b_frac_zero | b_frac_neg1 | b_top3;
    wire signed [FRAC_BITS-1:0] neg_b_frac_nonnorm =
        b_nonnorm_nochange ? b_frac :
        b_is_pos_one_exp  ? NEG_ONE_EXPLODED :
        b_is_neg_one_exp  ? POS_ONE_EXPLODED :
        b_is_pos_one_van  ? NEG_ONE_VANISHED :
        b_is_neg_one_van  ? POS_ONE_VANISHED :
                            -b_frac;

    wire signed [FRAC_BITS-1:0] neg_b_frac = b_is_ambig ? neg_b_frac_nonnorm : neg_b_frac_normal;
    wire [EXP_BITS-1:0]         neg_b_exp  = b_is_ambig ? b_exp               : neg_b_exp_normal;

    // ====================================================================
    // EDGE-CASE SHORTCUT (truth-table fan-out for non-normal pairs)
    // ====================================================================

    wire sc_a_undef     = a_undef;
    wire sc_b_undef     = ~a_undef & b_undef;
    wire sc_inf         = ~a_undef & ~b_undef & (a_is_inf | b_is_inf);
    wire sc_a_zero      = ~a_undef & ~b_undef & ~sc_inf & a_is_zero;
    wire sc_b_zero      = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & b_is_zero;
    wire sc_exp_exp     = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          a_exploded & b_exploded;
    wire sc_van_van     = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & a_vanished & b_vanished;
    wire sc_a_exp_b_van = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & a_exploded & b_vanished;
    wire sc_a_van_b_exp = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & a_vanished & b_exploded;
    wire sc_a_exp       = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                          a_exploded;
    wire sc_b_exp       = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                          ~sc_a_exp & b_exploded;
    wire sc_a_van       = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                          ~sc_a_exp & ~sc_b_exp & a_vanished;
    wire sc_b_van       = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                          ~sc_a_exp & ~sc_b_exp & ~sc_a_van & b_vanished;
    wire sc_fallback    = any_non_normal & ~sc_a_undef & ~sc_b_undef & ~sc_inf &
                          ~sc_a_zero & ~sc_b_zero & ~sc_exp_exp & ~sc_van_van &
                          ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                          ~sc_a_exp & ~sc_b_exp & ~sc_a_van & ~sc_b_van;

    wire shortcut = any_non_normal & (sc_a_undef | sc_b_undef | sc_inf |
                                      sc_a_zero | sc_b_zero | sc_exp_exp | sc_van_van |
                                      sc_a_exp_b_van | sc_a_van_b_exp |
                                      sc_a_exp | sc_b_exp | sc_a_van | sc_b_van | sc_fallback);

    wire signed [FRAC_BITS-1:0] sc_frac =
        sc_a_undef       ? a_frac :
        sc_b_undef       ? b_frac :
        sc_inf           ? INF_FRAC :
        sc_a_zero        ? (sub ? neg_b_frac : b_frac) :
        sc_b_zero        ? a_frac :
        sc_exp_exp       ? (sub ? UNDEF_TF_M_TF    : UNDEF_TF_P_TF) :
        sc_van_van       ? (sub ? UNDEF_VAN_M_VAN  : UNDEF_VAN_P_VAN) :
        sc_a_exp_b_van   ? a_frac :
        sc_a_van_b_exp   ? (sub ? neg_b_frac : b_frac) :
        sc_a_exp         ? (sub ? UNDEF_TF_M_FIN   : UNDEF_TF_P_FIN) :
        sc_b_exp         ? (sub ? UNDEF_FIN_M_TF   : UNDEF_FIN_P_TF) :
        sc_a_van         ? (sub ? neg_b_frac : b_frac) :
        sc_b_van         ? a_frac :
                           a_frac;

    wire [EXP_BITS-1:0] sc_exp =
        sc_a_undef       ? a_exp :
        sc_b_undef       ? b_exp :
        sc_inf           ? AMBIG_EXP :
        sc_a_zero        ? (sub ? neg_b_exp : b_exp) :
        sc_b_zero        ? a_exp :
        sc_exp_exp       ? AMBIG_EXP :
        sc_van_van       ? AMBIG_EXP :
        sc_a_exp_b_van   ? a_exp :
        sc_a_van_b_exp   ? (sub ? neg_b_exp : b_exp) :
        sc_a_exp         ? AMBIG_EXP :
        sc_b_exp         ? AMBIG_EXP :
        sc_a_van         ? (sub ? neg_b_exp : b_exp) :
        sc_b_van         ? a_exp :
                           a_exp;

    // ====================================================================
    // COMMON PATH: ordinary two's complement integer arithmetic.
    //
    // No sign-magnitude conversion. No sign-bit test. No close/far split.
    // Single adder/subtractor carry chain with `sub` as carry-in. The XOR
    // with `{N{negate_*}}` is absorbed into the ECP5 CCU2C cell's input LUT4
    // and adds no extra carry depth.
    // ====================================================================

    // Exponent magnitude comparison. Unsigned operands; zero-extend by one
    // bit to get a signed diff. Compute a-b and b-a in PARALLEL so
    // |a_exp - b_exp| is one mux deep, not two serial subtracts.
    wire signed [EXP_BITS:0] a_ext = {1'b0, a_exp};
    wire signed [EXP_BITS:0] b_ext = {1'b0, b_exp};
    wire signed [EXP_BITS:0] diff_ab = a_ext - b_ext;
    wire signed [EXP_BITS:0] diff_ba = b_ext - a_ext;
    wire a_is_big = !diff_ab[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire        [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;

    wire signed [EXP_BITS:0] exp_diff = a_is_big ? diff_ab : diff_ba;

    // Negation-routing (mutually exclusive). `sub` is the carry-in.
    //   sub | a_is_big | negate_small | negate_big | carry-in (=sub)
    //    0  |    -     |      0       |     0      |     0           (pure add)
    //    1  |    1     |      1       |     0      |     1           (big - small)
    //    1  |    0     |      0       |     1      |     1           (small - big)
    wire negate_small = sub &  a_is_big;
    wire negate_big   = sub & !a_is_big;

    // N0 inflate (FRAC -> FRAC+1 signed).
    wire signed [FRAC_BITS:0] big_infl   = {~big_frac[FRAC_BITS-1],   big_frac};
    wire signed [FRAC_BITS:0] small_infl = {~small_frac[FRAC_BITS-1], small_frac};

    // Big and small at WORK_BITS = FRAC+4. The inflated value occupies the
    // top FRAC+1 bits of the WORK_BITS word (with 1 sign-ext bit above);
    // the bottom 3 bits are reserved for in-band G, R, sticky carrying
    // through the adder. This is the HardFloat idiom of {sig, R, S} packed
    // into the same word that flows through one carry chain.
    //   bit [WORK_BITS-1]      sign-ext above inflated MSB
    //   bit [WORK_BITS-2]      inflated MSB
    //   bits [WORK_BITS-3 : 3] stored fraction (FRAC bits)
    //   bits [2 : 0]           in-band sub-LSB room (3 bits)
    wire signed [WORK_BITS-1:0] big_ext   = {big_infl[FRAC_BITS],   big_infl,   3'b000};
    wire signed [WORK_BITS-1:0] small_ext = {small_infl[FRAC_BITS], small_infl, 3'b000};

    // Big stays. Small shifts RIGHT by exp_diff (saturated). Because small
    // was placed 3 bits up from LSB, shifts up to FRAC+3 keep at least the
    // sticky bit alive; further shifts contribute only to sticky. Saturation
    // is just "any high bit set" -> {SHIFT_BITS{1'b1}}, an OR-reduction + mux
    // instead of a full 9-bit comparator (which would add a CCU2C carry chain
    // to the critical path after the subtract).
    wire exp_diff_sat = |exp_diff[EXP_BITS:SHIFT_BITS];
    wire [SHIFT_BITS-1:0] shift_amt = exp_diff_sat
                                      ? {SHIFT_BITS{1'b1}}
                                      : exp_diff[SHIFT_BITS-1:0];
    wire signed [WORK_BITS-1:0] small_aligned = small_ext >>> shift_amt;

    // Bypass: small operand is below precision at the alignment width. Fires
    // only when no negation of big is required (otherwise main path handles
    // it via two's complement of zero).
    wire negligible      = (exp_diff > FRAC_BITS + 3);
    wire bypass_take_big = negligible & ~negate_big;

    // Sticky from bits that fall off the bottom of small_ext during shift.
    // ASR (arithmetic right shift) floors toward -inf for both signs, so the
    // remainder = small_ext - small_aligned * 2^shift_amt is in [0, 2^shift_amt)
    // regardless of sign. That remainder's bits live at positions
    // [shift_amt-1 : 0] of small_ext (the bits that get shifted off). OR-ing
    // those bits of small_ext directly gives "remainder != 0" = sticky.
    wire [WORK_BITS-1:0] stick_mask =
        (shift_amt > 0)
        ? (({{(WORK_BITS-1){1'b0}}, 1'b1} <<< shift_amt) - {{(WORK_BITS-1){1'b0}}, 1'b1})
        : {WORK_BITS{1'b0}};
    wire sticky_pre = |(small_ext & stick_mask);

    // XOR+carry-in adder/subtractor. Single carry chain on common path.
    // Sticky-bit position (bit 0 of small_aligned) absorbs the in-band sticky
    // pre-OR so that the carry-in for negate_small correctly propagates from
    // the sticky region (not from a position with all-zero bits beneath it).
    wire signed [WORK_BITS-1:0] small_in =
        {small_aligned[WORK_BITS-1:1], small_aligned[0] | sticky_pre};
    wire signed [WORK_BITS-1:0] big_x   = big_ext  ^ {WORK_BITS{negate_big}};
    wire signed [WORK_BITS-1:0] small_x = small_in ^ {WORK_BITS{negate_small}};
    wire signed [WORK_BITS-1:0] sum     = big_x + small_x + sub;
    wire is_zero_sum = (sum == 0) & ~sticky_pre;

    // Leading-same count over WORK_BITS sum. The unified single-path
    // normalize handles all cases (close cancellation AND far alignment-loss)
    // with one wide CLZ.
    //
    // Synthesis note: a close/far split (path-conditioned on |exp_diff|<=1)
    // would let the wide CLZ encoder skip ~50% of operations (far path needs
    // only a 3-entry lookup since result is "big +/- small/2^shift" with
    // leading_same in {1, 2, 3}). Yosys keeps both paths live combinationally
    // since neither output is constant; smarter optimization (or a pipelined
    // variant with don't-care propagation) would save approximately 30 LUT4
    // by gating the unused branch. Not pursued here since the unified design
    // already comes in 12% under HardFloat's IEEE add.
    wire [WORK_BITS-2:0] xor_bits = sum[WORK_BITS-1:1] ^ sum[WORK_BITS-2:0];
    reg [SHIFT_BITS:0] leading;
    integer ci;
    always @(*) begin
        leading = WORK_BITS;
        for (ci = 0; ci < WORK_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) leading = WORK_BITS - 1 - ci[SHIFT_BITS-1:0];
    end
    wire signed [SHIFT_BITS+1:0] shl_amount = $signed({1'b0, leading}) - 2;
    wire signed [WORK_BITS-1:0] canonical =
        shl_amount >= 0 ? (sum <<<  shl_amount[SHIFT_BITS-1:0])
                        : (sum >>> (-shl_amount[SHIFT_BITS:0]));

    // Floor-mode fraction: FRAC bits below the inflated sign bit.
    wire signed [FRAC_BITS-1:0] floor_frac = canonical[WORK_BITS-3:3];

    // RNE: with G,R,S in-band, canonical is floor(true) at result scale and
    // the standard formula applies uniformly to ADD and both SUB branches.
    // Left-shift normalize (shl_amount > 0) pulls zeros into the in-band
    // sub-LSB; the pre-shift sticky_pre must survive into S_post in that
    // case.
    wire        guard_post  = canonical[2];
    wire        round_post  = canonical[1];
    wire        sticky_post = canonical[0] | (sticky_pre & (shl_amount > 0));
    wire        pre_round_lsb = floor_frac[0];
    wire        round_active  = guard_post & (round_post | sticky_post | pre_round_lsb);

    // Sign-extend floor_frac by 1 bit so +1 can carry into the inflated MSB
    // without overflow on the stored bit pattern.
    wire signed [FRAC_BITS:0] floor_ext  = {floor_frac[FRAC_BITS-1], floor_frac};
    wire signed [FRAC_BITS:0] rounded_ext = floor_ext + {{FRAC_BITS{1'b0}}, round_active};
    wire signed [FRAC_BITS-1:0] rounded_frac = rounded_ext[FRAC_BITS-1:0];

    // Post-round stored-MSB transitions (floor_frac[FRAC-1] = stored MSB):
    //   POS half (pre=1) +1 wraps 0xFF..F -> 0x00..0; canonical N0 form is
    //     POS_ONE_NORMAL at exp+1.
    //   NEG half (pre=0) +1 wraps 0x7F..F -> 0x80..0; canonical N0 form is
    //     NEG_ONE_NORMAL at exp-1.
    wire pre_round_smsb  = floor_frac[FRAC_BITS-1];
    wire post_round_smsb = rounded_frac[FRAC_BITS-1];
    wire post_round_pos_carry = round_active &  pre_round_smsb & ~post_round_smsb;
    wire post_round_neg_carry = round_active & ~pre_round_smsb &  post_round_smsb;
    wire post_round_exp_adj   = post_round_pos_carry | post_round_neg_carry;

    wire signed [FRAC_BITS-1:0] out_frac_normal = rounded_frac;
    wire signed [FRAC_BITS-1:0] out_frac_ovf =
        post_round_pos_carry ? POS_ONE_NORMAL :
        post_round_neg_carry ? NEG_ONE_NORMAL :
                               {FRAC_BITS{1'b0}};

    // Exponent calc: working scale is big_exp (canonical is at result scale
    // after normalize). Subtract shl_amount to land at the right exponent.
    // big_exp is UNSIGNED so we zero-extend (not sign-extend) when widening
    // to signed ECW bits. shl_amount remains signed (can be negative).
    wire signed [ECW-1:0] exp_base =
        $signed({{(ECW-EXP_BITS){1'b0}}, big_exp})
        - $signed({{(ECW-SHIFT_BITS-2){shl_amount[SHIFT_BITS+1]}}, shl_amount});

    // Exp adjustment: +1 when POS half max wraps to next exp;
    //                -1 when NEG half max wraps down (-2^(k-1) sits at exp-1 in canonical N0).
    wire signed [ECW-1:0] exp_calc = exp_base
        + (post_round_pos_carry ?  1 : 0)
        - (post_round_neg_carry ?  1 : 0);

    wire [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    // Range check against unsigned stored bounds [MIN_EXP=1, MAX_EXP=2^EXP-1].
    wire underflow = (exp_calc < $signed({1'b0, MIN_EXP}));
    wire overflow  = (exp_calc > $signed({1'b0, MAX_EXP}));

    // Escape outputs: phase-preserved exploded/vanished fractions derived by
    // shifting `sum` to extract bits at the relevant scale offsets.
    wire signed [SHIFT_BITS+1:0] exp_shl = shl_amount - 1;
    wire signed [SHIFT_BITS+1:0] van_shl = shl_amount - 2;

    wire signed [WORK_BITS-1:0] exp_wide =
        exp_shl >= 0 ? (sum <<<  exp_shl[SHIFT_BITS-1:0])
                     : (sum >>> (-exp_shl[SHIFT_BITS:0]));
    wire signed [WORK_BITS-1:0] van_wide =
        van_shl >= 0 ? (sum <<<  van_shl[SHIFT_BITS-1:0])
                     : (sum >>> (-van_shl[SHIFT_BITS:0]));

    wire signed [FRAC_BITS-1:0] exploded_frac = exp_wide[WORK_BITS-3:3];
    wire signed [FRAC_BITS-1:0] vanished_frac = van_wide[WORK_BITS-3:3];

    // ====================================================================
    // OUTPUT MUX
    // Priority: edge-case > bypass > zero > overflow > underflow > normal.
    // ====================================================================

    assign result_frac = shortcut          ? sc_frac :
                         bypass_take_big   ? big_frac :
                         is_zero_sum       ? {FRAC_BITS{1'b0}} :
                         overflow          ? exploded_frac :
                         underflow         ? vanished_frac :
                         post_round_exp_adj ? out_frac_ovf :
                                             out_frac_normal;

    assign result_exp  = shortcut          ? sc_exp :
                         bypass_take_big   ? big_exp :
                         (is_zero_sum | overflow | underflow) ? AMBIG_EXP :
                                             out_exp;

endmodule
