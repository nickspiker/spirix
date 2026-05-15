// Sequential testbench for spirix_sqrt at FRAC=24, EXP=8.

`timescale 1ns/1ps
module tb_sqrt_p4;
    localparam FRAC = 24;
    localparam EXP  = 8;
    localparam signed [EXP-1:0] AMB = -(1 <<< (EXP-1));

    reg clk = 0;
    always #5 clk = ~clk;

    reg start = 0;
    reg signed [FRAC-1:0] a_frac;
    reg signed [EXP-1:0]  a_exp;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;
    wire busy, done;

    spirix_sqrt #(.FRAC_BITS(FRAC), .EXP_BITS(EXP), .PARALLEL(4)) dut(
        .clk(clk), .start(start),
        .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(r_frac), .result_exp(r_exp),
        .busy(busy), .done(done)
    );

    function [3:0] classify;
        input [FRAC-1:0] storage; input signed [EXP-1:0] e;
        reg [FRAC-1:0] s; reg msb; integer i; integer count;
        begin
            s = storage;
            if (e !== AMB) classify = 0;
            else if (s === {FRAC{1'b0}}) classify = 1;
            else if (&s)               classify = 6;
            else begin
                msb = s[FRAC-1];
                count = 1;
                for (i = FRAC-2; i >= 0; i = i - 1) begin
                    if (s[i] === msb) count = count + 1;
                    else i = -1;
                end
                if (count == 1) classify = (msb == 1'b0) ? 4 : 5;
                else if (count == 2) classify = (msb == 1'b0) ? 2 : 3;
                else classify = 7;
            end
        end
    endfunction

    integer fd, line, total, exact, ulp1, state_match, fail, fail_examples;
    reg [FRAC-1:0] a_in, gold_frac;
    integer a_e_in, gold_e_in, gold_st_in;
    reg signed [EXP-1:0] gold_exp;
    reg [3:0] dut_state, expected_state;

    initial begin
        fd = $fopen("verilog/sqrt_vectors.txt", "r");
        if (fd == 0) begin $display("FATAL: cannot open vectors"); $finish; end

        total = 0; exact = 0; ulp1 = 0; state_match = 0; fail = 0;
        fail_examples = 0; line = 0;
        @(posedge clk);

        while (!$feof(fd)) begin
            integer rc;
            rc = $fscanf(fd, "%h %d %h %d %d\n",
                         a_in, a_e_in, gold_frac, gold_e_in, gold_st_in);
            if (rc !== 5) begin ; end
            else begin
                line = line + 1;
                gold_exp = gold_e_in[EXP-1:0];
                expected_state = gold_st_in[3:0];

                a_frac = a_in; a_exp = a_e_in[EXP-1:0];
                @(posedge clk); #1;
                start = 1;
                @(posedge clk); #1;
                start = 0;
                while (!done) begin @(posedge clk); #1; end

                dut_state = classify(r_frac, r_exp);
                total = total + 1;
                if (expected_state !== 4'd0 && expected_state !== 4'd1) begin
                    if ((expected_state == 4'd7 && dut_state == 4'd7 && r_exp === AMB) ||
                        ((expected_state == 4'd2 || expected_state == 4'd3) &&
                         (dut_state == 4'd2 || dut_state == 4'd3) && r_exp === AMB) ||
                        ((expected_state == 4'd4 || expected_state == 4'd5 || expected_state == 4'd6) &&
                         (dut_state == 4'd4 || dut_state == 4'd5 || dut_state == 4'd6) && r_exp === AMB)) begin
                        state_match = state_match + 1;
                        exact = exact + 1;
                    end else begin
                        fail = fail + 1;
                        if (fail_examples < 8) begin
                            $display("FAIL[%0d]: expected_st=%0d, got st=%0d (%h,%0d) for sqrt(%h,%0d)",
                                     line, expected_state, dut_state, r_frac, r_exp, a_in, a_e_in);
                            fail_examples = fail_examples + 1;
                        end
                    end
                end else if (r_frac === gold_frac && r_exp === gold_exp) begin
                    exact = exact + 1;
                end else if (expected_state === 4'd0 && dut_state === 4'd0
                             && r_exp === gold_exp
                             && ((r_frac - gold_frac == 24'sd1) ||
                                 (gold_frac - r_frac == 24'sd1))) begin
                    ulp1 = ulp1 + 1;
                    if (ulp1 <= 5) begin
                        $display("  ULP1[%0d]: sqrt(%h,%0d) → got %h want %h",
                                 line, a_in, a_e_in, r_frac, gold_frac);
                    end
                end else begin
                    fail = fail + 1;
                    if (fail_examples < 8) begin
                        $display("FAIL[%0d]: sqrt(%h,%0d) → got (%h,%0d) want (%h,%0d) [exp_st=%0d, dut_st=%0d]",
                                 line, a_in, a_e_in,
                                 r_frac, r_exp, gold_frac, gold_exp,
                                 expected_state, dut_state);
                        fail_examples = fail_examples + 1;
                    end
                end
            end
        end
        $fclose(fd);
        $display("");
        $display("Total:        %0d", total);
        $display("Exact:        %0d  (%0d state-class matches)", exact, state_match);
        $display("1-ULP off:    %0d", ulp1);
        $display(">1-ULP fail:  %0d", fail);
        if (fail == 0) $display("OK"); else $display("FAILED");
        $finish;
    end
endmodule
