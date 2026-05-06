// top_extclk.v — external-clock silicon Fmax test for spirix DUTs (Stage 2)
//
// Self-test mode: counter runs on `clk` (25 MHz reference) so a freshly flashed
// bitstream displays "25.00 MHz" on the OLED. To switch to the RC oscillator
// for real testing, change `count_clk` source from `clk` to `ext_clk` (one
// line, marked SELF_TEST below).
//
// Architecture:
//   ext_clk pin (E16) → DCC global clock → DUT clock (PARALLEL=4 divide)
//   osc_drive pin (F15) = ~ext_clk (combinational, forms RC oscillator)
//
//   count_clk → 32-stage async ripple counter, 1-second window, snapshot
//   count_snap × 1717987 >> 34 = centi-MHz integer
//   Double-dabble bin→BCD (6 digits, max 9999.99)
//   Render glyphs to 8bpp framebuf, dither out via SH1106 I2C
//
// BRAM (256 × 128 bits) feeds inputs to DUT, sticky-fail on (output ≠ expected).
// Status: PASS / FAIL bitmap below the frequency line.
//
// Glyph ROM: 11 glyphs (0-9, .) at 16w × 16h × 8bpp = 256 bytes per glyph.
// Loaded from data/decimal_glyphs.mem at synth time.

module top_extclk #(
    parameter integer CLK_DIV = 7   // SSD1306 I2C clock divisor (25 MHz / 7 ≈ 3.57 MHz)
)(
    input  wire clk,            // 25 MHz reference oscillator (P6)
    input  wire ext_clk,        // RC oscillator input (E16)
    output wire osc_drive,      // FPGA inverter output → pot (F15)
    output reg  led,            // T6, active-low
    input  wire btn,            // R7, active-low (hold = reset all)
    output wire ntsc_sync,      // C4 — held low (CRT not used in this design)
    output wire ntsc_vid,       // D4
    output wire oled_scl,       // P2 — SH1106 OLED clock
    output wire oled_sda        // R2 — SH1106 OLED data
);

    // =========================================================================
    // Power-on reset (25 MHz domain)
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

    assign osc_drive = ~ext_clk;
    assign ntsc_sync = 1'b0;
    assign ntsc_vid  = 1'b0;

    // =========================================================================
    // SELF-TEST mode: BOTH the counter AND the DUT run on the 25 MHz reference,
    // so the OLED reads "25.00 MHz" and the DUT passes its vectors cleanly
    // before any RC pot is wired. To switch to RC oscillator testing, comment
    // out `SELF_TEST` and ext_clk drives both.
    // =========================================================================
`define SELF_TEST

`ifdef SELF_TEST
    wire count_clk = clk;
    wire dut_clk   = clk;
`else
    wire count_clk = ext_clk;
    wire dut_clk   = ext_clk;
`endif

    // =========================================================================
    // 32-stage free-running async ripple counter — never reset, never disabled.
    // Sampling is done by reading `count` on a snap pulse and differencing
    // against the previous sample. No settling/freeze period needed; any
    // metastability in mid-ripple bits costs at most ±1 LSB which is invisible
    // at our centi-MHz display precision.
    // =========================================================================
    reg [31:0] count;
    always @(posedge count_clk) count[0] <= ~count[0];
    genvar gi;
    generate
        for (gi = 1; gi < 32; gi = gi + 1) begin : ripple
            always @(negedge count[gi-1]) count[gi] <= ~count[gi];
        end
    endgenerate

    // =========================================================================
    // 60 Hz snap timer (25 MHz reference domain)
    //
    // SNAP_INTERVAL = 25_000_000 / 60 = 416,666 ref cycles. Every snap, we
    // capture `count` into count_snap, push prior count_snap into count_prev,
    // and `count_diff = count_snap - count_prev` is the count over one frame.
    //
    // Multiplier K is calibrated so 416,666 × K >> 32 = 25,000 milli-MHz, i.e.,
    // 25.000 MHz for a count_diff of 416,666 (= 25 MHz × 1/60 sec).
    //   K_milli = ceil(25,000 × 2^32 / 416,666) = 257,698,453
    // Result: count_diff × MULT_K >> 32 = 23-bit milli-MHz value (max ~9999.999 MHz).
    // =========================================================================
    localparam integer SNAP_INTERVAL = 25_000_000 / 60;  // = 416,666
    localparam [27:0]  MULT_K        = 28'd257_698_453;

    reg [19:0] snap_timer = 0;
    reg        snap_pulse_ref = 0;
    reg        snap_pulse_d = 0;     // delayed by 1 — count_diff is fresh on this pulse
    always @(posedge clk) begin
        snap_pulse_ref <= 1'b0;
        snap_pulse_d   <= snap_pulse_ref;
        if (global_reset) begin
            snap_timer <= 0;
        end else if (snap_timer == SNAP_INTERVAL - 1) begin
            snap_timer     <= 0;
            snap_pulse_ref <= 1'b1;
        end else begin
            snap_timer <= snap_timer + 1;
        end
    end

    // Capture count into count_snap on snap_pulse_ref. Push old count_snap
    // into count_prev. After this cycle, count_diff = count_snap - count_prev
    // is the count over the most recent 1/60 sec window.
    reg [31:0] count_snap = 0;
    reg [31:0] count_prev = 0;
    always @(posedge clk) begin
        if (snap_pulse_ref) begin
            count_prev <= count_snap;
            count_snap <= count;
        end
    end

    wire [31:0] count_diff = count_snap - count_prev;

    // Multiplier: count_diff × K >> 32 → milli-MHz (max ~9999.999 MHz, 23 bits)
    wire [59:0] product = count_diff * MULT_K;
    wire [22:0] milli_mhz = product[54:32];

    // snap_pulse_d fires one cycle AFTER snap_pulse_ref, by which time
    // count_snap and count_prev have been latched and count_diff is fresh.
    // That's the trigger for double-dabble.
    wire snap_pulse = snap_pulse_d;

    // =========================================================================
    // Double-dabble: 23-bit binary → 7 BCD digits (28 bits)
    // Sequential, runs once per snap_pulse. 23 cycles to complete.
    // =========================================================================
    reg [50:0] dd_reg;        // [50:23] = 7 BCD digits, [22:0] = binary working area
    reg [4:0]  dd_step;
    reg        dd_busy;
    reg [27:0] bcd_value;     // final 7 BCD digits

    // Standard double-dabble layout (51 bits = 28 BCD + 23 binary):
    //   dd_reg[22: 0]  = binary working area (milli_mhz, shifts out the top)
    //   dd_reg[26:23]  = BCD digit 0 (LSB)
    //   dd_reg[30:27]  = BCD digit 1
    //   dd_reg[34:31]  = BCD digit 2
    //   dd_reg[38:35]  = BCD digit 3
    //   dd_reg[42:39]  = BCD digit 4
    //   dd_reg[46:43]  = BCD digit 5
    //   dd_reg[50:47]  = BCD digit 6 (MSB)
    //
    // Each iteration: for any BCD nibble currently >=5, add 3; then shift left 1.
    // After 23 iterations: binary is gone, dd_reg[50:23] holds the 7 BCD digits.
    wire [50:0] dd_adjusted;
    assign dd_adjusted[22: 0] = dd_reg[22: 0];                                                            // binary unchanged
    assign dd_adjusted[26:23] = (dd_reg[26:23] >= 4'd5) ? dd_reg[26:23] + 4'd3 : dd_reg[26:23];           // BCD 0
    assign dd_adjusted[30:27] = (dd_reg[30:27] >= 4'd5) ? dd_reg[30:27] + 4'd3 : dd_reg[30:27];           // BCD 1
    assign dd_adjusted[34:31] = (dd_reg[34:31] >= 4'd5) ? dd_reg[34:31] + 4'd3 : dd_reg[34:31];           // BCD 2
    assign dd_adjusted[38:35] = (dd_reg[38:35] >= 4'd5) ? dd_reg[38:35] + 4'd3 : dd_reg[38:35];           // BCD 3
    assign dd_adjusted[42:39] = (dd_reg[42:39] >= 4'd5) ? dd_reg[42:39] + 4'd3 : dd_reg[42:39];           // BCD 4
    assign dd_adjusted[46:43] = (dd_reg[46:43] >= 4'd5) ? dd_reg[46:43] + 4'd3 : dd_reg[46:43];           // BCD 5
    assign dd_adjusted[50:47] = (dd_reg[50:47] >= 4'd5) ? dd_reg[50:47] + 4'd3 : dd_reg[50:47];           // BCD 6
    wire [50:0] dd_next = {dd_adjusted[49:0], 1'b0};

    always @(posedge clk) begin
        if (global_reset) begin
            dd_busy <= 0;
            bcd_value <= 0;
        end else if (snap_pulse) begin
            dd_reg <= {28'd0, milli_mhz};
            dd_step <= 0;
            dd_busy <= 1;
        end else if (dd_busy) begin
            dd_reg <= dd_next;
            dd_step <= dd_step + 1;
            if (dd_step == 5'd22) begin
                dd_busy <= 0;
                bcd_value <= dd_next[50:23];   // 7 BCD digits at the top of dd_reg
            end
        end
    end

    // =========================================================================
    // BRAM with packed test vectors. 256 × 128 bits.
    // =========================================================================
    reg [127:0] vec_rom [0:255];
    initial $readmemh("div_p4_vectors.mem", vec_rom);

    reg [7:0] vec_addr = 0;
    reg [127:0] vec_data;
    always @(posedge dut_clk) vec_data <= vec_rom[vec_addr];

    wire signed [23:0] dut_a_frac    = vec_data[127:104];
    wire signed  [7:0] dut_a_exp     = vec_data[103: 96];
    wire signed [23:0] dut_b_frac    = vec_data[ 95: 72];
    wire signed  [7:0] dut_b_exp     = vec_data[ 71: 64];
    wire signed [23:0] exp_q_frac    = vec_data[ 63: 40];
    wire signed  [7:0] exp_q_exp     = vec_data[ 39: 32];
    wire        [3:0]  exp_state     = vec_data[ 31: 28];

    // =========================================================================
    // DUT: spirix_divide PARALLEL=4 on dut_clk (= ext_clk)
    // =========================================================================
    wire signed [23:0] dut_r_frac;
    wire signed  [7:0] dut_r_exp;
    wire dut_busy_w, dut_done_w;

    reg iter_busy = 0;
    reg iter_start = 0;
    always @(posedge dut_clk) begin
        iter_start <= 1'b0;
        if (!iter_busy && !dut_busy_w) begin
            iter_start <= 1'b1;
            iter_busy <= 1'b1;
        end else if (dut_done_w) begin
            iter_busy <= 1'b0;
            vec_addr  <= vec_addr + 1'b1;
        end
    end

    spirix_divide #(.FRAC_BITS(24), .EXP_BITS(8), .PARALLEL(4)) dut (
        .clk(dut_clk), .start(iter_start),
        .a_frac(dut_a_frac), .a_exp(dut_a_exp),
        .b_frac(dut_b_frac), .b_exp(dut_b_exp),
        .result_frac(dut_r_frac), .result_exp(dut_r_exp),
        .busy(dut_busy_w), .done(dut_done_w)
    );

    // =========================================================================
    // Compare DUT output vs expected (ext_clk domain)
    // =========================================================================
    localparam signed [7:0] AMBIG_EXP_LOC = 8'h80;

    function [3:0] classify_state;
        input [23:0] storage; input signed [7:0] e;
        reg msb; integer i; integer count_lsbc;
        begin
            classify_state = 0;
            if (e !== AMBIG_EXP_LOC) classify_state = 0;
            else if (storage == 24'h000000) classify_state = 1;
            else if (&storage) classify_state = 6;
            else begin
                msb = storage[23];
                count_lsbc = 1;
                for (i = 22; i >= 0; i = i - 1) begin
                    if (storage[i] == msb) count_lsbc = count_lsbc + 1;
                    else i = -1;
                end
                if (count_lsbc == 1) classify_state = (msb == 1'b0) ? 4 : 5;
                else if (count_lsbc == 2) classify_state = (msb == 1'b0) ? 2 : 3;
                else classify_state = 7;
            end
        end
    endfunction

    wire [3:0] dut_state_w = classify_state(dut_r_frac, dut_r_exp);
    wire is_normal_expected = (exp_state == 4'd0);
    wire exact_match = (dut_r_frac == exp_q_frac) && (dut_r_exp == exp_q_exp);
    wire state_match =
        (exp_state == 4'd7 && dut_state_w == 4'd7 && dut_r_exp == AMBIG_EXP_LOC) ||
        ((exp_state == 4'd2 || exp_state == 4'd3) &&
         (dut_state_w == 4'd2 || dut_state_w == 4'd3) && dut_r_exp == AMBIG_EXP_LOC) ||
        ((exp_state == 4'd4 || exp_state == 4'd5 || exp_state == 4'd6) &&
         (dut_state_w == 4'd4 || dut_state_w == 4'd5 || dut_state_w == 4'd6) && dut_r_exp == AMBIG_EXP_LOC) ||
        ((exp_state == 4'd1) && (dut_state_w == 4'd1) && dut_r_exp == AMBIG_EXP_LOC);
    wire vector_pass = is_normal_expected ? exact_match : state_match;
    wire check_now = dut_done_w;

    reg fail_latched_dut = 0;
    always @(posedge dut_clk) begin
        if (check_now && !vector_pass) fail_latched_dut <= 1;
    end

    // Sync into 25 MHz domain
    reg [1:0] fail_sync = 0;
    always @(posedge clk) fail_sync <= {fail_sync[0], fail_latched_dut};
    wire fail_latched = fail_sync[1];

    // =========================================================================
    // Frame-coherent display registers
    // ----
    // All values that drive the OLED render are latched into "display_*"
    // copies on snap_pulse_ref (60 Hz). The renderer reads the display_*
    // copies, so the picture only changes at frame boundaries — no tearing
    // mid-frame, all metrics for one frame are visually consistent.
    //
    // Currently snapshotted: BCD digits, fail_latched.
    // Future: anything else added to the per-frame status (vector pass count,
    // current vector address, DUT busy %, etc.) goes here too.
    // =========================================================================
    reg [27:0] display_bcd  = 0;
    reg        display_fail = 0;
    always @(posedge clk) if (snap_pulse_ref) begin
        display_bcd  <= bcd_value;
        display_fail <= fail_latched;
    end

    // =========================================================================
    // Glyph ROM: 11 glyphs × 16×16 × 8bpp = 2816 bytes
    // Font rendered at 19.3 px in 16-wide tiles with content limited to 12 wide
    // (centered, 2 px blank padding each side). The renderer below slices off
    // the 2 px padding cols at display time, so cells appear 12 wide visually
    // without resizing the font.
    // =========================================================================
    reg [7:0] glyph_rom [0:2815];
    initial $readmemh("decimal_glyphs.mem", glyph_rom);

    // =========================================================================
    // Dither source: 32-bit Galois LFSR (deterministic, no combinational loops).
    // Synchronous, predictable, easy to debug. Updates every clk cycle.
    // =========================================================================
    reg [31:0] lfsr = 32'hCAFE_BABE;
    always @(posedge clk) begin
        // x^32 + x^22 + x^2 + x + 1 (maximal-length Galois LFSR)
        lfsr <= {lfsr[0], lfsr[31:1]} ^ ({32{lfsr[0]}} & 32'h80200003);
    end
    wire [7:0] trng_out = lfsr[7:0];

    // =========================================================================
    // Framebuffer: 8bpp, 128×64 = 8192 bytes
    // =========================================================================
    reg [7:0] fb [0:8191];

    // Layout (top half): 7-glyph frequency display "0000.00", 16x16 each =
    //                    7 × 16 = 112 px wide, centered with 8 px margin.
    //                    Rows 0-15.
    // Bottom half: blank for now (PASS/FAIL TODO).
    //
    // Glyph slot mapping for "ABCD.EF":
    //   col 0..15:   bcd[23:20] → "thousands"
    //   col 16..31:  bcd[19:16] → "hundreds"
    //   col 32..47:  bcd[15:12] → "tens"
    //   col 48..63:  bcd[11: 8] → "ones"
    //   col 64..79:  '.'
    //   col 80..95:  bcd[ 7: 4] → "tenths"
    //   col 96..111: bcd[ 3: 0] → "hundredths"
    //   col 112..127: blank
    //
    // FB write port: continuous 25 MHz scan over the framebuffer addressing
    // every byte. For each (col,row), look up which glyph slot covers that
    // (col, row) cell and read the corresponding glyph_rom byte.

    // Simple approach: walk the framebuffer write address linearly and
    // synthesize the pixel value from (col, row).
    reg [12:0] fb_wr_addr = 0;
    wire [6:0] wr_col = fb_wr_addr[6:0];
    wire [5:0] wr_row = fb_wr_addr[12:7];

    // Map (wr_col, wr_row) to a glyph slot and intra-glyph (gx, gy)
    // Top band: rows 0-15
    wire in_freq_band = (wr_row < 6'd16);
    wire in_status_band = (wr_row >= 6'd16) && (wr_row < 6'd32);

    // 12-wide cells: 8 cells × 12 = 96 px digit area, centered with 16 px
    // padding each side (cols 0-15 and 112-127 blank).
    localparam integer DIGIT_AREA_START = 16;
    localparam integer DIGIT_AREA_END   = 16 + 8 * 12;  // 112
    wire in_digit_area = (wr_col >= DIGIT_AREA_START) && (wr_col < DIGIT_AREA_END);
    wire [6:0] rel_col = wr_col - DIGIT_AREA_START[6:0];   // 0..95 within digit area

    // freq_pos = rel_col / 12, gx_in_cell = rel_col mod 12. Both via case.
    reg [2:0] freq_pos;
    reg [3:0] gx;
    always @(*) begin
        if      (rel_col < 7'd12) begin freq_pos = 3'd0; gx = rel_col[3:0]; end
        else if (rel_col < 7'd24) begin freq_pos = 3'd1; gx = (rel_col - 7'd12); end
        else if (rel_col < 7'd36) begin freq_pos = 3'd2; gx = (rel_col - 7'd24); end
        else if (rel_col < 7'd48) begin freq_pos = 3'd3; gx = (rel_col - 7'd36); end
        else if (rel_col < 7'd60) begin freq_pos = 3'd4; gx = (rel_col - 7'd48); end
        else if (rel_col < 7'd72) begin freq_pos = 3'd5; gx = (rel_col - 7'd60); end
        else if (rel_col < 7'd84) begin freq_pos = 3'd6; gx = (rel_col - 7'd72); end
        else                      begin freq_pos = 3'd7; gx = (rel_col - 7'd84); end
    end
    wire [3:0] gy = wr_row[3:0];

    // Leading-zero suppression: blank pos 0/1/2 when all higher digits are zero.
    // Pos 3 (units of MHz) always shown so even "0.000" displays the leading zero.
    wire suppress_p0 = (display_bcd[27:24] == 4'd0);
    wire suppress_p1 = suppress_p0 && (display_bcd[23:20] == 4'd0);
    wire suppress_p2 = suppress_p1 && (display_bcd[19:16] == 4'd0);

    // Resolve glyph slot for each freq position — reads frame-coherent display_bcd
    //   pos 0: bcd[27:24]  thousands of MHz   (blanked if leading zero)
    //   pos 1: bcd[23:20]  hundreds  of MHz   (blanked if leading zero)
    //   pos 2: bcd[19:16]  tens      of MHz   (blanked if leading zero)
    //   pos 3: bcd[15:12]  ones      of MHz   (always shown)
    //   pos 4: '.' (period glyph, slot 10)
    //   pos 5: bcd[11: 8]  tenths    of MHz
    //   pos 6: bcd[ 7: 4]  hundredths of MHz
    //   pos 7: bcd[ 3: 0]  thousandths of MHz
    reg [3:0] freq_slot;
    reg       freq_slot_valid;
    always @(*) begin
        case (freq_pos)
            3'd0: begin freq_slot = display_bcd[27:24]; freq_slot_valid = !suppress_p0; end
            3'd1: begin freq_slot = display_bcd[23:20]; freq_slot_valid = !suppress_p1; end
            3'd2: begin freq_slot = display_bcd[19:16]; freq_slot_valid = !suppress_p2; end
            3'd3: begin freq_slot = display_bcd[15:12]; freq_slot_valid = 1'b1;          end
            3'd4: begin freq_slot = 4'd10;              freq_slot_valid = 1'b1;          end // '.'
            3'd5: begin freq_slot = display_bcd[11: 8]; freq_slot_valid = 1'b1;          end
            3'd6: begin freq_slot = display_bcd[ 7: 4]; freq_slot_valid = 1'b1;          end
            3'd7: begin freq_slot = display_bcd[ 3: 0]; freq_slot_valid = 1'b1;          end
        endcase
    end

    // Glyph storage is 16×16 with content centered at cols 2-13 (2 px blank
    // padding each side). Cell width is 12, so we read cols 2-13 of the
    // stored glyph: glyph_gx = gx_in_cell + 2.
    //   addr = slot × 256 + gy × 16 + glyph_gx
    //        = {slot[3:0], gy[3:0], glyph_gx[3:0]}
    wire [3:0] glyph_gx = gx + 4'd2;            // gx in cell (0..11) → glyph col (2..13)
    wire [11:0] glyph_addr = {freq_slot, gy, glyph_gx};
    wire [7:0]  glyph_pixel_freq;
    assign glyph_pixel_freq = glyph_rom[glyph_addr];

    // Final pixel: glyph in freq band+digit area, status indicator in status band, blank elsewhere.
    wire [7:0] freq_pixel = (in_freq_band && in_digit_area && freq_slot_valid)
                          ? glyph_pixel_freq : 8'h00;

    // Status band: simple — fully white if pass, fully dark if fail, blink if no activity
    reg [23:0] heartbeat = 0;
    always @(posedge clk) heartbeat <= heartbeat + 1;
    wire heartbeat_on = heartbeat[23];
    wire [7:0] status_pixel =
        display_fail ? 8'h00 :
        heartbeat_on ? 8'hFF :
                       8'h40;  // dim grey when off-pulse

    wire [7:0] fb_wr_pixel =
        in_freq_band   ? freq_pixel :
        in_status_band ? status_pixel :
                         8'h00;

    always @(posedge clk) begin
        fb[fb_wr_addr] <= fb_wr_pixel;
        fb_wr_addr <= fb_wr_addr + 1;
    end

    // =========================================================================
    // CE for OLED FSM (slows the I2C state machine to ~3.57 MHz at CLK_DIV=7)
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
    // SH1106/SSD1306 init command sequence
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
    // OLED FSM — gather + dither for SH1106
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

    reg [7:0] fb_dout = 0;

    always @(posedge clk) if (ce) begin
        fb_dout <= fb[{page, gather_cnt[2:0], col}];
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
                                3'd0: begin i2c_data <= I2C_ADDR;             i2c_send_stop <= 0; end
                                3'd1: begin i2c_data <= CMD_PREFIX;            i2c_send_stop <= 0; end
                                3'd2: begin i2c_data <= 8'hB0 | {5'd0, page}; i2c_send_stop <= 0; end
                                3'd3: begin i2c_data <= 8'h02;                i2c_send_stop <= 0; end
                                3'd4: begin i2c_data <= 8'h10;                i2c_send_stop <= 1; end
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
                    // TRNG-dithered greyscale: per-pixel random threshold.
                    // Same pattern as pin scanner / calc — soft edges instead
                    // of hard binarization.
                    px_byte <= {(fb_dout > trng_out), px_byte[7:1]};
                end
                if (gather_cnt == 4'd8) begin
                    gather_cnt <= 0;
                    oled_state <= ST_SEND;
                end
            end
        endcase
    end

    // =========================================================================
    // LED status: solid on while passing, off on fail, slow blink if no DUT activity
    // =========================================================================
    reg [23:0] led_blink = 0;
    always @(posedge clk) begin
        led_blink <= led_blink + 1;
        if (global_reset)
            led <= 1'b1;
        else if (fail_latched)
            led <= 1'b1;          // OFF (active-low)
        else
            led <= 1'b0;          // ON
    end

endmodule
