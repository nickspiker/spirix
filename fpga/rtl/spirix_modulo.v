// Spirix Modulo: a % b (combinational)
// Parameterized combinational modulo for Spirix floating-point scalars.
//
// Definition: a % b = a - floor(a/b) * b
// Sign convention: mathematical modulus (result sign matches divisor).
//
// Implementation: compose divide → floor → multiply → subtract.
// Each sub-operation uses a dedicated module instance.

module spirix_modulo #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output wire signed [FRAC_BITS-1:0] result_frac,
    output wire signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};

    // =========================================================================
    // Step 1: Divide a / b
    // =========================================================================
    wire signed [FRAC_BITS-1:0] q_frac;
    wire signed [EXP_BITS-1:0]  q_exp;

    spirix_divide #(.FRAC_BITS(FRAC_BITS), .EXP_BITS(EXP_BITS)) div (
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(q_frac), .result_exp(q_exp)
    );

    // =========================================================================
    // Step 2: Floor the quotient
    //   For N1 scalar (frac, exp):
    //     exp >= FRAC-1: value is integer, floor = self
    //     exp < 0:       |value| < 1, floor = 0 (pos) or (-1,0) (neg non-zero)
    //     0 <= exp < FRAC-1: mask off fractional bits, round toward -inf
    // =========================================================================
    wire signed [EXP_BITS-1:0] q_exp_clamped = (q_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]) ?
                                                 q_exp : q_exp;
    wire q_is_ambiguous = (q_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]);
    wire q_is_negative = q_frac[FRAC_BITS-1];

    // Number of fractional bits to discard: (FRAC-1) - exp
    // If exp >= FRAC-1: discard 0 (all integer)
    // If exp < 0: discard all (FRAC-1 or more)
    wire signed [EXP_BITS:0] frac_count = (FRAC_BITS - 1) - $signed({q_exp[EXP_BITS-1], q_exp});
    wire exp_large = (frac_count <= 0);     // all integer, no truncation needed
    wire exp_small = (frac_count >= FRAC_BITS); // all fractional

    // Build mask to zero out fractional bits
    // mask = ~((1 << frac_count) - 1) when 0 < frac_count < FRAC
    wire [FRAC_BITS-1:0] ones_mask;
    wire [$clog2(FRAC_BITS)-1:0] shift_amt = frac_count[$clog2(FRAC_BITS)-1:0];

    // Decoder: generate (1 << shift_amt) - 1
    wire [FRAC_BITS-1:0] low_mask = ({{(FRAC_BITS-1){1'b0}}, 1'b1} << shift_amt) - 1;
    wire [FRAC_BITS-1:0] keep_mask = ~low_mask;

    // Truncated fraction (toward zero)
    wire signed [FRAC_BITS-1:0] trunc_frac = q_frac & $signed(keep_mask);

    // For floor (toward -inf): if negative and any fractional bits were set,
    // subtract 1 from the integer part. But since we work in N1, "subtract 1"
    // means calling the subtract module. Instead, detect if truncation lost
    // any bits and add -1 step (via the truncated integer representation).
    wire has_frac_bits = |(q_frac & $signed(low_mask));
    wire need_floor_adj = q_is_negative & has_frac_bits;

    // Floor adjustment for negative: trunc_frac - (1 << frac_count)
    // This is equivalent to decrementing the integer part by 1.
    // In the N1 fraction representation, subtracting the unit at position frac_count
    // is: trunc_frac - low_mask - 1 = trunc_frac + ~low_mask + 1 - low_mask - 1
    // Simpler: trunc_frac - (low_mask + 1) = trunc_frac - (1 << frac_count)
    wire [FRAC_BITS-1:0] unit = {{(FRAC_BITS-1){1'b0}}, 1'b1} << shift_amt;
    wire signed [FRAC_BITS-1:0] floor_frac_adj = trunc_frac - $signed(unit);

    // Choose floor result
    wire signed [FRAC_BITS-1:0] floor_frac_mid = need_floor_adj ? floor_frac_adj : trunc_frac;

    // Handle the edge cases: need to renormalize if floor result is not N1
    // Check N1: top 2 bits must differ
    wire floor_mid_n1 = (floor_frac_mid[FRAC_BITS-1] != floor_frac_mid[FRAC_BITS-2]);

    // If result is zero (all fractional bits masked off and integer part is 0)
    wire floor_is_zero = (floor_frac_mid == 0);

    // If not N1 and not zero, need to renormalize (find leading same, shift)
    // For the floor operation, the result might need multi-bit normalization.
    // Use a simple CLZ-based approach.

    // Leading same count (combinational)
    reg [$clog2(FRAC_BITS):0] floor_leading;
    integer fi;
    always @(*) begin
        floor_leading = FRAC_BITS;
        for (fi = FRAC_BITS-2; fi >= 0; fi = fi - 1)
            if (floor_frac_mid[fi] != floor_frac_mid[FRAC_BITS-1] && floor_leading == FRAC_BITS)
                floor_leading = FRAC_BITS - 1 - fi;
    end

    wire [$clog2(FRAC_BITS):0] floor_norm_shift = floor_leading - 1;
    wire signed [FRAC_BITS-1:0] floor_frac_norm = floor_frac_mid <<< floor_norm_shift;
    wire signed [EXP_BITS-1:0]  floor_exp_norm = q_exp - $signed({{1'b0}, floor_norm_shift[$clog2(FRAC_BITS)-1:0]});

    // Select floor output
    // ONE = POS_HALF with exp+1 (represents 1.0)
    wire signed [FRAC_BITS-1:0] neg_one_frac = NEG_ONE;
    wire signed [EXP_BITS-1:0]  neg_one_exp  = 1'b0; // -1.0 * 2^0 = -1.0

    wire signed [FRAC_BITS-1:0] floor_frac;
    wire signed [EXP_BITS-1:0]  floor_exp;

    // AMB_EXP quotient: underflowed value near zero.
    //   Positive: floor ≈ 0 (handled downstream by floor_zero)
    //   Negative: floor = -1.0 = (NEG_ONE, 0)
    assign floor_frac = q_is_ambiguous ? (q_is_negative ? NEG_ONE : q_frac) :
                         exp_large      ? q_frac :          // all integer
                         exp_small      ? (q_is_negative ? NEG_ONE : {FRAC_BITS{1'b0}}) :
                         floor_is_zero  ? {FRAC_BITS{1'b0}} :
                         floor_mid_n1   ? floor_frac_mid :
                                          floor_frac_norm;

    assign floor_exp  = q_is_ambiguous ? (q_is_negative ? {EXP_BITS{1'b0}} : q_exp) :
                         exp_large      ? q_exp :
                         exp_small      ? (q_is_negative ? {EXP_BITS{1'b0}} : AMBIGUOUS_EXP[EXP_BITS-1:0]) :
                         floor_is_zero  ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                         floor_mid_n1   ? q_exp :
                                          floor_exp_norm;

    // =========================================================================
    // Step 3: Multiply floor(a/b) * b
    // =========================================================================
    wire signed [FRAC_BITS-1:0] prod_frac;
    wire signed [EXP_BITS-1:0]  prod_exp;

    // If floor is zero, product is zero (skip multiply)
    wire floor_zero = (floor_frac == 0) ||
                      (floor_exp == AMBIGUOUS_EXP[EXP_BITS-1:0] && !floor_frac[FRAC_BITS-1]);

    spirix_multiply #(.FRAC_BITS(FRAC_BITS), .EXP_BITS(EXP_BITS)) mul (
        .a_frac(floor_frac), .a_exp(floor_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(prod_frac), .result_exp(prod_exp)
    );

    // =========================================================================
    // Step 4: Subtract a - floor(a/b) * b
    // =========================================================================
    wire signed [FRAC_BITS-1:0] sub_frac;
    wire signed [EXP_BITS-1:0]  sub_exp;

    spirix_subtract #(.FRAC_BITS(FRAC_BITS), .EXP_BITS(EXP_BITS)) sub (
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(floor_zero ? {FRAC_BITS{1'b0}} : prod_frac),
        .b_exp(floor_zero ? AMBIGUOUS_EXP[EXP_BITS-1:0] : prod_exp),
        .result_frac(sub_frac), .result_exp(sub_exp)
    );

    assign result_frac = sub_frac;
    assign result_exp  = sub_exp;

endmodule
