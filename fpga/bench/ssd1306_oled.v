// SH1106 128x64 OLED controller (I2C, write-only)
//
// Displays 4 × 128-bit register values as horizontal bit bars.
// Each register gets 2 pages (16 pixels tall), 1 column per bit (MSB left).
// Pages 0-1: reg0, Pages 2-3: reg1, Pages 4-5: reg2, Pages 6-7: reg3
// Full 128×64 display utilization.
//
// SH1106 notes:
//   - Page addressing only (no horizontal auto-wrap)
//   - 132-column RAM, visible offset = 2 (cols 2..129)
//   - Must set page + column address before each 128-byte page write
//   - Push-pull buffer: drive SDA low during ACK to prevent bus contention

module ssd1306_oled #(
    parameter OVERLAY_FILE = "",
    parameter ENABLE_OVERLAY = 0
)(
    input  wire         clk,   // 25 MHz
    input  wire         rst,
    input  wire [127:0] reg0,  // pages 0-1 (top)
    input  wire [127:0] reg1,  // pages 2-3
    input  wire [127:0] reg2,  // pages 4-5
    input  wire [127:0] reg3,  // pages 6-7 (bottom)

    output wire         scl,
    output wire         sda
);

    // I2C slave address (0x3C write = 0x78)
    localparam I2C_ADDR    = 8'h78;
    localparam CMD_PREFIX  = 8'h00;  // Co=0, D/C#=0: command stream
    localparam DATA_PREFIX = 8'h40;  // Co=0, D/C#=1: data stream

    // I2C driver interface
    reg  [7:0] i2c_data;
    reg        i2c_start;
    reg        i2c_send_start;
    reg        i2c_send_stop;
    wire       i2c_busy;

    ssd1306_i2c #(.CLK_DIV(64)) i2c (
        .clk(clk), .rst(rst),
        .data(i2c_data),
        .start(i2c_start),
        .send_start(i2c_send_start),
        .send_stop(i2c_send_stop),
        .busy(i2c_busy),
        .scl(scl), .sda(sda)
    );

    // =========================================================================
    // Init command table
    // =========================================================================
    localparam INIT_LEN = 25;
    reg [7:0] init_cmds [0:INIT_LEN-1];
    initial begin
        init_cmds[ 0] = 8'hAE;  // display off
        init_cmds[ 1] = 8'hD5;  // set clock divide
        init_cmds[ 2] = 8'h80;  //   default
        init_cmds[ 3] = 8'hA8;  // set multiplex
        init_cmds[ 4] = 8'h3F;  //   64-1
        init_cmds[ 5] = 8'hD3;  // display offset
        init_cmds[ 6] = 8'h00;  //   0
        init_cmds[ 7] = 8'h40;  // start line 0
        init_cmds[ 8] = 8'h8D;  // charge pump (SSD1306)
        init_cmds[ 9] = 8'h14;  //   enable
        init_cmds[10] = 8'hAD;  // DC-DC control (SH1106)
        init_cmds[11] = 8'h8B;  //   DC-DC on
        init_cmds[12] = 8'hA1;  // segment remap (col 127 = SEG0)
        init_cmds[13] = 8'hC8;  // COM scan descending
        init_cmds[14] = 8'hDA;  // COM pins
        init_cmds[15] = 8'h12;  //   alternative, no remap
        init_cmds[16] = 8'h81;  // set contrast
        init_cmds[17] = 8'hCF;  //   high
        init_cmds[18] = 8'hD9;  // pre-charge
        init_cmds[19] = 8'hF1;  //   phase1=1, phase2=15
        init_cmds[20] = 8'hDB;  // VCOMH deselect
        init_cmds[21] = 8'h40;  //   ~0.77xVcc
        init_cmds[22] = 8'hA4;  // display from RAM
        init_cmds[23] = 8'hA6;  // normal (not inverted)
        init_cmds[24] = 8'hAF;  // display on
    end

    // =========================================================================
    // FSM
    // =========================================================================
    localparam [2:0]
        ST_RESET    = 3'd0,
        ST_SEND     = 3'd1,
        ST_WAIT     = 3'd2,
        ST_NEXT     = 3'd3,
        ST_BUSFREE  = 3'd4;

    reg [2:0]  state = ST_RESET;
    reg [19:0] reset_cnt = 0;
    reg [9:0]  busfree_cnt = 0;

    reg [1:0]  phase;
    localparam PH_INIT = 2'd0, PH_PAGE_CMD = 2'd1, PH_PAGE_DATA = 2'd2;

    reg [4:0]  cmd_idx;
    reg [2:0]  page;
    reg [6:0]  col;

    // Latched register values (snapshot at frame start)
    reg [127:0] lat0, lat1, lat2, lat3;

    // Each register = 128 columns (1 col/bit, MSB left), 2 pages (16px tall).
    // page[2:1] selects register, col[6:3] selects byte, col[2:0] selects bit.
    // Both pages of a register show the same bit value (solid 16px bar).
    wire [127:0] sel_reg = (page[2:1] == 2'd0) ? lat0 :
                           (page[2:1] == 2'd1) ? lat1 :
                           (page[2:1] == 2'd2) ? lat2 : lat3;

    // Fixed-range byte select (avoids 128-bit variable indexing)
    reg [7:0] sel_byte;
    always @(*) begin
        case (col[6:3])
            4'd0:  sel_byte = sel_reg[127:120];
            4'd1:  sel_byte = sel_reg[119:112];
            4'd2:  sel_byte = sel_reg[111:104];
            4'd3:  sel_byte = sel_reg[103: 96];
            4'd4:  sel_byte = sel_reg[ 95: 88];
            4'd5:  sel_byte = sel_reg[ 87: 80];
            4'd6:  sel_byte = sel_reg[ 79: 72];
            4'd7:  sel_byte = sel_reg[ 71: 64];
            4'd8:  sel_byte = sel_reg[ 63: 56];
            4'd9:  sel_byte = sel_reg[ 55: 48];
            4'd10: sel_byte = sel_reg[ 47: 40];
            4'd11: sel_byte = sel_reg[ 39: 32];
            4'd12: sel_byte = sel_reg[ 31: 24];
            4'd13: sel_byte = sel_reg[ 23: 16];
            4'd14: sel_byte = sel_reg[ 15:  8];
            default: sel_byte = sel_reg[  7:  0];
        endcase
    end
    wire bit_val = sel_byte[3'd7 - col[2:0]];
    wire [7:0] bar_byte = bit_val ? 8'hFF : 8'h00;

    // Optional text overlay ROM (1024 bytes = 8 pages × 128 cols)
    // XOR'd with bar data: black text on white (pass), white text on black (fail)
    wire [7:0] px_byte;
    generate if (ENABLE_OVERLAY) begin : g_overlay
        reg [7:0] overlay_rom [0:1023];
        initial $readmemh(OVERLAY_FILE, overlay_rom);
        wire [9:0] overlay_addr = {page, col};
        wire [7:0] overlay_byte = overlay_rom[overlay_addr];
        assign px_byte = bar_byte ^ overlay_byte;
    end else begin : g_no_overlay
        assign px_byte = bar_byte;
    end endgenerate

    always @(posedge clk) begin
        if (rst) begin
            state     <= ST_RESET;
            reset_cnt <= 0;
            i2c_start <= 0;
        end else begin
            i2c_start <= 0;

            case (state)

                ST_RESET: begin
                    reset_cnt <= reset_cnt + 1;
                    if (&reset_cnt) begin
                        phase   <= PH_INIT;
                        cmd_idx <= 0;
                        page    <= 0;
                        col     <= 0;
                        lat0 <= reg0;
                        lat1 <= reg1;
                        lat2 <= reg2;
                        lat3 <= reg3;
                        state   <= ST_SEND;
                    end
                end

                ST_SEND: begin
                    if (!i2c_busy) begin
                        case (phase)
                            PH_INIT: begin
                                if (cmd_idx == 0) begin
                                    i2c_data       <= I2C_ADDR;
                                    i2c_send_start <= 1;
                                    i2c_send_stop  <= 0;
                                end else if (cmd_idx == 1) begin
                                    i2c_data       <= CMD_PREFIX;
                                    i2c_send_start <= 0;
                                    i2c_send_stop  <= 0;
                                end else begin
                                    i2c_data       <= init_cmds[cmd_idx - 2];
                                    i2c_send_start <= 0;
                                    i2c_send_stop  <= (cmd_idx == INIT_LEN + 1);
                                end
                            end

                            PH_PAGE_CMD: begin
                                case (cmd_idx[2:0])
                                    3'd0: begin
                                        i2c_data       <= I2C_ADDR;
                                        i2c_send_start <= 1;
                                        i2c_send_stop  <= 0;
                                    end
                                    3'd1: begin
                                        i2c_data       <= CMD_PREFIX;
                                        i2c_send_start <= 0;
                                        i2c_send_stop  <= 0;
                                    end
                                    3'd2: begin
                                        i2c_data       <= 8'hB0 | {5'b0, page};
                                        i2c_send_start <= 0;
                                        i2c_send_stop  <= 0;
                                    end
                                    3'd3: begin
                                        i2c_data       <= 8'h02;
                                        i2c_send_start <= 0;
                                        i2c_send_stop  <= 0;
                                    end
                                    3'd4: begin
                                        i2c_data       <= 8'h10;
                                        i2c_send_start <= 0;
                                        i2c_send_stop  <= 1;
                                    end
                                    default: ;
                                endcase
                            end

                            PH_PAGE_DATA: begin
                                if (cmd_idx == 0) begin
                                    i2c_data       <= I2C_ADDR;
                                    i2c_send_start <= 1;
                                    i2c_send_stop  <= 0;
                                end else if (cmd_idx == 1) begin
                                    i2c_data       <= DATA_PREFIX;
                                    i2c_send_start <= 0;
                                    i2c_send_stop  <= 0;
                                end else begin
                                    i2c_data       <= px_byte;
                                    i2c_send_start <= 0;
                                    i2c_send_stop  <= (col == 127);
                                end
                            end
                        endcase

                        i2c_start <= 1;
                        state     <= ST_WAIT;
                    end
                end

                ST_WAIT: begin
                    if (i2c_busy)
                        state <= ST_NEXT;
                end

                ST_NEXT: begin
                    if (!i2c_busy) begin
                        case (phase)
                            PH_INIT: begin
                                if (cmd_idx == INIT_LEN + 1) begin
                                    phase   <= PH_PAGE_CMD;
                                    cmd_idx <= 0;
                                    page    <= 0;
                                    state   <= ST_BUSFREE;
                                end else begin
                                    cmd_idx <= cmd_idx + 1;
                                    state   <= ST_SEND;
                                end
                            end

                            PH_PAGE_CMD: begin
                                if (cmd_idx == 4) begin
                                    phase   <= PH_PAGE_DATA;
                                    cmd_idx <= 0;
                                    col     <= 0;
                                    state   <= ST_BUSFREE;
                                end else begin
                                    cmd_idx <= cmd_idx + 1;
                                    state   <= ST_SEND;
                                end
                            end

                            PH_PAGE_DATA: begin
                                if (cmd_idx < 2) begin
                                    cmd_idx <= cmd_idx + 1;
                                    state   <= ST_SEND;
                                end else if (col == 127) begin
                                    if (page == 7) begin
                                        // Frame done — latch new values
                                        phase   <= PH_PAGE_CMD;
                                        cmd_idx <= 0;
                                        page    <= 0;
                                        lat0    <= reg0;
                                        lat1    <= reg1;
                                        lat2    <= reg2;
                                        lat3    <= reg3;
                                    end else begin
                                        phase   <= PH_PAGE_CMD;
                                        cmd_idx <= 0;
                                        page    <= page + 1;
                                    end
                                    state <= ST_BUSFREE;
                                end else begin
                                    col   <= col + 1;
                                    state <= ST_SEND;
                                end
                            end
                        endcase
                    end
                end

                ST_BUSFREE: begin
                    busfree_cnt <= busfree_cnt + 1;
                    if (&busfree_cnt)
                        state <= ST_SEND;
                end

            endcase
        end
    end

endmodule
