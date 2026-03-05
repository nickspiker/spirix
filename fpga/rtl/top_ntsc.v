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
    output reg  ntsc_vid   // J1 G0 (D4) — 220Ω
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

    // Protocol: 10-bit counter, bit taps for events (zero comparisons)
    //   [0..127]   warmup (LFSR runs, no accumulation)
    //   [128..511]  accumulate (384 cycles)
    //   bit 9 high  → done
    localparam PROTO_BITS = 10;

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
    localparam [1:0] PH_GOLD = 2'd0, PH_SWITCH = 2'd1,
                     PH_TEST = 2'd2, PH_DONE   = 2'd3;
    reg [1:0] phase = PH_GOLD;

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

    // =========================================================================
    // DUT — switchable via defines
    //
    // DUT_SPIRIX_FMA (default), DUT_HF_FMA, DUT_HF_MUL, DUT_HF_ADD
    // All combinational + registered output with CE. Output: 32-bit mul_fold.
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

    // =========================================================================
    // Protocol counter + accumulator + phase FSM
    // =========================================================================

    reg [PROTO_BITS-1:0] proto_cnt;
    wire       proto_done = proto_cnt[9];           // bit tap: done at 512
    wire       accumulating = proto_cnt[7] & ~proto_done;  // bit tap: accum from 128..511
    reg [31:0] accum;
    reg [31:0] gold_reg = 0, test_reg = 0;
    reg        test_done_sys = 0;

    always @(posedge sys_clk) begin
        if (!pll_lock || !por_done || btn_held_sys) begin
            // Global reset (PLL not locked OR button held)
            phase         <= PH_GOLD;
            lfsr          <= LFSR_SEED;
            lfsr2         <= LFSR_SEED2;
            proto_cnt      <= 0;
            accum          <= 0;
            gold_reg       <= 0;
            test_reg       <= 0;
            test_done_sys  <= 0;
        end else begin

            // =================================================================
            // Phase transitions (not CE-gated, run every sys_clk cycle)
            // =================================================================
            case (phase)
                PH_GOLD: begin
                    if (proto_done) begin
                        gold_reg <= accum;
                        phase    <= PH_SWITCH;
                    end
                end
                PH_SWITCH: begin
                    // Reset datapath for test phase
                    lfsr         <= LFSR_SEED;
                    lfsr2        <= LFSR_SEED2;
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
                PH_DONE: ;
            endcase

            // =================================================================
            // CE-gated datapath (only runs when ce=1)
            // =================================================================
            if (ce && phase != PH_SWITCH && phase != PH_DONE) begin
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
    // Button debounce (clk domain)
    // =========================================================================
    reg [1:0] btn_sync = 2'b11;
    reg [17:0] btn_deb = 0;
    reg btn_clean = 1, btn_prev = 1;
    wire btn_press = btn_prev & ~btn_clean;

    always @(posedge clk) begin
        btn_sync <= {btn_sync[0], btn};
        if (btn_sync[1] != btn_clean) begin
            btn_deb <= btn_deb + 1;
            if (&btn_deb) btn_clean <= btn_sync[1];
        end else
            btn_deb <= 0;
        btn_prev <= btn_clean;
    end

    // CDC: btn_clean → sys_clk domain (btn_clean=0 when pressed, active-low)
    reg [1:0] btn_sync_sys = 2'b11;
    always @(posedge sys_clk) btn_sync_sys <= {btn_sync_sys[0], btn_clean};
    wire btn_held_sys = ~btn_sync_sys[1];  // 1 when button is held down

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
    wire pass = (gold_reg == test_reg);
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
    wire btn_held = ~btn_clean;  // clk domain: 1 when button held

    ntsc_framebuf #(
        .FB_W    (320),
        .FB_H    (240),
        .H_SCALE (4)
    ) ntsc (
        .clk       (clk),
        .status    (status),
        .hash      (gold_reg),
        .hash2     (test_reg),
        .sync_pin  (ntsc_sync_w),
        .video_pin (ntsc_vid_w)
    );

    // Kill NTSC output when button held (both low = no signal)
    always @(posedge clk) begin
        ntsc_sync <= btn_held ? 1'b0 : ntsc_sync_w;
        ntsc_vid  <= btn_held ? 1'b0 : ntsc_vid_w;
    end

endmodule
