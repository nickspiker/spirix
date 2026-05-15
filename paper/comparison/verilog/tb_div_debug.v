`timescale 1ns/1ps
module tb_div_debug;
    localparam FRAC = 24;
    localparam EXP  = 8;

    reg clk = 0;
    always #5 clk = ~clk;

    reg start = 0;
    reg signed [FRAC-1:0] a_frac, b_frac;
    reg signed [EXP-1:0]  a_exp,  b_exp;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;
    wire busy, done;

    spirix_divide #(.FRAC_BITS(FRAC), .EXP_BITS(EXP)) dut(
        .clk(clk), .start(start),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(r_frac), .result_exp(r_exp),
        .busy(busy), .done(done)
    );

    integer cycle = 0;
    always @(posedge clk) cycle = cycle + 1;

    initial begin
        $display("Test: 1.0 / 0.5 = 2.0");
        $display("  1.0 = (POS_ONE_NORMAL=0x800000, exp=1)  -- value = +0.5*2^1");
        $display("  0.5 = (POS_ONE_NORMAL=0x800000, exp=0)  -- value = +0.5*2^0");
        $display("  Expected: 2.0 = (POS_ONE_NORMAL=0x800000, exp=2)");
        a_frac = 24'h800000; a_exp = 8'sd1;
        b_frac = 24'h800000; b_exp = 8'sd0;

        @(posedge clk);
        start = 1;
        $display("[cyc %0d] start=1 a=(%h,%0d) b=(%h,%0d)", cycle, a_frac, a_exp, b_frac, b_exp);
        @(posedge clk);
        start = 0;
        $display("[cyc %0d] start=0, busy=%b, done=%b", cycle, busy, done);

        while (!done) begin
            @(posedge clk);
            $display("[cyc %0d] busy=%b done=%b q=%h r=%h", cycle, busy, done, dut.q_reg, dut.r_reg);
            if (cycle > 50) begin $display("TIMEOUT"); $finish; end
        end
        #1;
        $display("[cyc %0d] DONE: result=(%h, %0d)", cycle, r_frac, r_exp);
        $finish;
    end
endmodule
