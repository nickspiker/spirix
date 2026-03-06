// spirix_addsub — Minimal combinational add/subtract for Spirix scalars
//
// Floor-only (no rounding), single-path design.
// Parameterized: FRAC_BITS in {2..256}, EXP_BITS in {2..256}.
//
// Architecture: left-shift big -> add -> CLZ -> normalize -> extract (floor).
//   Matches the Rust reference model exactly (2x intermediate width,
//   single floor point at extraction).
//   No close/far split. No rounding. No sticky tracking.
//   Barrel shifters inferred by synthesis from <<< operators.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction: signed two's complement, N1-normalized (top two bits differ).
// Exponent: signed two's complement. AMBIGUOUS_EXP = -2^(EXP_BITS-1)
// encodes zero/exploded/vanished/infinity/undefined.

module spirix_addsub #(
    parameter FRAC_BITS = 32,
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

    // 2x width intermediate — matches Rust i16/i32/i64 promotion.
    // Single floor at extraction; no precision loss from alignment.
    localparam INT_BITS   = 2 * FRAC_BITS;
    localparam SHIFT_BITS = $clog2(INT_BITS);
    localparam AMBIG_EXP  = -(1 <<< (EXP_BITS - 1));

    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};

    // Exponent calc width: enough to hold small_exp + FRAC - norm_shift
    localparam ECW = (SHIFT_BITS + 1 > EXP_BITS + 1) ? SHIFT_BITS + 2 : EXP_BITS + 2;

    // == Step 1: Exponent difference + swap ===================================

    wire signed [EXP_BITS:0] raw_diff = $signed({a_exp[EXP_BITS-1], a_exp})
                                       - $signed({b_exp[EXP_BITS-1], b_exp});
    wire a_is_big = !raw_diff[EXP_BITS];

    wire signed [FRAC_BITS-1:0] big_frac   = a_is_big ? a_frac : b_frac;
    wire signed [EXP_BITS-1:0]  big_exp    = a_is_big ? a_exp  : b_exp;
    wire signed [FRAC_BITS-1:0] small_frac = a_is_big ? b_frac : a_frac;
    wire signed [EXP_BITS-1:0]  small_exp  = a_is_big ? b_exp  : a_exp;

    wire signed [EXP_BITS:0] exp_diff = raw_diff[EXP_BITS] ? -raw_diff : raw_diff;
    wire negligible = (exp_diff >= FRAC_BITS);

    wire negate_small = sub &  a_is_big;
    wire negate_big   = sub & !a_is_big;

    // == Bypass: negligible operand ===========================================
    //
    // When exp_diff >= FRAC_BITS, small is less than 1 ULP of big.
    // Add: return big. Sub with b as big: return -big (Spirix negation).
    // Negation special cases: -POS_HALF and -NEG_ONE need exponent adjust
    // to stay N1-normalized.

    wire big_is_pos_half = (big_frac == POS_HALF);
    wire big_is_neg_one  = (big_frac == NEG_ONE);

    wire signed [FRAC_BITS-1:0] neg_big_frac = big_is_pos_half ? NEG_ONE  :
                                                 big_is_neg_one  ? POS_HALF :
                                                 -big_frac;
    wire signed [EXP_BITS-1:0]  neg_big_exp  = big_is_pos_half ? (big_exp - 1) :
                                                 big_is_neg_one  ? (big_exp + 1) :
                                                 big_exp;

    wire bypass_add = negligible & !negate_big;
    wire bypass_sub = negligible &  negate_big;

    // == Step 2: Extend to 2x width ==========================================

    wire signed [INT_BITS-1:0] big_ext   = $signed(big_frac);
    wire signed [INT_BITS-1:0] small_ext = $signed(small_frac);

    wire [SHIFT_BITS-1:0] shift_amt = (exp_diff >= FRAC_BITS)
                                     ? FRAC_BITS[SHIFT_BITS-1:0] - 1
                                     : exp_diff[SHIFT_BITS-1:0];
    wire signed [INT_BITS-1:0] big_shifted = big_ext <<< shift_amt;

    // == Step 3: Add / subtract ===============================================

    wire signed [INT_BITS-1:0] sum = (big_shifted ^ {INT_BITS{negate_big}})
                                   + (small_ext   ^ {INT_BITS{negate_small}})
                                   + {{(INT_BITS-1){1'b0}}, sub};
    wire is_zero = (sum == 0);

    // == Step 4: CLZ (count leading sign bits - 1) ============================

    wire [INT_BITS-2:0] xor_bits = sum[INT_BITS-1:1] ^ sum[INT_BITS-2:0];

    reg [SHIFT_BITS-1:0] norm_shift;
    integer ci;
    always @(*) begin
        norm_shift = INT_BITS - 1;
        for (ci = 0; ci < INT_BITS - 1; ci = ci + 1)
            if (xor_bits[ci]) norm_shift = (INT_BITS - 2) - ci[SHIFT_BITS-1:0];
    end

    // == Step 5: Normalize (left shift) + extract top FRAC_BITS (floor) =======

    wire signed [INT_BITS-1:0] normalized = sum <<< norm_shift;
    wire signed [FRAC_BITS-1:0] out_frac = normalized[INT_BITS-1 -: FRAC_BITS];

    // == Step 6: Exponent =====================================================
    //
    // result_exp = small_exp + (FRAC_BITS - leading) + 1
    //            = small_exp + FRAC_BITS - (norm_shift + 1) + 1
    //            = small_exp + FRAC_BITS - norm_shift

    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS){small_exp[EXP_BITS-1]}}, small_exp})
        + FRAC_BITS
        - $signed({{(ECW-SHIFT_BITS){1'b0}}, norm_shift});

    wire signed [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    wire underflow = (exp_calc < (AMBIG_EXP + 1));

    // Vanished fraction: duplicate sign bit -> N2 prefix pattern
    wire signed [FRAC_BITS-1:0] vanished_frac = {normalized[INT_BITS-1],
                                                   normalized[INT_BITS-1 -: FRAC_BITS-1]};

    // == Step 7: Output mux ===================================================

    assign result_frac = bypass_add ? big_frac :
                         bypass_sub ? neg_big_frac :
                         is_zero    ? {FRAC_BITS{1'b0}} :
                         underflow  ? vanished_frac :
                                      out_frac;

    assign result_exp  = bypass_add ? big_exp :
                         bypass_sub ? neg_big_exp :
                         (is_zero | underflow) ? AMBIG_EXP[EXP_BITS-1:0] :
                                                  out_exp;

endmodule
