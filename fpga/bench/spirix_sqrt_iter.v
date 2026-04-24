// spirix_sqrt_iter — Iterative sequential square root for Spirix scalars
//
// Computes sqrt(a) on N1-normalized signed fractions with signed exponents.
// Fully parameterized. Restoring binary square root: 1 result bit per clock.
//
// Latency: FRAC_BITS + 2 cycles from start to done.
//   Cycle 0 (start):              Setup radicand, registers.
//   Cycles 1..FRAC_BITS+1:        Restoring sqrt (shift-in 2 bits, trial subtract).
//   Last cycle:                   Final trial subtract + normalize/round/output.
//
// Interface:
//   start:  pulse high for 1 cycle to begin. Inputs sampled on this edge.
//   busy:   high while computing. Do not assert start while busy.
//   done:   pulses high for 1 cycle when result_frac/result_exp are valid.
//
// Area: one (FRAC+3)-bit subtractor (trial subtract), one (FRAC+3)-bit
// remainder register, one (FRAC+1)-bit root shift register, one 52-bit
// radicand shift register, one counter. No DSP blocks. No multipliers.
//
// Algorithm: restoring binary square root on the scaled radicand.
//   Even exponent: radicand = |frac| << (2N - MAG)
//   Odd exponent:  radicand = |frac| << (2N - MAG - 1)
// where N = FRAC_BITS + 1 iterations produce an (N)-bit integer square root Q.
// Result magnitude = Q >> 2, with Q[1:0] and remainder providing rounding info.
// Banker's round applied. Output is always non-negative (sqrt of negative = 0).
//
// Edge cases follow the Spirix state model:
//   AMBIG exponent → undef passthrough, N0 passthrough, vanished, exploded
//   Negative normal → UNDEF_SQRT_NEG
//   Positive normal → compute
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_sqrt_iter #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire start,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp,
    output wire busy,
    output reg  done = 0
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    localparam MAG     = FRAC_BITS - 1;     // 24 unsigned magnitude bits
    localparam N_ITER  = FRAC_BITS + 1;     // 26 iterations
    localparam RAD_BITS = 2 * N_ITER;       // 52-bit radicand
    localparam R_BITS  = N_ITER + 2;        // 28-bit remainder
    localparam CTR_BITS = $clog2(N_ITER);

    localparam S_IDLE     = 0;
    localparam S_COMPUTE  = 1;
    localparam S_SHORTCUT = 2;
    reg [1:0] state = S_IDLE;

    // Edge case constants
    localparam signed [EXP_BITS-1:0]  AMBIG_E  = AMBIGUOUS_EXP[EXP_BITS-1:0];
    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    // Undefined prefix constants — match src/core/undefined.rs L6 Roots.
    localparam signed [FRAC_BITS-1:0] UNDEF_SQRT_NEG    = $signed({8'h02, {UPAD{1'b0}}}); //  0x02 SQRT_NEGATIVE
    localparam signed [FRAC_BITS-1:0] UNDEF_SQRT_EXPLOD = $signed({8'hFC, {UPAD{1'b0}}}); // -0x04 SQRT_EXPLODED
    localparam signed [FRAC_BITS-1:0] UNDEF_SQRT_VANISH = $signed({8'h03, {UPAD{1'b0}}}); //  0x03 SQRT_VANISHED

    // Input state classification (combinational)
    wire a_is_ambig = (a_exp == AMBIG_E);
    wire a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire a_n2 = !a_n1 && (a_frac[FRAC_BITS-2] != a_frac[FRAC_BITS-3]);
    wire a_vanished = a_n2;
    wire [7:0] a_pref = a_frac[FRAC_BITS-1 -: 8];
    wire a_n0 = (a_pref == 8'h00) || (a_pref == 8'hFF);
    wire a_top3 = (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &&
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire a_undef = !a_n0 && a_top3;
    wire a_is_normal = a_n1 && !a_is_ambig;

    assign busy = (state != S_IDLE);

    // Registers
    reg [RAD_BITS-1:0]      rad_reg;        // radicand shift register
    reg [R_BITS-1:0]        rem_reg;        // partial remainder
    reg [N_ITER-1:0]        root_reg;       // root accumulator
    reg [CTR_BITS-1:0]      counter;
    reg                     exp_odd_reg;
    reg signed [EXP_BITS:0] exp_half_reg;
    reg signed [FRAC_BITS-1:0] sc_frac_r;
    reg signed [EXP_BITS-1:0]  sc_exp_r;

    // =========================================================================
    // Trial subtraction (restoring sqrt iteration)
    //
    // Each iteration: shift in 2 radicand bits, trial = (root << 2) | 1.
    // If remainder >= trial: subtract, set root bit. Else: keep remainder.
    // =========================================================================
    wire [1:0]        rad_top2    = rad_reg[RAD_BITS-1:RAD_BITS-2];
    wire [R_BITS-1:0] rem_shifted = {rem_reg[R_BITS-3:0], rad_top2};
    wire [R_BITS-1:0] trial       = {root_reg, 2'b01};
    wire [R_BITS:0]   trial_sub   = {1'b0, rem_shifted} - {1'b0, trial};
    wire              ge          = ~trial_sub[R_BITS];

    // =========================================================================
    // Finalization (from live/combinational values on last cycle)
    //
    // root_live is the 26-bit integer square root Q. The result magnitude
    // is Q[25:2] (24 bits), with Q[1:0] and the remainder providing
    // guard/round/sticky bits for banker's rounding.
    //
    // Scaling: Q = isqrt(radicand), where radicand = |frac| << K.
    //   Even exp: K=2N-MAG, Q = sqrt(|frac|) * 2^((2N-MAG)/2) = sqrt(|frac|)*2^14
    //             Q >> 2 = sqrt(|frac|) * 2^12, fractional = sqrt(|frac|/2^24). ✓
    //   Odd exp:  K=2N-MAG-1, Q = sqrt(2*|frac|) * 2^13
    //             Q >> 2 = sqrt(2*|frac|) * 2^11, with exp adjusted +1. ✓
    // =========================================================================
    wire [N_ITER-1:0] root_live = {root_reg[N_ITER-2:0], ge};
    wire [R_BITS-1:0] rem_live  = ge ? trial_sub[R_BITS-1:0] : rem_shifted;

    wire [MAG-1:0] frac_raw = root_live[N_ITER-1:2];
    wire f_guard    = root_live[1];
    wire f_round    = root_live[0];
    wire f_sticky   = |rem_live;
    wire f_lsb      = frac_raw[0];
    wire f_round_up = f_guard & (f_round | f_sticky | f_lsb);

    wire [MAG:0]   frac_rounded = {1'b0, frac_raw} + {{MAG{1'b0}}, f_round_up};
    wire           round_ovf    = frac_rounded[MAG];
    wire [MAG-1:0] pos_frac     = round_ovf ? POS_HALF[MAG-1:0] : frac_rounded[MAG-1:0];

    // Exponent: (a_exp >>> 1) + exp_odd + round_ovf
    wire signed [EXP_BITS:0] exp_base = exp_half_reg
                                       + {{EXP_BITS{1'b0}}, exp_odd_reg}
                                       + {{EXP_BITS{1'b0}}, round_ovf};
    wire exp_too_big   = (exp_base > MAX_EXP);
    wire exp_too_small = (exp_base < MIN_EXP);

    wire clamp = exp_too_big | exp_too_small;
    wire signed [FRAC_BITS-1:0] out_frac = clamp ? {FRAC_BITS{1'b0}} :
                                            $signed({1'b0, pos_frac});
    wire signed [EXP_BITS-1:0]  out_exp  = clamp ? AMBIGUOUS_EXP[EXP_BITS-1:0] :
                                            exp_base[EXP_BITS-1:0];

    // =========================================================================
    // State machine
    //
    // IDLE + start: set up radicand shift register, clear remainder/root,
    //     record exponent info. Transition to COMPUTE with counter = N-1.
    // COMPUTE: one restoring-sqrt iteration per cycle. On counter == 0,
    //     finalize from live root/remainder and register output directly.
    // =========================================================================
    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    if (!a_is_normal) begin
                        // Edge case chain: undef → N0 → vanished → exploded
                        state <= S_SHORTCUT;
                        sc_exp_r <= AMBIG_E;
                        if (a_undef) begin
                            sc_frac_r <= a_frac; sc_exp_r <= a_exp;
                        end else if (a_n0) begin
                            sc_frac_r <= a_frac; sc_exp_r <= a_exp;
                        end else if (a_vanished) begin
                            sc_frac_r <= UNDEF_SQRT_VANISH;
                        end else begin
                            sc_frac_r <= UNDEF_SQRT_EXPLOD;
                        end
                    end else if (a_frac[FRAC_BITS-1]) begin
                        // Negative normal → sqrt of negative
                        state <= S_SHORTCUT;
                        sc_frac_r <= UNDEF_SQRT_NEG;
                        sc_exp_r  <= AMBIG_E;
                    end else begin
                        // Positive normal — compute sqrt
                        exp_odd_reg  <= a_exp[0];
                        exp_half_reg <= $signed({a_exp[EXP_BITS-1], a_exp}) >>> 1;

                        if (a_exp[0])
                            rad_reg <= {1'b0, a_frac[MAG-1:0], {(RAD_BITS-MAG-1){1'b0}}};
                        else
                            rad_reg <= {a_frac[MAG-1:0], {(RAD_BITS-MAG){1'b0}}};

                        rem_reg  <= {R_BITS{1'b0}};
                        root_reg <= {N_ITER{1'b0}};
                        counter  <= N_ITER[CTR_BITS-1:0] - 1;
                        state    <= S_COMPUTE;
                    end
                end
            end

            S_COMPUTE: begin
                // Restoring sqrt iteration
                rem_reg  <= ge ? trial_sub[R_BITS-1:0] : rem_shifted;
                root_reg <= {root_reg[N_ITER-2:0], ge};
                rad_reg  <= rad_reg << 2;

                if (counter == 0) begin
                    // Last iteration: finalize and output directly
                    result_frac <= out_frac;
                    result_exp  <= out_exp;
                    done  <= 1'b1;
                    state <= S_IDLE;
                end else begin
                    counter <= counter - 1;
                end
            end

            S_SHORTCUT: begin
                result_frac <= sc_frac_r;
                result_exp  <= sc_exp_r;
                done  <= 1'b1;
                state <= S_IDLE;
            end
        endcase
    end

endmodule
