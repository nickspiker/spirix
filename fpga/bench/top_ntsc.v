// Single-instance CE-gated BLAKE3 self-test for Colorlight 5A-75B v8.0
//
// One blake3 instance, run twice: slow CE (gold, guaranteed correct) then
// fast CE (test, full PLL speed). Counter-based protocol ensures both
// phases process identical PRNG sequences. Compare accumulated hashes.
//
// CRT shows: gold hash (bottom strip), test hash (top strip, flipped CRT).
// Output: J1 pin 1 (R0, C4) = sync, J1 pin 2 (G0, D4) = video

module top_ntsc (
    input  wire clk,       // 25 MHz oscillator (P6)
    output reg  led,       // LED on T6 (active-low)
    input  wire btn,       // User button on R7 (active-low)
    output reg  ntsc_sync, // J1 R0 (C4) — 560Ω
    output reg  ntsc_vid,  // J1 G0 (D4) — 220Ω
    output wire oled_scl,  // J1 B0 (E4) — I2C SCL
    output wire oled_sda   // J1 R1 (D3) — I2C SDA
);

    // =========================================================================
    // PLL: high-speed clock for test
    // =========================================================================
    wire sys_clk, pll_lock;

    // Power-on reset: hold blake3 in reset for 15 sys_clk cycles after PLL lock
    reg [3:0] por_cnt = 0;
    wire      por_done = &por_cnt;

`ifdef PLL_CLKFB_DIV
    (* ICP_CURRENT="12" *) (* LPF_RESISTOR="8" *)
    (* MFG_ENABLE_FILTEROPAMP="1" *) (* MFG_GMCREF_SEL="2" *)
    EHXPLLL #(
        .PLLRST_ENA       ("DISABLED"),
        .INTFB_WAKE       ("DISABLED"),
        .STDBY_ENABLE     ("DISABLED"),
        .DPHASE_SOURCE    ("DISABLED"),
        .OUTDIVIDER_MUXA  ("DIVA"),
        .OUTDIVIDER_MUXB  ("DIVB"),
        .OUTDIVIDER_MUXC  ("DIVC"),
        .OUTDIVIDER_MUXD  ("DIVD"),
        .CLKI_DIV         (`PLL_CLKI_DIV),
        .CLKFB_DIV        (`PLL_CLKFB_DIV),
        .CLKOP_ENABLE     ("ENABLED"),
        .CLKOP_DIV        (`PLL_CLKOP_DIV),
        .CLKOP_CPHASE     (`PLL_CLKOP_CPHASE),
        .CLKOP_FPHASE     (0),
        .FEEDBK_PATH      ("CLKOP")
    ) pll (
        .RST(1'b0), .STDBY(1'b0), .CLKI(clk),
        .CLKOP(sys_clk), .CLKFB(sys_clk), .CLKINTFB(),
        .PHASESEL0(1'b0), .PHASESEL1(1'b0),
        .PHASEDIR(1'b1), .PHASESTEP(1'b1), .PHASELOADREG(1'b1),
        .PLLWAKESYNC(1'b0), .ENCLKOP(1'b0), .LOCK(pll_lock)
    );
`else
    assign sys_clk  = clk;
    assign pll_lock = 1'b1;
`endif

    always @(posedge sys_clk)
        if (!pll_lock)    por_cnt <= 0;
        else if (!por_done) por_cnt <= por_cnt + 1;

    // Blake3 reset: active-low. Assert during POR and button hold.
    wire b3_reset_n = por_done & pll_lock & ~btn_held_sys;

    // =========================================================================
    // Constants
    // =========================================================================
    localparam CE_GOLD_DIV = 256;      // gold CE divider (effective freq = PLL/256)

    // Protocol: 18-bit counter, bit taps for events (zero comparisons)
    //   [0..32767]      warmup (LFSR runs, no accumulation)
    //   [32768..131071] accumulate (98304 cycles, 4× previous protocol)
    //   bit 17 high     → done (≥131072)
    localparam PROTO_BITS = 18;

    // =========================================================================
    // XOR-fold: 512 bits → 32 bits
    // =========================================================================
    function [31:0] xor_fold;
        input [511:0] h;
        integer i;
        begin
            xor_fold = 32'b0;
            for (i = 0; i < 16; i = i + 1)
                xor_fold = xor_fold ^ h[i*32 +: 32];
        end
    endfunction

    // =========================================================================
    // Phase + CE
    // =========================================================================
    localparam [2:0] PH_IDLE   = 3'd0, PH_GOLD = 3'd1, PH_SWITCH = 3'd2,
                     PH_TEST   = 3'd3, PH_DONE = 3'd4;
    reg [2:0] phase = PH_IDLE;
    reg [63:0] captured_seed, captured_seed2;

`ifdef DUT_SPIRIX_BITWISE
`define DUT_MULTI_COMBO
`endif
`ifdef DUT_SPIRIX_ADDBIT
`define DUT_MULTI_COMBO
`endif
`ifdef DUT_SPIRIX_UNIFIED
`define DUT_MULTI_COMBO
`endif

`ifdef DUT_MULTI_COMBO
    // Multi-round: cycle through all 16 width combos (4 frac × 4 exp)
    reg [3:0]  combo = 0;          // sequential counter 0..15
    reg [3:0]  combo_mask = 0;     // random XOR mask (captured at start)
    wire [3:0] combo_actual = combo ^ combo_mask;  // shuffled combo index
    reg [15:0] combo_results = 0;  // pass/fail per combo (indexed by combo_actual)
    reg [15:0] combo_tested = 0;  // 1 = this combo has been tested
`endif

    // CE generator (registered for clean fanout at high freq)
    reg [7:0] ce_div = 0;
    reg       ce = 0;
    always @(posedge sys_clk) begin
        ce_div <= (ce_div == CE_GOLD_DIV - 1) ? 8'd0 : ce_div + 1;
        ce     <= (phase == PH_TEST) || (ce_div == CE_GOLD_DIV - 1);
    end

    // =========================================================================
    // PRNG: 64-bit Galois LFSR (maximal length, period 2^64-1)
    // Critical path: lfsr[0] → 1 LUT (AND tap) → FF.  ~0.7ns.
    // Taps: x^64 + x^63 + x^61 + x^60  (maximal, from Xilinx XAPP052)
    // Output: lfsr[31:0] window
    // =========================================================================
    localparam [63:0] LFSR_SEED  = 64'hCAFE_BABE_DEAD_BEEF;
    localparam [63:0] LFSR_SEED2 = 64'h0123_4567_89AB_CDEF;
    localparam [63:0] LFSR_TAPS = 64'hD800000000000000;  // bits 63,62,60,59

    reg [63:0] lfsr;
    wire       lfsr_fb = lfsr[0];
    wire [63:0] lfsr_next = {1'b0, lfsr[63:1]} ^ (lfsr_fb ? LFSR_TAPS : 64'b0);

    // Free-running entropy counter (never stops, XORed into seed on capture)
    reg [63:0] entropy = 0;
    always @(posedge sys_clk) entropy <= entropy + 1;

    // =========================================================================
    // DUT — switchable via defines
    //
    // DUT_SPIRIX_FMA (default), DUT_SPIRIX_MUL, DUT_SPIRIX_MUL_PIPE2,
    // DUT_SPIRIX_DIV_ITER, DUT_SPIRIX_DIVMOD_NR, DUT_SPIRIX_SQRT_NR,
    // DUT_SPIRIX_NR_DIV, DUT_SPIRIX_NR_SQRT, DUT_SPIRIX_SQRT_ITER,
    // DUT_HF_FMA, DUT_HF_MUL, DUT_HF_ADD, DUT_HF_DIV, DUT_HF_SQRT,
    // DUT_FPN_FMA, DUT_FPN_MUL, DUT_FPN_ADD, DUT_FPN_DIV, DUT_FPN_SQRT
    // All produce: 32-bit mul_fold. Iterative DUTs also define dut_advance.
    // =========================================================================

    // Second LFSR for 3-operand DUTs (FMA)
    reg [63:0] lfsr2;
    wire       lfsr2_fb = lfsr2[0];
    wire [63:0] lfsr2_next = {1'b0, lfsr2[63:1]} ^ (lfsr2_fb ? LFSR_TAPS : 64'b0);

`ifdef DUT_HF_FMA
    // ----- HardFloat mulAddRecFN (binary32 FMA), IEEE 754 I/O -----
    // LFSR provides 32-bit IEEE inputs; fNToRecFN/recFNToFN convert at boundary
    wire [31:0] ieee_a = lfsr[31:0];
    wire [31:0] ieee_b = lfsr[63:32];
    wire [31:0] ieee_c = lfsr2[31:0];
    wire [1:0]  hf_op  = lfsr2[33:32];

    wire [32:0] rec_a, rec_b, rec_c;
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_a (.in(ieee_a), .out(rec_a));
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_b (.in(ieee_b), .out(rec_b));
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_c (.in(ieee_c), .out(rec_c));

    wire [32:0] rec_out;
    wire [4:0]  hf_flags;
    mulAddRecFN #(.expWidth(8), .sigWidth(24)) dut_hf_fma (
        .control      (1'b1),
        .op           (hf_op),
        .a            (rec_a),
        .b            (rec_b),
        .c            (rec_c),
        .roundingMode (3'b000),
        .out          (rec_out),
        .exceptionFlags(hf_flags)
    );

    wire [31:0] hf_ieee_out;
    recFNToFN #(.expWidth(8), .sigWidth(24)) cvt_out (.in(rec_out), .out(hf_ieee_out));

    reg [31:0] hf_out_r;
    always @(posedge sys_clk) if (ce) hf_out_r <= hf_ieee_out;
    wire [31:0] mul_fold = hf_out_r;

`elsif DUT_HF_MUL
    // ----- HardFloat mulRecFN (binary32 multiply), IEEE 754 I/O -----
    wire [31:0] ieee_a = lfsr[31:0];
    wire [31:0] ieee_b = lfsr[63:32];

    wire [32:0] rec_a, rec_b;
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_a (.in(ieee_a), .out(rec_a));
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_b (.in(ieee_b), .out(rec_b));

    wire [32:0] rec_out;
    wire [4:0]  hf_flags;
    mulRecFN #(.expWidth(8), .sigWidth(24)) dut_hf_mul (
        .control      (1'b1),
        .a            (rec_a),
        .b            (rec_b),
        .roundingMode (3'b000),
        .out          (rec_out),
        .exceptionFlags(hf_flags)
    );

    wire [31:0] hf_ieee_out;
    recFNToFN #(.expWidth(8), .sigWidth(24)) cvt_out (.in(rec_out), .out(hf_ieee_out));

    reg [31:0] hf_out_r;
    always @(posedge sys_clk) if (ce) hf_out_r <= hf_ieee_out;
    wire [31:0] mul_fold = hf_out_r;

`elsif DUT_HF_ADD
    // ----- HardFloat addRecFN (binary32 add/sub), IEEE 754 I/O -----
    wire [31:0] ieee_a = lfsr[31:0];
    wire [31:0] ieee_b = lfsr[63:32];
    wire        hf_sub = lfsr[0];

    wire [32:0] rec_a, rec_b;
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_a (.in(ieee_a), .out(rec_a));
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_b (.in(ieee_b), .out(rec_b));

    wire [32:0] rec_out;
    wire [4:0]  hf_flags;
    addRecFN #(.expWidth(8), .sigWidth(24)) dut_hf_add (
        .control      (1'b1),
        .subOp        (hf_sub),
        .a            (rec_a),
        .b            (rec_b),
        .roundingMode (3'b000),
        .out          (rec_out),
        .exceptionFlags(hf_flags)
    );

    wire [31:0] hf_ieee_out;
    recFNToFN #(.expWidth(8), .sigWidth(24)) cvt_out (.in(rec_out), .out(hf_ieee_out));

    reg [31:0] hf_out_r;
    always @(posedge sys_clk) if (ce) hf_out_r <= hf_ieee_out;
    wire [31:0] mul_fold = hf_out_r;

`elsif DUT_FPN_FMA
    // ----- FPnew FMA (binary32), native IEEE 754 I/O -----
    wire [31:0] fpn_a = lfsr[31:0];
    wire [31:0] fpn_b = lfsr[63:32];
    wire [31:0] fpn_c = lfsr2[31:0];
    wire        fpn_sub = lfsr2[32];

    wire [31:0] fpn_result;
    fpnew_fma dut_fpn_fma (
        .clk_i          (sys_clk),
        .rst_ni         (b3_reset_n),
        .operands_i     ({fpn_c, fpn_b, fpn_a}),
        .is_boxed_i     (3'b111),
        .rnd_mode_i     (3'b000),       // RNE
        .op_i           (4'd0),         // FMADD
        .op_mod_i       (fpn_sub),
        .tag_i          (1'b0),
        .mask_i         (1'b1),
        .aux_i          (1'b0),
        .in_valid_i     (1'b1),
        .flush_i        (1'b0),
        .out_ready_i    (1'b1),
        .result_o       (fpn_result),
        .status_o       (),
        .extension_bit_o(),
        .tag_o          (),
        .mask_o         (),
        .aux_o          (),
        .out_valid_o    (),
        .in_ready_o     (),
        .busy_o         (),
        .reg_ena_i      (1'b0),
        .early_out_valid_o()
    );

    reg [31:0] fpn_out_r;
    always @(posedge sys_clk) if (ce) fpn_out_r <= fpn_result;
    wire [31:0] mul_fold = fpn_out_r;

`elsif DUT_FPN_MUL
    // ----- FPnew multiply (FMA with c=0), native IEEE 754 I/O -----
    wire [31:0] fpn_a = lfsr[31:0];
    wire [31:0] fpn_b = lfsr[63:32];

    wire [31:0] fpn_result;
    fpnew_fma dut_fpn_mul (
        .clk_i          (sys_clk),
        .rst_ni         (b3_reset_n),
        .operands_i     ({32'h00000000, fpn_b, fpn_a}),
        .is_boxed_i     (3'b111),
        .rnd_mode_i     (3'b000),       // RNE
        .op_i           (4'd3),         // MUL
        .op_mod_i       (1'b0),
        .tag_i          (1'b0),
        .mask_i         (1'b1),
        .aux_i          (1'b0),
        .in_valid_i     (1'b1),
        .flush_i        (1'b0),
        .out_ready_i    (1'b1),
        .result_o       (fpn_result),
        .status_o       (),
        .extension_bit_o(),
        .tag_o          (),
        .mask_o         (),
        .aux_o          (),
        .out_valid_o    (),
        .in_ready_o     (),
        .busy_o         (),
        .reg_ena_i      (1'b0),
        .early_out_valid_o()
    );

    reg [31:0] fpn_out_r;
    always @(posedge sys_clk) if (ce) fpn_out_r <= fpn_result;
    wire [31:0] mul_fold = fpn_out_r;

`elsif DUT_FPN_ADD
    // ----- FPnew add (FMA with b=1.0), native IEEE 754 I/O -----
    wire [31:0] fpn_a = lfsr[31:0];
    wire [31:0] fpn_b = lfsr[63:32];
    wire        fpn_sub = lfsr[0];

    wire [31:0] fpn_result;
    fpnew_fma dut_fpn_add (
        .clk_i          (sys_clk),
        .rst_ni         (b3_reset_n),
        .operands_i     ({fpn_b, 32'h3F800000, fpn_a}),
        .is_boxed_i     (3'b111),
        .rnd_mode_i     (3'b000),       // RNE
        .op_i           (4'd2),         // ADD
        .op_mod_i       (fpn_sub),
        .tag_i          (1'b0),
        .mask_i         (1'b1),
        .aux_i          (1'b0),
        .in_valid_i     (1'b1),
        .flush_i        (1'b0),
        .out_ready_i    (1'b1),
        .result_o       (fpn_result),
        .status_o       (),
        .extension_bit_o(),
        .tag_o          (),
        .mask_o         (),
        .aux_o          (),
        .out_valid_o    (),
        .in_ready_o     (),
        .busy_o         (),
        .reg_ena_i      (1'b0),
        .early_out_valid_o()
    );

    reg [31:0] fpn_out_r;
    always @(posedge sys_clk) if (ce) fpn_out_r <= fpn_result;
    wire [31:0] mul_fold = fpn_out_r;

`elsif DUT_SPIRIX_ADDSUB
    // ----- Spirix addsub (combinational + registered output) -----
    wire signed [24:0] as_a_frac = lfsr[24:0];
    wire signed  [7:0] as_a_exp  = lfsr[32:25];
    wire signed [24:0] as_b_frac = lfsr[57:33];
    wire signed  [7:0] as_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};
    wire               as_sub    = lfsr[0];

    wire signed [24:0] as_r_frac;
    wire signed  [7:0] as_r_exp;

    spirix_addsub #(.FRAC_BITS(25), .EXP_BITS(8)) dut_as (
        .a_frac(as_a_frac), .a_exp(as_a_exp),
        .b_frac(as_b_frac), .b_exp(as_b_exp),
        .sub(as_sub),
        .result_frac(as_r_frac), .result_exp(as_r_exp)
    );

    reg signed [24:0] as_r_frac_r;
    reg signed  [7:0] as_r_exp_r;
    always @(posedge sys_clk) if (ce) begin
        as_r_frac_r <= as_r_frac;
        as_r_exp_r  <= as_r_exp;
    end

    wire [32:0] as_out = {as_r_exp_r, as_r_frac_r};
    wire [31:0] mul_fold = as_out[31:0] ^ {31'b0, as_out[32]};

`elsif DUT_SPIRIX_ADDSUB_PIPE2
    // ----- Spirix addsub_pipe2 (2-stage pipeline) -----
    wire signed [24:0] as_a_frac = lfsr[24:0];
    wire signed  [7:0] as_a_exp  = lfsr[32:25];
    wire signed [24:0] as_b_frac = lfsr[57:33];
    wire signed  [7:0] as_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};
    wire               as_sub    = lfsr[0];

    wire signed [24:0] as_r_frac;
    wire signed  [7:0] as_r_exp;

    spirix_addsub_pipe2 #(.FRAC_BITS(25), .EXP_BITS(8)) dut_as (
        .clk(sys_clk), .ce(ce),
        .a_frac(as_a_frac), .a_exp(as_a_exp),
        .b_frac(as_b_frac), .b_exp(as_b_exp),
        .sub(as_sub),
        .result_frac(as_r_frac), .result_exp(as_r_exp)
    );

    wire [32:0] as_out = {as_r_exp, as_r_frac};
    wire [31:0] mul_fold = as_out[31:0] ^ {31'b0, as_out[32]};

`elsif DUT_SPIRIX_MUL
    // ----- Spirix multiply (standalone) -----
    wire signed [24:0] mul_a_frac = lfsr[24:0];
    wire signed  [7:0] mul_a_exp  = lfsr[32:25];
    wire signed [24:0] mul_b_frac = lfsr[57:33];
    wire signed  [7:0] mul_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};

    wire signed [24:0] mul_r_frac;
    wire signed  [7:0] mul_r_exp;

    spirix_multiply #(.FRAC_BITS(25), .EXP_BITS(8)) dut_mul (
        .a_frac(mul_a_frac), .a_exp(mul_a_exp),
        .b_frac(mul_b_frac), .b_exp(mul_b_exp),
        .negate(1'b0),
        .result_frac(mul_r_frac), .result_exp(mul_r_exp)
    );

    reg signed [24:0] mul_r_frac_r;
    reg signed  [7:0] mul_r_exp_r;
    always @(posedge sys_clk) if (ce) begin
        mul_r_frac_r <= mul_r_frac;
        mul_r_exp_r  <= mul_r_exp;
    end

    wire [32:0] mul_out = {mul_r_exp_r, mul_r_frac_r};
    wire [31:0] mul_fold = mul_out[31:0] ^ {31'b0, mul_out[32]};

`elsif DUT_SPIRIX_MUL_PIPE2
    // ----- Spirix multiply_pipe2 (2-stage pipeline) -----
    wire signed [24:0] mul_a_frac = lfsr[24:0];
    wire signed  [7:0] mul_a_exp  = lfsr[32:25];
    wire signed [24:0] mul_b_frac = lfsr[57:33];
    wire signed  [7:0] mul_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};

    wire signed [24:0] mul_r_frac;
    wire signed  [7:0] mul_r_exp;

    spirix_multiply_pipe2 #(.FRAC_BITS(25), .EXP_BITS(8)) dut_mul (
        .clk(sys_clk), .ce(ce),
        .a_frac(mul_a_frac), .a_exp(mul_a_exp),
        .b_frac(mul_b_frac), .b_exp(mul_b_exp),
        .negate(1'b0),
        .result_frac(mul_r_frac), .result_exp(mul_r_exp)
    );

    wire [32:0] mul_out = {mul_r_exp, mul_r_frac};
    wire [31:0] mul_fold = mul_out[31:0] ^ {31'b0, mul_out[32]};

`elsif DUT_SPIRIX_DIV_ITER
    // ----- Spirix divide_iter (iterative, 0 DSP) -----
    wire signed [24:0] dut_a_frac = lfsr[24:0];
    wire signed  [7:0] dut_a_exp  = lfsr[32:25];
    wire signed [24:0] dut_b_frac = lfsr[57:33];
    wire signed  [7:0] dut_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};

    wire signed [24:0] dut_r_frac;
    wire signed  [7:0] dut_r_exp;
    wire dut_busy_w, dut_done_w;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge sys_clk) begin
        iter_start <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && !dut_busy_w)
            begin iter_start <= 1; iter_busy <= 1; end
        else if (dut_done_w)
            iter_busy <= 0;
    end

    spirix_divide_iter #(.FRAC_BITS(25), .EXP_BITS(8)) dut (
        .clk(sys_clk), .start(iter_start),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .b_frac(dut_b_frac), .b_exp(dut_b_exp),
        .result_frac(dut_r_frac), .result_exp(dut_r_exp),
        .busy(dut_busy_w), .done(dut_done_w)
    );

    wire [32:0] dut_out = {dut_r_exp, dut_r_frac};
    wire [31:0] mul_fold = dut_out[31:0] ^ {31'b0, dut_out[32]};
    wire dut_advance = dut_done_w;
`define DUT_ITER_ADVANCE

`elsif DUT_SPIRIX_DIVMOD_NR
    // ----- Spirix divmod_nr (8-stage pipeline, 20 DSP) -----
    wire signed [24:0] dut_a_frac = lfsr[24:0];
    wire signed  [7:0] dut_a_exp  = lfsr[32:25];
    wire signed [24:0] dut_b_frac = lfsr[57:33];
    wire signed  [7:0] dut_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};

    wire signed [24:0] dut_q_frac;
    wire signed  [7:0] dut_q_exp;

    spirix_divmod_nr #(.FRAC_BITS(25), .EXP_BITS(8), .ENABLE_MOD(0)) dut (
        .clk(sys_clk), .ce(ce),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .b_frac(dut_b_frac), .b_exp(dut_b_exp),
        .q_frac(dut_q_frac), .q_exp(dut_q_exp),
        .mod_frac(), .mod_exp()
    );

    wire [32:0] dut_out = {dut_q_exp, dut_q_frac};
    wire [31:0] mul_fold = dut_out[31:0] ^ {31'b0, dut_out[32]};

`elsif DUT_SPIRIX_SQRT_NR
    // ----- Spirix sqrt_nr (10-stage pipeline, 27 DSP) -----
    wire signed [24:0] dut_a_frac = lfsr[24:0];
    wire signed  [7:0] dut_a_exp  = lfsr[32:25];

    wire signed [24:0] dut_r_frac;
    wire signed  [7:0] dut_r_exp;

    spirix_sqrt_nr #(.FRAC_BITS(25), .EXP_BITS(8)) dut (
        .clk(sys_clk), .ce(ce),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .result_frac(dut_r_frac), .result_exp(dut_r_exp)
    );

    wire [32:0] dut_out = {dut_r_exp, dut_r_frac};
    wire [31:0] mul_fold = dut_out[31:0] ^ {31'b0, dut_out[32]};

`elsif DUT_SPIRIX_NR_DIV
    // ----- Spirix nr_divsqrt (divide mode, iterative, 1 DSP) -----
    wire signed [24:0] dut_a_frac = lfsr[24:0];
    wire signed  [7:0] dut_a_exp  = lfsr[32:25];
    wire signed [24:0] dut_b_frac = lfsr[57:33];
    wire signed  [7:0] dut_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};

    wire signed [24:0] dut_r_frac;
    wire signed  [7:0] dut_r_exp;
    wire dut_busy_w, dut_done_w;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge sys_clk) begin
        iter_start <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && !dut_busy_w)
            begin iter_start <= 1; iter_busy <= 1; end
        else if (dut_done_w)
            iter_busy <= 0;
    end

    spirix_nr_divsqrt #(.FRAC_BITS(25), .EXP_BITS(8)) dut (
        .clk(sys_clk), .start(iter_start), .mode(1'b0),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .b_frac(dut_b_frac), .b_exp(dut_b_exp),
        .result_frac(dut_r_frac), .result_exp(dut_r_exp),
        .busy(dut_busy_w), .done(dut_done_w)
    );

    wire [32:0] dut_out = {dut_r_exp, dut_r_frac};
    wire [31:0] mul_fold = dut_out[31:0] ^ {31'b0, dut_out[32]};
    wire dut_advance = dut_done_w;
`define DUT_ITER_ADVANCE

`elsif DUT_SPIRIX_NR_SQRT
    // ----- Spirix nr_divsqrt (sqrt mode, iterative, 1 DSP) -----
    wire signed [24:0] dut_a_frac = lfsr[24:0];
    wire signed  [7:0] dut_a_exp  = lfsr[32:25];

    wire signed [24:0] dut_r_frac;
    wire signed  [7:0] dut_r_exp;
    wire dut_busy_w, dut_done_w;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge sys_clk) begin
        iter_start <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && !dut_busy_w)
            begin iter_start <= 1; iter_busy <= 1; end
        else if (dut_done_w)
            iter_busy <= 0;
    end

    spirix_nr_divsqrt #(.FRAC_BITS(25), .EXP_BITS(8)) dut (
        .clk(sys_clk), .start(iter_start), .mode(1'b1),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .b_frac(25'sb0), .b_exp(8'sb0),
        .result_frac(dut_r_frac), .result_exp(dut_r_exp),
        .busy(dut_busy_w), .done(dut_done_w)
    );

    wire [32:0] dut_out = {dut_r_exp, dut_r_frac};
    wire [31:0] mul_fold = dut_out[31:0] ^ {31'b0, dut_out[32]};
    wire dut_advance = dut_done_w;
`define DUT_ITER_ADVANCE

`elsif DUT_SPIRIX_SQRT_ITER
    // ----- Spirix sqrt_iter (iterative, 0 DSP) -----
    wire signed [24:0] dut_a_frac = lfsr[24:0];
    wire signed  [7:0] dut_a_exp  = lfsr[32:25];

    wire signed [24:0] dut_r_frac;
    wire signed  [7:0] dut_r_exp;
    wire dut_busy_w, dut_done_w;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge sys_clk) begin
        iter_start <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && !dut_busy_w)
            begin iter_start <= 1; iter_busy <= 1; end
        else if (dut_done_w)
            iter_busy <= 0;
    end

    spirix_sqrt_iter #(.FRAC_BITS(25), .EXP_BITS(8)) dut (
        .clk(sys_clk), .start(iter_start),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .result_frac(dut_r_frac), .result_exp(dut_r_exp),
        .busy(dut_busy_w), .done(dut_done_w)
    );

    wire [32:0] dut_out = {dut_r_exp, dut_r_frac};
    wire [31:0] mul_fold = dut_out[31:0] ^ {31'b0, dut_out[32]};
    wire dut_advance = dut_done_w;
`define DUT_ITER_ADVANCE

`elsif DUT_HF_DIV
    // ----- HardFloat divSqrtRecFN_small (divide), IEEE 754 I/O -----
    wire [31:0] ieee_a = lfsr[31:0];
    wire [31:0] ieee_b = lfsr[63:32];

    wire [32:0] rec_a, rec_b;
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_a (.in(ieee_a), .out(rec_a));
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_b (.in(ieee_b), .out(rec_b));

    wire hf_inReady, hf_outValid;
    wire [32:0] hf_rec_out;
    wire [4:0]  hf_flags;

    reg iter_busy = 0;
    reg iter_inValid = 0;
    always @(posedge sys_clk) begin
        iter_inValid <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && hf_inReady)
            begin iter_inValid <= 1; iter_busy <= 1; end
        else if (hf_outValid)
            iter_busy <= 0;
    end

    div_f32 dut_hf_div (
        .nReset    (b3_reset_n),
        .clock     (sys_clk),
        .inReady   (hf_inReady),
        .inValid   (iter_inValid),
        .a         (rec_a),
        .b         (rec_b),
        .roundingMode(3'b000),
        .outValid  (hf_outValid),
        .out       (hf_rec_out),
        .exceptionFlags(hf_flags)
    );

    wire [31:0] hf_ieee_out;
    recFNToFN #(.expWidth(8), .sigWidth(24)) cvt_out (.in(hf_rec_out), .out(hf_ieee_out));

    reg [31:0] hf_out_r;
    always @(posedge sys_clk) if (hf_outValid) hf_out_r <= hf_ieee_out;
    wire [31:0] mul_fold = hf_out_r;
    wire dut_advance = hf_outValid;
`define DUT_ITER_ADVANCE

`elsif DUT_HF_SQRT
    // ----- HardFloat divSqrtRecFN_small (sqrt), IEEE 754 I/O -----
    wire [31:0] ieee_a = lfsr[31:0];

    wire [32:0] rec_a;
    fNToRecFN #(.expWidth(8), .sigWidth(24)) cvt_a (.in(ieee_a), .out(rec_a));

    wire hf_inReady, hf_outValid;
    wire [32:0] hf_rec_out;
    wire [4:0]  hf_flags;

    reg iter_busy = 0;
    reg iter_inValid = 0;
    always @(posedge sys_clk) begin
        iter_inValid <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && hf_inReady)
            begin iter_inValid <= 1; iter_busy <= 1; end
        else if (hf_outValid)
            iter_busy <= 0;
    end

    sqrt_f32 dut_hf_sqrt (
        .nReset    (b3_reset_n),
        .clock     (sys_clk),
        .inReady   (hf_inReady),
        .inValid   (iter_inValid),
        .a         (rec_a),
        .roundingMode(3'b000),
        .outValid  (hf_outValid),
        .out       (hf_rec_out),
        .exceptionFlags(hf_flags)
    );

    wire [31:0] hf_ieee_out;
    recFNToFN #(.expWidth(8), .sigWidth(24)) cvt_out (.in(hf_rec_out), .out(hf_ieee_out));

    reg [31:0] hf_out_r;
    always @(posedge sys_clk) if (hf_outValid) hf_out_r <= hf_ieee_out;
    wire [31:0] mul_fold = hf_out_r;
    wire dut_advance = hf_outValid;
`define DUT_ITER_ADVANCE

`elsif DUT_FPN_DIV
    // ----- FPnew div (div_sqrt_mvp_wrapper, Div_start) -----
    wire [31:0] ieee_a = lfsr[31:0];
    wire [31:0] ieee_b = lfsr[63:32];

    wire fpn_ready, fpn_done;
    wire [63:0] fpn_result64;
    wire [4:0]  fpn_fflags;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge sys_clk) begin
        iter_start <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && fpn_ready)
            begin iter_start <= 1; iter_busy <= 1; end
        else if (fpn_done)
            iter_busy <= 0;
    end

    div_sqrt_mvp_wrapper #(.PrePipeline_depth_S(0), .PostPipeline_depth_S(0)) dut_fpn_div (
        .Clk_CI          (sys_clk),
        .Rst_RBI         (b3_reset_n),
        .Div_start_SI    (iter_start),
        .Sqrt_start_SI   (1'b0),
        .Operand_a_DI    ({32'b0, ieee_a}),
        .Operand_b_DI    ({32'b0, ieee_b}),
        .RM_SI           (3'b000),
        .Precision_ctl_SI(6'b0),
        .Format_sel_SI   (2'b00),       // FP32
        .Kill_SI         (1'b0),
        .Result_DO       (fpn_result64),
        .Fflags_SO       (fpn_fflags),
        .Ready_SO        (fpn_ready),
        .Done_SO         (fpn_done)
    );

    reg [31:0] fpn_out_r;
    always @(posedge sys_clk) if (fpn_done) fpn_out_r <= fpn_result64[31:0];
    wire [31:0] mul_fold = fpn_out_r;
    wire dut_advance = fpn_done;
`define DUT_ITER_ADVANCE

`elsif DUT_FPN_SQRT
    // ----- FPnew sqrt (div_sqrt_mvp_wrapper, Sqrt_start) -----
    wire [31:0] ieee_a = lfsr[31:0];

    wire fpn_ready, fpn_done;
    wire [63:0] fpn_result64;
    wire [4:0]  fpn_fflags;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge sys_clk) begin
        iter_start <= 0;
        if (btn_held_sys || !por_done)
            iter_busy <= 0;
        else if (ce && !iter_busy && fpn_ready)
            begin iter_start <= 1; iter_busy <= 1; end
        else if (fpn_done)
            iter_busy <= 0;
    end

    div_sqrt_mvp_wrapper #(.PrePipeline_depth_S(0), .PostPipeline_depth_S(0)) dut_fpn_sqrt (
        .Clk_CI          (sys_clk),
        .Rst_RBI         (b3_reset_n),
        .Div_start_SI    (1'b0),
        .Sqrt_start_SI   (iter_start),
        .Operand_a_DI    ({32'b0, ieee_a}),
        .Operand_b_DI    (64'b0),
        .RM_SI           (3'b000),
        .Precision_ctl_SI(6'b0),
        .Format_sel_SI   (2'b00),       // FP32
        .Kill_SI         (1'b0),
        .Result_DO       (fpn_result64),
        .Fflags_SO       (fpn_fflags),
        .Ready_SO        (fpn_ready),
        .Done_SO         (fpn_done)
    );

    reg [31:0] fpn_out_r;
    always @(posedge sys_clk) if (fpn_done) fpn_out_r <= fpn_result64[31:0];
    wire [31:0] mul_fold = fpn_out_r;
    wire dut_advance = fpn_done;
`define DUT_ITER_ADVANCE

`elsif DUT_SPIRIX_UNIFIED
    // ----- Spirix ALU unified (2-stage, 13 ops, 64-bit datapath) -----
    // Mixed exponent proximity: 25% identical, 25% close (±3), 25% moderate
    // (±127), 25% fully random. Ensures barrel/alignment paths get exercised
    // at all exp widths (E4+ would otherwise never hit close path).

    wire signed [63:0] un_a_frac = lfsr[63:0];
    wire signed [63:0] un_a_exp  = lfsr2[63:0];
    wire signed [63:0] un_b_frac = {lfsr[31:0], lfsr2[31:0]};

    // Exponent perturbation scaled to MSB-aligned exp_one_msb
    wire [1:0] un_exp_w  = combo_actual[3:2];  // 00=8 01=16 10=32 11=64
    wire [5:0] un_exp_rsh = (un_exp_w == 2'd0) ? 6'd56 :   // E3: top 8 bits active
                            (un_exp_w == 2'd1) ? 6'd48 :   // E4: top 16 bits active
                            (un_exp_w == 2'd2) ? 6'd32 :   // E5: top 32 bits active
                                                 6'd0;     // E6: all 64 bits active

    // 4 exponent proximity modes (2 LFSR bits select)
    wire [1:0] exp_mode = lfsr2[37:36];
    // Small perturbation: ±3 in exponent space (sign-extend 3 bits)
    wire signed [63:0] exp_pert_close = {{61{lfsr[2]}}, lfsr[2:0]} <<< un_exp_rsh;
    // Moderate perturbation: ±127 in exponent space (sign-extend 7 bits)
    wire signed [63:0] exp_pert_mod   = {{57{lfsr[6]}}, lfsr[6:0]} <<< un_exp_rsh;

    wire signed [63:0] un_b_exp =
        (exp_mode == 2'b00) ? un_a_exp :                              // identical
        (exp_mode == 2'b01) ? (un_a_exp + exp_pert_close) :           // close ±3
        (exp_mode == 2'b10) ? (un_a_exp + exp_pert_mod) :             // moderate ±127
                              {lfsr2[31:0], lfsr[31:0]};              // fully random

    // Map lfsr2 bits to valid op range 0..12 (13 ops)
    wire [3:0] un_raw_op = lfsr2[35:32];
    wire [3:0] un_op = (un_raw_op > 4'd12) ? (un_raw_op - 4'd3) : un_raw_op;

    // Frac width from shuffled combo (exp_w already declared above for perturbation)
    wire [1:0] un_frac_w = combo_actual[1:0];  // 00=8 01=16 10=32 11=64

    wire signed [63:0] un_r_frac, un_r_exp;
    wire un_cmp_lt, un_cmp_eq, un_cmp_gt, un_cmp_unord;

    spirix_alu_unified #(.MAX_FRAC(64), .MAX_EXP(64)) dut_un (
        .clk(sys_clk), .ce(ce),
        .op(un_op),
        .frac_width(un_frac_w),
        .exp_width(un_exp_w),
        .a_frac(un_a_frac), .a_exp(un_a_exp),
        .b_frac(un_b_frac), .b_exp(un_b_exp),
        .result_frac(un_r_frac), .result_exp(un_r_exp),
        .cmp_lt(un_cmp_lt), .cmp_eq(un_cmp_eq),
        .cmp_gt(un_cmp_gt), .cmp_unord(un_cmp_unord)
    );

    wire [31:0] mul_fold = un_r_frac[63:32] ^ un_r_frac[31:0]
                         ^ un_r_exp[63:32]  ^ un_r_exp[31:0]
                         ^ {28'b0, un_cmp_lt, un_cmp_eq, un_cmp_gt, un_cmp_unord};

`elsif DUT_SPIRIX_ADDBIT
    // ----- Spirix ALU addbit (2-stage pipeline, 5 ops, 64-bit datapath) -----
    // ADD/SUB/AND/OR/XOR with close/far split + shared barrel.
    // Full 64-bit inputs, random width + op selection from LFSR.

    wire signed [63:0] ab_a_frac = lfsr[63:0];
    wire signed [63:0] ab_a_exp  = lfsr2[63:0];
    wire signed [63:0] ab_b_frac = {lfsr[31:0], lfsr2[31:0]};
    wire signed [63:0] ab_b_exp  = {lfsr2[31:0], lfsr[31:0]};

    // Map lfsr2 bits to valid op range 0..4 (5 ops)
    wire [2:0] ab_raw_op = lfsr2[34:32];
    wire [2:0] ab_op = (ab_raw_op > 3'd4) ? (ab_raw_op - 3'd5) : ab_raw_op;

    // Width from shuffled combo: random execution order each run
    wire [1:0] ab_frac_w = combo_actual[1:0];  // 00=8 01=16 10=32 11=64
    wire [1:0] ab_exp_w  = combo_actual[3:2];  // 00=8 01=16 10=32 11=64

    wire signed [63:0] ab_r_frac, ab_r_exp;

    spirix_alu_addbit #(.MAX_FRAC(64), .MAX_EXP(64)) dut_ab (
        .clk(sys_clk), .ce(ce),
        .op(ab_op),
        .frac_width(ab_frac_w),
        .exp_width(ab_exp_w),
        .a_frac(ab_a_frac), .a_exp(ab_a_exp),
        .b_frac(ab_b_frac), .b_exp(ab_b_exp),
        .result_frac(ab_r_frac), .result_exp(ab_r_exp)
    );

    wire [31:0] mul_fold = ab_r_frac[63:32] ^ ab_r_frac[31:0]
                         ^ ab_r_exp[63:32]  ^ ab_r_exp[31:0];

`elsif DUT_SPIRIX_BITWISE
    // ----- Spirix ALU bitwise (single-stage, 11 ops, 64-bit datapath) -----
    // Full 64-bit inputs, random width + op selection from LFSR.
    // Exercises all 4 frac widths (8/16/32/64) and all 4 exp widths.

    wire signed [63:0] bw_a_frac = lfsr[63:0];
    wire signed [63:0] bw_a_exp  = lfsr2[63:0];
    wire signed [63:0] bw_b_frac = {lfsr[31:0], lfsr2[31:0]};
    wire signed [63:0] bw_b_exp  = {lfsr2[31:0], lfsr[31:0]};

    // Map lfsr2 bits to valid op range 0..9 (10 ops, no FLOOR)
    wire [3:0] bw_raw_op = lfsr2[35:32];
    wire [3:0] bw_op = (bw_raw_op > 4'd9) ? (bw_raw_op - 4'd6) : bw_raw_op;

    // Width from shuffled combo: random execution order each run
    wire [1:0] bw_frac_w = combo_actual[1:0];  // 00=8 01=16 10=32 11=64
    wire [1:0] bw_exp_w  = combo_actual[3:2];  // 00=8 01=16 10=32 11=64

    wire signed [63:0] bw_r_frac, bw_r_exp;
    wire bw_cmp_lt, bw_cmp_eq, bw_cmp_gt, bw_cmp_unord;

    spirix_alu_bitwise #(.MAX_FRAC(64), .MAX_EXP(64)) dut_bw (
        .clk(sys_clk), .ce(ce),
        .op(bw_op),
        .frac_width(bw_frac_w),
        .exp_width(bw_exp_w),
        .a_frac(bw_a_frac), .a_exp(bw_a_exp),
        .b_frac(bw_b_frac), .b_exp(bw_b_exp),
        .result_frac(bw_r_frac), .result_exp(bw_r_exp),
        .cmp_lt(bw_cmp_lt), .cmp_eq(bw_cmp_eq),
        .cmp_gt(bw_cmp_gt), .cmp_unord(bw_cmp_unord)
    );

    wire [31:0] mul_fold = bw_r_frac[63:32] ^ bw_r_frac[31:0]
                         ^ bw_r_exp[63:32]  ^ bw_r_exp[31:0]
                         ^ {28'b0, bw_cmp_lt, bw_cmp_eq, bw_cmp_gt, bw_cmp_unord};

`else
    // ----- Spirix FMA (default) -----
    wire signed [24:0] fma_a_frac = lfsr[24:0];
    wire signed  [7:0] fma_a_exp  = lfsr[32:25];
    wire signed [24:0] fma_b_frac = lfsr[57:33];
    wire signed  [7:0] fma_b_exp  = {lfsr[63], lfsr[63], lfsr[63:58]};
    wire signed [24:0] fma_c_frac = lfsr2[24:0];
    wire signed  [7:0] fma_c_exp  = lfsr2[32:25];
    wire               fma_sub    = lfsr[0];

    wire signed [24:0] fma_r_frac;
    wire signed  [7:0] fma_r_exp;

    spirix_fma #(.FRAC_BITS(25), .EXP_BITS(8)) dut_fma (
        .a_frac(fma_a_frac), .a_exp(fma_a_exp),
        .b_frac(fma_b_frac), .b_exp(fma_b_exp),
        .c_frac(fma_c_frac), .c_exp(fma_c_exp),
        .sub(fma_sub),
        .result_frac(fma_r_frac), .result_exp(fma_r_exp)
    );

    reg signed [24:0] fma_r_frac_r;
    reg signed  [7:0] fma_r_exp_r;
    always @(posedge sys_clk) if (ce) begin
        fma_r_frac_r <= fma_r_frac;
        fma_r_exp_r  <= fma_r_exp;
    end

    wire [32:0] fma_out = {fma_r_exp_r, fma_r_frac_r};
    wire [31:0] mul_fold = fma_out[31:0] ^ {31'b0, fma_out[32]};
`endif

`ifndef DUT_ITER_ADVANCE
    wire dut_advance = ce;
`endif

    // =========================================================================
    // Protocol counter + accumulator + phase FSM
    // =========================================================================

    reg [PROTO_BITS-1:0] proto_cnt;
    wire       proto_done = proto_cnt[17];           // bit tap: done at 131072
    wire       accumulating = proto_cnt[15] & ~proto_done;  // bit tap: accum from 32768..131071
    reg [31:0] accum;
    reg [31:0] gold_reg = 0, test_reg = 0;
    reg        test_done_sys = 0;

    always @(posedge sys_clk) begin
        if (!pll_lock || !por_done) begin
            // Hard reset (PLL not locked or POR)
            phase          <= PH_IDLE;
            lfsr           <= LFSR_SEED;
            lfsr2          <= LFSR_SEED2;
            captured_seed  <= LFSR_SEED;
            captured_seed2 <= LFSR_SEED2;
            proto_cnt      <= 0;
            accum          <= 0;
            gold_reg       <= 0;
            test_reg       <= 0;
            test_done_sys  <= 0;
`ifdef DUT_MULTI_COMBO
            combo          <= 0;
            combo_mask     <= 0;
            combo_results  <= 0;
            combo_tested   <= 0;
`endif
        end else if (btn_held_sys) begin
            // Button held: reset to idle, LFSR free-runs for entropy
            phase         <= PH_IDLE;
            proto_cnt     <= 0;
            accum         <= 0;
            gold_reg      <= 0;
            test_reg      <= 0;
            test_done_sys <= 0;
`ifdef DUT_MULTI_COMBO
            combo         <= 0;
            combo_mask     <= 0;
            combo_results <= 0;
            combo_tested  <= 0;
`endif
            lfsr          <= lfsr_next;
            lfsr2         <= lfsr2_next;
        end else begin

            // =================================================================
            // Phase transitions
            // =================================================================
            case (phase)
                PH_IDLE: begin
                    // Button just released (or first run after POR):
                    // XOR entropy counter into LFSR for unique seed each run
                    captured_seed  <= lfsr ^ entropy;
                    captured_seed2 <= lfsr2 ^ entropy;
                    lfsr           <= lfsr ^ entropy;
                    lfsr2          <= lfsr2 ^ entropy;
                    proto_cnt      <= 0;
                    accum          <= 0;
`ifdef DUT_MULTI_COMBO
                    // Capture shuffle mask on first combo only
                    if (combo == 0)
                        combo_mask <= entropy[3:0];
`endif
                    phase          <= PH_GOLD;
                end
                PH_GOLD: begin
                    if (proto_done) begin
                        gold_reg <= accum;
                        phase    <= PH_SWITCH;
                    end
                end
                PH_SWITCH: begin
                    // Reset LFSR to captured seed for test phase
                    lfsr           <= captured_seed;
                    lfsr2          <= captured_seed2;
                    proto_cnt      <= 0;
                    accum          <= 0;
                    phase          <= PH_TEST;
                end
                PH_TEST: begin
                    if (proto_done) begin
                        test_reg      <= accum;
                        test_done_sys <= 1;
                        phase         <= PH_DONE;
                    end
                end
                PH_DONE: begin
`ifdef DUT_MULTI_COMBO
                    // Record pass/fail + tested for this width combo, advance to next
                    combo_results <= combo_results | ({15'b0, (gold_reg == test_reg)} << combo_actual);
                    combo_tested  <= combo_tested  | (16'b1 << combo_actual);
                    if (combo != 4'd15) begin
                        combo         <= combo + 1;
                        test_done_sys <= 0;
                        proto_cnt     <= 0;
                        accum         <= 0;
                        gold_reg      <= 0;
                        test_reg      <= 0;
                        phase         <= PH_IDLE;
                    end
`endif
                end
            endcase

            // =================================================================
            // CE-gated datapath (gold & test phases only)
            // =================================================================
            if (dut_advance && (phase == PH_GOLD || phase == PH_TEST)) begin
                lfsr  <= lfsr_next;
                lfsr2 <= lfsr2_next;

                if (accumulating)
                    accum <= {accum[30:0], accum[31]} ^ mul_fold;

                if (!proto_done)
                    proto_cnt <= proto_cnt + 1;
            end
        end
    end

    // =========================================================================
    // Button sync (no debounce — bouncing just re-runs the test, more entropy)
    // =========================================================================
    reg [1:0] btn_sync_sys = 2'b11;
    always @(posedge sys_clk) btn_sync_sys <= {btn_sync_sys[0], btn};
    wire btn_held_sys = ~btn_sync_sys[1];  // 1 when button held (sys_clk domain)

    // =========================================================================
    // CDC: test_done_sys → clk domain
    // =========================================================================
    reg done_sync1 = 0, done_sync2 = 0;
    always @(posedge clk) begin
        done_sync1 <= test_done_sys;
        done_sync2 <= done_sync1;
    end

    reg lock_sync1 = 0, lock_sync2 = 0;
    always @(posedge clk) begin
        lock_sync1 <= pll_lock;
        lock_sync2 <= lock_sync1;
    end

    // =========================================================================
    // Status (clk domain)
    // =========================================================================
`ifdef DUT_MULTI_COMBO
    wire pass = &combo_results;  // all 16 combos must pass
`else
    wire pass = (gold_reg == test_reg);
`endif
    wire [1:0] status = !lock_sync2  ? 2'd2 :
                        !done_sync2  ? 2'd0 :
                        pass         ? 2'd1 : 2'd2;

    // =========================================================================
    // LED (clk domain)
    // =========================================================================
    reg [24:0] blink_ctr = 0;
    always @(posedge clk) blink_ctr <= blink_ctr + 1;

    always @(posedge clk) begin
        case (status)
            2'd0:    led <= blink_ctr[24];
            2'd2:    led <= (blink_ctr[24:22] != 3'b000);
            default: led <= (blink_ctr[24:22] == 3'b000);
        endcase
    end

    // =========================================================================
    // NTSC display (clk domain = 25 MHz)
    // =========================================================================
    wire ntsc_sync_w, ntsc_vid_w;

    ntsc_framebuf #(
        .FB_W    (320),
        .FB_H    (240),
        .H_SCALE (4)
    ) ntsc (
        .clk       (clk),
        .status    (status),
        .hash      (test_done_sys ? gold_reg : lfsr[31:0]),
        .hash2     (test_done_sys ? test_reg : lfsr[63:32]),
        .sync_pin  (ntsc_sync_w),
        .video_pin (ntsc_vid_w)
    );

    always @(posedge clk) begin
        ntsc_sync <= ntsc_sync_w;
        ntsc_vid  <= ntsc_vid_w;
    end

    // =========================================================================
    // OLED display (25 MHz clk domain, I2C)
    // =========================================================================
    // CDC: snapshot sys_clk registers into clk domain (decorative, not critical)
    // 128-bit wide: full LFSR state, DUT output + accumulators, entropy, control
    reg [127:0] oled_r0, oled_r1, oled_r2, oled_r3;
    reg [15:0] oled_gate;
    always @(posedge clk) begin
`ifdef DUT_MULTI_COMBO
        // Live 4×4 grid: 3 states per cell (blank/fail-text/pass-white).
        // combo_results = pass/fail, combo_tested = has been tested.
        oled_gate <= combo_tested;
        oled_r0 <= {{32{combo_results[ 3]}}, {32{combo_results[ 2]}},
                    {32{combo_results[ 1]}}, {32{combo_results[ 0]}}};
        oled_r1 <= {{32{combo_results[ 7]}}, {32{combo_results[ 6]}},
                    {32{combo_results[ 5]}}, {32{combo_results[ 4]}}};
        oled_r2 <= {{32{combo_results[11]}}, {32{combo_results[10]}},
                    {32{combo_results[ 9]}}, {32{combo_results[ 8]}}};
        oled_r3 <= {{32{combo_results[15]}}, {32{combo_results[14]}},
                    {32{combo_results[13]}}, {32{combo_results[12]}}};
`else
        oled_gate <= 16'hFFFF;  // no gating for non-combo DUTs
        oled_r0 <= {lfsr, lfsr2};                              // both LFSRs
        oled_r1 <= {mul_fold, accum, gold_reg, test_reg};      // DUT + test state
        oled_r2 <= {entropy, captured_seed};                   // entropy + seed
        oled_r3 <= {captured_seed2, 64'b0};                    // seed2 + spare
`endif
    end

`ifdef DUT_MULTI_COMBO
    ssd1306_oled #(.OVERLAY_FILE("build/oled_overlay.mem"), .ENABLE_OVERLAY(1)) oled (
`else
    ssd1306_oled oled (
`endif
        .clk(clk),
        .rst(~pll_lock),
        .reg0(oled_r0),
        .reg1(oled_r1),
        .reg2(oled_r2),
        .reg3(oled_r3),
        .overlay_gate(oled_gate),
        .scl(oled_scl),
        .sda(oled_sda)
    );

endmodule
