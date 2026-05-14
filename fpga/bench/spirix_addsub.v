// spirix_addsub — Combinational add/subtract for Spirix scalars with edge cases
//
// Floor-only (no rounding), single-path design. N0 format.
// Parameterized: FRAC_BITS in {8..256}, EXP_BITS in {2..256}.
//
// N0 storage convention (matches Rust Scalar<Fn, En>):
//   value = inflate(fraction) / 2^FRAC * 2^exponent
//   inflate(x) = sign_ext(x) XOR high_mask, where high_mask = (-1) << FRAC.
//   Equivalently: wide = {{FRAC{~x[FRAC-1]}}, x} (low FRAC bits = stored; high
//   FRAC bits = complement of stored MSB, giving implicit ~MSB sign).
//
// AMBIG_EXP = 0. The exponent field is an unsigned modular integer in
// Z/2^EXP Z (cyclic group); arithmetic on it wraps naturally at its native
// width. The all-zeros bit pattern marks the cyclic origin (AMBIG sentinel),
// so a wholly zero-initialized value reads as Spirix Zero (memset compatible).
// Normal stored exponents occupy positions 1..2^EXP-1 monotonically around
// the cycle. Overflow past MAX_EXP=2^EXP-1 wraps UP to 0 = AMBIG; underflow
// past MIN_EXP=1 also wraps to 0 = AMBIG via natural unsigned modular
// arithmetic. Both saturation directions reach the same sentinel without a
// branch — detection is a single `new_exp == 0` test.
//
// Ambiguous (exponent == AMBIG_EXP) encodes non-normal states:
//   all-0 fraction     → Zero
//   all-1 fraction     → Infinity
//   N-1 pattern         → Exploded (top 2 bits differ)
//   N-2 pattern         → Vanished (top 2 bits same, bit 3 differs)
//   N-3+ pattern        → Undefined (top 3+ bits all same, non-uniform)
//
// Edge case precedence matches Rust scalar_add_scalar exactly.
// Undefined-prefix constants match src/core/undefined.rs.

module spirix_addsub #(
    parameter FRAC_BITS = 32,
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

    // FRAC+2 working width (shift-small-RIGHT, big never shifts).
    // The sub-LSB bits lost during small's right-shift are OR-ed into a
    // single sticky flag; that flag disambiguates true-zero from tiny-positive
    // (POS_ONE_VANISHED) when the close-path sum cancels exactly.
    // Was 2*FRAC in the shift-big-LEFT scheme; FRAC+2 + sticky suffices.
    localparam WORK_BITS  = FRAC_BITS + 2;
    localparam SHIFT_BITS = $clog2(WORK_BITS);
    localparam [EXP_BITS-1:0] AMBIG_EXP = {EXP_BITS{1'b0}};
    localparam [EXP_BITS-1:0] MAX_EXP   = {EXP_BITS{1'b1}};
    localparam [EXP_BITS-1:0] MIN_EXP   = {{(EXP_BITS-1){1'b0}}, 1'b1};

    // Exponent calc width: need enough bits to detect over/underflow without wrapping.
    localparam ECW = EXP_BITS + SHIFT_BITS + 2;

    // N0 normal boundary constants.
    localparam signed [FRAC_BITS-1:0] POS_ONE_NORMAL = {1'b1, {(FRAC_BITS-1){1'b0}}}; // 0x80...0
    localparam signed [FRAC_BITS-1:0] NEG_ONE_NORMAL = {FRAC_BITS{1'b0}};             // 0x00...0

    // Escaped (N-1 / N-2) boundary patterns — same in N0 and N1.
    localparam signed [FRAC_BITS-1:0] POS_ONE_EXPLODED = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}}; // 0x40
    localparam signed [FRAC_BITS-1:0] NEG_ONE_EXPLODED = {1'b1, 1'b0, {(FRAC_BITS-2){1'b0}}}; // 0x80
    localparam signed [FRAC_BITS-1:0] POS_ONE_VANISHED = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}}; // 0x20
    localparam signed [FRAC_BITS-1:0] NEG_ONE_VANISHED = {2'b11, {(FRAC_BITS-2){1'b0}}};       // 0xC0

    // Undefined prefix constants — match src/core/undefined.rs exactly.
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_TF   = {8'h1A, {UPAD{1'b0}}}; // ℘⬆+⬆ TRANSFINITE_PLUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_TF   = {8'hE5, {UPAD{1'b0}}}; // ℘⬆-⬆ TRANSFINITE_MINUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_P_VAN = {8'h1D, {UPAD{1'b0}}}; // ℘↓+↓ VANISHED_PLUS_VANISHED
    localparam signed [FRAC_BITS-1:0] UNDEF_VAN_M_VAN = {8'hE2, {UPAD{1'b0}}}; // ℘↓-↓ VANISHED_MINUS_VANISHED
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_P_FIN  = {8'h1C, {UPAD{1'b0}}}; // ℘⬆+ TRANSFINITE_PLUS_FINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_M_FIN  = {8'hE3, {UPAD{1'b0}}}; // ℘⬆- TRANSFINITE_MINUS_FINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_P_TF  = {8'h1B, {UPAD{1'b0}}}; // ℘+⬆ FINITE_PLUS_TRANSFINITE
    localparam signed [FRAC_BITS-1:0] UNDEF_FIN_M_TF  = {8'hE4, {UPAD{1'b0}}}; // ℘-⬆ FINITE_MINUS_TRANSFINITE

    // ========== State detection =================================================

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
    wire a_transf    = a_is_inf | a_exploded;
    wire a_vanished  = a_is_ambig & a_n2;
    wire a_undef     = a_is_ambig & a_top3;
    wire a_is_normal = ~a_is_ambig;   // N0: any fraction pattern + non-AMBIG exp

    wire b_is_zero   = b_is_ambig & b_frac_zero;
    wire b_is_inf    = b_is_ambig & b_frac_neg1;
    wire b_exploded  = b_is_ambig & b_n1;
    wire b_transf    = b_is_inf | b_exploded;
    wire b_vanished  = b_is_ambig & b_n2;
    wire b_undef     = b_is_ambig & b_top3;
    wire b_is_normal = ~b_is_ambig;

    wire any_non_normal = ~a_is_normal | ~b_is_normal;

    // ========== Spirix negation of b (for sub edge cases) =======================
    //
    // N0 normal negation:
    //   POS_ONE_NORMAL (0x80) → (NEG_ONE_NORMAL, exp-1) or NEG_ONE_VANISHED if exp-1==AMBIG
    //   NEG_ONE_NORMAL (0x00) → (POS_ONE_NORMAL, exp+1) or POS_ONE_EXPLODED if exp+1==AMBIG
    //   else → (-frac, exp) — two's-complement negate in place
    //
    // Non-normal negation:
    //   zero/inf/undef (top3_same) → unchanged (signless)
    //   POS_ONE_EXPLODED ↔ NEG_ONE_EXPLODED
    //   POS_ONE_VANISHED ↔ NEG_ONE_VANISHED
    //   else → wrapping_neg (exp unchanged)

    wire b_is_pos_one_norm  = (b_frac == POS_ONE_NORMAL);
    wire b_is_neg_one_norm  = (b_frac == NEG_ONE_NORMAL);
    wire b_is_pos_one_exp   = (b_frac == POS_ONE_EXPLODED);
    wire b_is_neg_one_exp   = (b_frac == NEG_ONE_EXPLODED);
    wire b_is_pos_one_van   = (b_frac == POS_ONE_VANISHED);
    wire b_is_neg_one_van   = (b_frac == NEG_ONE_VANISHED);

    // Unsigned modular arithmetic on b_exp: b_exp=MIN→b_exp-1=0=AMBIG, and
    // b_exp=MAX→b_exp+1=0=AMBIG, both via natural unsigned wrap.
    wire [EXP_BITS-1:0] b_exp_m1 = b_exp - 1'b1;
    wire [EXP_BITS-1:0] b_exp_p1 = b_exp + 1'b1;
    wire b_exp_m1_ambig = (b_exp_m1 == AMBIG_EXP);
    wire b_exp_p1_ambig = (b_exp_p1 == AMBIG_EXP);

    // Normal negation (b_is_normal)
    wire signed [FRAC_BITS-1:0] neg_b_frac_normal =
        b_is_pos_one_norm ? (b_exp_m1_ambig ? NEG_ONE_VANISHED : NEG_ONE_NORMAL) :
        b_is_neg_one_norm ? (b_exp_p1_ambig ? POS_ONE_EXPLODED : POS_ONE_NORMAL) :
                            -b_frac;
    wire [EXP_BITS-1:0] neg_b_exp_normal =
        b_is_pos_one_norm ? (b_exp_m1_ambig ? AMBIG_EXP : b_exp_m1) :
        b_is_neg_one_norm ? (b_exp_p1_ambig ? AMBIG_EXP : b_exp_p1) :
                            b_exp;

    // Non-normal negation (b_is_ambig)
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

    // ========== Edge case priority (matches Rust scalar_add_scalar) =============
    //
    // Add:                                          Sub:
    // 1.  a undef           -> a                    1.  a undef           -> a
    // 2.  b undef           -> b                    2.  b undef           -> b
    // 3.  any [∞]           -> [∞]                  3.  any [∞]           -> [∞]
    // 4.  a_zero            -> b                    4.  a_zero            -> -b
    // 5.  b_zero            -> a                    5.  b_zero            -> a
    // 6.  exp & exp         -> TF_P_TF              6.  exp & exp         -> TF_M_TF
    // 7.  van & van         -> VAN_P_VAN            7.  van & van         -> VAN_M_VAN
    // 8.  a_exp & b_van     -> a                    8.  a_exp & b_van     -> a
    // 9.  a_van & b_exp     -> b                    9.  a_van & b_exp     -> -b
    // 10. a_exp             -> TF_P_FIN             10. a_exp             -> TF_M_FIN
    // 11. b_exp             -> FIN_P_TF             11. b_exp             -> FIN_M_TF
    // 12. a_van             -> b                    12. a_van             -> -b
    // 13. b_van             -> a                    13. b_van             -> a
    // 14. fallback          -> a                    14. fallback          -> a
    //
    // [∞] absorbs (signless Riemann singularity, no − op).
    // Vanished is negligible against exploded: [↑]±[↓]=[↑] (steps 8/9).

    localparam signed [FRAC_BITS-1:0] INF_FRAC = {FRAC_BITS{1'b1}};
    wire sc_a_undef   = a_undef;
    wire sc_b_undef   = ~a_undef & b_undef;
    wire sc_inf       = ~a_undef & ~b_undef & (a_is_inf | b_is_inf);
    wire sc_a_zero    = ~a_undef & ~b_undef & ~sc_inf & a_is_zero;
    wire sc_b_zero    = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & b_is_zero;
    wire sc_exp_exp   = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                        a_exploded & b_exploded;
    wire sc_van_van   = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_exp_exp & a_vanished & b_vanished;
    wire sc_a_exp_b_van = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & a_exploded & b_vanished;
    wire sc_a_van_b_exp = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                          ~sc_exp_exp & ~sc_van_van & a_vanished & b_exploded;
    wire sc_a_exp     = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                        a_exploded;
    wire sc_b_exp     = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                        ~sc_a_exp & b_exploded;
    wire sc_a_van     = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                        ~sc_a_exp & ~sc_b_exp & a_vanished;
    wire sc_b_van     = ~a_undef & ~b_undef & ~sc_inf & ~sc_a_zero & ~sc_b_zero &
                        ~sc_exp_exp & ~sc_van_van & ~sc_a_exp_b_van & ~sc_a_van_b_exp &
                        ~sc_a_exp & ~sc_b_exp & ~sc_a_van & b_vanished;
    wire sc_fallback  = any_non_normal & ~sc_a_undef & ~sc_b_undef & ~sc_inf &
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

    // ========== Normal arithmetic (used when !shortcut) ==========================

    // Exponent difference + big/small selection. Unsigned operands;
    // zero-extend by one bit to get a signed diff for sign-detection.
    wire signed [EXP_BITS:0] raw_diff = $signed({1'b0, a_exp})
                                       - $signed({1'b0, b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire        [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;
    wire        [EXP_BITS-1:0]  small_exp  = a_is_big ? b_exp  : a_exp;

    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;
    wire negligible = (exp_diff >= FRAC_BITS - 1);

    wire negate_small = sub &  a_is_big;
    wire negate_big   = sub & !a_is_big;

    // Bypass: operand is negligible at the other's precision.
    wire big_is_pos_one = (big_frac == POS_ONE_NORMAL);
    wire big_is_neg_one = (big_frac == NEG_ONE_NORMAL);

    wire [EXP_BITS-1:0] big_exp_m1 = big_exp - 1'b1;
    wire [EXP_BITS-1:0] big_exp_p1 = big_exp + 1'b1;
    wire big_exp_m1_ambig = (big_exp_m1 == AMBIG_EXP);
    wire big_exp_p1_ambig = (big_exp_p1 == AMBIG_EXP);

    wire signed [FRAC_BITS-1:0] neg_big_frac =
        big_is_pos_one ? (big_exp_m1_ambig ? NEG_ONE_VANISHED : NEG_ONE_NORMAL) :
        big_is_neg_one ? (big_exp_p1_ambig ? POS_ONE_EXPLODED : POS_ONE_NORMAL) :
                         -big_frac;
    wire [EXP_BITS-1:0] neg_big_exp =
        big_is_pos_one ? (big_exp_m1_ambig ? AMBIG_EXP : big_exp_m1) :
        big_is_neg_one ? (big_exp_p1_ambig ? AMBIG_EXP : big_exp_p1) :
                         big_exp;

    wire bypass_add = negligible & !negate_big;
    wire bypass_sub = negligible &  negate_big;

    // N0 inflate to FRAC+1 bits, then sign-extend to FRAC+2 (WORK_BITS).
    wire signed [FRAC_BITS:0]   big_infl   = {~big_frac[FRAC_BITS-1],   big_frac};
    wire signed [FRAC_BITS:0]   small_infl = {~small_frac[FRAC_BITS-1], small_frac};
    wire signed [WORK_BITS-1:0] big_ext    = {big_infl[FRAC_BITS],   big_infl};
    wire signed [WORK_BITS-1:0] small_ext  = {small_infl[FRAC_BITS], small_infl};

    // Conditional negation (in FRAC+2 — wider than FRAC+1 — so negating
    // the most-negative value −2^FRAC doesn't overflow).
    wire signed [WORK_BITS-1:0] big_eff   = negate_big   ? -big_ext   : big_ext;
    wire signed [WORK_BITS-1:0] small_eff = negate_small ? -small_ext : small_ext;

    // Big never shifts. Small shifts RIGHT by exp_diff. The single GUARD bit
    // (= bit (shift_amt - 1) of small_eff, i.e., the highest bit discarded by
    // the arith-shr) is enough to compute floor correctly at the result LSB:
    //   • For close path (shift <= 1), this is also the only lost bit.
    //   • For far path (shift >= 2), |result| ≈ |big| so shl_amount ≤ 1; the
    //     residual at canonical bit 0 is exactly the guard bit (bits below
    //     guard floor away at the result's LSB).
    // The sticky-OR (= OR of all lost bits) would over-count: it can't tell
    // "guard=0, lower=1" (floor truncates to 0) from "guard=1, lower=0"
    // (floor keeps the 1) when shift ≥ 2.
    wire [SHIFT_BITS-1:0] shift_amt = exp_diff[SHIFT_BITS-1:0];
    wire signed [WORK_BITS-1:0] small_aligned = small_eff >>> shift_amt;
    wire [SHIFT_BITS-1:0] guard_shr = (shift_amt > 0) ? (shift_amt - 1'b1) : {SHIFT_BITS{1'b0}};
    wire signed [WORK_BITS-1:0] guard_shifted = small_eff >>> guard_shr;
    wire guard_bit = (shift_amt > 0) & guard_shifted[0];

    // Sum in FRAC+2 bits. Max per-operand magnitude is 2^FRAC, so sum is
    // bounded by 2^(FRAC+1), fitting in FRAC+2 signed.
    wire signed [WORK_BITS-1:0] sum = big_eff + small_aligned;

    // Extended sum: append guard bit at position −1, giving a (WORK_BITS+1)-bit
    // value at "half-big-ULP" scale (same scale as Rust's shift-big-LEFT for
    // shift=1). Working at this scale lets the sum_floored=0 cancellation case
    // resolve naturally: when sum=0 and guard=1, the extended sum is just 1
    // (= one ULP at the finer scale), which normalize handles as a normal
    // POS_ONE_NORMAL @ (big_exp - FRAC) or vanished if that underflows.
    localparam WORK_BITS_EXT = WORK_BITS + 1;
    localparam SHIFT_BITS_EXT = $clog2(WORK_BITS_EXT);
    wire signed [WORK_BITS_EXT-1:0] extended_sum = {sum, guard_bit};
    wire is_zero_ext = (extended_sum == 0);

    // Leading-same count on extended_sum at WORK_BITS_EXT bits.
    // Target for normalized N0 in FRAC+3 form is leading-same = 3 (top 3 are sign).
    wire [WORK_BITS_EXT-2:0] xor_bits = extended_sum[WORK_BITS_EXT-1:1]
                                     ^ extended_sum[WORK_BITS_EXT-2:0];
    reg [SHIFT_BITS_EXT:0] leading;
    integer ci;
    always @(*) begin
        leading = WORK_BITS_EXT;
        for (ci = 0; ci < WORK_BITS_EXT - 1; ci = ci + 1)
            if (xor_bits[ci]) leading = WORK_BITS_EXT - 1 - ci[SHIFT_BITS_EXT-1:0];
    end

    // Normalize: shl_amount > 0 → left shift (cancellation), < 0 → right shift (overflow).
    wire signed [SHIFT_BITS_EXT+1:0] shl_amount = $signed({1'b0, leading}) - 3;

    wire signed [WORK_BITS_EXT-1:0] canonical =
        shl_amount >= 0 ? (extended_sum <<<  shl_amount[SHIFT_BITS_EXT-1:0])
                        : (extended_sum >>> (-shl_amount[SHIFT_BITS_EXT:0]));

    wire signed [FRAC_BITS-1:0] out_frac = canonical[FRAC_BITS-1:0];

    // We worked at extended scale (= big_exp - 1), so out_exp shifts by one extra.
    // big_exp is UNSIGNED so we zero-extend (not sign-extend) when widening
    // to signed ECW bits. shl_amount remains signed (can be negative).
    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS){1'b0}}, big_exp})
        - 1'b1
        - $signed({{(ECW-SHIFT_BITS_EXT-2){shl_amount[SHIFT_BITS_EXT+1]}}, shl_amount});

    wire [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    // Range check against unsigned stored bounds [MIN_EXP=1, MAX_EXP=2^EXP-1].
    wire underflow = (exp_calc < $signed({1'b0, MIN_EXP}));
    wire overflow  = (exp_calc > $signed({1'b0, MAX_EXP}));

    // Escape outputs.
    wire signed [SHIFT_BITS_EXT+1:0] exp_shl = shl_amount - 1;
    wire signed [SHIFT_BITS_EXT+1:0] van_shl = shl_amount - 2;

    wire signed [WORK_BITS_EXT-1:0] exp_wide =
        exp_shl >= 0 ? (extended_sum <<<  exp_shl[SHIFT_BITS_EXT-1:0])
                     : (extended_sum >>> (-exp_shl[SHIFT_BITS_EXT:0]));
    wire signed [WORK_BITS_EXT-1:0] van_wide =
        van_shl >= 0 ? (extended_sum <<<  van_shl[SHIFT_BITS_EXT-1:0])
                     : (extended_sum >>> (-van_shl[SHIFT_BITS_EXT:0]));

    wire signed [FRAC_BITS-1:0] exploded_frac = exp_wide[FRAC_BITS-1:0];
    wire signed [FRAC_BITS-1:0] vanished_frac = van_wide[FRAC_BITS-1:0];

    // ========== Output mux ==========================================================
    //
    // Priority: shortcut > bypass > zero > overflow > underflow > normal.

    assign result_frac = shortcut                   ? sc_frac :
                         bypass_add                 ? big_frac :
                         bypass_sub                 ? neg_big_frac :
                         is_zero_ext                ? {FRAC_BITS{1'b0}} :
                         overflow                   ? exploded_frac :
                         underflow                  ? vanished_frac :
                                                      out_frac;

    assign result_exp  = shortcut                  ? sc_exp :
                         bypass_add                ? big_exp :
                         bypass_sub                ? neg_big_exp :
                         (is_zero_ext | overflow | underflow) ? AMBIG_EXP :
                                                                out_exp;

endmodule
