// Hand-computed smoke tests for spirix_addsub at FRAC=16, EXP=8 (F4E3).
// Verifies: zero identity, basic add, subtract, cancellation to zero.

`timescale 1ns/1ps
module tb_smoke;
    localparam FRAC = 16;
    localparam EXP  = 8;
    localparam signed [EXP-1:0] AMB = -(1 <<< (EXP-1));

    reg signed [FRAC-1:0] a_frac, b_frac;
    reg signed [EXP-1:0]  a_exp,  b_exp;
    reg                    sub;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;

    spirix_addsub #(.FRAC_BITS(FRAC), .EXP_BITS(EXP)) dut(
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .sub(sub),
        .result_frac(r_frac), .result_exp(r_exp)
    );

    integer pass, fail;

    task check;
        input [127:0] name;
        input signed [FRAC-1:0] af; input signed [EXP-1:0] ae;
        input signed [FRAC-1:0] bf; input signed [EXP-1:0] be;
        input s;
        input signed [FRAC-1:0] ef; input signed [EXP-1:0] ee;
        begin
            a_frac = af; a_exp = ae;
            b_frac = bf; b_exp = be;
            sub = s;
            #1;
            if (r_frac === ef && r_exp === ee) begin
                pass = pass + 1;
                $display("PASS  %s : (%h,%0d) %s (%h,%0d) = (%h,%0d)",
                         name, af, ae, s ? "-" : "+", bf, be, r_frac, r_exp);
            end else begin
                fail = fail + 1;
                $display("FAIL  %s : (%h,%0d) %s (%h,%0d) = (%h,%0d), wanted (%h,%0d)",
                         name, af, ae, s ? "-" : "+", bf, be, r_frac, r_exp, ef, ee);
            end
        end
    endtask

    initial begin
        pass = 0; fail = 0;
        // N0 storage:
        //   POS_ONE_NORMAL  = 16'h8000 (= +0.5·2^exp = the "+1.0·2^(exp-1)" boundary)
        //   NEG_ONE_NORMAL  = 16'h0000 (= -1.0·2^exp)

        // 1.0 + 1.0 = 2.0  → (8000,1) + (8000,1) = (8000,2)
        check("1+1=2", 16'h8000, 8'd1, 16'h8000, 8'd1, 1'b0, 16'h8000, 8'd2);

        // 1.0 - 1.0 = 0  → (8000,1) - (8000,1) = (0,AMB)
        check("1-1=0", 16'h8000, 8'd1, 16'h8000, 8'd1, 1'b1, 16'h0000, AMB);

        // (-1.0) + (+1.0) = 0  → -1 stored as (0,0), +1 as (8000,1), sum=0
        check("-1+1=0", 16'h0000, 8'd0, 16'h8000, 8'd1, 1'b0, 16'h0000, AMB);

        // 0 + 1.0 = 1.0  (zero-as-identity)
        check("0+1=1", 16'h0000, AMB, 16'h8000, 8'd1, 1'b0, 16'h8000, 8'd1);

        // 1.0 + 0 = 1.0
        check("1+0=1", 16'h8000, 8'd1, 16'h0000, AMB, 1'b0, 16'h8000, 8'd1);

        // 0 - 1.0 = -1.0  → +1 was (8000,1)=+0.5·2¹; negated drops exp by 1
        // → -1 = (0,0) (NEG_ONE_NORMAL at exp=0 = -1·2⁰)
        check("0-1=-1", 16'h0000, AMB, 16'h8000, 8'd1, 1'b1, 16'h0000, 8'd0);

        // INF + 1.0 = INF (zero-priority is below b_undef so INF+1 falls
        //                  to a_transf=INF, returns UNDEF_TF_P_FIN)
        // Actually a is INF (transfinite), b is 1.0 finite.
        // sc_a_transf fires (a_transf=1, others gating it false).
        // Result: UNDEF_TF_P_FIN = 8'h1C followed by FRAC-8=8 zeros = 16'h1C00.
        check("inf+1=undef_tf+fin", 16'hFFFF, AMB, 16'h8000, 8'd1, 1'b0,
              16'h1C00, AMB);

        // 1.0 - INF = UNDEF_FIN_M_TF = 16'hE400
        check("1-inf=undef_fin-tf", 16'h8000, 8'd1, 16'hFFFF, AMB, 1'b1,
              16'hE400, AMB);

        // 0 + INF = INF (zero-as-identity wins over b_transf)
        check("0+inf=inf", 16'h0000, AMB, 16'hFFFF, AMB, 1'b0,
              16'hFFFF, AMB);

        // 0 - INF = -INF — Spirix infinity is signless, so negate(INF) = INF
        check("0-inf=inf", 16'h0000, AMB, 16'hFFFF, AMB, 1'b1,
              16'hFFFF, AMB);

        // 0.5 + 0.5 = 1.0
        // 0.5·2^0 = +0.5 stored at exp=0 = POS_ONE_NORMAL = (8000,0)
        // 1.0·2^0 = 0.5·2^1 = (8000,1)
        check("0.5+0.5=1", 16'h8000, 8'd0, 16'h8000, 8'd0, 1'b0,
              16'h8000, 8'd1);

        $display("");
        $display("Total: %0d pass, %0d fail", pass, fail);
        if (fail == 0) $display("OK"); else $display("FAILED");
        $finish;
    end
endmodule
