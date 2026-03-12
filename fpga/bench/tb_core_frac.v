// tb_core_frac — Test spirix_core FRAC micro-op against Rust gold vectors
//
// Reads frac_vectors.hex: "03 fw ew a_frac a_exp r_frac r_exp"
// Writes A to R0 via ext port, issues OP_FRAC(ra=0,rb=0,rd=1), reads R1.

`timescale 1ns/1ps

module tb_core_frac;

    parameter MAX_FRAC = 64;
    parameter MAX_EXP  = 64;
    parameter VECFILE  = "fpga/build/frac_vectors.hex";

    reg clk = 0;
    always #5 clk = ~clk;  // 100 MHz

    reg rst = 1;
    reg [17:0] instr;
    reg exec = 0;
    wire busy, done;

    reg [2:0] ext_addr;
    reg signed [MAX_FRAC-1:0] ext_wfrac;
    reg signed [MAX_EXP-1:0]  ext_wexp;
    reg ext_we = 0;
    wire signed [MAX_FRAC-1:0] ext_rfrac;
    wire signed [MAX_EXP-1:0]  ext_rexp;

    spirix_core #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) uut (
        .clk(clk), .rst(rst),
        .instr(instr), .exec(exec), .busy(busy), .done(done),
        .ext_addr(ext_addr), .ext_wfrac(ext_wfrac), .ext_wexp(ext_wexp),
        .ext_we(ext_we), .ext_rfrac(ext_rfrac), .ext_rexp(ext_rexp)
    );

    // Opcodes
    localparam [4:0] OP_FRAC = 5'd15;

    // Build instruction word: {opcode[4:0], ra[2:0], rb[2:0], rd[2:0], frac_w[1:0], exp_w[1:0]}
    function [17:0] make_instr;
        input [4:0] op;
        input [2:0] ra, rb, rd;
        input [1:0] fw, ew;
        make_instr = {op, ra, rb, rd, fw, ew};
    endfunction

    integer fd, r, line_num;
    reg [7:0]  op_hex;
    reg [3:0]  fw_hex, ew_hex;
    reg [63:0] a_frac, a_exp, exp_frac, exp_exp;
    integer pass_count = 0, fail_count = 0;

    initial begin
        // Reset
        #20 rst = 0;
        #10;

        fd = $fopen(VECFILE, "r");
        if (fd == 0) begin
            $display("ERROR: Cannot open %s", VECFILE);
            $finish;
        end

        line_num = 0;
        while (!$feof(fd)) begin
            r = $fscanf(fd, "%x %d %d %x %x %x %x\n",
                         op_hex, fw_hex, ew_hex, a_frac, a_exp, exp_frac, exp_exp);
            line_num = line_num + 1;
            if (r != 7) begin
                if (!$feof(fd))
                    $display("WARN: parse error at line %0d (got %0d fields)", line_num, r);
            end else begin

            // Write A to R0
            @(posedge clk);
            ext_addr  <= 3'd0;
            ext_wfrac <= a_frac;
            ext_wexp  <= a_exp;
            ext_we    <= 1;
            @(posedge clk);
            ext_we <= 0;

            // Issue FRAC instruction: ra=0, rb=0 (unused for FRAC), rd=1
            @(posedge clk);
            instr <= make_instr(OP_FRAC, 3'd0, 3'd0, 3'd1, fw_hex[1:0], ew_hex[1:0]);
            exec  <= 1;
            @(posedge clk);
            exec <= 0;

            // Wait for done
            @(posedge clk);
            while (!done) @(posedge clk);

            // Read R1
            @(posedge clk);
            ext_addr <= 3'd1;
            @(posedge clk);  // async read settles

            if (ext_rfrac === exp_frac && ext_rexp === exp_exp) begin
                pass_count = pass_count + 1;
            end else begin
                fail_count = fail_count + 1;
                if (fail_count <= 50)
                    $display("FAIL line %0d: F%0dE%0d a=(%016x,%016x) exp=(%016x,%016x) got=(%016x,%016x)",
                             line_num, fw_hex, ew_hex,
                             a_frac, a_exp, exp_frac, exp_exp,
                             ext_rfrac, ext_rexp);
            end
            end // else (r == 7)
        end

        $fclose(fd);
        $display("\n%0d/%0d pass, %0d fail", pass_count, pass_count + fail_count, fail_count);
        if (fail_count == 0)
            $display("ALL PASS");
        else
            $display("FAILED");
        $finish;
    end

endmodule
