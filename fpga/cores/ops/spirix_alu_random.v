// spirix_alu_random — 2-cycle pipelined hardware TRNG
//
// 72 ring oscillators with frequency diversity (24×A, 24×B, 24×C LUT4 pins).
// ROs run continuously after first activation. prev_sample updated every idle clock.
//
// S1 (capture): temporal XOR of 72 ROs vs previous sample → 72-bit jitter
// S2 (normalize): rotation mix (coprime 9,31) + XOR-fold 72→64 + CLZ + normalize
//
// Latency: 2 clocks (warm) or 4 clocks (cold, first-ever start needs priming).
// Throughput: 1 result per 2 clocks.
// Core: 72 RO LUTs + 72 mix LUTs + 8 fold LUTs + ~150 CLZ/barrel.
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64

module spirix_alu_random #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire                      clk,
    input  wire                      ce,
    input  wire                      start,
    input  wire [1:0]                frac_width,
    input  wire [1:0]                exp_width,
    output reg signed [MAX_FRAC-1:0] result_frac,
    output reg signed [MAX_EXP-1:0]  result_exp,
    output reg                       busy,
    output reg                       done,
    output wire [71:0]               dbg_xor
);

    localparam signed [MAX_EXP-1:0] AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // =====================================================================
    // 72 Ring Oscillators — 3 frequency bands × 24 each
    // =====================================================================
    reg ro_gate = 0;
    wire [23:0] osc_a, osc_b, osc_c;

    genvar gi;
    generate
        for (gi = 0; gi < 24; gi = gi + 1) begin : ro_a_gen
            (* keep, syn_keep="true" *) wire fb;
            (* keep *) LUT4 #(.INIT(16'h0100)) ro (
                .A(fb), .B(1'b0), .C(1'b0), .D(ro_gate), .Z(fb)
            );
            assign osc_a[gi] = fb;
        end
        for (gi = 0; gi < 24; gi = gi + 1) begin : ro_b_gen
            (* keep, syn_keep="true" *) wire fb;
            (* keep *) LUT4 #(.INIT(16'h3300)) ro (
                .A(1'b0), .B(fb), .C(1'b0), .D(ro_gate), .Z(fb)
            );
            assign osc_b[gi] = fb;
        end
        for (gi = 0; gi < 24; gi = gi + 1) begin : ro_c_gen
            (* keep, syn_keep="true" *) wire fb;
            (* keep *) LUT4 #(.INIT(16'h0F00)) ro (
                .A(1'b0), .B(1'b0), .C(fb), .D(ro_gate), .Z(fb)
            );
            assign osc_c[gi] = fb;
        end
    endgenerate

    wire [71:0] ro_out = {osc_c, osc_b, osc_a};

    // =====================================================================
    // Continuous temporal XOR (prev_sample updated every idle clock)
    // =====================================================================
    reg [71:0] prev_sample = 0;
    wire [71:0] jitter = ro_out ^ prev_sample;

    // =====================================================================
    // S1 output register: captured jitter
    // =====================================================================
    reg [71:0] jitter_s1 = 0;
    assign dbg_xor = jitter_s1;

    // =====================================================================
    // S2: Rotation mix (coprime offsets 9, 31) — 1 LUT4 per bit
    // =====================================================================
    wire [71:0] mixed;
    genvar gj;
    generate
        for (gj = 0; gj < 72; gj = gj + 1) begin : mix_gen
            assign mixed[gj] = jitter_s1[gj]
                              ^ jitter_s1[(gj + 9) % 72]
                              ^ jitter_s1[(gj + 31) % 72];
        end
    endgenerate

    // =====================================================================
    // CLZ + normalize on full 72 bits, then truncate to 64
    // Sign is random (bit 71). Supports both positive (01...) and negative (10...).
    // =====================================================================
    wire [71:0] rng72 = mixed;
    wire sign = rng72[71];
    wire [70:0] magnitude = rng72[70:0];

    // Width mask on the 71-bit magnitude (positions 70..0)
    wire [70:0] mag_mask;
    assign mag_mask[70:56] = 15'h7FFF;   // always active (8-bit+ frac)
    assign mag_mask[55:48] = {8{frac_width[1] | frac_width[0]}};
    assign mag_mask[47:32] = {16{frac_width[1]}};
    assign mag_mask[31: 0] = {32{frac_width[1] & frac_width[0]}};

    wire [70:0] mag_masked = magnitude & mag_mask;

    // XOR magnitude with sign: for negative (sign=1), flips leading ones → zeros
    // Then CLZ works uniformly for both signs (counts redundant sign-extension bits)
    wire [70:0] mag_xor = mag_masked ^ {71{sign}};

    // 7-level CLZ on {1'b0, mag_xor} (72 bits, padded to 128)
    wire [127:0] clz_pad = {57'b0, mag_xor};
    wire clz6 = ~|clz_pad[127:64];
    wire [63:0] clz_l6 = clz6 ? clz_pad[63:0] : clz_pad[127:64];
    wire clz5 = ~|clz_l6[63:32];
    wire [31:0] clz_l5 = clz5 ? clz_l6[31:0] : clz_l6[63:32];
    wire clz4 = ~|clz_l5[31:16];
    wire [15:0] clz_l4 = clz4 ? clz_l5[15:0] : clz_l5[31:16];
    wire clz3 = ~|clz_l4[15:8];
    wire [7:0]  clz_l3 = clz3 ? clz_l4[7:0] : clz_l4[15:8];
    wire clz2 = ~|clz_l3[7:4];
    wire [3:0]  clz_l2 = clz2 ? clz_l3[3:0] : clz_l3[7:4];
    wire clz1 = ~|clz_l2[3:2];
    wire [1:0]  clz_l1 = clz1 ? clz_l2[1:0] : clz_l2[3:2];
    wire clz0 = ~clz_l1[1];
    wire [6:0] clz = {clz6, clz5, clz4, clz3, clz2, clz1, clz0};

    // Subtract the 57 padding zeros to get actual leading redundant bits
    wire [6:0] nshift = clz - 7'd57;

    // Normalize: shift 72-bit value left, then take top 64 bits
    wire [127:0] shifted128 = {rng72, 56'b0} << nshift;
    wire [MAX_FRAC-1:0] normed_frac = shifted128[127:64];
    wire signed [MAX_EXP-1:0] normed_exp = -{{(MAX_EXP-7){1'b0}}, nshift};

    wire is_zero = ~|mag_xor;
    wire [MAX_FRAC-1:0] out_frac = is_zero ? {MAX_FRAC{1'b0}} : normed_frac;
    wire signed [MAX_EXP-1:0] out_exp = is_zero ? AMBIG_EXP : normed_exp;

    // =====================================================================
    // FSM: 2-cycle pipeline (warm) or 4-cycle (cold priming)
    // =====================================================================
    localparam [2:0] S_IDLE    = 3'd0,
                     S_PRIME1  = 3'd1,  // cold: ROs just turned on, wait 1 clock
                     S_PRIME2  = 3'd2,  // cold: capture first prev_sample
                     S_CAPTURE = 3'd3,  // S1: capture jitter
                     S_NORM    = 3'd4;  // S2: mix + normalize → output
    reg [2:0] state = S_IDLE;
    reg ro_warm = 0;  // set after first priming, stays 1 forever

    always @(posedge clk) if (ce) begin
        done <= 0;

        // Keep prev_sample fresh while idle (ROs still running)
        if (ro_warm && state == S_IDLE)
            prev_sample <= ro_out;

        case (state)
            S_IDLE: begin
                if (start) begin
                    busy <= 1;
                    if (ro_warm) begin
                        // Warm: capture immediately (prev_sample is fresh)
                        jitter_s1   <= jitter;
                        prev_sample <= ro_out;
                        state       <= S_NORM;
                    end else begin
                        // Cold: need to prime ROs
                        ro_gate <= 1;
                        state   <= S_PRIME1;
                    end
                end
            end

            S_PRIME1: begin
                // ROs have been on for 1 clock — capture first sample
                prev_sample <= ro_out;
                state       <= S_PRIME2;
            end

            S_PRIME2: begin
                // Now have real temporal XOR
                jitter_s1   <= ro_out ^ prev_sample;
                prev_sample <= ro_out;
                ro_warm     <= 1;
                state       <= S_NORM;
            end

            S_CAPTURE: begin
                // S1: capture jitter (used for back-to-back operation)
                jitter_s1   <= jitter;
                prev_sample <= ro_out;
                state       <= S_NORM;
            end

            S_NORM: begin
                // S2: mix + CLZ + normalize (all combinational), register output
                result_frac <= out_frac;
                result_exp  <= out_exp;
                done        <= 1;
                busy        <= 0;
                state       <= S_IDLE;
            end
        endcase
    end

endmodule
