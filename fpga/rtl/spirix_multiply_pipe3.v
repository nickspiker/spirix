// spirix_multiply_pipe3 — 3-stage pipelined multiply for Spirix scalars
//
// Computes a * b on N1-normalized signed fractions with signed exponents.
// Fully parameterized. Latency: 3 cycles. Throughput: 1 result per clock.
//
//   Stage 1: DSP multiply + exponent sum.
//       The signed multiply infers MULT18X18D with output registers on ECP5.
//       Exponent add runs in parallel on fabric.
//
//   Stage 2: Normalize + extract + banker's round + early rovf flags.
//       Critical path: norm mux -> sticky OR -> round_up -> FRAC-bit add.
//       Early rovf flags computed from pre-round signals (independent of
//       rounding adder), registered alongside frac_rounded.
//
//   Stage 3: Rounding overflow mux + exponent adjust + overflow/underflow
//       clamp. Light logic: rovf mux + EXP-bit add + compare + output mux.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_multiply_pipe3 #(
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
    localparam PROD_BITS = 2 * FRAC_BITS - 1;
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    // =========================================================================
    // Stage 1: DSP multiply + exponent sum
    // =========================================================================
    wire signed [PROD_BITS-1:0] product_comb = a_frac * b_frac;
    wire signed [EXP_BITS:0] exp_sum = $signed({a_exp[EXP_BITS-1], a_exp})
                                      + $signed({b_exp[EXP_BITS-1], b_exp});

    reg signed [PROD_BITS-1:0] s1_product;
    reg signed [EXP_BITS:0]    s1_exp_sum;

    always @(posedge clk) begin
        s1_product <= product_comb;
        s1_exp_sum <= exp_sum;
    end

    // =========================================================================
    // Stage 2: Normalize + extract + round + early rovf flags
    //
    // Critical path: norm mux -> OR-reduce sticky -> round_up -> FRAC-bit add.
    // Early rovf flags are computed from frac_raw + round_up (independent of
    // the rounding adder result), so they don't extend the critical path.
    // =========================================================================
    wire s2_is_n1 = (s1_product[PROD_BITS-1] != s1_product[PROD_BITS-2]);
    wire s2_norm_shift = !s2_is_n1;
    wire signed [PROD_BITS-1:0] s2_normalized = s2_norm_shift ?
                                                 (s1_product <<< 1) : s1_product;

    wire signed [FRAC_BITS-1:0] s2_frac_raw = s2_normalized[PROD_BITS-1 -: FRAC_BITS];
    wire s2_guard  = s2_normalized[PROD_BITS - 1 - FRAC_BITS];
    wire s2_sticky = (PROD_BITS - 2 - FRAC_BITS >= 0) ?
                     |s2_normalized[PROD_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire s2_lsb    = s2_frac_raw[0];
    wire s2_round_up = s2_guard & (s2_sticky | s2_lsb);

    wire signed [FRAC_BITS-1:0] s2_frac_rounded = s2_frac_raw
                                                   + {{(FRAC_BITS-1){1'b0}}, s2_round_up};

    // Early rounding overflow detection (from pre-round signals only)
    wire s2_rovf_pos = !s2_frac_raw[FRAC_BITS-1]
                     & (&s2_frac_raw[FRAC_BITS-2:0])
                     & s2_round_up;
    wire s2_rovf_neg = s2_frac_raw[FRAC_BITS-1]
                     & !s2_frac_raw[FRAC_BITS-2]
                     & (&s2_frac_raw[FRAC_BITS-3:0])
                     & s2_round_up;

    reg signed [FRAC_BITS-1:0] s2_frac_rounded_r;
    reg s2_rovf_pos_r, s2_rovf_neg_r, s2_norm_shift_r;
    reg signed [EXP_BITS:0] s2_exp_sum_r;

    always @(posedge clk) begin
        s2_frac_rounded_r <= s2_frac_rounded;
        s2_rovf_pos_r     <= s2_rovf_pos;
        s2_rovf_neg_r     <= s2_rovf_neg;
        s2_norm_shift_r   <= s2_norm_shift;
        s2_exp_sum_r      <= s1_exp_sum;
    end

    // =========================================================================
    // Stage 3: Rounding overflow mux + exponent adjust + clamp
    //
    // Light logic: rovf mux (2 gates) + EXP-bit add + compare + output mux.
    // =========================================================================
    wire signed [FRAC_BITS-1:0] s3_out_frac = s2_rovf_pos_r ? POS_HALF :
                                               s2_rovf_neg_r ? NEG_ONE  :
                                               s2_frac_rounded_r;

    wire signed [EXP_BITS:0] s3_exp_wide = s2_exp_sum_r
                                           - {{EXP_BITS{1'b0}}, s2_norm_shift_r}
                                           + {{EXP_BITS{1'b0}}, s2_rovf_pos_r}
                                           - {{EXP_BITS{1'b0}}, s2_rovf_neg_r};

    wire s3_exp_too_big   = (s3_exp_wide > MAX_EXP);
    wire s3_exp_too_small = (s3_exp_wide < MIN_EXP);
    wire signed [EXP_BITS-1:0] s3_out_exp = s3_exp_wide[EXP_BITS-1:0];

    always @(posedge clk) begin
        result_frac <= s3_exp_too_big   ? s3_out_frac :
                       s3_exp_too_small ? {s3_out_frac[FRAC_BITS-1],
                                           s3_out_frac[FRAC_BITS-1:1]} :
                                          s3_out_frac;

        result_exp  <= (s3_exp_too_big | s3_exp_too_small) ?
                        AMBIGUOUS_EXP[EXP_BITS-1:0] : s3_out_exp;
    end

endmodule
