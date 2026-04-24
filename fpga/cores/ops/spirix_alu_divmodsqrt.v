// spirix_alu_divmodsqrt — Unified iterative divider/sqrt/mod for multi-width Spirix scalars
//
// Restoring binary algorithms: 1 result bit per clock, 0 DSP.
// Runtime-selectable precision via frac_width/exp_width inputs.
//
// Operations:
//   op=00: Division (a / b).  Latency: ACTIVE_FRAC + 3 cycles.
//   op=01: Square root (sqrt(a)).  Latency: ACTIVE_FRAC + 4 cycles.
//   op=10: Modulo (a mod b, floored).  Latency: d + 3 cycles (early-term).
//   Edge case inputs: 2 cycles (S_SHORTCUT).
//
// Modulo: captures intermediate remainder at iteration d = a_exp - b_exp.
// Early-terminates division once remainder is captured (saves FRAC-d cycles).
// Result sign matches divisor (Python-style floored modulo).
// d < 0: passthrough a (same signs) or barrel-shift subtract (diff signs).
// d > ACTIVE_FRAC: returns (0, AMBIG) — insufficient precision.
//
// Width-dependent iteration count: matches divide_iter/sqrt_iter precision per active width (8/16/32/64-bit fractions → FRAC+1 / FRAC+2 iterations).
// Banker's rounding at active-width boundary.
//
// Interface:
//   start:  pulse high for 1 cycle. Inputs sampled on this edge.
//   busy:   high while computing. Do not assert start while busy.
//   done:   pulses high for 1 cycle when result is valid.
//
// Fractions MSB-aligned (sa() convention). Exponents LSB-aligned.
// Universal AMBIG: 0x8000...0 for all widths.
// Full edge case handling: undef, zero, inf, exploded, vanished, negative sqrt.

module spirix_alu_divmodsqrt #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire clk,
    input  wire start,
    input  wire [1:0]  op,                       // 00=DIV, 01=SQRT, 10=MOD
    input  wire [1:0]  frac_width,               // 00=8 01=16 10=32 11=64
    input  wire [1:0]  exp_width,                // 00=8 01=16 10=32 11=64
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output reg  signed [MAX_FRAC-1:0] result_frac,
    output reg  signed [MAX_EXP-1:0]  result_exp,
    output wire busy,
    output reg  done = 0
);

    // ================================================================
    //  CONSTANTS
    // ================================================================
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {1'b0, 1'b1, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam MAG = MAX_FRAC - 1;    // 63 unsigned magnitude bits

    // Sqrt sizing (for max width = 64-bit)
    localparam N_ITER   = MAX_FRAC + 1;   // 65 iterations max
    localparam RAD_BITS = 2 * N_ITER;     // 130-bit radicand
    localparam R_BITS   = N_ITER + 2;     // 67-bit remainder

    localparam CTR_BITS = $clog2(N_ITER + 1);  // 7

    // FSM (3-bit: 2-stage finalization pipeline)
    localparam S_IDLE     = 3'd0;
    localparam S_COMPUTE  = 3'd1;
    localparam S_FINAL1   = 3'd2;   // finalize stage 1: extract/normalize/CLZ
    localparam S_FINAL2   = 3'd3;   // finalize stage 2: sign, exp clamp, output
    localparam S_SHORTCUT = 3'd4;
    reg [2:0] state = S_IDLE;

    assign busy = (state != S_IDLE);

    // Op decode helpers
    wire op_is_sqrt = (op == 2'b01);
    wire op_is_mod  = (op == 2'b10);
    // op_is_div = ~op[1] & ~op[0], but we use default case

    // Edge case prefixes (MSB-aligned, top 8 bits)
    localparam signed [MAX_FRAC-1:0] UNDEF_NEG_DIV_NEG = {8'hE9, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_DIV_TF   = {8'h16, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_GENERAL      = {8'hFE, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_SQRT_NEG     = {8'hF6, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_SQRT_EXPLOD  = {8'h08, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_SQRT_VANISH  = {8'hF7, {(MAX_FRAC-8){1'b0}}};
    // MOD-specific undefined prefixes
    localparam signed [MAX_FRAC-1:0] UNDEF_TF_MOD       = {8'h15, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_MOD_VANISH   = {8'h14, {(MAX_FRAC-8){1'b0}}};
    localparam signed [MAX_FRAC-1:0] UNDEF_MOD_EXPLOD   = {8'h13, {(MAX_FRAC-8){1'b0}}};

    // ================================================================
    //  WIDTH-DEPENDENT CONSTANTS (from input frac_width/exp_width)
    // ================================================================
    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                                {64'hFFFFFFFFFFFFFFFF};

    // Registered width for finalization
    reg [1:0] frac_width_r, exp_width_r;

    wire [MAX_FRAC-1:0] frac_mask_r =
        (frac_width_r == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width_r == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width_r == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                                  {64'hFFFFFFFFFFFFFFFF};

    wire signed [MAX_EXP:0] exp_max_r =
        (exp_width_r == 2'd0) ? 65'sd127 :
        (exp_width_r == 2'd1) ? 65'sd32767 :
        (exp_width_r == 2'd2) ? 65'sd2147483647 :
                                 {1'b0, {(MAX_EXP-1){1'b1}}};
    wire signed [MAX_EXP:0] exp_min_r = -exp_max_r;

    // ================================================================
    //  STATE CLASSIFICATION (width-independent for MSB-aligned)
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);
    wire a_n1 = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire b_n1 = (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-2]);
    wire a_n2 = ~a_n1 & (a_frac[MAX_FRAC-2] != a_frac[MAX_FRAC-3]);
    wire b_n2 = ~b_n1 & (b_frac[MAX_FRAC-2] != b_frac[MAX_FRAC-3]);
    wire [7:0] a_pref = a_frac[MAX_FRAC-1 -: 8];
    wire [7:0] b_pref = b_frac[MAX_FRAC-1 -: 8];
    wire a_n0 = (a_pref == 8'h00) | (a_pref == 8'hFF);
    wire b_n0 = (b_pref == 8'h00) | (b_pref == 8'hFF);
    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire b_top3 = (b_frac[MAX_FRAC-1] == b_frac[MAX_FRAC-2]) &
                  (b_frac[MAX_FRAC-2] == b_frac[MAX_FRAC-3]);

    wire a_undef    = ~a_n0 & a_top3;
    wire b_undef    = ~b_n0 & b_top3;
    wire a_is_zero  = a_is_ambig & a_n0 & ~a_frac[MAX_FRAC-1];
    wire b_is_zero  = b_is_ambig & b_n0 & ~b_frac[MAX_FRAC-1];
    wire a_is_inf   = a_is_ambig & a_n0 & a_frac[MAX_FRAC-1];
    wire b_is_inf   = b_is_ambig & b_n0 & b_frac[MAX_FRAC-1];
    wire a_vanished = a_n2;
    wire b_vanished = b_n2;
    wire a_exploded = a_is_ambig & a_n1;
    wire b_exploded = b_is_ambig & b_n1;

    // a_transfinite = AMBIG exp AND (N1 frac OR infinity) — matches Rust is_transfinite()
    wire a_transfinite = a_exploded | a_is_inf;

    // Same-sign check for MOD edge cases (raw input comparison)
    wire same_sign_input = (a_frac[MAX_FRAC-1] == b_frac[MAX_FRAC-1]);

    // ================================================================
    //  EDGE CASE SHORTCUT
    // ================================================================
    reg ec_shortcut;
    reg signed [MAX_FRAC-1:0] ec_sc_frac;
    reg signed [MAX_EXP-1:0]  ec_sc_exp;

    always @(*) begin
        ec_shortcut = 1'b0;
        ec_sc_frac  = {MAX_FRAC{1'b0}};
        ec_sc_exp   = AMBIG_EXP;

        if (op_is_sqrt) begin
            // SQRT edge cases
            if (a_is_ambig) begin
                ec_shortcut = 1'b1;
                if (a_undef) begin
                    ec_sc_frac = a_frac; ec_sc_exp = a_exp;
                end else if (a_is_zero | a_is_inf) begin
                    ec_sc_frac = a_frac; ec_sc_exp = a_exp;
                end else if (a_vanished) begin
                    ec_sc_frac = UNDEF_SQRT_VANISH;
                end else begin
                    ec_sc_frac = UNDEF_SQRT_EXPLOD;
                end
            end else if (a_frac[MAX_FRAC-1]) begin
                ec_shortcut = 1'b1;
                ec_sc_frac  = UNDEF_SQRT_NEG;
            end
        end else if (op_is_mod) begin
            // MOD edge cases (Rust: scalar_modulus_scalar)
            if (a_is_ambig | b_is_ambig) begin
                ec_shortcut = 1'b1;
                if (a_undef) begin
                    ec_sc_frac = a_frac; ec_sc_exp = a_exp;
                end else if (b_undef) begin
                    ec_sc_frac = b_frac; ec_sc_exp = b_exp;
                end else if (a_is_zero | b_is_zero) begin
                    ec_sc_frac = {MAX_FRAC{1'b0}}; // ZERO
                end else if (a_transfinite) begin
                    ec_sc_frac = UNDEF_TF_MOD;
                end else if (b_is_inf) begin
                    ec_sc_frac = a_frac; ec_sc_exp = a_exp;
                end else if (b_vanished) begin
                    ec_sc_frac = UNDEF_MOD_VANISH;
                end else if (same_sign_input) begin
                    ec_sc_frac = a_frac; ec_sc_exp = a_exp;
                end else begin
                    ec_sc_frac = UNDEF_MOD_EXPLOD;
                end
            end
        end else begin
            // DIV edge cases
            if (a_is_ambig | b_is_ambig) begin
                ec_shortcut = 1'b1;
                if (a_undef) begin
                    ec_sc_frac = a_frac; ec_sc_exp = a_exp;
                end else if (b_undef) begin
                    ec_sc_frac = b_frac; ec_sc_exp = b_exp;
                end else if (b_is_zero) begin
                    ec_sc_frac = a_is_zero ? UNDEF_NEG_DIV_NEG : frac_neg1_val;
                end else if (a_is_inf) begin
                    ec_sc_frac = b_is_inf ? UNDEF_TF_DIV_TF : frac_neg1_val;
                end else if (a_is_zero | b_is_inf) begin
                    ec_sc_frac = {MAX_FRAC{1'b0}};
                end else if (a_exploded & b_exploded) begin
                    ec_sc_frac = UNDEF_TF_DIV_TF;
                end else if (a_vanished & b_vanished) begin
                    ec_sc_frac = UNDEF_NEG_DIV_NEG;
                end else begin
                    ec_sc_frac = UNDEF_GENERAL;
                end
            end
        end
    end

    // ================================================================
    //  ABS VALUE EXTRACTION (division/mod, NEG_ONE handling)
    // ================================================================
    wire a_is_neg_one = (a_frac == NEG_ONE);
    wire b_is_neg_one = (b_frac == NEG_ONE);

    wire [MAG-1:0] abs_a = a_is_neg_one ? POS_HALF[MAG-1:0] :
                            (a_frac[MAX_FRAC-1] ? (~a_frac[MAG-1:0] + 1'b1) :
                                                    a_frac[MAG-1:0]);
    wire [MAG-1:0] abs_b = b_is_neg_one ? POS_HALF[MAG-1:0] :
                            (b_frac[MAX_FRAC-1] ? (~b_frac[MAG-1:0] + 1'b1) :
                                                    b_frac[MAG-1:0]);

    // ================================================================
    //  WIDTH-DEPENDENT COUNTER INIT
    // ================================================================
    wire [CTR_BITS-1:0] div_counter_init =
        (frac_width == 2'd0) ? 7'd7 :
        (frac_width == 2'd1) ? 7'd15 :
        (frac_width == 2'd2) ? 7'd31 :
                                7'd63;

    wire [CTR_BITS-1:0] sqrt_counter_init =
        (frac_width == 2'd0) ? 7'd8 :
        (frac_width == 2'd1) ? 7'd16 :
        (frac_width == 2'd2) ? 7'd32 :
                                7'd64;

    // Active frac bits = div_counter_init + 1
    wire [CTR_BITS-1:0] active_frac = div_counter_init + 7'd1;

    // ================================================================
    //  MOD: EXPONENT DIFFERENCE (d = a_exp_adj - b_exp_adj)
    // ================================================================
    wire signed [MAX_EXP:0] mod_a_exp_adj = $signed({a_exp[MAX_EXP-1], a_exp})
                                          + {{MAX_EXP{1'b0}}, a_is_neg_one};
    wire signed [MAX_EXP:0] mod_b_exp_adj = $signed({b_exp[MAX_EXP-1], b_exp})
                                          + {{MAX_EXP{1'b0}}, b_is_neg_one};
    wire signed [MAX_EXP:0] mod_d = mod_a_exp_adj - mod_b_exp_adj;

    wire mod_d_neg = mod_d[MAX_EXP];  // d < 0
    wire mod_d_gt_frac = ~mod_d_neg & (mod_d > $signed({1'b0, {(MAX_EXP-CTR_BITS){1'b0}}, active_frac}));
    wire mod_d_is_zero = (mod_d == 0);

    // Counter target for capture: div_counter_init + 1 - d (only valid when 1 <= d <= active_frac)
    wire [CTR_BITS-1:0] mod_capture_ctr = active_frac - mod_d[CTR_BITS-1:0];

    // Active magnitude mask: top (frac_bits-1) bits of the MAG-bit field
    wire [MAG-1:0] mag_mask =
        (frac_width == 2'd0) ? {{7{1'b1}},  56'b0} :
        (frac_width == 2'd1) ? {{15{1'b1}}, 48'b0} :
        (frac_width == 2'd2) ? {{31{1'b1}}, 32'b0} :
                                {63{1'b1}};

    // d < 0 diff-signs: barrel shift abs_a right by |d| (clamped to 63)
    // Edge case: d = -64 (mod_d[5:0] = 0) wraps ~0+1 to 0. Clamp when bottom 6 bits are 0.
    wire mod_d_bottom_zero = (mod_d[5:0] == 6'b0);
    wire [5:0] mod_bypass_shift =
        (mod_d[MAX_EXP:6] != {(MAX_EXP-5){mod_d[MAX_EXP]}}) ? 6'd63 :
        mod_d_bottom_zero                                      ? 6'd63 :
                                                                 (~mod_d[5:0] + 6'd1);
    wire [MAG-1:0] mod_bypass_shifted = (abs_a >> mod_bypass_shift) & mag_mask;
    wire [MAG-1:0] mod_bypass_mag = abs_b - mod_bypass_shifted;

    // ================================================================
    //  REGISTERS
    // ================================================================
    reg [1:0]                   op_reg;
    reg [CTR_BITS-1:0]          counter;

    // Division registers (shared with MOD)
    reg [MAG-1:0]               d_reg;          // divisor magnitude
    reg [MAX_FRAC-1:0]          div_r_reg;      // partial remainder
    reg [MAX_FRAC:0]            q_reg;          // quotient shift register
    reg                         sign_reg;       // a_sign XOR b_sign
    reg                         a_neg_reg;      // original a was negative
    reg                         a_negone_reg;   // original a was NEG_ONE
    reg signed [MAX_EXP:0]      a_exp_adj_r;
    reg signed [MAX_EXP:0]      b_exp_adj_r;

    // Sqrt registers
    reg [RAD_BITS-1:0]          rad_reg;
    reg [R_BITS-1:0]            sqrt_rem_reg;
    reg [N_ITER-1:0]            root_reg;
    reg                         exp_odd_reg;
    reg signed [MAX_EXP:0]      exp_half_reg;
    reg [N_ITER-1:0]            sqrt_root_final;
    reg [R_BITS-1:0]            sqrt_rem_final;

    // MOD registers
    reg [MAX_FRAC-1:0]          mod_rem_r;      // captured remainder at iteration d
    reg [CTR_BITS-1:0]          mod_d_target_r; // counter value at which to capture
    reg                         mod_bypass_r;   // d<0 diff-signs (pre-corrected in mod_rem_r)
    reg                         b_neg_reg;      // b's sign for floored mod result

    // Finalization stage 1 → stage 2 pipeline registers
    reg [MAX_FRAC-1:0]          f1_frac_raw;
    reg                         f1_neg_trunc;
    reg [MAX_FRAC-1:0]          f1_neg_inc;
    reg                         f1_neg_adj_ovf;
    reg                         f1_sign;
    reg signed [MAX_EXP:0]      f1_exp_base;
    reg [MAX_FRAC-1:0]          f1_frac_mask;

    // Shortcut registers
    reg signed [MAX_FRAC-1:0]   sc_frac_r;
    reg signed [MAX_EXP-1:0]    sc_exp_r;

    // ================================================================
    //  DIVISION TRIAL SUBTRACTOR
    // ================================================================
    // Fires for DIV (op=00) and MOD (op=10): both have op[0]=0
    wire starting = (state == S_IDLE) & start & ~ec_shortcut & ~op[0];

    wire [MAX_FRAC-1:0] div_r_in = starting ? {1'b0, abs_a} :
                                    {div_r_reg[MAX_FRAC-2:0], 1'b0};
    wire [MAG-1:0] div_d_mux = starting ? abs_b : d_reg;

    wire [MAX_FRAC:0] div_trial = {1'b0, div_r_in} - {2'b00, div_d_mux};
    wire div_ge = ~div_trial[MAX_FRAC];
    wire [MAX_FRAC-1:0] div_r_next = div_ge ? div_trial[MAX_FRAC-1:0] : div_r_in;

    // ================================================================
    //  SQRT TRIAL SUBTRACTOR
    // ================================================================
    wire [1:0]        sqrt_rad_top2  = rad_reg[RAD_BITS-1:RAD_BITS-2];
    wire [R_BITS-1:0] sqrt_rem_shift = {sqrt_rem_reg[R_BITS-3:0], sqrt_rad_top2};
    wire [R_BITS-1:0] sqrt_trial_val = {root_reg, 2'b01};
    wire [R_BITS:0]   sqrt_trial_sub = {1'b0, sqrt_rem_shift} - {1'b0, sqrt_trial_val};
    wire              sqrt_ge        = ~sqrt_trial_sub[R_BITS];

    wire [N_ITER-1:0] sqrt_root_live = {root_reg[N_ITER-2:0], sqrt_ge};
    wire [R_BITS-1:0] sqrt_rem_live  = sqrt_ge ? sqrt_trial_sub[R_BITS-1:0] : sqrt_rem_shift;

    // ================================================================
    //  DIVISION FINALIZATION (reads from q_reg, div_r_reg in S_FINAL1)
    // ================================================================
    wire div_euclid_adj = a_neg_reg & ~a_negone_reg & (|div_r_reg);
    wire [MAX_FRAC:0] q_adj = q_reg + {{MAX_FRAC{1'b0}}, div_euclid_adj};

    wire div_norm_shift =
        (frac_width_r == 2'd0) ? q_adj[8] :
        (frac_width_r == 2'd1) ? q_adj[16] :
        (frac_width_r == 2'd2) ? q_adj[32] : q_adj[64];

    wire [MAX_FRAC:0] div_q_norm = div_norm_shift ? (q_adj >> 1) : q_adj;

    wire [MAX_FRAC-1:0] div_frac_raw =
        (frac_width_r == 2'd0) ? {div_q_norm[8:1], 56'b0} :
        (frac_width_r == 2'd1) ? {div_q_norm[16:1], 48'b0} :
        (frac_width_r == 2'd2) ? {div_q_norm[32:1], 32'b0} :
                                  div_q_norm[64:1];

    wire div_trunc_nonzero = div_q_norm[0] | (div_norm_shift & q_adj[0])
                           | (a_negone_reg & (|div_r_reg));
    wire div_neg_trunc_adj = sign_reg & div_trunc_nonzero;

    wire [MAX_FRAC-1:0] div_neg_inc =
        (frac_width_r == 2'd0) ? (64'h1 << 56) :
        (frac_width_r == 2'd1) ? (64'h1 << 48) :
        (frac_width_r == 2'd2) ? (64'h1 << 32) : 64'h1;

    wire div_neg_adj_ovf =
        (frac_width_r == 2'd0) ? (&div_frac_raw[62:56] & div_neg_trunc_adj) :
        (frac_width_r == 2'd1) ? (&div_frac_raw[62:48] & div_neg_trunc_adj) :
        (frac_width_r == 2'd2) ? (&div_frac_raw[62:32] & div_neg_trunc_adj) :
                                  (&div_frac_raw[62:0]  & div_neg_trunc_adj);

    wire signed [MAX_EXP:0] div_exp_base = a_exp_adj_r - b_exp_adj_r
        + {{MAX_EXP{1'b0}}, div_norm_shift}
        + {{MAX_EXP{1'b0}}, div_neg_adj_ovf};

    // ================================================================
    //  SQRT FINALIZATION
    // ================================================================
    wire [MAX_FRAC-1:0] sqrt_pos_frac =
        (frac_width_r == 2'd0) ? {1'b0, sqrt_root_final[8:2], 56'b0} :
        (frac_width_r == 2'd1) ? {1'b0, sqrt_root_final[16:2], 48'b0} :
        (frac_width_r == 2'd2) ? {1'b0, sqrt_root_final[32:2], 32'b0} :
                                  {1'b0, sqrt_root_final[64:2]};

    wire signed [MAX_EXP:0] sqrt_exp_base = exp_half_reg
        + {{MAX_EXP{1'b0}}, exp_odd_reg};

    // ================================================================
    //  MOD FINALIZATION (reads from mod_rem_r, d_reg, sign_reg)
    // ================================================================
    // Floored-mod sign correction: if signs differ and remainder nonzero,
    // complement with abs_b (result has b's sign, Python-style).
    wire [MAG-1:0] mod_euclid_rem = mod_rem_r[MAG-1:0];
    wire mod_rem_nz = |mod_euclid_rem;
    wire [MAG-1:0] mod_complement = d_reg - mod_euclid_rem;
    // For bypass (d<0 diff-signs): mod_rem_r already holds final magnitude
    wire [MAG-1:0] mod_corrected = mod_bypass_r ? mod_euclid_rem :
                                    (sign_reg & mod_rem_nz) ? mod_complement :
                                     mod_euclid_rem;
    wire mod_is_zero_fin = ~|mod_corrected;

    // CLZ on MAG-bit unsigned value
    reg [5:0] mod_clz;
    integer mi;
    always @(*) begin
        mod_clz = MAG[5:0];  // 63 = all zeros
        for (mi = 0; mi <= MAG - 1; mi = mi + 1)
            if (mod_corrected[mi])
                mod_clz = MAG[5:0] - 1 - mi[5:0];
    end

    wire [MAG-1:0] mod_norm = mod_corrected << mod_clz;
    wire [MAX_FRAC-1:0] mod_pos_frac = {1'b0, mod_norm};

    wire signed [MAX_EXP:0] mod_exp_base = b_exp_adj_r
        - {{(MAX_EXP-5){1'b0}}, mod_clz};

    // ================================================================
    //  FINALIZATION STAGE 2 COMBINATIONAL (from f1_* pipeline regs)
    // ================================================================
    wire [MAX_FRAC-1:0] f2_pos_frac =
        f1_neg_adj_ovf ? POS_HALF[MAX_FRAC-1:0] :
        f1_neg_trunc   ? (f1_frac_raw + f1_neg_inc) :
                          f1_frac_raw;

    wire f2_neg_is_ph = (f2_pos_frac == POS_HALF);

    wire signed [MAX_FRAC-1:0] f2_final_frac =
        f1_sign ? (f2_neg_is_ph ? NEG_ONE : (~f2_pos_frac + 1'b1)) :
                   $signed(f2_pos_frac);

    wire signed [MAX_EXP:0] f2_exp_final =
        (f1_sign & f2_neg_is_ph) ? (f1_exp_base - 1) : f1_exp_base;

    wire f2_exp_big   = (f2_exp_final > $signed(exp_max_r));
    wire f2_exp_small = (f2_exp_final < $signed(exp_min_r));

    wire signed [MAX_FRAC-1:0] f2_masked_frac = f2_final_frac & $signed(f1_frac_mask);
    wire signed [MAX_FRAC-1:0] f2_small_frac  = {f2_final_frac[MAX_FRAC-1],
                                                   f2_final_frac[MAX_FRAC-1:1]}
                                                 & $signed(f1_frac_mask);

    wire signed [MAX_FRAC-1:0] f2_out_frac =
        f2_exp_big   ? f2_masked_frac :
        f2_exp_small ? f2_small_frac  :
                        f2_masked_frac;

    wire signed [MAX_EXP-1:0] f2_out_exp =
        (f2_exp_big | f2_exp_small) ? AMBIG_EXP : f2_exp_final[MAX_EXP-1:0];

    // ================================================================
    //  STATE MACHINE
    // ================================================================
    always @(posedge clk) begin
        done <= 1'b0;

        case (state)
            S_IDLE: begin
                if (start) begin
                    frac_width_r <= frac_width;
                    exp_width_r  <= exp_width;
                    op_reg       <= op;

                    if (ec_shortcut) begin
                        sc_frac_r <= ec_sc_frac;
                        sc_exp_r  <= ec_sc_exp;
                        state <= S_SHORTCUT;

                    end else if (op_is_sqrt) begin
                        // SQRT: setup radicand
                        exp_odd_reg  <= a_exp[0];
                        exp_half_reg <= $signed({a_exp[MAX_EXP-1], a_exp}) >>> 1;

                        if (a_exp[0])
                            rad_reg <= {1'b0, a_frac[MAG-1:0], {(RAD_BITS-MAG-1){1'b0}}};
                        else
                            rad_reg <= {a_frac[MAG-1:0], {(RAD_BITS-MAG){1'b0}}};

                        sqrt_rem_reg <= {R_BITS{1'b0}};
                        root_reg     <= {N_ITER{1'b0}};
                        counter      <= sqrt_counter_init;
                        state        <= S_COMPUTE;

                    end else if (op_is_mod) begin
                        // MOD: setup shared div registers + mod-specific
                        sign_reg     <= a_frac[MAX_FRAC-1] ^ b_frac[MAX_FRAC-1];
                        a_neg_reg    <= a_frac[MAX_FRAC-1];
                        b_neg_reg    <= b_frac[MAX_FRAC-1];
                        a_negone_reg <= a_is_neg_one;
                        a_exp_adj_r  <= mod_a_exp_adj;
                        b_exp_adj_r  <= mod_b_exp_adj;
                        d_reg        <= abs_b;
                        mod_bypass_r <= 1'b0;

                        if (mod_d_neg & same_sign_input) begin
                            // d < 0, same signs: mod = a (passthrough)
                            sc_frac_r <= a_frac;
                            sc_exp_r  <= a_exp;
                            state <= S_SHORTCUT;
                        end else if (mod_d_neg) begin
                            // d < 0, diff signs: mod = abs_b - (abs_a >> |d|)
                            mod_rem_r    <= {{(MAX_FRAC-MAG){1'b0}}, mod_bypass_mag};
                            mod_bypass_r <= 1'b1;
                            state <= S_FINAL1;
                        end else if (mod_d_gt_frac) begin
                            // d > active_frac: ambiguous (0, AMBIG)
                            sc_frac_r <= {MAX_FRAC{1'b0}};
                            sc_exp_r  <= AMBIG_EXP;
                            state <= S_SHORTCUT;
                        end else if (mod_d_is_zero) begin
                            // d == 0: capture R_0 from this cycle's trial subtract
                            mod_rem_r <= div_r_next;
                            // Still register div state for finalization (d_reg etc.)
                            div_r_reg <= div_r_next;
                            q_reg     <= {{MAX_FRAC{1'b0}}, div_ge};
                            state <= S_FINAL1;
                        end else begin
                            // 1 <= d <= active_frac: run division, capture at iteration d
                            mod_d_target_r <= mod_capture_ctr;
                            div_r_reg      <= div_r_next;
                            q_reg          <= {{MAX_FRAC{1'b0}}, div_ge};
                            counter        <= div_counter_init;
                            state          <= S_COMPUTE;
                        end

                    end else begin
                        // DIV: first trial subtract folded into start
                        sign_reg     <= a_frac[MAX_FRAC-1] ^ b_frac[MAX_FRAC-1];
                        a_neg_reg    <= a_frac[MAX_FRAC-1];
                        a_negone_reg <= a_is_neg_one;
                        a_exp_adj_r  <= $signed({a_exp[MAX_EXP-1], a_exp})
                                     + {{MAX_EXP{1'b0}}, a_is_neg_one};
                        b_exp_adj_r  <= $signed({b_exp[MAX_EXP-1], b_exp})
                                     + {{MAX_EXP{1'b0}}, b_is_neg_one};

                        d_reg     <= abs_b;
                        div_r_reg <= div_r_next;
                        q_reg     <= {{MAX_FRAC{1'b0}}, div_ge};

                        counter <= div_counter_init;
                        state   <= S_COMPUTE;
                    end
                end
            end

            S_COMPUTE: begin
                if (op_reg[0]) begin
                    // SQRT iteration
                    sqrt_rem_reg <= sqrt_ge ? sqrt_trial_sub[R_BITS-1:0] : sqrt_rem_shift;
                    root_reg     <= {root_reg[N_ITER-2:0], sqrt_ge};
                    rad_reg      <= rad_reg << 2;

                    if (counter == 0) begin
                        sqrt_root_final <= sqrt_root_live;
                        sqrt_rem_final  <= sqrt_rem_live;
                        state <= S_FINAL1;
                    end else begin
                        counter <= counter - 1;
                    end
                end else begin
                    // DIV/MOD iteration
                    div_r_reg <= div_r_next;
                    q_reg     <= {q_reg[MAX_FRAC-1:0], div_ge};

                    if (op_reg[1] && (counter == mod_d_target_r)) begin
                        // MOD: capture R_d and early-terminate
                        mod_rem_r <= div_r_next;
                        state <= S_FINAL1;
                    end else if (counter == 0) begin
                        state <= S_FINAL1;
                    end else begin
                        counter <= counter - 1;
                    end
                end
            end

            S_FINAL1: begin
                f1_frac_mask <= frac_mask_r;
                if (op_reg[0]) begin
                    // SQRT finalization
                    f1_frac_raw    <= sqrt_pos_frac;
                    f1_neg_trunc   <= 1'b0;
                    f1_neg_inc     <= {MAX_FRAC{1'b0}};
                    f1_neg_adj_ovf <= 1'b0;
                    f1_sign        <= 1'b0;
                    f1_exp_base    <= sqrt_exp_base;
                end else if (op_reg[1]) begin
                    // MOD finalization: CLZ normalize, b's sign, b's exponent
                    if (mod_is_zero_fin) begin
                        // Exact division: result is ZERO
                        f1_frac_raw    <= {MAX_FRAC{1'b0}};
                        f1_neg_trunc   <= 1'b0;
                        f1_neg_inc     <= {MAX_FRAC{1'b0}};
                        f1_neg_adj_ovf <= 1'b0;
                        f1_sign        <= 1'b0;
                        f1_exp_base    <= {1'b1, {MAX_EXP{1'b0}}}; // force underflow → AMBIG
                    end else begin
                        f1_frac_raw    <= mod_pos_frac;
                        f1_neg_trunc   <= 1'b0;
                        f1_neg_inc     <= {MAX_FRAC{1'b0}};
                        f1_neg_adj_ovf <= 1'b0;
                        f1_sign        <= b_neg_reg;
                        f1_exp_base    <= mod_exp_base;
                    end
                end else begin
                    // DIV finalization
                    f1_frac_raw    <= div_frac_raw;
                    f1_neg_trunc   <= div_neg_trunc_adj;
                    f1_neg_inc     <= div_neg_inc;
                    f1_neg_adj_ovf <= div_neg_adj_ovf;
                    f1_sign        <= sign_reg;
                    f1_exp_base    <= div_exp_base;
                end
                state <= S_FINAL2;
            end

            S_FINAL2: begin
                result_frac <= f2_out_frac;
                result_exp  <= f2_out_exp;
                done  <= 1'b1;
                state <= S_IDLE;
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
