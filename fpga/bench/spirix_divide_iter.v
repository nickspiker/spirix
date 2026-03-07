// spirix_divide_iter — Iterative sequential divider for Spirix scalars
//
// Computes a / b on N1-normalized signed fractions with signed exponents.
// Fully parameterized. Restoring division: 1 quotient bit per clock.
//
// Latency: FRAC_BITS + 2 cycles from start to done.
//   Cycle 0 (start):       First trial subtract folded into start cycle.
//   Cycles 1..FRAC:        Restoring division (shift + subtract).
//   Cycle FRAC+1:          Finalize (normalize/round/sign from registers).
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
// Timing: every COMPUTE cycle is uniform (trial subtract only, ~FRAC/2
// CCU2C deep). Finalization (normalize + round + sign) runs in its own
// cycle from registered quotient/remainder, so it doesn't stack on the
// subtractor.
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

    localparam S_IDLE     = 0;
    localparam S_COMPUTE  = 1;
    localparam S_FINALIZE = 2;
    localparam S_SHORTCUT = 3;
    reg [1:0] state = S_IDLE;

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

    // Edge case constants
    localparam signed [EXP_BITS-1:0]  AMBIG_E  = AMBIGUOUS_EXP[EXP_BITS-1:0];
    localparam signed [FRAC_BITS-1:0] ALL_ONES = {FRAC_BITS{1'b1}};
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_NEG_DIV_NEG = $signed({8'hE9, {UPAD{1'b0}}});
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_DIV_TF   = $signed({8'h16, {UPAD{1'b0}}});
    localparam signed [FRAC_BITS-1:0] UNDEF_GENERAL      = $signed({8'hFE, {UPAD{1'b0}}});

    // Edge case classification (combinational, from input ports)
    wire ec_a_is_ambig = (a_exp == AMBIG_E);
    wire ec_b_is_ambig = (b_exp == AMBIG_E);
    wire ec_a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire ec_b_n1 = (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-2]);
    wire [7:0] ec_a_pref = a_frac[FRAC_BITS-1 -: 8];
    wire [7:0] ec_b_pref = b_frac[FRAC_BITS-1 -: 8];
    wire ec_a_n0 = (ec_a_pref == 8'h00) || (ec_a_pref == 8'hFF);
    wire ec_b_n0 = (ec_b_pref == 8'h00) || (ec_b_pref == 8'hFF);
    wire ec_a_top3 = (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &&
                     (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire ec_b_top3 = (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &&
                     (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);
    wire ec_a_undef = !ec_a_n0 && ec_a_top3;
    wire ec_b_undef = !ec_b_n0 && ec_b_top3;
    wire ec_a_n2 = !ec_a_n1 && (a_frac[FRAC_BITS-2] != a_frac[FRAC_BITS-3]);
    wire ec_b_n2 = !ec_b_n1 && (b_frac[FRAC_BITS-2] != b_frac[FRAC_BITS-3]);
    wire ec_a_is_zero_st = ec_a_n0 && !a_frac[FRAC_BITS-1];
    wire ec_b_is_zero_st = ec_b_n0 && !b_frac[FRAC_BITS-1];
    wire ec_a_is_inf = ec_a_n0 && a_frac[FRAC_BITS-1];
    wire ec_b_is_inf = ec_b_n0 && b_frac[FRAC_BITS-1];
    wire ec_a_vanished = ec_a_n2;
    wire ec_b_vanished = ec_b_n2;
    wire ec_a_exploded = ec_a_is_ambig && ec_a_n1;
    wire ec_b_exploded = ec_b_is_ambig && ec_b_n1;
    wire ec_a_is_normal = ec_a_n1 && !ec_a_is_ambig;
    wire ec_b_is_normal = ec_b_n1 && !ec_b_is_ambig;
    wire ec_any_non_normal = !ec_a_is_normal || !ec_b_is_normal;

    reg ec_shortcut;
    reg signed [FRAC_BITS-1:0] ec_sc_frac;
    reg signed [EXP_BITS-1:0]  ec_sc_exp;

    always @(*) begin
        ec_shortcut = 1'b0;
        ec_sc_frac  = {FRAC_BITS{1'b0}};
        ec_sc_exp   = AMBIG_E;
        if (ec_any_non_normal) begin
            ec_shortcut = 1'b1;
            if (ec_a_undef) begin
                ec_sc_frac = a_frac; ec_sc_exp = a_exp;
            end else if (ec_b_undef) begin
                ec_sc_frac = b_frac; ec_sc_exp = b_exp;
            end else if (ec_b_is_zero_st) begin
                ec_sc_frac = ec_a_is_zero_st ? UNDEF_NEG_DIV_NEG : ALL_ONES;
            end else if (ec_a_is_inf) begin
                ec_sc_frac = ec_b_is_inf ? UNDEF_TF_DIV_TF : ALL_ONES;
            end else if (ec_a_is_zero_st || ec_b_is_inf) begin
                ec_sc_frac = {FRAC_BITS{1'b0}};
            end else if (ec_a_exploded && ec_b_exploded) begin
                ec_sc_frac = UNDEF_TF_DIV_TF;
            end else if (ec_a_vanished && ec_b_vanished) begin
                ec_sc_frac = UNDEF_NEG_DIV_NEG;
            end else if (ec_a_vanished || ec_b_vanished) begin
                ec_sc_frac = UNDEF_GENERAL;
            end else begin
                ec_sc_frac = UNDEF_GENERAL;
            end
        end else if (b_frac == 0) begin
            ec_shortcut = 1'b1;
            ec_sc_frac  = ALL_ONES;
        end
    end

    // Shortcut registers (for S_SHORTCUT state)
    reg signed [FRAC_BITS-1:0] sc_frac_r;
    reg signed [EXP_BITS-1:0]  sc_exp_r;

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
    // Finalization logic (reads from registered q_reg/r_reg in S_FINALIZE)
    //
    // After the last COMPUTE cycle registers q and r, S_FINALIZE reads
    // them purely from flops — no dependency on the trial subtractor.
    // =========================================================================

    // Bounded normalization
    wire norm_shift = q_reg[FRAC_BITS];
    wire [FRAC_BITS:0] q_norm = norm_shift ? (q_reg >> 1) : q_reg;
    wire [FRAC_BITS-1:0] frac_pos_raw = q_norm[FRAC_BITS:1];

    // Banker's round
    wire f_guard      = q_norm[0];
    wire f_norm_sticky = norm_shift & q_reg[0];
    wire f_rem_sticky  = |r_reg;
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
    // COMPUTE: shift-and-subtract each cycle. On counter == 0, register
    //     final q/r and transition to FINALIZE.
    // FINALIZE: normalize/round/sign from registered q_reg/r_reg, output.
    // =========================================================================
    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    if (ec_shortcut) begin
                        // Edge case — skip computation
                        sc_frac_r <= ec_sc_frac;
                        sc_exp_r  <= ec_sc_exp;
                        state <= S_SHORTCUT;
                    end else begin
                        // Sign and exponent adjustment
                        sign_reg <= a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
                        b_zero_reg <= 1'b0;

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
            end

            S_COMPUTE: begin
                // Shift-and-subtract iteration
                r_reg <= r_next;
                q_reg <= {q_reg[FRAC_BITS-1:0], ge};

                if (counter == 0) begin
                    // Last iteration done — register q/r, finalize next cycle
                    state <= S_FINALIZE;
                end else begin
                    counter <= counter - 1;
                end
            end

            S_FINALIZE: begin
                // All finalization reads from registered q_reg/r_reg
                result_frac <= out_frac;
                result_exp  <= out_exp;
                done <= 1'b1;
                state <= S_IDLE;
            end

            S_SHORTCUT: begin
                result_frac <= sc_frac_r;
                result_exp  <= sc_exp_r;
                done <= 1'b1;
                state <= S_IDLE;
            end
        endcase
    end

endmodule
