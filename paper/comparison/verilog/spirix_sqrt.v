// spirix_sqrt — Banker's-rounded N0 iterative sqrt for paper comparison.
//
// Specialty module for the paper/comparison harness. Ports production
// spirix_sqrt_iter to N0 storage at FRAC=24 (binary32-equivalent).
//
// Architecture: restoring binary square root. 1 result bit per clock.
// Latency: COMPUTE_FRAC + 2 cycles (= 27 at FRAC=24) from start to done.
//
// Algorithm (unchanged from production):
//   Even exp:  radicand = |compute_q| << (2N - MAG)
//   Odd exp:   radicand = |compute_q| << (2N - MAG - 1)
//   N = COMPUTE_FRAC + 1 = 26 iterations produce N-bit integer sqrt Q.
//   Result magnitude = Q >> 2; Q[1:0] + remainder give G/R/S for banker's.
//
// Output is always non-negative (sqrt of negative input → UNDEF_SQRT_NEG).

module spirix_sqrt #(
    parameter FRAC_BITS = 24,
    parameter EXP_BITS  = 8,
    parameter PARALLEL  = 1   // root bits emitted per clock (1, 2, 4, ...)
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
    localparam COMPUTE_FRAC = FRAC_BITS + 1;            // = 25
    localparam MAG          = COMPUTE_FRAC - 1;         // = 24 unsigned magnitude bits
    localparam N_ITER       = COMPUTE_FRAC + 1;         // = 26 iterations needed
    localparam RAD_BITS     = 2 * N_ITER;               // = 52
    // Parallel-N: each cycle emits PARALLEL root bits and consumes 2*PARALLEL radicand bits.
    // ITER_TOTAL × PARALLEL >= N_ITER; any excess root bits fold into sticky.
    localparam ITER_TOTAL   = (N_ITER + PARALLEL - 1) / PARALLEL;
    localparam Q_BITS       = ITER_TOTAL * PARALLEL;    // root accumulator width
    localparam EXCESS_Q     = Q_BITS - N_ITER;          // extra root bits (folded into sticky)
    localparam R_BITS       = Q_BITS + 2;               // remainder width (sized for trial = root×4+1)
    localparam CTR_BITS     = $clog2(ITER_TOTAL + 1);

    localparam signed [EXP_BITS-1:0] AMBIG_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [EXP_BITS:0]   MAX_EXP_W = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0]   MIN_EXP_W = -(1 <<< (EXP_BITS - 1)) + 1;

    // N0 boundary patterns
    localparam signed [FRAC_BITS-1:0] POS_ONE_NORMAL   = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_NORMAL   = {FRAC_BITS{1'b0}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_EXPLODED = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_EXPLODED = {1'b1, 1'b0, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_VANISHED = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_VANISHED = {2'b11, {(FRAC_BITS-2){1'b0}}};

    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_SQRT_NEG    = $signed({8'h02, {UPAD{1'b0}}});
    localparam signed [FRAC_BITS-1:0] UNDEF_SQRT_EXPLOD = $signed({8'hFC, {UPAD{1'b0}}});
    localparam signed [FRAC_BITS-1:0] UNDEF_SQRT_VANISH = $signed({8'h03, {UPAD{1'b0}}});

    // ───── State detection (N0 conventions) ─────────────────────────────
    wire a_is_ambig  = (a_exp == AMBIG_EXP);
    wire a_frac_zero = (a_frac == {FRAC_BITS{1'b0}});
    wire a_frac_neg1 = &a_frac;
    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire a_n2 = ~a_n1 & ~a_n0 & (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-3]);
    wire a_top3 = ~a_n0 & (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);

    wire a_is_zero  = a_is_ambig & a_frac_zero;
    wire a_is_inf   = a_is_ambig & a_frac_neg1;
    wire a_exploded = a_is_ambig & a_n1;
    wire a_vanished = a_is_ambig & a_n2;
    wire a_undef    = a_is_ambig & a_top3;
    wire a_is_normal = ~a_is_ambig;

    // In N0 storage, MSB=1 means positive value, MSB=0 means negative.
    wire a_is_negative_normal = a_is_normal & ~a_frac[FRAC_BITS-1];

    // ───── State machine ─────────────────────────────────────────────────
    localparam S_IDLE     = 0;
    localparam S_COMPUTE  = 1;
    localparam S_SHORTCUT = 2;
    reg [1:0] state = S_IDLE;
    assign busy = (state != S_IDLE);

    // Registers
    reg [RAD_BITS-1:0]      rad_reg     = {RAD_BITS{1'b0}};
    reg [R_BITS-1:0]        rem_reg     = {R_BITS{1'b0}};
    reg [Q_BITS-1:0]        root_reg    = {Q_BITS{1'b0}};
    reg [CTR_BITS-1:0]      counter     = {CTR_BITS{1'b0}};
    reg                     exp_odd_reg = 1'b0;
    reg signed [EXP_BITS:0] exp_half_reg = {(EXP_BITS+1){1'b0}};
    reg signed [FRAC_BITS-1:0] sc_frac_r = {FRAC_BITS{1'b0}};
    reg signed [EXP_BITS-1:0]  sc_exp_r  = {EXP_BITS{1'b0}};

    // ───── Trial-subtraction chain (PARALLEL iterations per clock) ─────
    // Each chain step consumes the top 2 bits of the current radicand,
    // does one trial subtract using the running root, and emits one root bit.
    // Step 0 sees rem_reg/root_reg/rad_reg (the registered state); each
    // subsequent step uses the previous step's combinational outputs.
    wire [R_BITS-1:0]   chain_rem  [0:PARALLEL];
    wire [Q_BITS-1:0]   chain_root [0:PARALLEL];
    wire [RAD_BITS-1:0] chain_rad  [0:PARALLEL];
    wire [PARALLEL-1:0] chain_ge;

    assign chain_rem[0]  = rem_reg;
    assign chain_root[0] = root_reg;
    assign chain_rad[0]  = rad_reg;

    genvar gi;
    generate
        for (gi = 0; gi < PARALLEL; gi = gi + 1) begin : sqrt_chain
            wire [1:0]        rad_top2_g    = chain_rad[gi][RAD_BITS-1:RAD_BITS-2];
            wire [R_BITS-1:0] rem_shifted_g = {chain_rem[gi][R_BITS-3:0], rad_top2_g};
            wire [R_BITS-1:0] trial_g       = {chain_root[gi], 2'b01};
            wire [R_BITS:0]   trial_sub_g   = {1'b0, rem_shifted_g} - {1'b0, trial_g};
            wire              ge_g          = ~trial_sub_g[R_BITS];

            assign chain_ge[gi]    = ge_g;
            assign chain_rem[gi+1] = ge_g ? trial_sub_g[R_BITS-1:0] : rem_shifted_g;
            assign chain_root[gi+1] = {chain_root[gi][Q_BITS-2:0], ge_g};
            assign chain_rad[gi+1]  = chain_rad[gi] << 2;
        end
    endgenerate

    // Final-cycle live values for finalize use the chain output.
    wire [Q_BITS-1:0] root_live = chain_root[PARALLEL];
    wire [R_BITS-1:0] rem_live  = chain_rem[PARALLEL];

    // Top MAG bits = magnitude; next bit = guard; next = round; below = sticky.
    // For PARALLEL=1 (Q_BITS=N_ITER=26), this matches the original [25:2]/[1]/[0].
    wire [MAG-1:0] frac_raw = root_live[Q_BITS-1 : Q_BITS-MAG];
    wire f_guard    = root_live[Q_BITS-MAG-1];
    wire f_round    = root_live[Q_BITS-MAG-2];
    wire excess_root_sticky;
    generate
        if (EXCESS_Q > 0) begin : excess_sticky_gen
            assign excess_root_sticky = |root_live[EXCESS_Q-1:0];
        end else begin : excess_sticky_zero
            assign excess_root_sticky = 1'b0;
        end
    endgenerate
    wire f_sticky   = |rem_live | excess_root_sticky;
    wire f_lsb      = frac_raw[0];
    wire f_round_up = f_guard & (f_round | f_sticky | f_lsb);

    wire [MAG:0]   frac_rounded = {1'b0, frac_raw} + {{MAG{1'b0}}, f_round_up};
    wire           round_ovf    = frac_rounded[MAG];

    // For N0: result is positive normal. Storage is FRAC=24 bits = compute_q[23:0].
    // Canonical positive in N0: storage MSB=1, magnitude top differs.
    // pos_frac (24 bits) is the magnitude bits = compute_q[23:0]. For canonical
    // normalized positive sqrt result, magnitude top (= bit 23 of pos_frac) = 1.
    // So pos_frac IS the N0 storage directly.
    wire [MAG-1:0] pos_frac =
        round_ovf ? {1'b1, {(MAG-1){1'b0}}}     // = 0x800000 = POS_ONE_NORMAL after rovf
                  : frac_rounded[MAG-1:0];

    // Exponent: (a_exp >>> 1) + exp_odd + round_ovf
    wire signed [EXP_BITS:0] exp_base = exp_half_reg
                                       + {{EXP_BITS{1'b0}}, exp_odd_reg}
                                       + {{EXP_BITS{1'b0}}, round_ovf};
    wire exp_too_big   = (exp_base > MAX_EXP_W);
    wire exp_too_small = (exp_base < MIN_EXP_W);
    wire clamp = exp_too_big | exp_too_small;

    // Saturation: positive result, so exploded = POS_ONE_EXPLODED, vanished = POS_ONE_VANISHED.
    wire signed [FRAC_BITS-1:0] out_frac =
        exp_too_big   ? POS_ONE_EXPLODED :
        exp_too_small ? POS_ONE_VANISHED :
                        $signed(pos_frac);
    wire signed [EXP_BITS-1:0]  out_exp = clamp ? AMBIG_EXP : exp_base[EXP_BITS-1:0];

    // ───── State machine ─────────────────────────────────────────────────
    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    if (a_undef) begin
                        // Undefined passthrough
                        state <= S_SHORTCUT;
                        sc_frac_r <= a_frac; sc_exp_r <= a_exp;
                    end else if (a_is_zero) begin
                        // sqrt(0) = 0
                        state <= S_SHORTCUT;
                        sc_frac_r <= {FRAC_BITS{1'b0}}; sc_exp_r <= AMBIG_EXP;
                    end else if (a_is_inf) begin
                        // sqrt(inf) = inf
                        state <= S_SHORTCUT;
                        sc_frac_r <= {FRAC_BITS{1'b1}}; sc_exp_r <= AMBIG_EXP;
                    end else if (a_exploded) begin
                        // sqrt(exploded) → UNDEF_SQRT_EXPLOD
                        state <= S_SHORTCUT;
                        sc_frac_r <= UNDEF_SQRT_EXPLOD; sc_exp_r <= AMBIG_EXP;
                    end else if (a_vanished) begin
                        // sqrt(vanished) → UNDEF_SQRT_VANISH
                        state <= S_SHORTCUT;
                        sc_frac_r <= UNDEF_SQRT_VANISH; sc_exp_r <= AMBIG_EXP;
                    end else if (a_is_negative_normal) begin
                        // sqrt of negative → UNDEF_SQRT_NEG
                        state <= S_SHORTCUT;
                        sc_frac_r <= UNDEF_SQRT_NEG; sc_exp_r <= AMBIG_EXP;
                    end else begin
                        // Positive normal — compute sqrt
                        // Magnitude in N0 = a_q[23:0] = a_frac (24 bits, since
                        // for positive a_frac MSB=1, the inflated compute_q
                        // bits 23..0 equal a_frac bits 23..0 directly).
                        exp_odd_reg  <= a_exp[0];
                        exp_half_reg <= $signed({a_exp[EXP_BITS-1], a_exp}) >>> 1;

                        if (a_exp[0])
                            rad_reg <= {1'b0, a_frac[MAG-1:0], {(RAD_BITS-MAG-1){1'b0}}};
                        else
                            rad_reg <= {a_frac[MAG-1:0], {(RAD_BITS-MAG){1'b0}}};

                        rem_reg  <= {R_BITS{1'b0}};
                        root_reg <= {N_ITER{1'b0}};
                        counter  <= ITER_TOTAL[CTR_BITS-1:0] - 1'd1;
                        state    <= S_COMPUTE;
                    end
                end
            end

            S_COMPUTE: begin
                rem_reg  <= chain_rem[PARALLEL];
                root_reg <= chain_root[PARALLEL];
                rad_reg  <= chain_rad[PARALLEL];

                if (counter == 0) begin
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
