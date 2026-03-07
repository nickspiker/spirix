// spirix_sqrt — Minimal iterative square root for Spirix scalars
//
// Floor-only (no rounding), restoring binary square root.
// Parameterized: FRAC_BITS in {2..256}, EXP_BITS in {2..256}.
//
// Latency: FRAC_BITS + 2 cycles from start to done.
//   Cycle 0 (start):       Setup radicand, exponent, registers.
//   Cycles 1..FRAC:        Restoring sqrt (shift 2 bits, trial subtract).
//   Cycle FRAC+1:          Finalize (extract fraction, apply exponent).
//
// Interface:
//   start:  pulse high for 1 cycle to begin. Inputs sampled on this edge.
//   busy:   high while computing. Do not assert start while busy.
//   done:   pulses high for 1 cycle when result_frac/result_exp are valid.
//
// Algorithm:
//   1. Invalid check: negative, zero, or AMBIG exponent → (0, AMBIG_EXP).
//   2. Radicand setup: 2*FRAC bits from |frac|, shifted for even/odd exp.
//   3. FRAC iterations of restoring binary sqrt: 1 root bit per cycle.
//   4. Extract: fraction = {0, root[FRAC-1:1]} (floor, no rounding).
//   5. Exponent = (a_exp >>> 1) + (a_exp & 1).
//
// Matches Rust scalar sqrt exactly (floor extraction, restoring algorithm).
// 0 DSP. No multiplier.

module spirix_sqrt #(
    parameter FRAC_BITS = 32,
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

    localparam AMBIG_EXP = -(1 <<< (EXP_BITS - 1));
    localparam MAG       = FRAC_BITS - 1;      // unsigned magnitude bits
    localparam RAD_BITS  = 2 * FRAC_BITS;      // radicand width
    localparam R_BITS    = FRAC_BITS + 2;       // remainder width
    localparam CTR_BITS  = $clog2(FRAC_BITS + 1);

    localparam S_IDLE     = 0;
    localparam S_COMPUTE  = 1;
    localparam S_FINALIZE = 2;
    reg [1:0] state = S_IDLE;

    assign busy = (state != S_IDLE);

    // Registers
    reg [RAD_BITS-1:0]      rad_reg;        // radicand shift register
    reg [R_BITS-1:0]        rem_reg;        // partial remainder
    reg [FRAC_BITS-1:0]     root_reg;       // root accumulator
    reg [CTR_BITS-1:0]      counter;
    reg                     invalid_reg;
    reg signed [EXP_BITS:0] exp_out_reg;

    // == Restoring sqrt iteration ================================================
    //
    // Each cycle: shift in 2 radicand bits into remainder, trial subtract
    // against {root, 2'b01}. If remainder >= trial: subtract and set root bit.
    // ============================================================================
    wire [1:0]        rad_top2    = rad_reg[RAD_BITS-1:RAD_BITS-2];
    wire [R_BITS-1:0] rem_shifted = {rem_reg[R_BITS-3:0], rad_top2};
    wire [R_BITS-1:0] trial_val   = {root_reg, 2'b01};
    wire [R_BITS:0]   trial_sub   = {1'b0, rem_shifted} - {1'b0, trial_val};
    wire              ge          = ~trial_sub[R_BITS];

    // == Finalization (from registered root_reg in S_FINALIZE) ===================
    //
    // root_reg[FRAC-1] is always 1 for valid N1-positive inputs.
    // fraction = {0, root[FRAC-1:1]} = positive N1-normalized result.
    // root[0] is the truncated bit (floor behavior).
    // ============================================================================
    wire [FRAC_BITS-1:0] final_frac = invalid_reg ? {FRAC_BITS{1'b0}} :
                                       {1'b0, root_reg[FRAC_BITS-1:1]};
    wire signed [EXP_BITS-1:0] final_exp = invalid_reg ? AMBIG_EXP[EXP_BITS-1:0] :
                                             exp_out_reg[EXP_BITS-1:0];

    // == State machine ===========================================================

    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    // Invalid: negative, zero, or ambiguous exponent
                    invalid_reg <= a_frac[FRAC_BITS-1] | (a_frac == 0) |
                                   (a_exp == AMBIG_EXP[EXP_BITS-1:0]);

                    // Exponent: (a_exp >>> 1) + (a_exp & 1)
                    // Matches Rust: exp/2 + even, with -1 for negative odd.
                    exp_out_reg <= ($signed({a_exp[EXP_BITS-1], a_exp}) >>> 1)
                                 + {{EXP_BITS{1'b0}}, a_exp[0]};

                    // Radicand: 2*FRAC bits, magnitude positioned for even/odd
                    if (a_exp[0])  // odd exponent: shift right by 1
                        rad_reg <= {1'b0, a_frac[MAG-1:0], {(RAD_BITS-MAG-1){1'b0}}};
                    else           // even exponent
                        rad_reg <= {a_frac[MAG-1:0], {(RAD_BITS-MAG){1'b0}}};

                    rem_reg  <= {R_BITS{1'b0}};
                    root_reg <= {FRAC_BITS{1'b0}};
                    counter  <= FRAC_BITS[CTR_BITS-1:0] - 1;
                    state    <= S_COMPUTE;
                end
            end

            S_COMPUTE: begin
                rem_reg  <= ge ? trial_sub[R_BITS-1:0] : rem_shifted;
                root_reg <= {root_reg[FRAC_BITS-2:0], ge};
                rad_reg  <= rad_reg << 2;

                if (counter == 0)
                    state <= S_FINALIZE;
                else
                    counter <= counter - 1;
            end

            S_FINALIZE: begin
                result_frac <= final_frac;
                result_exp  <= final_exp;
                done <= 1'b1;
                state <= S_IDLE;
            end
        endcase
    end

endmodule
