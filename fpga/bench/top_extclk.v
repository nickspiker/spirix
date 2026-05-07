// top_extclk.v — pure frequency counter for an external RC-oscillator clock.
//
// Pot wired between ext_clk (E16) and osc_drive (F15). FPGA inverts ext_clk
// onto osc_drive, forming an RC oscillator whose frequency is set by the pot.
//
// Each frame (60 Hz, 1/60 sec window):
//   1. SAMPLE         — counter accumulates ext_clk edges for SNAP_INTERVAL
//                       cycles of the 25 MHz reference clock.
//   2. DISCONNECT     — gate counter input off, wait a couple of cycles for
//                       any in-flight ripple to settle.
//   3. READ           — snapshot count into count_snap (which directly equals
//                       the count over the window since the counter was reset
//                       at the start of this window).
//   4. RESET          — async-reset the counter back to zero.
//   5. RECONNECT      — re-enable counter input, restart the window.
//
// count_snap × K >> 32 = milli-MHz integer (K calibrated so 25 MHz reads
// 25.000). Double-dabble produces 7 BCD digits → glyph render → OLED.
//
// Below the digits, a 32-cell × 16-row binary ruler shows count_snap with
// light-grey blocks for 1s and dark-grey blocks for 0s, MSB on the left.
//
// All display values are latched into display_* registers on snap_pulse so
// the OLED only changes at frame boundaries.
//
// Build with `-DPLL_TEST` to replace ext_clk with a 25→400 MHz PLL output
// (sanity-check that the counter matches a known reference). RC osc still
// runs but its output is ignored.

module top_extclk #(
    parameter integer CLK_DIV = 7   // SSD1306 I2C SCL divisor (25 MHz / 7 ≈ 3.57 MHz)
)(
    input  wire clk,            // 25 MHz reference oscillator (P6)
    input  wire ext_clk,        // RC oscillator input (E16)
    output wire osc_drive,      // FPGA inverter output → pot (F15)
    output reg  led,            // T6 — heartbeat (slow blink)
    input  wire btn,            // R7 — global reset (active-low)
    output wire ntsc_sync,      // C4 — held low (CRT not used)
    output wire ntsc_vid,       // D4
    output wire oled_scl,       // P2
    output wire oled_sda        // R2
);

    // =========================================================================
    // POR + button-driven global reset
    // =========================================================================
    reg [11:0] por_cnt = 0;
    reg por_done = 0;
    always @(posedge clk) begin
        if (!por_done) begin
            por_cnt <= por_cnt + 1;
            if (&por_cnt) por_done <= 1;
        end
    end

    reg [1:0] btn_sync = 2'b11;
    always @(posedge clk) btn_sync <= {btn_sync[0], btn};
    wire btn_held = ~btn_sync[1];
    wire global_reset = btn_held | ~por_done;

    // =========================================================================
    // RC oscillator: osc_drive = ~ext_clk_pin (combinational inverter).
    // External pot between the two pins forms the feedback delay.
    // =========================================================================
    assign osc_drive = ~ext_clk;
    assign ntsc_sync = 1'b0;
    assign ntsc_vid  = 1'b0;

`ifdef RING_TEST
    // 7-stage internal ring oscillator (pure fabric, no I/O round trip).
    // Each LUT4 with INIT=0x5555 is a NOT on input A. (* keep *) prevents
    // yosys from optimizing the combinational loop to a constant.
    // (* noglobal *) prevents nextpnr from promoting count_clk to a DCCA
    // global clock buffer, which would inject ~3-5 ns into the loop.
    (* noglobal *) wire [6:0] ring_q;
    genvar ri;
    generate
        for (ri = 0; ri < 7; ri = ri + 1) begin : ring_stages
            (* keep *) LUT4 #(.INIT(16'h5555)) inv (
                .A(ring_q[(ri == 0) ? 6 : ri - 1]),
                .B(1'b0), .C(1'b0), .D(1'b0),
                .Z(ring_q[ri])
            );
        end
    endgenerate
    (* noglobal *) wire count_clk = ring_q[0];
`elsif PLL_TEST
`ifndef PLL_FBDIV
  `define PLL_FBDIV 32
`endif
    // 25 MHz × PLL_FBDIV = VCO = CLKOP (CLKOP_DIV=1, CPHASE=0).
    // Datasheet VCO spec: 400-800 MHz (FBDIV 16-32). Beyond is silicon-lottery.
    wire pll_clk;
    wire pll_locked;
    (* FREQUENCY_PIN_CLKI="25" *)
    (* FREQUENCY_PIN_CLKOP="800" *)
    (* ICP_CURRENT="12" *) (* LPF_RESISTOR="8" *)
    (* MFG_ENABLE_FILTEROPAMP="1" *) (* MFG_GMCREF_SEL="2" *)
    EHXPLLL #(
        .PLLRST_ENA("DISABLED"),
        .INTFB_WAKE("DISABLED"),
        .STDBY_ENABLE("DISABLED"),
        .DPHASE_SOURCE("DISABLED"),
        .OUTDIVIDER_MUXA("DIVA"),
        .OUTDIVIDER_MUXB("DIVB"),
        .OUTDIVIDER_MUXC("DIVC"),
        .OUTDIVIDER_MUXD("DIVD"),
        .CLKI_DIV(1),
        .CLKOP_ENABLE("ENABLED"),
        .CLKOP_DIV(1),
        .CLKOP_CPHASE(0),
        .CLKOP_FPHASE(0),
        .FEEDBK_PATH("CLKOP"),
        .CLKFB_DIV(`PLL_FBDIV)
    ) pll_i (
        .RST(1'b0),
        .STDBY(1'b0),
        .CLKI(clk),
        .CLKOP(pll_clk),
        .CLKFB(pll_clk),
        .CLKINTFB(),
        .PHASESEL0(1'b0),
        .PHASESEL1(1'b0),
        .PHASEDIR(1'b1),
        .PHASESTEP(1'b1),
        .PHASELOADREG(1'b1),
        .PLLWAKESYNC(1'b0),
        .ENCLKOP(1'b0),
        .LOCK(pll_locked)
    );
    wire count_clk = pll_clk;
`else
    wire count_clk = ext_clk;
`endif

    // =========================================================================
    // Counter — 32-stage async ripple, gated by `counter_enable`,
    // async-resettable via `counter_reset`. Both signals are driven from the
    // 25 MHz reference domain and synchronized into count_clk before use.
    //
    // The buffer LUT on count_clk_buf isolates the counter chain's load from
    // the bare ext_clk pin, so any backwards-driven activity in the counter
    // logic doesn't perturb the RC oscillator loop.
    // =========================================================================
    reg counter_enable = 1;
    reg counter_reset  = 0;

    reg [1:0] ena_sync;
    reg [1:0] rst_sync;
    always @(posedge count_clk) begin
        ena_sync <= {ena_sync[0], counter_enable};
        rst_sync <= {rst_sync[0], counter_reset};
    end
    wire ena_ext = ena_sync[1];
    wire rst_ext = rst_sync[1];

    reg [31:0] count;
    always @(posedge count_clk or posedge rst_ext) begin
        if (rst_ext)        count[0] <= 1'b0;
        else if (ena_ext)   count[0] <= ~count[0];
    end
    genvar gi;
    generate
        for (gi = 1; gi < 32; gi = gi + 1) begin : ripple
            always @(negedge count[gi-1] or posedge rst_ext) begin
                if (rst_ext) count[gi] <= 1'b0;
                else         count[gi] <= ~count[gi];
            end
        end
    endgenerate

    // =========================================================================
    // 60 Hz windowed FSM
    //   state 0  SAMPLE       — counter accumulating, window_cnt → SNAP_INTERVAL
    //   state 1  DISCONNECT   — gate off, wait `settle` cycles for ripple to clear
    //   state 2  READ         — snapshot count → count_snap, fire snap_pulse
    //   state 3  RESET        — assert counter_reset, wait `settle` cycles
    //   state 4  RECONNECT    — drop reset, re-enable, restart window → state 0
    // =========================================================================
    localparam integer SNAP_INTERVAL = 25_000_000 / 60;  // = 416,666

    reg [19:0] window_cnt = 0;
    reg [3:0]  state      = 0;
    reg [3:0]  settle     = 0;
    reg [31:0] count_snap = 0;
    reg        snap_pulse = 0;

    always @(posedge clk) begin
        snap_pulse <= 1'b0;
        if (global_reset) begin
            state          <= 0;
            window_cnt     <= 0;
            settle         <= 0;
            counter_enable <= 1;
            counter_reset  <= 0;
        end else begin
            case (state)
                4'd0: begin // SAMPLE
                    window_cnt <= window_cnt + 1;
                    if (window_cnt == SNAP_INTERVAL - 1) begin
                        counter_enable <= 1'b0;
                        settle         <= 0;
                        state          <= 4'd1;
                    end
                end
                4'd1: begin // DISCONNECT (settle ripple)
                    settle <= settle + 1;
                    if (settle == 4'd4) state <= 4'd2;
                end
                4'd2: begin // READ
                    count_snap <= count;
                    snap_pulse <= 1'b1;
                    settle     <= 0;
                    state      <= 4'd3;
                end
                4'd3: begin // RESET counter
                    counter_reset <= 1'b1;
                    settle        <= settle + 1;
                    if (settle == 4'd4) state <= 4'd4;
                end
                4'd4: begin // RECONNECT
                    counter_reset  <= 1'b0;
                    counter_enable <= 1'b1;
                    window_cnt     <= 0;
                    state          <= 4'd0;
                end
            endcase
        end
    end

    // =========================================================================
    // Frequency math: count_snap × K >> 32 = milli-MHz integer.
    //   K = ceil(25,000 × 2^32 / 416,666) = 257,698,453
    //   For count_snap = 416,666 (= 25 MHz × 1/60 sec) → 25,000 milli-MHz
    // =========================================================================
    localparam [27:0] MULT_K = 28'd257_698_453;
    wire [59:0] product   = count_snap * MULT_K;
    wire [22:0] milli_mhz = product[54:32];

    // =========================================================================
    // Double-dabble: 23-bit milli_mhz → 7 BCD digits (28 bits)
    // 23 cycles to complete; kicks off one cycle after snap_pulse.
    // =========================================================================
    reg snap_pulse_d = 0;
    always @(posedge clk) snap_pulse_d <= snap_pulse;

    reg [50:0] dd_reg;
    reg [4:0]  dd_step;
    reg        dd_busy;
    reg [27:0] bcd_value = 0;

    wire [50:0] dd_adjusted;
    assign dd_adjusted[22: 0] = dd_reg[22: 0];
    assign dd_adjusted[26:23] = (dd_reg[26:23] >= 4'd5) ? dd_reg[26:23] + 4'd3 : dd_reg[26:23];
    assign dd_adjusted[30:27] = (dd_reg[30:27] >= 4'd5) ? dd_reg[30:27] + 4'd3 : dd_reg[30:27];
    assign dd_adjusted[34:31] = (dd_reg[34:31] >= 4'd5) ? dd_reg[34:31] + 4'd3 : dd_reg[34:31];
    assign dd_adjusted[38:35] = (dd_reg[38:35] >= 4'd5) ? dd_reg[38:35] + 4'd3 : dd_reg[38:35];
    assign dd_adjusted[42:39] = (dd_reg[42:39] >= 4'd5) ? dd_reg[42:39] + 4'd3 : dd_reg[42:39];
    assign dd_adjusted[46:43] = (dd_reg[46:43] >= 4'd5) ? dd_reg[46:43] + 4'd3 : dd_reg[46:43];
    assign dd_adjusted[50:47] = (dd_reg[50:47] >= 4'd5) ? dd_reg[50:47] + 4'd3 : dd_reg[50:47];
    wire [50:0] dd_next = {dd_adjusted[49:0], 1'b0};

    always @(posedge clk) begin
        if (global_reset) begin
            dd_busy   <= 0;
            bcd_value <= 0;
        end else if (snap_pulse_d) begin
            dd_reg  <= {28'd0, milli_mhz};
            dd_step <= 0;
            dd_busy <= 1;
        end else if (dd_busy) begin
            dd_reg  <= dd_next;
            dd_step <= dd_step + 1;
            if (dd_step == 5'd22) begin
                dd_busy   <= 0;
                bcd_value <= dd_next[50:23];
            end
        end
    end

    // =========================================================================
    // Frame-coherent display registers — latched on snap_pulse
    //   display_bcd   — 7 BCD digits to render at the top
    //   display_count — raw 32-bit count_snap to render as the binary ruler
    // =========================================================================
    reg [27:0] display_bcd   = 0;
    reg [31:0] display_count = 0;
    always @(posedge clk) if (snap_pulse) begin
        display_bcd   <= bcd_value;
        display_count <= count_snap;
    end

    // =========================================================================
    // Glyph ROM — 11 glyphs × 12 wide × 16 tall × 8bpp = 2112 bytes.
    // No padding: each tile is exactly the inked content. addr = slot*192 + gy*12 + gx.
    // =========================================================================
    reg [7:0] glyph_rom [0:2111];
    initial $readmemh("decimal_glyphs.mem", glyph_rom);

    // =========================================================================
    // LFSR dither (32-bit Galois, period 2^32-1)
    // =========================================================================
    reg [31:0] lfsr = 32'hCAFE_BABE;
    always @(posedge clk) begin
        lfsr <= {lfsr[0], lfsr[31:1]} ^ ({32{lfsr[0]}} & 32'h80200003);
    end
    wire [7:0] dither = lfsr[7:0];

    // =========================================================================
    // 8bpp framebuffer (128 × 64), continuous write at 25 MHz
    // =========================================================================
    reg [7:0]  fb [0:8191];
    reg [12:0] fb_wr_addr = 0;
    wire [6:0] wr_col = fb_wr_addr[6:0];
    wire [5:0] wr_row = fb_wr_addr[12:7];

    // Bands
    wire in_freq_band   = (wr_row < 6'd16);                                // rows 0..15
    wire in_binary_band = (wr_row >= 6'd16) && (wr_row < 6'd32);           // rows 16..31

`ifdef FONT_TEST
    // ----- Font test: 10 cells × 12 wide showing digits 0-9 at top -----
    localparam integer DIGIT_AREA_START = 4;
    localparam integer DIGIT_AREA_END   = 4 + 10 * 12;  // 124
    wire in_digit_area = (wr_col >= DIGIT_AREA_START[6:0]) && (wr_col < DIGIT_AREA_END[6:0]);
    wire [6:0] rel_col = wr_col - DIGIT_AREA_START[6:0];

    reg [3:0] freq_slot;
    reg [3:0] gx_in_cell;
    always @(*) begin
        if      (rel_col < 7'd12)  begin freq_slot = 4'd0; gx_in_cell = rel_col[3:0]; end
        else if (rel_col < 7'd24)  begin freq_slot = 4'd1; gx_in_cell = (rel_col - 7'd12); end
        else if (rel_col < 7'd36)  begin freq_slot = 4'd2; gx_in_cell = (rel_col - 7'd24); end
        else if (rel_col < 7'd48)  begin freq_slot = 4'd3; gx_in_cell = (rel_col - 7'd36); end
        else if (rel_col < 7'd60)  begin freq_slot = 4'd4; gx_in_cell = (rel_col - 7'd48); end
        else if (rel_col < 7'd72)  begin freq_slot = 4'd5; gx_in_cell = (rel_col - 7'd60); end
        else if (rel_col < 7'd84)  begin freq_slot = 4'd6; gx_in_cell = (rel_col - 7'd72); end
        else if (rel_col < 7'd96)  begin freq_slot = 4'd7; gx_in_cell = (rel_col - 7'd84); end
        else if (rel_col < 7'd108) begin freq_slot = 4'd8; gx_in_cell = (rel_col - 7'd96); end
        else                       begin freq_slot = 4'd9; gx_in_cell = (rel_col - 7'd108); end
    end
    wire [3:0] gy = wr_row[3:0];
    wire freq_slot_valid = 1'b1;
`else
    // ----- Frequency display: 8 cells × 12 wide centered at col 16 -----
    localparam integer DIGIT_AREA_START = 16;
    localparam integer DIGIT_AREA_END   = 16 + 8 * 12;  // 112
    wire in_digit_area = (wr_col >= DIGIT_AREA_START[6:0]) && (wr_col < DIGIT_AREA_END[6:0]);
    wire [6:0] rel_col = wr_col - DIGIT_AREA_START[6:0];

    reg [2:0] freq_pos;
    reg [3:0] gx_in_cell;
    always @(*) begin
        if      (rel_col < 7'd12) begin freq_pos = 3'd0; gx_in_cell = rel_col[3:0]; end
        else if (rel_col < 7'd24) begin freq_pos = 3'd1; gx_in_cell = (rel_col - 7'd12); end
        else if (rel_col < 7'd36) begin freq_pos = 3'd2; gx_in_cell = (rel_col - 7'd24); end
        else if (rel_col < 7'd48) begin freq_pos = 3'd3; gx_in_cell = (rel_col - 7'd36); end
        else if (rel_col < 7'd60) begin freq_pos = 3'd4; gx_in_cell = (rel_col - 7'd48); end
        else if (rel_col < 7'd72) begin freq_pos = 3'd5; gx_in_cell = (rel_col - 7'd60); end
        else if (rel_col < 7'd84) begin freq_pos = 3'd6; gx_in_cell = (rel_col - 7'd72); end
        else                      begin freq_pos = 3'd7; gx_in_cell = (rel_col - 7'd84); end
    end
    wire [3:0] gy = wr_row[3:0];

    // Leading-zero suppression (positions 0/1/2)
    wire suppress_p0 = (display_bcd[27:24] == 4'd0);
    wire suppress_p1 = suppress_p0 && (display_bcd[23:20] == 4'd0);
    wire suppress_p2 = suppress_p1 && (display_bcd[19:16] == 4'd0);

    reg [3:0] freq_slot;
    reg       freq_slot_valid;
    always @(*) begin
        case (freq_pos)
            3'd0: begin freq_slot = display_bcd[27:24]; freq_slot_valid = !suppress_p0; end
            3'd1: begin freq_slot = display_bcd[23:20]; freq_slot_valid = !suppress_p1; end
            3'd2: begin freq_slot = display_bcd[19:16]; freq_slot_valid = !suppress_p2; end
            3'd3: begin freq_slot = display_bcd[15:12]; freq_slot_valid = 1'b1; end
            3'd4: begin freq_slot = 4'd10;              freq_slot_valid = 1'b1; end // '.'
            3'd5: begin freq_slot = display_bcd[11: 8]; freq_slot_valid = 1'b1; end
            3'd6: begin freq_slot = display_bcd[ 7: 4]; freq_slot_valid = 1'b1; end
            3'd7: begin freq_slot = display_bcd[ 3: 0]; freq_slot_valid = 1'b1; end
        endcase
    end
`endif

    // 12-wide tiles. addr = slot*192 + gy*12 + gx_in_cell (gx_in_cell is 0..11).
    // gy*12 = gy*8 + gy*4 (no multiplier needed).
    reg [11:0] slot_offset;
    always @(*) begin
        case (freq_slot)
            4'd0:  slot_offset = 12'd0;
            4'd1:  slot_offset = 12'd192;
            4'd2:  slot_offset = 12'd384;
            4'd3:  slot_offset = 12'd576;
            4'd4:  slot_offset = 12'd768;
            4'd5:  slot_offset = 12'd960;
            4'd6:  slot_offset = 12'd1152;
            4'd7:  slot_offset = 12'd1344;
            4'd8:  slot_offset = 12'd1536;
            4'd9:  slot_offset = 12'd1728;
            4'd10: slot_offset = 12'd1920;
            default: slot_offset = 12'd0;
        endcase
    end
    wire [7:0]  gy_x12     = ({4'd0, gy} << 3) + ({4'd0, gy} << 2);
    wire [11:0] glyph_addr = slot_offset + {4'd0, gy_x12} + {8'd0, gx_in_cell};
    wire [7:0]  glyph_pixel = glyph_rom[glyph_addr];
    wire [7:0]  freq_pixel  = (in_freq_band && in_digit_area && freq_slot_valid)
                              ? glyph_pixel : 8'h00;

    // ----- Binary ruler: 32 cells × 4 px wide × 16 rows tall -----
    // Cell N (col 4N..4N+3) shows display_count[31 - N]. MSB on the left.
    // Light grey (0xC0) for 1, dark grey (0x40) for 0.
    wire [4:0] bit_idx       = wr_col[6:2];                  // 0..31
    wire       bin_bit_value = display_count[5'd31 - bit_idx];
    wire [7:0] binary_pixel  = bin_bit_value ? 8'hC0 : 8'h40;

    wire [7:0] fb_wr_pixel =
        in_freq_band   ? freq_pixel   :
        in_binary_band ? binary_pixel :
                         8'h00;

    always @(posedge clk) begin
        fb[fb_wr_addr] <= fb_wr_pixel;
        fb_wr_addr     <= fb_wr_addr + 1;
    end

    // =========================================================================
    // OLED I2C clock-enable (one ce pulse per CLK_DIV ref cycles)
    // =========================================================================
    reg [$clog2(CLK_DIV)-1:0] cnt = 0;
    wire at_top = (cnt == CLK_DIV - 1);
    always @(posedge clk) cnt <= at_top ? 0 : cnt + 1;
    reg ce = 0;
    always @(posedge clk) ce <= at_top;

    // =========================================================================
    // I2C driver
    // =========================================================================
    reg  [7:0] i2c_data;
    reg        i2c_start;
    reg        i2c_send_stop;
    wire       i2c_busy;

    ssd1306_i2c #(.CLK_DIV(CLK_DIV)) i2c (
        .clk(clk), .rst(1'b0),
        .data(i2c_data),
        .start(i2c_start),
        .send_start(1'b0),
        .send_stop(i2c_send_stop),
        .busy(i2c_busy),
        .scl(oled_scl), .sda(oled_sda)
    );

    // =========================================================================
    // SSD1306/SH1106 init
    // =========================================================================
    localparam I2C_ADDR    = 8'h78;
    localparam CMD_PREFIX  = 8'h00;
    localparam DATA_PREFIX = 8'h40;
    localparam INIT_LEN    = 25;

    reg [7:0] init_cmds [0:INIT_LEN-1];
    initial begin
        init_cmds[ 0] = 8'hAE; init_cmds[ 1] = 8'hD5;
        init_cmds[ 2] = 8'h80; init_cmds[ 3] = 8'hA8;
        init_cmds[ 4] = 8'h3F; init_cmds[ 5] = 8'hD3;
        init_cmds[ 6] = 8'h00; init_cmds[ 7] = 8'h40;
        init_cmds[ 8] = 8'h8D; init_cmds[ 9] = 8'h14;
        init_cmds[10] = 8'hAD; init_cmds[11] = 8'h8B;
        init_cmds[12] = 8'hA1; init_cmds[13] = 8'hC8;
        init_cmds[14] = 8'hDA; init_cmds[15] = 8'h12;
        init_cmds[16] = 8'h81; init_cmds[17] = 8'hCF;
        init_cmds[18] = 8'hD9; init_cmds[19] = 8'hF1;
        init_cmds[20] = 8'hDB; init_cmds[21] = 8'h40;
        init_cmds[22] = 8'hA4; init_cmds[23] = 8'hA6;
        init_cmds[24] = 8'hAF;
    end

    // =========================================================================
    // OLED FSM — gather + dither
    // =========================================================================
    localparam [2:0]
        ST_RESET   = 3'd0,
        ST_SEND    = 3'd1,
        ST_WAIT    = 3'd2,
        ST_NEXT    = 3'd3,
        ST_BUSFREE = 3'd4,
        ST_GATHER  = 3'd5;

    localparam [1:0]
        PH_INIT      = 2'd0,
        PH_PAGE_CMD  = 2'd1,
        PH_PAGE_DATA = 2'd2;

    reg [2:0]  oled_state   = ST_RESET;
    reg [1:0]  phase        = PH_INIT;
    reg [19:0] reset_cnt    = 0;
    reg [9:0]  busfree_cnt  = 0;
    reg [4:0]  cmd_idx      = 0;
    reg [2:0]  page         = 0;
    reg [6:0]  col          = 0;
    reg [7:0]  px_byte      = 0;
    reg [3:0]  gather_cnt   = 0;
    reg [7:0]  fb_dout      = 0;

    always @(posedge clk) if (ce) begin
        fb_dout   <= fb[{page, gather_cnt[2:0], col}];
        i2c_start <= 0;

        case (oled_state)
            ST_RESET: begin
                reset_cnt <= reset_cnt + 1;
                if (&reset_cnt) begin
                    phase   <= PH_INIT;
                    cmd_idx <= 0;
                    page    <= 0;
                    col     <= 0;
                    oled_state <= ST_SEND;
                end
            end
            ST_SEND: begin
                if (!i2c_busy) begin
                    case (phase)
                        PH_INIT: begin
                            if (cmd_idx == 0) begin
                                i2c_data      <= I2C_ADDR;
                                i2c_send_stop <= 0;
                            end else if (cmd_idx == 1) begin
                                i2c_data      <= CMD_PREFIX;
                                i2c_send_stop <= 0;
                            end else begin
                                i2c_data      <= init_cmds[cmd_idx - 2];
                                i2c_send_stop <= (cmd_idx == INIT_LEN + 1);
                            end
                        end
                        PH_PAGE_CMD: begin
                            case (cmd_idx[2:0])
                                3'd0: begin i2c_data <= I2C_ADDR;            i2c_send_stop <= 0; end
                                3'd1: begin i2c_data <= CMD_PREFIX;          i2c_send_stop <= 0; end
                                3'd2: begin i2c_data <= 8'hB0 | {5'd0, page}; i2c_send_stop <= 0; end
                                3'd3: begin i2c_data <= 8'h02;               i2c_send_stop <= 0; end
                                3'd4: begin i2c_data <= 8'h10;               i2c_send_stop <= 1; end
                                default: ;
                            endcase
                        end
                        PH_PAGE_DATA: begin
                            if (cmd_idx == 0) begin
                                i2c_data      <= I2C_ADDR;
                                i2c_send_stop <= 0;
                            end else if (cmd_idx == 1) begin
                                i2c_data      <= DATA_PREFIX;
                                i2c_send_stop <= 0;
                            end else begin
                                i2c_data      <= px_byte;
                                i2c_send_stop <= (col == 7'd127);
                            end
                        end
                    endcase
                    i2c_start  <= 1;
                    oled_state <= ST_WAIT;
                end
            end
            ST_WAIT: begin
                if (i2c_busy) oled_state <= ST_NEXT;
            end
            ST_NEXT: begin
                if (!i2c_busy) begin
                    case (phase)
                        PH_INIT: begin
                            if (cmd_idx == INIT_LEN + 1) begin
                                phase   <= PH_PAGE_CMD;
                                cmd_idx <= 0;
                                page    <= 0;
                                oled_state <= ST_BUSFREE;
                            end else begin
                                cmd_idx    <= cmd_idx + 1;
                                oled_state <= ST_SEND;
                            end
                        end
                        PH_PAGE_CMD: begin
                            if (cmd_idx == 4) begin
                                phase      <= PH_PAGE_DATA;
                                cmd_idx    <= 0;
                                col        <= 0;
                                gather_cnt <= 0;
                                oled_state <= ST_BUSFREE;
                            end else begin
                                cmd_idx    <= cmd_idx + 1;
                                oled_state <= ST_SEND;
                            end
                        end
                        PH_PAGE_DATA: begin
                            if (cmd_idx < 2) begin
                                cmd_idx <= cmd_idx + 1;
                                if (cmd_idx == 1) begin
                                    gather_cnt <= 0;
                                    oled_state <= ST_GATHER;
                                end else begin
                                    oled_state <= ST_SEND;
                                end
                            end else if (col == 7'd127) begin
                                phase   <= PH_PAGE_CMD;
                                cmd_idx <= 0;
                                page    <= (page == 3'd7) ? 3'd0 : page + 1;
                                oled_state <= ST_BUSFREE;
                            end else begin
                                col        <= col + 1;
                                gather_cnt <= 0;
                                oled_state <= ST_GATHER;
                            end
                        end
                    endcase
                end
            end
            ST_BUSFREE: begin
                busfree_cnt <= busfree_cnt + 1;
                if (&busfree_cnt) begin
                    busfree_cnt <= 0;
                    oled_state <= ST_SEND;
                end
            end
            ST_GATHER: begin
                gather_cnt <= gather_cnt + 1;
                if (gather_cnt >= 4'd1) begin
                    px_byte <= {(fb_dout > dither), px_byte[7:1]};
                end
                if (gather_cnt == 4'd8) begin
                    gather_cnt <= 0;
                    oled_state <= ST_SEND;
                end
            end
        endcase
    end

    // =========================================================================
    // LED — solid ON (active-low). Eliminates power-rail perturbation that a
    // blinking LED was causing on the RC oscillator (~0.15% frequency wiggle).
    // The OLED already indicates the bitstream is alive.
    // =========================================================================
    always @(posedge clk) led <= 1'b0;

endmodule
