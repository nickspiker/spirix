// spirix_divide — Minimal iterative divider for Spirix scalars
//
// Floor-only (no rounding), restoring binary long division.
// Parameterized: FRAC_BITS in {2..256}, EXP_BITS in {2..256}.
//
// Latency: FRAC_BITS + 2 cycles from start to done.
//   Cycle 0 (start):       Abs values, first trial subtract.
//   Cycles 1..FRAC:        Restoring division (shift + subtract).
//   Cycle FRAC+1:          Finalize (normalize/floor/sign from registers).
//
// Interface:
//   start:  pulse high for 1 cycle to begin. Inputs sampled on this edge.
//   busy:   high while computing. Do not assert start while busy.
//   done:   pulses high for 1 cycle when result_frac/result_exp are valid.
//
// Algorithm:
//   1. Extract signs, take absolute values (NEG_ONE → POS_HALF with exp+1).
//   2. Unsigned restoring division: FRAC+1 quotient bits, 1 bit per cycle.
//   3. Euclidean adjustment: if numerator was negative and remainder != 0,
//      increment quotient (matches Rust's div_euclid semantics).
//   4. Bounded normalize (0 or 1 bit shift).
//   5. Extract FRAC magnitude bits. For negative results with lost precision
//      (truncated bits or remainder), add 1 to magnitude (floor toward -inf).
//   6. Apply sign, clamp exponent.
//
// Matches Rust scalar_divide_scalar exactly (Euclidean division, floor).
// 0 DSP. No barrel shifter.

module spirix_divide #(
    parameter FRAC_BITS = 32,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire start,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp,
    output wire busy,
    output reg  done = 0
);

    localparam AMBIG_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam CTR_BITS = $clog2(FRAC_BITS + 1);
    localparam ECW = (EXP_BITS + 2 > $clog2(FRAC_BITS) + 2)
                   ? EXP_BITS + 2 : $clog2(FRAC_BITS) + 2;

    localparam S_IDLE     = 0;
    localparam S_COMPUTE  = 1;
    localparam S_FINALIZE = 2;
    reg [1:0] state = S_IDLE;

    assign busy = (state != S_IDLE);

    // Registered operands
    reg [FRAC_BITS-2:0]     d_reg;       // divisor magnitude
    reg [FRAC_BITS-1:0]     r_reg;       // partial remainder
    reg [FRAC_BITS:0]       q_reg;       // quotient shift register (FRAC+1 bits)
    reg [CTR_BITS-1:0]      counter;
    reg                      sign_reg;    // result sign (XOR of input signs)
    reg                      a_neg_reg;   // numerator was negative (for Euclidean adj)
    reg signed [EXP_BITS:0] a_exp_adj_r; // adjusted exponents
    reg signed [EXP_BITS:0] b_exp_adj_r;
    reg                      b_zero_reg;

    // == Combinational: absolute values from input ports ========================

    wire a_is_neg_one = (a_frac == NEG_ONE);
    wire b_is_neg_one = (b_frac == NEG_ONE);

    wire [FRAC_BITS-2:0] abs_a_comb = a_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                       (a_frac[FRAC_BITS-1] ? (~a_frac[FRAC_BITS-2:0] + 1'b1) :
                                                               a_frac[FRAC_BITS-2:0]);
    wire [FRAC_BITS-2:0] abs_b_comb = b_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                       (b_frac[FRAC_BITS-1] ? (~b_frac[FRAC_BITS-2:0] + 1'b1) :
                                                               b_frac[FRAC_BITS-2:0]);

    // == Shared trial subtraction ===============================================

    wire starting = (state == S_IDLE) & start;

    wire [FRAC_BITS-1:0] r_in = starting ? {1'b0, abs_a_comb} :
                                            {r_reg[FRAC_BITS-2:0], 1'b0};
    wire [FRAC_BITS-2:0] d_mux = starting ? abs_b_comb : d_reg;

    wire [FRAC_BITS:0] trial = {1'b0, r_in} - {2'b00, d_mux};
    wire ge = !trial[FRAC_BITS];
    wire [FRAC_BITS-1:0] r_next = ge ? trial[FRAC_BITS-1:0] : r_in;

    // == Finalization (from registered q_reg/r_reg in S_FINALIZE) ===============

    // Euclidean adjustment: when numerator was negative and remainder nonzero,
    // Rust's div_euclid returns a quotient that's 1 larger in magnitude.
    wire has_remainder = |r_reg;
    wire euclid_adj = a_neg_reg & has_remainder;
    wire [FRAC_BITS:0] q_adj = q_reg + {{FRAC_BITS{1'b0}}, euclid_adj};

    // Bounded normalize: if quotient MSB is set, shift right by 1.
    wire norm_shift = q_adj[FRAC_BITS];
    wire [FRAC_BITS:0] q_norm = norm_shift ? (q_adj >> 1) : q_adj;

    // Extract FRAC magnitude bits: q_norm[FRAC:1].
    // q_norm[0] is truncated during extraction.
    // With norm_shift, q_adj[0] was also truncated.
    wire [FRAC_BITS-1:0] frac_pos = q_norm[FRAC_BITS:1];

    // Floor adjustment for negative results:
    // Unsigned truncation rounds toward zero. For negative results, floor
    // rounds toward -inf, which means adding 1 to the unsigned magnitude
    // whenever bits were truncated during extraction.
    // Note: has_remainder is NOT included — after Euclidean adjustment,
    // Q' is the exact integer quotient; only the extraction loses bits.
    wire trunc_bits = q_norm[0] | (norm_shift & q_adj[0]);
    wire neg_floor_adj = sign_reg & trunc_bits;
    wire [FRAC_BITS-1:0] frac_adj = frac_pos + {{(FRAC_BITS-1){1'b0}}, neg_floor_adj};

    // Floor adjustment overflow: frac_pos was 0111...1, +1 → POS_HALF magnitude.
    wire floor_ovf = neg_floor_adj & (&frac_pos[FRAC_BITS-2:0]);

    // Apply sign
    wire mag_is_half = (frac_adj == POS_HALF);
    wire signed [FRAC_BITS-1:0] neg_frac = mag_is_half ?
                                            NEG_ONE :
                                            (~frac_adj + 1'b1);
    wire signed [FRAC_BITS-1:0] pos_frac_signed = $signed(frac_adj);
    wire signed [FRAC_BITS-1:0] final_frac = sign_reg ? neg_frac : pos_frac_signed;

    // Exponent = a_exp - b_exp + norm_shift + floor_ovf - neg_half_adj
    wire neg_half_adj = sign_reg & mag_is_half;
    wire signed [ECW-1:0] exp_calc =
        $signed({{(ECW-EXP_BITS-1){a_exp_adj_r[EXP_BITS]}}, a_exp_adj_r})
        - $signed({{(ECW-EXP_BITS-1){b_exp_adj_r[EXP_BITS]}}, b_exp_adj_r})
        + {{(ECW-1){1'b0}}, norm_shift}
        + {{(ECW-1){1'b0}}, floor_ovf}
        - {{(ECW-1){1'b0}}, neg_half_adj};

    localparam signed [ECW-1:0] MAX_EXP_W = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [ECW-1:0] MIN_EXP_W = -(1 <<< (EXP_BITS - 1)) + 1;

    wire overflow  = (exp_calc > MAX_EXP_W);
    wire underflow = (exp_calc < MIN_EXP_W);
    wire signed [EXP_BITS-1:0] out_exp = exp_calc[EXP_BITS-1:0];

    wire signed [FRAC_BITS-1:0] vanished_frac = {final_frac[FRAC_BITS-1],
                                                   final_frac[FRAC_BITS-1:1]};

    wire signed [FRAC_BITS-1:0] out_frac = b_zero_reg  ? {FRAC_BITS{1'b0}} :
                                            overflow    ? final_frac :
                                            underflow   ? vanished_frac :
                                                          final_frac;

    wire signed [EXP_BITS-1:0] out_exp_final =
        (b_zero_reg | overflow | underflow) ? AMBIG_EXP[EXP_BITS-1:0] : out_exp;

    // == State machine ==========================================================

    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    sign_reg <= a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
                    a_neg_reg <= a_frac[FRAC_BITS-1];
                    b_zero_reg <= (b_frac == 0);

                    a_exp_adj_r <= $signed({a_exp[EXP_BITS-1], a_exp})
                                 + {{EXP_BITS{1'b0}}, a_is_neg_one};
                    b_exp_adj_r <= $signed({b_exp[EXP_BITS-1], b_exp})
                                 + {{EXP_BITS{1'b0}}, b_is_neg_one};

                    d_reg <= abs_b_comb;
                    r_reg <= r_next;
                    q_reg <= {{FRAC_BITS{1'b0}}, ge};

                    counter <= FRAC_BITS[CTR_BITS-1:0] - 1;
                    state <= S_COMPUTE;
                end
            end

            S_COMPUTE: begin
                r_reg <= r_next;
                q_reg <= {q_reg[FRAC_BITS-1:0], ge};

                if (counter == 0) begin
                    state <= S_FINALIZE;
                end else begin
                    counter <= counter - 1;
                end
            end

            S_FINALIZE: begin
                result_frac <= out_frac;
                result_exp  <= out_exp_final;
                done <= 1'b1;
                state <= S_IDLE;
            end
        endcase
    end

endmodule
