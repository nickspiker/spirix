// Ring oscillator TRNG array — 128×64 gated LUT4 inverter loops on OLED
//
// Protocol per frame:
//   1. Gate all 8192 ROs to 0 (254 sys_clk cycles)
//   2. Release gate for 1 sys_clk cycle (ROs oscillate from 0)
//   3. Capture all 8192 outputs simultaneously into FFs
//   4. Shift-copy to framebuffer EBR (1024 cycles, 8 bits/cycle)
//   5. Repeat
//
// OLED (128×64 SH1106, I2C) displays each RO as one pixel.
// Low PLL freq: noise (ROs desync from jitter).
// High PLL freq: all black (ROs can't toggle in 1 clock).

module top_trng (
    input  wire clk,       // 25 MHz oscillator (P6)
    output reg  led,       // LED on T6 (active-low)
    input  wire btn,       // User button on R7 (active-low)
    output reg  ntsc_sync, // J1 R0 (C4) — unused
    output reg  ntsc_vid,  // J1 G0 (D4) — unused
    output wire oled_scl,  // J1 B0 (E4) — I2C SCL
    output wire oled_sda   // J1 R1 (D3) — I2C SDA
);

    // Unused NTSC outputs
    always @(posedge clk) begin ntsc_sync <= 0; ntsc_vid <= 0; end

    // =========================================================================
    // PLL: variable-frequency system clock
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

    // =========================================================================
    // Button sync
    // =========================================================================
    reg [1:0] btn_sync = 2'b11;
    always @(posedge sys_clk) btn_sync <= {btn_sync[0], btn};
    wire btn_held = ~btn_sync[1];

    // =========================================================================
    // Ring Oscillator Array — 8192 gated LUT4 inverter loops
    // =========================================================================
    // Each RO: LUT4 as gated inverter. Z = gate ? ~A : 0.
    // Output Z fed back to input A. Gate=0 forces output to 0.
    // Gate=1: free-running oscillation (~1-2 GHz on ECP5).
    //
    // Organized in OLED byte order so shift-copy writes sequential EBR bytes:
    //   samp index = page*1024 + col*8 + bit_within_page
    //   page = row/8, bit_within_page = row%8
    //   Byte i = samp[i*8+7 : i*8] → EBR address i → OLED page i/128, col i%128

    reg ro_gate = 0;
    wire [8191:0] ro_out;

    genvar gi;
    generate
        for (gi = 0; gi < 8192; gi = gi + 1) begin : ro
            // LUT4 truth table: Z = INIT[{D,C,B,A}]
            // D=gate, A=feedback. B=C=0.
            // D=0: Z=0 (held low). D=1: Z=~A (inverter).
            // INIT = 16'h0100
            (* keep, syn_keep="true" *)
            wire osc;
            (* keep *)
            LUT4 #(.INIT(16'h0100)) ro_lut (
                .A(osc), .B(1'b0), .C(1'b0), .D(ro_gate), .Z(osc)
            );
            assign ro_out[gi] = osc;
        end
    endgenerate

    // =========================================================================
    // Control FSM + shift register → EBR copy
    // =========================================================================
    localparam [1:0] S_GATE = 2'd0, S_ENABLE = 2'd1, S_COPY = 2'd2;
    reg [1:0]    state = S_GATE;
    reg [25:0]   gate_cnt = 0;  // ~0.13s at 500 MHz (2^25 = 33.5M clocks, ~7.5 Hz)
    reg [9:0]    copy_cnt = 0;
    reg [8191:0] samp = 0;
    reg          fb_wr = 0;

    // Framebuffer RAM: 1024 × 8 (inferred as DP16KD)
    // Write port: sys_clk (shift-copy). Read port: clk (OLED).
    reg [7:0] fb_ram [0:1023];

    // Write port — 1-cycle pipeline: fb_wr/copy_cnt/samp read as previous-cycle values
    always @(posedge sys_clk)
        if (fb_wr) fb_ram[copy_cnt] <= samp[7:0];

    always @(posedge sys_clk) begin
        fb_wr <= 0;

        if (btn_held || !pll_lock) begin
            state    <= S_GATE;
            gate_cnt <= 0;
            ro_gate  <= 0;
        end else begin
            case (state)
                S_GATE: begin
                    ro_gate <= 0;
                    if (gate_cnt[25]) begin  // bit tap: ~33.5M clocks = ~0.13s @ 500 MHz (~8 Hz)
                        ro_gate  <= 1;       // takes effect next clock
                        gate_cnt <= 0;
                        state    <= S_ENABLE;
                    end else
                        gate_cnt <= gate_cnt + 1;
                end

                S_ENABLE: begin
                    // ROs oscillated for 1 sys_clk period. Capture now.
                    // ro_gate was 1 during this clock period; goes low next cycle.
                    // samp <= ro_out captures instantaneous RO state before gate drops.
                    ro_gate  <= 0;
                    samp     <= ro_out;
                    copy_cnt <= 0;
                    state    <= S_COPY;
                end

                S_COPY: begin
                    // Write samp[7:0] to fb_ram[copy_cnt] (via 1-cycle pipeline)
                    // then shift right by 8 for next byte.
                    fb_wr <= 1;
                    samp  <= {8'b0, samp[8191:8]};
                    if (copy_cnt == 10'd1023)
                        state <= S_GATE;
                    else
                        copy_cnt <= copy_cnt + 1;
                end

                default: state <= S_GATE;
            endcase
        end
    end

    // =========================================================================
    // OLED display — 128×64 bitmap from EBR (25 MHz clk domain)
    // =========================================================================

    // Read port of framebuffer
    reg [7:0] fb_rd;
    reg [2:0] ol_page;
    reg [6:0] ol_col;
    always @(posedge clk) fb_rd <= fb_ram[{ol_page, ol_col}];

    // I2C driver (reuse existing ssd1306_i2c module)
    reg  [7:0] i2c_data;
    reg        i2c_start;
    reg        i2c_send_start;
    reg        i2c_send_stop;
    wire       i2c_busy;

    ssd1306_i2c #(.CLK_DIV(64)) i2c (
        .clk(clk), .rst(~pll_lock),
        .data(i2c_data),
        .start(i2c_start),
        .send_start(i2c_send_start),
        .send_stop(i2c_send_stop),
        .busy(i2c_busy),
        .scl(oled_scl), .sda(oled_sda)
    );

    // Init command table
    localparam INIT_LEN = 25;
    reg [7:0] init_cmds [0:INIT_LEN-1];
    initial begin
        init_cmds[ 0] = 8'hAE;  // display off
        init_cmds[ 1] = 8'hD5;  // clock divide
        init_cmds[ 2] = 8'h80;
        init_cmds[ 3] = 8'hA8;  // multiplex
        init_cmds[ 4] = 8'h3F;  // 64-1
        init_cmds[ 5] = 8'hD3;  // display offset
        init_cmds[ 6] = 8'h00;
        init_cmds[ 7] = 8'h40;  // start line 0
        init_cmds[ 8] = 8'h8D;  // charge pump (SSD1306)
        init_cmds[ 9] = 8'h14;
        init_cmds[10] = 8'hAD;  // DC-DC (SH1106)
        init_cmds[11] = 8'h8B;
        init_cmds[12] = 8'hA1;  // segment remap
        init_cmds[13] = 8'hC8;  // COM scan descending
        init_cmds[14] = 8'hDA;  // COM pins
        init_cmds[15] = 8'h12;
        init_cmds[16] = 8'h81;  // contrast
        init_cmds[17] = 8'hCF;
        init_cmds[18] = 8'hD9;  // pre-charge
        init_cmds[19] = 8'hF1;
        init_cmds[20] = 8'hDB;  // VCOMH
        init_cmds[21] = 8'h40;
        init_cmds[22] = 8'hA4;  // display from RAM
        init_cmds[23] = 8'hA6;  // normal
        init_cmds[24] = 8'hAF;  // display on
    end

    // OLED FSM
    localparam [2:0]
        OL_RESET   = 3'd0,
        OL_SEND    = 3'd1,
        OL_WAIT    = 3'd2,
        OL_NEXT    = 3'd3,
        OL_BUSFREE = 3'd4;

    reg [2:0]  ol_state = OL_RESET;
    reg [19:0] ol_reset_cnt = 0;
    reg [9:0]  ol_busfree_cnt = 0;

    reg [1:0]  ol_phase;
    localparam [1:0] PH_INIT = 2'd0, PH_PAGE_CMD = 2'd1, PH_PAGE_DATA = 2'd2;

    reg [4:0]  ol_cmd_idx;

    localparam I2C_ADDR    = 8'h78;
    localparam CMD_PREFIX  = 8'h00;
    localparam DATA_PREFIX = 8'h40;

    always @(posedge clk) begin
        if (~pll_lock) begin
            ol_state     <= OL_RESET;
            ol_reset_cnt <= 0;
            i2c_start    <= 0;
        end else begin
            i2c_start <= 0;

            case (ol_state)
                OL_RESET: begin
                    ol_reset_cnt <= ol_reset_cnt + 1;
                    if (&ol_reset_cnt) begin
                        ol_phase   <= PH_INIT;
                        ol_cmd_idx <= 0;
                        ol_page    <= 0;
                        ol_col     <= 0;
                        ol_state   <= OL_SEND;
                    end
                end

                OL_SEND: begin
                    if (!i2c_busy) begin
                        case (ol_phase)
                            PH_INIT: begin
                                if (ol_cmd_idx == 0) begin
                                    i2c_data <= I2C_ADDR; i2c_send_start <= 1; i2c_send_stop <= 0;
                                end else if (ol_cmd_idx == 1) begin
                                    i2c_data <= CMD_PREFIX; i2c_send_start <= 0; i2c_send_stop <= 0;
                                end else begin
                                    i2c_data <= init_cmds[ol_cmd_idx - 2];
                                    i2c_send_start <= 0;
                                    i2c_send_stop  <= (ol_cmd_idx == INIT_LEN + 1);
                                end
                            end

                            PH_PAGE_CMD: begin
                                case (ol_cmd_idx[2:0])
                                    3'd0: begin i2c_data <= I2C_ADDR;                    i2c_send_start <= 1; i2c_send_stop <= 0; end
                                    3'd1: begin i2c_data <= CMD_PREFIX;                  i2c_send_start <= 0; i2c_send_stop <= 0; end
                                    3'd2: begin i2c_data <= 8'hB0 | {5'b0, ol_page};    i2c_send_start <= 0; i2c_send_stop <= 0; end
                                    3'd3: begin i2c_data <= 8'h02;                       i2c_send_start <= 0; i2c_send_stop <= 0; end
                                    3'd4: begin i2c_data <= 8'h10;                       i2c_send_start <= 0; i2c_send_stop <= 1; end
                                    default: ;
                                endcase
                            end

                            PH_PAGE_DATA: begin
                                if (ol_cmd_idx == 0) begin
                                    i2c_data <= I2C_ADDR; i2c_send_start <= 1; i2c_send_stop <= 0;
                                end else if (ol_cmd_idx == 1) begin
                                    i2c_data <= DATA_PREFIX; i2c_send_start <= 0; i2c_send_stop <= 0;
                                end else begin
                                    i2c_data <= fb_rd;
                                    i2c_send_start <= 0;
                                    i2c_send_stop  <= (ol_col == 127);
                                end
                            end
                        endcase

                        i2c_start <= 1;
                        ol_state  <= OL_WAIT;
                    end
                end

                OL_WAIT: begin
                    if (i2c_busy) ol_state <= OL_NEXT;
                end

                OL_NEXT: begin
                    if (!i2c_busy) begin
                        case (ol_phase)
                            PH_INIT: begin
                                if (ol_cmd_idx == INIT_LEN + 1) begin
                                    ol_phase   <= PH_PAGE_CMD;
                                    ol_cmd_idx <= 0;
                                    ol_page    <= 0;
                                    ol_state   <= OL_BUSFREE;
                                end else begin
                                    ol_cmd_idx <= ol_cmd_idx + 1;
                                    ol_state   <= OL_SEND;
                                end
                            end

                            PH_PAGE_CMD: begin
                                if (ol_cmd_idx == 4) begin
                                    ol_phase   <= PH_PAGE_DATA;
                                    ol_cmd_idx <= 0;
                                    ol_col     <= 0;
                                    ol_state   <= OL_BUSFREE;
                                end else begin
                                    ol_cmd_idx <= ol_cmd_idx + 1;
                                    ol_state   <= OL_SEND;
                                end
                            end

                            PH_PAGE_DATA: begin
                                if (ol_cmd_idx < 2) begin
                                    ol_cmd_idx <= ol_cmd_idx + 1;
                                    ol_state   <= OL_SEND;
                                end else if (ol_col == 127) begin
                                    if (ol_page == 7) begin
                                        ol_phase   <= PH_PAGE_CMD;
                                        ol_cmd_idx <= 0;
                                        ol_page    <= 0;
                                    end else begin
                                        ol_phase   <= PH_PAGE_CMD;
                                        ol_cmd_idx <= 0;
                                        ol_page    <= ol_page + 1;
                                    end
                                    ol_state <= OL_BUSFREE;
                                end else begin
                                    ol_col   <= ol_col + 1;
                                    ol_state <= OL_SEND;
                                end
                            end
                        endcase
                    end
                end

                OL_BUSFREE: begin
                    ol_busfree_cnt <= ol_busfree_cnt + 1;
                    if (&ol_busfree_cnt) ol_state <= OL_SEND;
                end
            endcase
        end
    end

    // =========================================================================
    // LED: blink ~0.75 Hz to show system alive
    // =========================================================================
    reg [24:0] led_cnt = 0;
    always @(posedge clk) begin
        led_cnt <= led_cnt + 1;
        led     <= led_cnt[24];
    end

endmodule
