// spirix_core — Register-machine ALU, 21 ops, variable latency
//
// 8×128-bit register file (R0-R7), software-sequenced instructions.
// All ALU modules wired in parallel, selected by opcode.
// Target: 170 MHz on ECP5-25F (multiply_pipe bottleneck).
//
// Instruction word (18 bits):
//   [17:13] opcode   5-bit operation select (21 ops, room for 32)
//   [12:10] ra       source register A
//   [9:7]   rb       source register B
//   [6:4]   rd       destination register
//   [3:2]   frac_w   fraction width: 00=8 01=16 10=32 11=64
//   [1:0]   exp_w    exponent width:  00=8 01=16 10=32 11=64
//
// Latency (variable, from exec pulse to done pulse):
//   1 clk:  basic(NEG/ABS/SIGN/SHL/SHR), minmax(MIN/MAX), round(FLOOR/CEIL/ROUND)
//   1|3 clk: FRAC micro-op: non-normal→1clk shortcut, normal→3clk SUB(a,FLOOR(a))
//   3 clk:  addbit_pipe(ADD/SUB/AND/OR/XOR), multiply_pipe(MUL)
//   4 clk:  random(RNG, warm) — 2-cycle FSM + WB
//   N clk:  divmodsqrt(DIV/SQRT/MOD) — FRAC+3 to FRAC+5 cycles
//
// External register access: ext ports for loading inputs / reading results.

module spirix_core #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire        clk,
    input  wire        rst,

    // Instruction interface
    input  wire [17:0] instr,
    input  wire        exec,        // pulse: execute instruction
    output wire        busy,        // high while executing
    output reg         done = 0,    // pulse: instruction complete

    // External register access (active when not busy)
    input  wire [2:0]  ext_addr,
    input  wire signed [MAX_FRAC-1:0] ext_wfrac,
    input  wire signed [MAX_EXP-1:0]  ext_wexp,
    input  wire        ext_we,
    output wire signed [MAX_FRAC-1:0] ext_rfrac,
    output wire signed [MAX_EXP-1:0]  ext_rexp
);

    // ================================================================
    //  OPCODE ENCODING
    // ================================================================
    localparam [4:0]
        OP_NEG   = 5'd0,  OP_ABS   = 5'd1,  OP_SIGN  = 5'd2,
        OP_SHL   = 5'd3,  OP_SHR   = 5'd4,
        OP_MIN   = 5'd5,  OP_MAX   = 5'd6,
        OP_ADD   = 5'd7,  OP_SUB   = 5'd8,
        OP_AND   = 5'd9,  OP_OR    = 5'd10, OP_XOR   = 5'd11,
        OP_FLOOR = 5'd12, OP_CEIL  = 5'd13,
        OP_ROUND = 5'd14, OP_FRAC  = 5'd15,
        OP_MUL   = 5'd16,
        OP_DIV   = 5'd17, OP_SQRT  = 5'd18, OP_MOD   = 5'd19,
        OP_RNG   = 5'd20;

    // ================================================================
    //  INSTRUCTION DECODE
    // ================================================================
    reg [17:0] instr_r;
    wire [4:0] opcode = instr_r[17:13];
    wire [2:0] ra     = instr_r[12:10];
    wire [2:0] rb     = instr_r[9:7];
    wire [2:0] rd     = instr_r[6:4];
    wire [1:0] frac_w = instr_r[3:2];
    wire [1:0] exp_w  = instr_r[1:0];

    // Module select (one-hot from registered opcode)
    wire sel_basic  = (opcode <= OP_SHR);
    wire sel_minmax = (opcode == OP_MIN) | (opcode == OP_MAX);
    wire sel_addbit = ((opcode >= OP_ADD) & (opcode <= OP_XOR)) | (opcode == OP_FRAC);
    wire sel_round  = (opcode >= OP_FLOOR) & (opcode <= OP_ROUND);
    wire sel_mul    = (opcode == OP_MUL);
    wire sel_ds     = (opcode >= OP_DIV) & (opcode <= OP_MOD);
    wire sel_rng    = (opcode == OP_RNG);

    // Latency class
    wire is_1cycle = sel_basic | sel_minmax | sel_round;
    wire is_pipe   = sel_addbit | sel_mul;              // CE-gated 2-stage
    wire is_iter   = sel_ds | sel_rng;                  // start/done handshake

    // Internal op encoding for each module
    wire [2:0] basic_op  = opcode[2:0];            // 0-4
    wire       minmax_op = opcode[0];               // 0=MIN, 1=MAX
    wire is_frac = (opcode == OP_FRAC);
    wire [2:0] addbit_op = is_frac ? 3'd1 : (opcode[2:0] - 3'd7);  // FRAC→SUB, else ADD=0..XOR=4
    wire [1:0] round_op  = is_frac ? 2'd0 : opcode[1:0];           // FRAC→FLOOR, else FLOOR=0..ROUND=2
    wire [1:0] ds_op     = opcode[1:0] - 2'd1;     // DIV=0, SQRT=1, MOD=2

    // ================================================================
    //  REGISTER FILE — 8 × (64 frac + 64 exp), async read
    // ================================================================
    reg signed [MAX_FRAC-1:0] rf_frac [0:7];
    reg signed [MAX_EXP-1:0]  rf_exp  [0:7];

    // Async read ports
    wire signed [MAX_FRAC-1:0] a_frac = rf_frac[ra];
    wire signed [MAX_EXP-1:0]  a_exp  = rf_exp[ra];
    wire signed [MAX_FRAC-1:0] b_frac = rf_frac[rb];
    wire signed [MAX_EXP-1:0]  b_exp  = rf_exp[rb];

    // External read
    assign ext_rfrac = rf_frac[ext_addr];
    assign ext_rexp  = rf_exp[ext_addr];

    // ================================================================
    //  FRAC NON-NORMAL SHORTCUT
    // ================================================================
    // FRAC micro-op = SUB(a, FLOOR(a)) handles normal values via pipeline.
    // Non-normal inputs (AMBIG_EXP) need 1-cycle shortcut to match Rust spec:
    //   vanished neg → EFFECTIVELY_POS_ONE  (max_frac, exp=0)
    //   exploded     → ZERO                 (0, AMBIG_EXP)
    //   infinite     → FRACTIONAL_INFINITY  (0x1B, AMBIG_EXP)
    //   else         → self                 (zero, undef)
    localparam signed [MAX_EXP-1:0] AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_w == 2'd0) ? {8'hFF,  56'b0} :
        (frac_w == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_w == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                            {MAX_FRAC{1'b1}};

    wire frac_a_is_ambig = (a_exp == AMBIG_EXP);
    wire frac_a_neg      = a_frac[MAX_FRAC-1];
    wire frac_a_n1       = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire frac_a_n2       = ~frac_a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire frac_a_is_inf   = frac_a_is_ambig & (a_frac == frac_neg1_val);
    wire frac_a_exploded = frac_a_is_ambig & frac_a_n1;
    wire frac_a_vanished = frac_a_n2;
    wire frac_non_normal = is_frac & frac_a_is_ambig;

    wire signed [MAX_FRAC-1:0] max_frac_val =
        (frac_w == 2'd0) ? {8'h7F,  56'b0} :
        (frac_w == 2'd1) ? {16'h7FFF, 48'b0} :
        (frac_w == 2'd2) ? {32'h7FFFFFFF, 32'b0} :
                            {1'b0, {(MAX_FRAC-1){1'b1}}};

    wire signed [MAX_FRAC-1:0] frac_inf_val =
        (frac_w == 2'd0) ? {8'h1B,  56'b0} :
        (frac_w == 2'd1) ? {16'h001B, 48'b0} :
        (frac_w == 2'd2) ? {32'h0000001B, 32'b0} :
                            64'sh000000000000001B;

    wire signed [MAX_FRAC-1:0] frac_nn_frac =
        (frac_a_vanished & frac_a_neg) ? max_frac_val :
        frac_a_exploded                ? {MAX_FRAC{1'b0}} :
        frac_a_is_inf                  ? frac_inf_val :
                                          a_frac;

    wire signed [MAX_EXP-1:0] frac_nn_exp =
        (frac_a_vanished & frac_a_neg) ? {MAX_EXP{1'b0}} : AMBIG_EXP;

    // ================================================================
    //  ALU MODULE INSTANCES
    // ================================================================

    // --- basic (combinational, 1-clk) ---
    wire signed [MAX_FRAC-1:0] basic_rfrac, basic_rexp;
    spirix_alu_basic #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_basic (
        .op(basic_op), .frac_width(frac_w), .exp_width(exp_w),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(basic_rfrac), .result_exp(basic_rexp)
    );

    // --- minmax (combinational, 1-clk) ---
    wire signed [MAX_FRAC-1:0] minmax_rfrac, minmax_rexp;
    spirix_alu_minmax #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_minmax (
        .op(minmax_op), .frac_width(frac_w), .exp_width(exp_w),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(minmax_rfrac), .result_exp(minmax_rexp)
    );

    // --- round (combinational, 1-clk) ---
    wire signed [MAX_FRAC-1:0] round_rfrac, round_rexp;
    spirix_alu_round #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_round (
        .op(round_op), .frac_width(frac_w), .exp_width(exp_w),
        .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(round_rfrac), .result_exp(round_rexp)
    );

    // --- addbit_pipe (2-stage, CE-gated) ---
    // FRAC micro-op: b = FLOOR(a) from round module, op = SUB → result = a - FLOOR(a)
    wire signed [MAX_FRAC-1:0] addbit_b_frac = is_frac ? round_rfrac : b_frac;
    wire signed [MAX_EXP-1:0]  addbit_b_exp  = is_frac ? round_rexp  : b_exp;
    wire signed [MAX_FRAC-1:0] addbit_rfrac, addbit_rexp;
    reg pipe_ce;
    spirix_alu_addbit_pipe #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_addbit (
        .clk(clk), .ce(pipe_ce),
        .op(addbit_op), .frac_width(frac_w), .exp_width(exp_w),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(addbit_b_frac), .b_exp(addbit_b_exp),
        .result_frac(addbit_rfrac), .result_exp(addbit_rexp)
    );

    // --- multiply_pipe (2-stage, CE-gated) ---
    wire signed [MAX_FRAC-1:0] mul_rfrac, mul_rexp;
    spirix_alu_multiply_pipe #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_mul (
        .clk(clk), .ce(pipe_ce),
        .frac_width(frac_w), .exp_width(exp_w),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(mul_rfrac), .result_exp(mul_rexp)
    );

    // --- divmodsqrt (iterative, start/busy/done) ---
    wire signed [MAX_FRAC-1:0] ds_rfrac, ds_rexp;
    wire ds_busy, ds_done;
    reg  ds_start;
    spirix_alu_divmodsqrt #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_ds (
        .clk(clk), .start(ds_start),
        .op(ds_op), .frac_width(frac_w), .exp_width(exp_w),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(ds_rfrac), .result_exp(ds_rexp),
        .busy(ds_busy), .done(ds_done)
    );

    // --- random (2-cycle FSM, start/busy/done) ---
    wire signed [MAX_FRAC-1:0] rng_rfrac, rng_rexp;
    wire rng_busy, rng_done;
    reg  rng_start;
`ifdef NO_RNG
    assign rng_rfrac = {MAX_FRAC{1'b0}};
    assign rng_rexp  = {MAX_EXP{1'b0}};
    assign rng_busy  = 1'b0;
    assign rng_done  = 1'b0;
`else
    spirix_alu_random #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_rng (
        .clk(clk), .start(rng_start),
        .frac_width(frac_w), .exp_width(exp_w),
        .result_frac(rng_rfrac), .result_exp(rng_rexp),
        .busy(rng_busy), .done(rng_done),
        .dbg_xor()
    );
`endif

    // ================================================================
    //  RESULT MUX
    // ================================================================
    wire signed [MAX_FRAC-1:0] res_frac =
        sel_basic  ? basic_rfrac :
        sel_minmax ? minmax_rfrac :
        sel_round  ? round_rfrac :
        sel_addbit ? addbit_rfrac :
        sel_mul    ? mul_rfrac :
        sel_ds     ? ds_rfrac :
        sel_rng    ? rng_rfrac :
                     {MAX_FRAC{1'b0}};

    wire signed [MAX_EXP-1:0] res_exp =
        sel_basic  ? basic_rexp :
        sel_minmax ? minmax_rexp :
        sel_round  ? round_rexp :
        sel_addbit ? addbit_rexp :
        sel_mul    ? mul_rexp :
        sel_ds     ? ds_rexp :
        sel_rng    ? rng_rexp :
                     {MAX_EXP{1'b0}};

    // ================================================================
    //  FSM
    // ================================================================
    // Variable latency:
    //   1-cycle (combinational): EXEC0 → write → IDLE       (1 clk)
    //   3-cycle (pipelined):     EXEC0 → EXEC1 → WB → IDLE  (3 clk)
    //   N-cycle (iterative/rng): EXEC0 → WAIT... → WB → IDLE
    //
    // For pipelined modules (addbit_pipe, multiply_pipe):
    //   CE asserted in S_IDLE (on exec) so it's high during EXEC0+EXEC1.
    //   posedge leaving EXEC0: S1 captures (CE=1).
    //   posedge leaving EXEC1: S2 captures (CE=1) → output reg valid.
    //   WB: CE=0, read registered output, write to reg file.
    //
    // For iterative/random: start pulse in EXEC0, wait for done.
    //   done + result update at same posedge → need WB cycle to read.

    localparam [2:0]
        S_IDLE  = 3'd0,
        S_EXEC0 = 3'd1,
        S_EXEC1 = 3'd2,
        S_WAIT  = 3'd3,
        S_WB    = 3'd4;

    reg [2:0] state = S_IDLE;
    assign busy = (state != S_IDLE);

    always @(posedge clk) begin
        if (rst) begin
            state     <= S_IDLE;
            done      <= 0;
            pipe_ce   <= 0;
            ds_start  <= 0;
            rng_start <= 0;
        end else begin
            done      <= 0;
            ds_start  <= 0;
            rng_start <= 0;

            case (state)
                S_IDLE: begin
                    pipe_ce <= 0;
                    if (exec) begin
                        instr_r <= instr;
                        state   <= S_EXEC0;
                        // Assert CE early for pipe ops so it's high
                        // during both EXEC0 and EXEC1 (2 posedge captures).
                        // Decode from raw instr since instr_r not latched yet.
                        if (instr[17:13] >= OP_ADD && instr[17:13] <= OP_MUL)
                            pipe_ce <= 1;
                    end
                    // External register write (only when idle)
                    if (ext_we && !exec) begin
                        rf_frac[ext_addr] <= ext_wfrac;
                        rf_exp[ext_addr]  <= ext_wexp;
                    end
                end

                S_EXEC0: begin
                    // Instruction decoded, reg file read, ALUs computing
                    if (is_1cycle) begin
                        // Combinational result valid — write directly
                        rf_frac[rd] <= res_frac;
                        rf_exp[rd]  <= res_exp;
                        done        <= 1;
                        state       <= S_IDLE;
                    end else if (frac_non_normal) begin
                        // FRAC non-normal: 1-cycle shortcut (Rust spec)
                        rf_frac[rd] <= frac_nn_frac;
                        rf_exp[rd]  <= frac_nn_exp;
                        pipe_ce     <= 0;
                        done        <= 1;
                        state       <= S_IDLE;
                    end else if (is_pipe) begin
                        // CE already high from S_IDLE, S1 captures at this posedge
                        // Keep CE=1 for S2 capture at next posedge
                        state <= S_EXEC1;
                    end else begin
                        // Iterative (divmodsqrt) or random: pulse start
                        if (sel_ds)  ds_start  <= 1;
                        if (sel_rng) rng_start <= 1;
                        state <= S_WAIT;
                    end
                end

                S_EXEC1: begin
                    // Pipelined: CE=1 this cycle, S2 captures at next posedge
                    pipe_ce <= 0;  // CE off after this cycle
                    state   <= S_WB;
                end

                S_WAIT: begin
                    // Iterative/random: wait for done
                    if (ds_done | rng_done) begin
                        state <= S_WB;
                    end
                end

                S_WB: begin
                    // FRAC clamp: micro-op precision loss can produce 1.0;
                    // FRAC ∈ [0,1) so exp>0 means overflow → EFFECTIVELY_POS_ONE
                    if (is_frac & (res_exp > $signed({MAX_EXP{1'b0}}))) begin
                        rf_frac[rd] <= max_frac_val;
                        rf_exp[rd]  <= {MAX_EXP{1'b0}};
                    end else begin
                        rf_frac[rd] <= res_frac;
                        rf_exp[rd]  <= res_exp;
                    end
                    done        <= 1;
                    state       <= S_IDLE;
                end
            endcase
        end
    end

    // Initialize register file to zero (synthesis)
    integer i;
    initial begin
        for (i = 0; i < 8; i = i + 1) begin
            rf_frac[i] = {MAX_FRAC{1'b0}};
            rf_exp[i]  = {MAX_EXP{1'b0}};
        end
    end

endmodule
