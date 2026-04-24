`timescale 1ns / 1ps

// Minimal testbench: simulate just the PRNG + blake3 + XOR-fold + compare to verify the harness logic is correct in simulation.
module top_ntsc_tb;

    reg clk = 0;
    always #20 clk = ~clk;  // 25 MHz

    localparam [31:0] SEED = 32'hCAFE_BABE;
    localparam PRNG_FILL = 24;
    localparam NVEC = 4;  // just test 4 vectors for speed

    // ---- XOR-fold function ----
    function [31:0] xor_fold;
        input [511:0] h;
        integer i;
        begin
            xor_fold = 32'b0;
            for (i = 0; i < 16; i = i + 1)
                xor_fold = xor_fold ^ h[i*32 +: 32];
        end
    endfunction

    // ---- Reference side ----
    reg [31:0] ref_s1 = SEED, ref_s2 = 0, ref_s3 = 0;
    wire [31:0] ref_s1_next = ref_s3 ^ (ref_s3 << 13);
    wire [31:0] ref_s2_next = ref_s1 ^ (ref_s1 >> 17);
    wire [31:0] ref_s3_next = ref_s2 ^ (ref_s2 << 5);
    reg [767:0] ref_sr = 0;

    reg ref_b3_valid = 0;
    wire [511:0] ref_b3_hash;
    wire ref_b3_done;

    blake3 ref_b3 (
        .i_clk(clk), .i_reset(1'b1),
        .i_chain(ref_sr[255:0]),
        .i_mblock(ref_sr[767:256]),
        .i_counter(64'b0), .i_numbytes(32'd64),
        .i_dflags(32'h03), .i_valid(ref_b3_valid),
        .o_hash(ref_b3_hash), .o_valid(ref_b3_done)
    );

    // ---- Test side (same clock for sim) ----
    reg [31:0] test_s1 = SEED, test_s2 = 0, test_s3 = 0;
    wire [31:0] test_s1_next = test_s3 ^ (test_s3 << 13);
    wire [31:0] test_s2_next = test_s1 ^ (test_s1 >> 17);
    wire [31:0] test_s3_next = test_s2 ^ (test_s2 << 5);
    reg [767:0] test_sr = 0;

    reg test_b3_valid = 0;
    wire [511:0] test_b3_hash;
    wire test_b3_done;

    blake3 test_b3_inst (
        .i_clk(clk), .i_reset(1'b1),
        .i_chain(test_sr[255:0]),
        .i_mblock(test_sr[767:256]),
        .i_counter(64'b0), .i_numbytes(32'd64),
        .i_dflags(32'h03), .i_valid(test_b3_valid),
        .o_hash(test_b3_hash), .o_valid(test_b3_done)
    );

    // ---- BRAM ----
    reg [31:0] bram [0:255];
    reg [31:0] ref_results [0:NVEC-1];  // for debug display

    integer vec, fill;
    reg [31:0] ref_folded, test_folded;
    integer fail_count = 0;

    initial begin
        // Let blake3 initialize (it has no reset pulse here, same as hardware)
        // Actually give it a reset pulse to be safe
        // ... but i_reset is tied to 1'b1 above, matching the original (broken) config

        #100;  // wait a few cycles

        // =====================
        // Reference phase
        // =====================
        for (vec = 0; vec < NVEC; vec = vec + 1) begin
            // Fill shift register with 24 PRNG outputs
            for (fill = 0; fill < PRNG_FILL; fill = fill + 1) begin
                @(posedge clk);
                ref_s1 <= ref_s1_next;
                ref_s2 <= ref_s2_next;
                ref_s3 <= ref_s3_next;
                ref_sr <= {ref_sr[735:0], ref_s3};
            end
            @(posedge clk);

            // Strobe blake3
            @(posedge clk);
            ref_b3_valid <= 1;
            @(posedge clk);
            ref_b3_valid <= 0;

            // Wait for blake3 done
            wait(ref_b3_done);
            @(posedge clk);

            ref_folded = xor_fold(ref_b3_hash);
            bram[vec] = ref_folded;
            ref_results[vec] = ref_folded;
            $display("REF vec[%0d]: fold=%08h  sr[31:0]=%08h", vec, ref_folded, ref_sr[31:0]);
        end

        $display("");

        // =====================
        // Test phase (reset PRNG to same seed)
        // =====================
        test_s1 = SEED;
        test_s2 = 0;
        test_s3 = 0;
        test_sr = 0;

        for (vec = 0; vec < NVEC; vec = vec + 1) begin
            // Fill shift register
            for (fill = 0; fill < PRNG_FILL; fill = fill + 1) begin
                @(posedge clk);
                test_s1 <= test_s1_next;
                test_s2 <= test_s2_next;
                test_s3 <= test_s3_next;
                test_sr <= {test_sr[735:0], test_s3};
            end
            @(posedge clk);

            // Strobe blake3
            @(posedge clk);
            test_b3_valid <= 1;
            @(posedge clk);
            test_b3_valid <= 0;

            // Wait for blake3 done
            wait(test_b3_done);
            @(posedge clk);

            test_folded = xor_fold(test_b3_hash);
            $display("TST vec[%0d]: fold=%08h  bram=%08h  %s",
                     vec, test_folded, bram[vec],
                     (test_folded == bram[vec]) ? "OK" : "MISMATCH");
            if (test_folded != bram[vec]) fail_count = fail_count + 1;
        end

        $display("");
        if (fail_count == 0)
            $display("PASS: all %0d vectors match", NVEC);
        else
            $display("FAIL: %0d mismatches out of %0d vectors", fail_count, NVEC);

        #100;
        $finish;
    end

endmodule
