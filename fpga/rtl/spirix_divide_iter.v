// spirix_divide_iter — Iterative sequential divider for Spirix scalars
//
// Computes a / b on N1-normalized signed fractions with signed exponents.
// Fully parameterized. Restoring division: 1 quotient bit per clock.
//
// Latency: FRAC_BITS + 1 cycles from start to done.
//   Cycle 0 (start):       First trial subtract folded into start cycle.
//   Cycles 1..FRAC-1:      Restoring division (shift + subtract).
//   Cycle FRAC (last):     Final trial subtract + normalize/round/output.
//
// The first and last iterations are folded into setup and finalization
// respectively, eliminating two dead cycles vs the naive approach.
//
// Interface:
//   start:  pulse high for 1 cycle to begin. Inputs sampled on this edge.
//   busy:   high while computing. Do not assert start while busy.
//   done:   pulses high for 1 cycle when result_frac/result_exp are valid.
//
// Area: one FRAC-bit subtractor + comparator (shared as trial subtract),
// one FRAC-bit partial-remainder register, one (FRAC+1)-bit quotient
// shift register, one counter. No DSP blocks. No barrel shifter.
//
// Timing note: the last cycle has a longer critical path than normal
// iterations (trial subtract -> finalization -> output register).
// Normal iterations: ~FRAC/2 CCU2C deep. Last cycle: ~FRAC CCU2C deep
// (rounding + sign negate carry chains). On ECP5 this limits Fmax to
// roughly 60-80 MHz for FRAC=25, which is fine for most designs.
//
// Algorithm: same as spirix_divide (sign out, unsigned restoring division,
// bounded normalize, banker's round, sign back in, clamp). See that
// module's header for the mathematical rationale.
//
// Division by zero returns (0, AMBIGUOUS_EXP) after the normal latency.
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_divide_iter #(
    parameter FRAC_BITS = 25,
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

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    localparam CTR_BITS = $clog2(FRAC_BITS + 1);

    // Two states only: IDLE and COMPUTE. Init and finalization are folded.
    localparam S_IDLE    = 0;
    localparam S_COMPUTE = 1;
    reg state = S_IDLE;

    assign busy = (state != S_IDLE);

    // Registered operands
    reg [FRAC_BITS-2:0]     d_reg;         // divisor magnitude
    reg [FRAC_BITS-1:0]     r_reg;         // partial remainder (FRAC bits)
    reg [FRAC_BITS:0]       q_reg;         // quotient shift register (FRAC+1 bits)
    reg [CTR_BITS-1:0]      counter;       // iteration counter
    reg                      sign_reg;      // result sign
    reg signed [EXP_BITS:0] a_exp_adj_r;   // sign-extended adjusted exponents
    reg signed [EXP_BITS:0] b_exp_adj_r;
    reg                      b_zero_reg;    // division by zero flag

    // =========================================================================
    // Combinational: absolute values from input ports (for start cycle)
    //
    // NEG_ONE (10...0) has magnitude 2^(FRAC-1) which overflows FRAC-1
    // unsigned bits. Represent as POS_HALF magnitude with exponent+1.
    // =========================================================================
    wire a_is_neg_one = (a_frac == NEG_ONE);
    wire b_is_neg_one = (b_frac == NEG_ONE);

    wire [FRAC_BITS-2:0] abs_a_comb = a_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                       (a_frac[FRAC_BITS-1] ? (~a_frac[FRAC_BITS-2:0] + 1'b1) :
                                                               a_frac[FRAC_BITS-2:0]);
    wire [FRAC_BITS-2:0] abs_b_comb = b_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                       (b_frac[FRAC_BITS-1] ? (~b_frac[FRAC_BITS-2:0] + 1'b1) :
                                                               b_frac[FRAC_BITS-2:0]);

    // =========================================================================
    // Shared trial subtraction
    //
    // Start cycle: compare abs_a vs abs_b (no shift, first quotient bit).
    // Compute cycles: shift remainder left, compare vs registered divisor.
    // One physical subtractor with muxed inputs.
    // =========================================================================
    wire starting = (state == S_IDLE) & start;

    wire [FRAC_BITS-1:0] r_in = starting ? {1'b0, abs_a_comb} :
                                            {r_reg[FRAC_BITS-2:0], 1'b0};
    wire [FRAC_BITS-2:0] d_mux = starting ? abs_b_comb : d_reg;

    wire [FRAC_BITS:0] trial = {1'b0, r_in} - {2'b00, d_mux};
    wire ge = !trial[FRAC_BITS]; // no borrow: r_in >= divisor
    wire [FRAC_BITS-1:0] r_next = ge ? trial[FRAC_BITS-1:0] : r_in;

    // =========================================================================
    // Finalization logic (from live/combinational values on last cycle)
    //
    // On the last COMPUTE cycle, q_live and r_live reflect the state
    // AFTER this cycle's trial subtract (the values that WOULD be
    // registered if we had another cycle). The finalization reads these
    // directly, saving a dead cycle.
    //
    // Key timing observation: q_live's MSB (norm_shift) and most fraction
    // bits come from q_reg (registered), NOT from ge. Only the LSB of
    // q_live depends on the current cycle's ge. So the finalization
    // critical path is shorter than it first appears — the norm_shift
    // decision and most of frac_pos_raw are registered.
    // =========================================================================
    wire [FRAC_BITS:0] q_live = {q_reg[FRAC_BITS-1:0], ge};
    wire [FRAC_BITS-1:0] r_live = r_next;

    // Bounded normalization
    wire norm_shift = q_live[FRAC_BITS];
    wire [FRAC_BITS:0] q_norm = norm_shift ? (q_live >> 1) : q_live;
    wire [FRAC_BITS-1:0] frac_pos_raw = q_norm[FRAC_BITS:1];

    // Banker's round
    wire f_guard      = q_norm[0];
    wire f_norm_sticky = norm_shift & q_live[0];
    wire f_rem_sticky  = |r_live;
    wire f_sticky = f_norm_sticky | f_rem_sticky;
    wire f_lsb    = frac_pos_raw[0];
    wire f_round_up = f_guard & (f_sticky | f_lsb);

    wire [FRAC_BITS-1:0] frac_rounded = frac_pos_raw + {{(FRAC_BITS-1){1'b0}}, f_round_up};
    wire round_ovf = (&frac_pos_raw[FRAC_BITS-2:0]) & f_round_up;
    wire [FRAC_BITS-1:0] pos_frac = round_ovf ? POS_HALF : frac_rounded;

    // Apply sign
    wire neg_is_pos_half = (pos_frac == POS_HALF);
    wire signed [FRAC_BITS-1:0] neg_frac = neg_is_pos_half ? NEG_ONE :
                                            (~pos_frac + 1'b1);
    wire signed [FRAC_BITS-1:0] final_frac = sign_reg ? neg_frac : $signed(pos_frac);

    // Exponent (with sign adjustment)
    wire signed [EXP_BITS:0] exp_base = a_exp_adj_r - b_exp_adj_r
                                       + {{EXP_BITS{1'b0}}, norm_shift}
                                       + {{EXP_BITS{1'b0}}, round_ovf};
    wire signed [EXP_BITS:0] exp_final = (sign_reg & neg_is_pos_half) ?
                                          (exp_base - 1) : exp_base;
    wire exp_too_big   = (exp_final > MAX_EXP);
    wire exp_too_small = (exp_final < MIN_EXP);
    wire signed [EXP_BITS-1:0] final_exp = exp_final[EXP_BITS-1:0];

    // Output mux
    wire signed [FRAC_BITS-1:0] out_frac = b_zero_reg    ? {FRAC_BITS{1'b0}} :
                                            exp_too_big   ? final_frac :
                                            exp_too_small ? {final_frac[FRAC_BITS-1],
                                                             final_frac[FRAC_BITS-1:1]} :
                                                            final_frac;

    wire signed [EXP_BITS-1:0] out_exp = (b_zero_reg | exp_too_big | exp_too_small) ?
                                          AMBIGUOUS_EXP[EXP_BITS-1:0] : final_exp;

    // =========================================================================
    // State machine
    //
    // IDLE + start: compute abs values, first trial subtract, register
    //     all operands. Transition to COMPUTE with counter = FRAC-1.
    // COMPUTE: shift-and-subtract each cycle. On counter == 0, compute
    //     finalization from live q/r and register output directly.
    // =========================================================================
    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    // Sign and exponent adjustment
                    sign_reg <= a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
                    b_zero_reg <= (b_frac == 0);

                    a_exp_adj_r <= $signed({a_exp[EXP_BITS-1], a_exp})
                                 + {{EXP_BITS{1'b0}}, a_is_neg_one};
                    b_exp_adj_r <= $signed({b_exp[EXP_BITS-1], b_exp})
                                 + {{EXP_BITS{1'b0}}, b_is_neg_one};

                    // First trial subtract result (folded into start)
                    d_reg <= abs_b_comb;
                    r_reg <= r_next;
                    q_reg <= {{FRAC_BITS{1'b0}}, ge};

                    counter <= FRAC_BITS[CTR_BITS-1:0] - 1;
                    state <= S_COMPUTE;
                end
            end

            S_COMPUTE: begin
                // Shift-and-subtract iteration
                r_reg <= r_next;
                q_reg <= {q_reg[FRAC_BITS-1:0], ge};

                if (counter == 0) begin
                    // Last iteration: finalize and output directly
                    result_frac <= out_frac;
                    result_exp  <= out_exp;
                    done <= 1'b1;
                    state <= S_IDLE;
                end else begin
                    counter <= counter - 1;
                end
            end
        endcase
    end

endmodule
