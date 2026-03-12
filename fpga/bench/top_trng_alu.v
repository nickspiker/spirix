// TRNG ALU demo — button press generates random scalar,
// displays fraction/exponent on OLED with pi reference.
//
// OLED layout (128×64, 2×2 quadrants of 64×32 px):
//   top-left:  fraction    top-right: pi (fraction)
//   bot-left:  pi (frac)   bot-right: exponent

module top_trng_alu (
    input  wire clk,       // 25 MHz oscillator (P6)
    output reg  led,       // LED on T6 (active-low)
    input  wire btn,       // User button on R7 (active-low)
    output wire oled_scl,  // J1 B0 (E4)
    output wire oled_sda   // J1 R1 (D3)
);

    // =========================================================================
    // PLL
    // =========================================================================
    wire sys_clk, pll_lock;

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
        .CLKOP_DIV        (`PLL_CLKOP_DIV),
        .CLKOP_ENABLE     ("ENABLED"),
        .CLKOP_CPHASE     (`PLL_CLKOP_CPHASE),
        .FEEDBK_PATH      ("CLKOP")
    ) pll (
        .RST(1'b0), .STDBY(1'b0), .CLKI(clk), .CLKOP(sys_clk),
        .CLKFB(sys_clk), .LOCK(pll_lock),
        .PHASESEL0(1'b0), .PHASESEL1(1'b0),
        .PHASEDIR(1'b0), .PHASESTEP(1'b0), .PHASELOADREG(1'b0),
        .PLLWAKESYNC(1'b0), .ENCLKOP(1'b0)
    );
`else
    assign sys_clk  = clk;
    assign pll_lock = 1'b1;
`endif

    // =========================================================================
    // Button debounce + edge detect (sys_clk domain)
    // =========================================================================
    reg [2:0] btn_sync = 3'b111;
    reg [15:0] debounce_cnt = 0;
    reg btn_stable = 1;
    reg btn_prev = 1;
    wire btn_press = btn_prev & ~btn_stable;

    always @(posedge sys_clk) begin
        btn_sync <= {btn_sync[1:0], btn};
        if (btn_sync[2] != btn_stable) begin
            debounce_cnt <= debounce_cnt + 1;
            if (&debounce_cnt)
                btn_stable <= btn_sync[2];
        end else
            debounce_cnt <= 0;
        btn_prev <= btn_stable;
    end

    // =========================================================================
    // TRNG
    // =========================================================================
    wire signed [63:0] rng_frac, rng_exp;
    wire rng_busy, rng_done;
    wire [71:0] dbg_xor;
    reg  rng_start = 0;

    spirix_alu_random #(.MAX_FRAC(64), .MAX_EXP(64)) trng (
        .clk(sys_clk),
        .start(rng_start),
        .frac_width(2'b11),
        .exp_width(2'b11),
        .result_frac(rng_frac),
        .result_exp(rng_exp),
        .busy(rng_busy),
        .done(rng_done),
        .dbg_xor(dbg_xor)
    );

    // =========================================================================
    // Pi constant (F6E6: 64-bit fraction, exponent = 2)
    // 0x6487ED5110B4611A62633145C06E0E69 >> 64 = 0x6487ED5110B4611A
    // =========================================================================
    localparam [63:0] PI_FRAC = 64'h6487_ED51_10B4_611A;
    localparam [63:0] PI_EXP  = 64'h0000_0000_0000_0002;

    // =========================================================================
    // Capture result on done
    // =========================================================================
    reg [63:0] lat_frac = 0;
    reg [63:0] lat_exp  = 0;
    reg have_result = 0;

    always @(posedge sys_clk) begin
        rng_start <= 0;
        if (btn_press && !rng_busy)
            rng_start <= 1;
        if (rng_done) begin
            lat_frac    <= rng_frac;
            lat_exp     <= rng_exp;
            have_result <= 1;
        end
    end

    // =========================================================================
    // LED: off until first result, then solid on
    // =========================================================================
    always @(posedge sys_clk)
        led <= ~have_result;

    // =========================================================================
    // OLED layout — fractions LEFT column, exponents RIGHT column:
    //   TL: TRNG frac    TR: pi exp       (32px)
    //   BL: pi frac      BR: TRNG exp     (32px)
    // Fractions lined up vertically on left, exponents on right.
    // =========================================================================
    wire [127:0] top_row = {lat_frac, PI_EXP};
    wire [127:0] bot_row = {PI_FRAC,  lat_exp};

    ssd1306_oled #(.ENABLE_OVERLAY(0)) oled (
        .clk(clk),
        .rst(1'b0),
        .reg0(top_row),
        .reg1(top_row),
        .reg2(bot_row),
        .reg3(bot_row),
        .overlay_gate(16'b0),
        .scl(oled_scl),
        .sda(oled_sda)
    );

endmodule
