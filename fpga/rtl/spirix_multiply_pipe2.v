// spirix_multiply_pipe2 — 2-stage pipelined multiply for Spirix scalars
//
// Computes a * b on N1-normalized signed fractions with signed exponents.
// Fully parameterized. Latency: 2 cycles. Throughput: 1 result per clock.
//
//   Stage 1: DSP multiply + exponent sum.
//       The signed multiply infers MULT18X18D with output registers on ECP5.
//       Exponent add runs in parallel on fabric.
//
//   Stage 2: Normalize + extract + banker's round + early rovf +
//       exponent adjust + overflow/underflow clamp.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_multiply_pipe2 #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire ce,
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
    //
    // The signed multiply infers MULT18X18D with output registers on ECP5.
    // Exponent add runs in parallel on fabric.
    // =========================================================================
    wire signed [PROD_BITS-1:0] product_comb = a_frac * b_frac;

    wire signed [EXP_BITS:0] exp_sum = $signed({a_exp[EXP_BITS-1], a_exp})
                                      + $signed({b_exp[EXP_BITS-1], b_exp});

    reg signed [PROD_BITS-1:0] s1_product;
    reg signed [EXP_BITS:0]    s1_exp_sum;

    always @(posedge clk) if (ce) begin
        s1_product <= product_comb;
        s1_exp_sum <= exp_sum;
    end

    // =========================================================================
    // Stage 2: Normalize + round + exponent adjust + clamp
    //
    // All combinational from s1 registers, result captured in output register.
    // =========================================================================

    // Bounded normalization (0 or 1 bit)
    wire is_n1 = (s1_product[PROD_BITS-1] != s1_product[PROD_BITS-2]);
    wire norm_shift = !is_n1;
    wire signed [PROD_BITS-1:0] normalized = norm_shift ? (s1_product <<< 1) : s1_product;

    // Extract + banker's round (RNE)
    wire signed [FRAC_BITS-1:0] frac_raw = normalized[PROD_BITS-1 -: FRAC_BITS];
    wire guard  = normalized[PROD_BITS - 1 - FRAC_BITS];
    wire sticky = (PROD_BITS - 2 - FRAC_BITS >= 0) ?
                  |normalized[PROD_BITS - 2 - FRAC_BITS:0] : 1'b0;
    wire lsb    = frac_raw[0];
    wire round_up = guard & (sticky | lsb);

    wire signed [FRAC_BITS-1:0] frac_rounded = frac_raw + {{(FRAC_BITS-1){1'b0}}, round_up};

    // Early rounding overflow detection (from pre-round signals)
    wire rovf_pos = !frac_raw[FRAC_BITS-1]
                  & (&frac_raw[FRAC_BITS-2:0])
                  & round_up;
    wire rovf_neg = frac_raw[FRAC_BITS-1]
                  & !frac_raw[FRAC_BITS-2]
                  & (&frac_raw[FRAC_BITS-3:0])
                  & round_up;

    wire signed [FRAC_BITS-1:0] out_frac = rovf_pos ? POS_HALF :
                                             rovf_neg ? NEG_ONE  :
                                             frac_rounded;

    // Exponent adjust
    wire signed [EXP_BITS:0] exp_wide = s1_exp_sum
                                        - {{EXP_BITS{1'b0}}, norm_shift}
                                        + {{EXP_BITS{1'b0}}, rovf_pos}
                                        - {{EXP_BITS{1'b0}}, rovf_neg};

    wire exp_too_big   = (exp_wide > MAX_EXP);
    wire exp_too_small = (exp_wide < MIN_EXP);
    wire signed [EXP_BITS-1:0] out_exp = exp_wide[EXP_BITS-1:0];

    // Stage 2 output register
    always @(posedge clk) if (ce) begin
        result_frac <= exp_too_big   ? out_frac :
                       exp_too_small ? {out_frac[FRAC_BITS-1],
                                        out_frac[FRAC_BITS-1:1]} :
                                       out_frac;

        result_exp  <= (exp_too_big | exp_too_small) ?
                        AMBIGUOUS_EXP[EXP_BITS-1:0] : out_exp;
    end

endmodule
