// spirix_divide — Banker's-rounded N0 iterative divide for paper comparison.
//
// Specialty module for the paper/comparison harness. Ports production
// spirix_divide_iter to N0 storage at FRAC=24 (binary32-equivalent).
//
// Architecture: restoring sequential division. 1 quotient bit per clock.
// Latency: COMPUTE_FRAC + 2 cycles from start to done.
//   Cycle 0 (start):              Inflate, sign extract, abs, first trial
//                                 subtract folded into start cycle.
//   Cycles 1..COMPUTE_FRAC-1:     Restoring division (shift + trial subtract).
//   Cycle COMPUTE_FRAC:           Finalize (normalize/round/sign + deflate).
//
// Interface:
//   start: pulse for 1 cycle to begin. Inputs sampled on this edge.
//   busy:  high while computing.
//   done:  pulses when result is valid.

module spirix_divide #(
    parameter FRAC_BITS = 24,
    parameter EXP_BITS  = 8,
    parameter PARALLEL  = 1   // trial-subtractions chained per clock (1, 2, 4, ...)
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
    localparam COMPUTE_FRAC = FRAC_BITS + 1;        // = 25
    // We need (COMPUTE_FRAC + 1) total quotient bits (= 26 at FRAC=24).
    // Each iteration cycle (start + S_COMPUTE) emits PARALLEL q bits.
    // ITER_TOTAL = ceil((COMPUTE_FRAC+1) / PARALLEL) cycles total.
    // S_COMPUTE runs ITER_TOTAL - 1 cycles (start cycle handles the first PARALLEL trials).
    localparam ITER_TOTAL = (COMPUTE_FRAC + 1 + PARALLEL - 1) / PARALLEL;
    localparam Q_BITS     = ITER_TOTAL * PARALLEL;
    localparam EXCESS_Q   = Q_BITS - (COMPUTE_FRAC + 1); // 0 if exact fit, else folded into sticky
    localparam CTR_BITS = $clog2(ITER_TOTAL + 1);

    localparam signed [EXP_BITS-1:0] AMBIG_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [EXP_BITS:0]   MAX_EXP_W = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0]   MIN_EXP_W = -(1 <<< (EXP_BITS - 1)) + 1;

    // N0 storage boundary patterns.
    localparam signed [FRAC_BITS-1:0] POS_ONE_NORMAL   = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_NORMAL   = {FRAC_BITS{1'b0}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_EXPLODED = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_EXPLODED = {1'b1, 1'b0, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] POS_ONE_VANISHED = {2'b00, 1'b1, {(FRAC_BITS-3){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE_VANISHED = {2'b11, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] ALL_ONES         = {FRAC_BITS{1'b1}};

    localparam integer UPAD = (FRAC_BITS > 8) ? FRAC_BITS - 8 : 0;
    localparam signed [FRAC_BITS-1:0] UNDEF_NEG_DIV_NEG = $signed({8'hE9, {UPAD{1'b0}}});
    localparam signed [FRAC_BITS-1:0] UNDEF_TF_DIV_TF   = $signed({8'h16, {UPAD{1'b0}}});
    localparam signed [FRAC_BITS-1:0] UNDEF_GENERAL     = $signed({8'hFE, {UPAD{1'b0}}});

    // ───── State detection (N0 conventions) ─────────────────────────────
    wire a_is_ambig  = (a_exp == AMBIG_EXP);
    wire b_is_ambig  = (b_exp == AMBIG_EXP);
    wire a_frac_zero = (a_frac == {FRAC_BITS{1'b0}});
    wire a_frac_neg1 = &a_frac;
    wire b_frac_zero = (b_frac == {FRAC_BITS{1'b0}});
    wire b_frac_neg1 = &b_frac;
    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;
    wire a_n1 = (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-2]);
    wire b_n1 = (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-2]);
    wire a_n2 = ~a_n1 & ~a_n0 & (a_frac[FRAC_BITS-1] != a_frac[FRAC_BITS-3]);
    wire b_n2 = ~b_n1 & ~b_n0 & (b_frac[FRAC_BITS-1] != b_frac[FRAC_BITS-3]);
    wire a_top3 = ~a_n0 & (a_frac[FRAC_BITS-1] == a_frac[FRAC_BITS-2]) &
                  (a_frac[FRAC_BITS-2] == a_frac[FRAC_BITS-3]);
    wire b_top3 = ~b_n0 & (b_frac[FRAC_BITS-1] == b_frac[FRAC_BITS-2]) &
                  (b_frac[FRAC_BITS-2] == b_frac[FRAC_BITS-3]);

    wire a_is_zero  = a_is_ambig & a_frac_zero;
    wire a_is_inf   = a_is_ambig & a_frac_neg1;
    wire a_exploded = a_is_ambig & a_n1;
    wire a_vanished = a_is_ambig & a_n2;
    wire a_undef    = a_is_ambig & a_top3;
    wire b_is_zero  = b_is_ambig & b_frac_zero;
    wire b_is_inf   = b_is_ambig & b_frac_neg1;
    wire b_exploded = b_is_ambig & b_n1;
    wire b_vanished = b_is_ambig & b_n2;
    wire b_undef    = b_is_ambig & b_top3;

    wire any_non_normal = a_is_ambig | b_is_ambig;

    // ───── Edge-case shortcut ───────────────────────────────────────────
    //   1. a undef → a, b undef → b (passthrough)
    //   2. b zero (and a normal/non-zero) → infinity
    //   3. a zero / b zero → undefined (0/0 specifically)
    //   4. a inf, b inf → undefined (inf/inf)
    //   5. a inf, b normal → infinity
    //   6. a zero or b inf → zero (0/anything or normal/inf)
    //   7. exploded/exploded → undefined
    //   8. vanished/vanished → undefined
    //   9. exploded or vanished alone → undefined (general)
    reg ec_shortcut;
    reg signed [FRAC_BITS-1:0] ec_sc_frac;
    reg signed [EXP_BITS-1:0]  ec_sc_exp;
    always @(*) begin
        ec_shortcut = 1'b0;
        ec_sc_frac  = {FRAC_BITS{1'b0}};
        ec_sc_exp   = AMBIG_EXP;
        if (any_non_normal) begin
            ec_shortcut = 1'b1;
            if (a_undef) begin
                ec_sc_frac = a_frac; ec_sc_exp = a_exp;
            end else if (b_undef) begin
                ec_sc_frac = b_frac; ec_sc_exp = b_exp;
            end else if (b_is_zero) begin
                ec_sc_frac = a_is_zero ? UNDEF_NEG_DIV_NEG : ALL_ONES; // inf
            end else if (a_is_inf) begin
                ec_sc_frac = b_is_inf ? UNDEF_TF_DIV_TF : ALL_ONES;     // inf
            end else if (a_is_zero || b_is_inf) begin
                ec_sc_frac = {FRAC_BITS{1'b0}};                          // zero
            end else if (a_exploded && b_exploded) begin
                ec_sc_frac = UNDEF_TF_DIV_TF;
            end else if (a_vanished && b_vanished) begin
                ec_sc_frac = UNDEF_NEG_DIV_NEG;
            end else begin
                ec_sc_frac = UNDEF_GENERAL;
            end
        end
    end

    // ───── State machine ─────────────────────────────────────────────────
    localparam S_IDLE     = 0;
    localparam S_COMPUTE  = 1;
    localparam S_FINALIZE = 2;
    localparam S_SHORTCUT = 3;
    reg [1:0] state = S_IDLE;
    assign busy = (state != S_IDLE);

    // Registered operands (init to 0 to avoid X propagation in sim)
    reg [COMPUTE_FRAC-2:0]  d_reg = {(COMPUTE_FRAC-1){1'b0}};
    reg [COMPUTE_FRAC-1:0]  r_reg = {COMPUTE_FRAC{1'b0}};
    reg [Q_BITS-1:0]        q_reg = {Q_BITS{1'b0}};
    reg [CTR_BITS-1:0]      counter = {CTR_BITS{1'b0}};
    reg                      sign_reg = 1'b0;
    reg signed [EXP_BITS:0] a_exp_adj_r = {(EXP_BITS+1){1'b0}};
    reg signed [EXP_BITS:0] b_exp_adj_r = {(EXP_BITS+1){1'b0}};

    reg signed [FRAC_BITS-1:0] sc_frac_r = {FRAC_BITS{1'b0}};
    reg signed [EXP_BITS-1:0]  sc_exp_r  = {EXP_BITS{1'b0}};

    // ───── N0 inflate to compute Q + abs value ──────────────────────────
    // a_q = inflate(a_frac) is 25-bit signed compute Q.
    // For abs: NEG_ONE_NORMAL (storage 0x000000) inflates to compute_q
    // = -2^FRAC, whose magnitude 2^FRAC overflows COMPUTE_FRAC-1 = 24
    // unsigned bits. Represent as POS_ONE_NORMAL magnitude = +2^(FRAC-1)
    // with exponent + 1 (same numerical value).
    wire signed [COMPUTE_FRAC-1:0] a_q = {~a_frac[FRAC_BITS-1], a_frac};
    wire signed [COMPUTE_FRAC-1:0] b_q = {~b_frac[FRAC_BITS-1], b_frac};
    wire a_is_neg_one_norm = (a_frac == NEG_ONE_NORMAL);
    wire b_is_neg_one_norm = (b_frac == NEG_ONE_NORMAL);

    wire [COMPUTE_FRAC-2:0] abs_a_q =
        a_is_neg_one_norm ? {1'b1, {(COMPUTE_FRAC-2){1'b0}}} :  // 2^(FRAC-1) magnitude
        (a_q[COMPUTE_FRAC-1] ? (~a_q[COMPUTE_FRAC-2:0] + 1'b1) : a_q[COMPUTE_FRAC-2:0]);
    wire [COMPUTE_FRAC-2:0] abs_b_q =
        b_is_neg_one_norm ? {1'b1, {(COMPUTE_FRAC-2){1'b0}}} :
        (b_q[COMPUTE_FRAC-1] ? (~b_q[COMPUTE_FRAC-2:0] + 1'b1) : b_q[COMPUTE_FRAC-2:0]);

    // Sign of compute Q: top bit of a_q (1=negative). N0 storage MSB inverted.
    wire a_sign = a_q[COMPUTE_FRAC-1];
    wire b_sign = b_q[COMPUTE_FRAC-1];

    // ───── Trial subtractor chain (PARALLEL trials per clock) ──────────
    // Inlined restoring chain. Production choice: throughput-optimal at P=4 with
    // 121 MHz silicon Fmax + 7-cycle latency. See divide_step_chain.v and
    // divide_step_chain_fpnew.v + SYNTH_NOTES.md for the full Fmax/algorithm sweep.
    // Step 0 of the start cycle uses r_in_0 = {1'b0, abs_a_q} (no shift).
    // All other steps shift the previous step's result left by 1 first.
    wire starting = (state == S_IDLE) & start;
    wire [COMPUTE_FRAC-2:0] d_mux = starting ? abs_b_q : d_reg;

    wire [COMPUTE_FRAC-1:0] step_in  [0:PARALLEL-1];
    wire [COMPUTE_FRAC-1:0] step_out [0:PARALLEL-1];
    wire [PARALLEL-1:0]     step_ge;

    // step 0
    assign step_in[0]  = starting ? {1'b0, abs_a_q}
                                  : {r_reg[COMPUTE_FRAC-2:0], 1'b0};
    wire [COMPUTE_FRAC:0] trial0 = {1'b0, step_in[0]} - {2'b00, d_mux};
    assign step_ge[0]  = !trial0[COMPUTE_FRAC];
    assign step_out[0] = step_ge[0] ? trial0[COMPUTE_FRAC-1:0] : step_in[0];

    genvar k;
    generate
        for (k = 1; k < PARALLEL; k = k + 1) begin : ts_chain
            assign step_in[k] = {step_out[k-1][COMPUTE_FRAC-2:0], 1'b0};
            wire [COMPUTE_FRAC:0] trial_k = {1'b0, step_in[k]} - {2'b00, d_mux};
            assign step_ge[k] = !trial_k[COMPUTE_FRAC];
            assign step_out[k] = step_ge[k] ? trial_k[COMPUTE_FRAC-1:0] : step_in[k];
        end
    endgenerate

    wire [COMPUTE_FRAC-1:0] r_next = step_out[PARALLEL-1];
    wire [PARALLEL-1:0] q_chunk;
    generate
        for (k = 0; k < PARALLEL; k = k + 1) begin : qpack
            assign q_chunk[PARALLEL-1-k] = step_ge[k];
        end
    endgenerate

    // ───── Finalize: bounded normalize + banker's RNE + sign + deflate ─
    // q_reg width is Q_BITS = ITER_TOTAL * PARALLEL. We use the top
    // (COMPUTE_FRAC + 1) = 26 bits as the "logical quotient" matching the
    // PARALLEL=1 reference; the bottom EXCESS_Q bits (if any) are below the
    // round position and OR into sticky.
    wire [COMPUTE_FRAC:0] q_top = q_reg[Q_BITS-1 : Q_BITS-(COMPUTE_FRAC+1)];

    wire excess_sticky;
    generate
        if (EXCESS_Q > 0) begin : excess_sticky_gen
            assign excess_sticky = |q_reg[EXCESS_Q-1:0];
        end else begin : excess_sticky_zero
            assign excess_sticky = 1'b0;
        end
    endgenerate

    wire norm_shift = q_top[COMPUTE_FRAC];
    wire [COMPUTE_FRAC:0] q_norm = norm_shift ? (q_top >> 1) : q_top;
    wire [COMPUTE_FRAC-1:0] frac_pos_raw = q_norm[COMPUTE_FRAC:1];

    wire f_guard       = q_norm[0];
    wire f_norm_sticky = norm_shift & q_top[0];
    wire f_rem_sticky  = |r_reg | excess_sticky;
    wire f_sticky      = f_norm_sticky | f_rem_sticky;
    wire f_lsb         = frac_pos_raw[0];
    wire f_round_up    = f_guard & (f_sticky | f_lsb);

    wire [COMPUTE_FRAC-1:0] frac_rounded = frac_pos_raw + {{(COMPUTE_FRAC-1){1'b0}}, f_round_up};
    wire round_ovf = (&frac_pos_raw[COMPUTE_FRAC-2:0]) & f_round_up;
    // After rovf renorm at compute-Q level: pos_q becomes +2^(FRAC-1) = compute_q
    // form 0_1_000..._0 (25-bit). exp += 1.
    wire [COMPUTE_FRAC-1:0] pos_q =
        round_ovf ? {1'b0, 1'b1, {(COMPUTE_FRAC-2){1'b0}}} : frac_rounded;

    // Apply sign at compute-Q level.
    // Negation boundary: pos_q = +2^(FRAC-1) (= 0_1_000..0) negated = -2^(FRAC-1)
    // (= 1_1_000..0, non-canonical). Renorm: -2^FRAC at exp-1 = NEG_ONE_NORMAL
    // compute_q = 1_0_000..0 at exp - 1.
    wire pos_q_is_half = (pos_q == {1'b0, 1'b1, {(COMPUTE_FRAC-2){1'b0}}});
    wire signed [COMPUTE_FRAC-1:0] neg_q =
        pos_q_is_half ? {1'b1, 1'b0, {(COMPUTE_FRAC-2){1'b0}}} :  // = -2^FRAC
                        (~pos_q + 1'b1);
    wire signed [COMPUTE_FRAC-1:0] final_q = sign_reg ? neg_q : $signed(pos_q);

    // Exponent
    wire signed [EXP_BITS:0] exp_base = a_exp_adj_r - b_exp_adj_r
                                       + {{EXP_BITS{1'b0}}, norm_shift}
                                       + {{EXP_BITS{1'b0}}, round_ovf};
    wire signed [EXP_BITS:0] exp_final = (sign_reg & pos_q_is_half) ?
                                          (exp_base - 1) : exp_base;
    wire exp_too_big   = (exp_final > MAX_EXP_W);
    wire exp_too_small = (exp_final < MIN_EXP_W);
    wire signed [EXP_BITS-1:0] final_exp = exp_final[EXP_BITS-1:0];

    // Deflate compute Q to N0 storage = compute_q[FRAC-1:0].
    wire signed [FRAC_BITS-1:0] final_storage = final_q[FRAC_BITS-1:0];

    // Saturation outputs (sign-preserving). Sign = compute Q top bit = ~storage MSB.
    wire result_is_pos = ~final_q[COMPUTE_FRAC-1];
    wire signed [FRAC_BITS-1:0] sat_exploded =
        result_is_pos ? POS_ONE_EXPLODED : NEG_ONE_EXPLODED;
    wire signed [FRAC_BITS-1:0] sat_vanished =
        result_is_pos ? POS_ONE_VANISHED : NEG_ONE_VANISHED;

    wire signed [FRAC_BITS-1:0] out_frac =
        exp_too_big   ? sat_exploded :
        exp_too_small ? sat_vanished :
                        final_storage;

    wire signed [EXP_BITS-1:0] out_exp =
        (exp_too_big | exp_too_small) ? AMBIG_EXP : final_exp;

    // ───── Sequential state machine ─────────────────────────────────────
    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    if (ec_shortcut) begin
                        sc_frac_r <= ec_sc_frac;
                        sc_exp_r  <= ec_sc_exp;
                        state <= S_SHORTCUT;
                    end else begin
                        // Sign at compute Q level (XOR of signs).
                        sign_reg <= a_sign ^ b_sign;

                        a_exp_adj_r <= $signed({a_exp[EXP_BITS-1], a_exp})
                                     + {{EXP_BITS{1'b0}}, a_is_neg_one_norm};
                        b_exp_adj_r <= $signed({b_exp[EXP_BITS-1], b_exp})
                                     + {{EXP_BITS{1'b0}}, b_is_neg_one_norm};

                        d_reg <= abs_b_q;
                        r_reg <= r_next;
                        q_reg <= {{(Q_BITS-PARALLEL){1'b0}}, q_chunk};

                        // S_COMPUTE runs ITER_TOTAL - 1 cycles. Counter init
                        // = ITER_TOTAL - 2; transition to FINALIZE on counter==0.
                        // If ITER_TOTAL <= 1 (very high PARALLEL), skip S_COMPUTE.
                        counter <= (ITER_TOTAL >= 2) ? (ITER_TOTAL[CTR_BITS-1:0] - 2'd2)
                                                     : {CTR_BITS{1'b0}};
                        state   <= (ITER_TOTAL >= 2) ? S_COMPUTE : S_FINALIZE;
                    end
                end
            end

            S_COMPUTE: begin
                r_reg <= r_next;
                q_reg <= {q_reg[Q_BITS-1-PARALLEL:0], q_chunk};
                if (counter == 0)
                    state <= S_FINALIZE;
                else
                    counter <= counter - 1;
            end

            S_FINALIZE: begin
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
