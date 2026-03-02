// Exhaustive testbench for spirix_multiply (F5E4).
// Gold model: signed multiply at double width, normalize, round, extract.

`timescale 1ns/1ps
module tb;
    parameter FRAC = 5;
    parameter EXP  = 4;
    localparam PROD_BITS = 2 * FRAC - 1;

    reg signed [FRAC-1:0] a_frac, b_frac;
    reg signed [EXP-1:0]  a_exp,  b_exp;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;

    spirix_multiply #(.FRAC_BITS(FRAC),.EXP_BITS(EXP)) dut(
        .a_frac(a_frac),.a_exp(a_exp),.b_frac(b_frac),.b_exp(b_exp),
        .result_frac(r_frac),.result_exp(r_exp));

    localparam AMB_EXP = -(1<<(EXP-1));
    localparam signed [FRAC-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC-2){1'b0}}};
    localparam signed [FRAC-1:0] NEG_ONE  = {1'b1, {(FRAC-1){1'b0}}};
    localparam MAX_EXP_VAL = (1 << (EXP-1)) - 1;
    localparam MIN_EXP_VAL = -(1 << (EXP-1)) + 1;

    function is_n1;
        input signed [FRAC-1:0] frac;
        begin is_n1 = (frac[FRAC-1] != frac[FRAC-2]); end
    endfunction

    task gold_multiply;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg signed [PROD_BITS-1:0] product, normalized;
        reg signed [FRAC-1:0] frac_raw, frac_rounded;
        reg norm_shift, guard_bit, sticky_bit, lsb_bit, round_up_bit;
        reg rovf_pos, rovf_neg;
        reg signed [EXP:0] exp_wide;
        begin
            product = af * bf;

            // Bounded normalization
            if (product[PROD_BITS-1] != product[PROD_BITS-2]) begin
                norm_shift = 0;
                normalized = product;
            end else begin
                norm_shift = 1;
                normalized = product <<< 1;
            end

            frac_raw = normalized[PROD_BITS-1 -: FRAC];
            guard_bit = normalized[PROD_BITS - 1 - FRAC];
            sticky_bit = (PROD_BITS - 2 - FRAC >= 0) ?
                         |normalized[PROD_BITS - 2 - FRAC:0] : 1'b0;
            lsb_bit = frac_raw[0];
            round_up_bit = guard_bit & (sticky_bit | lsb_bit);

            frac_rounded = frac_raw + round_up_bit;

            rovf_pos = !frac_raw[FRAC-1] & frac_rounded[FRAC-1];
            rovf_neg = frac_rounded[FRAC-1] & frac_rounded[FRAC-2];

            if (rovf_pos)      of_ = POS_HALF;
            else if (rovf_neg) of_ = NEG_ONE;
            else               of_ = frac_rounded;

            exp_wide = $signed({{1{ae[EXP-1]}}, ae})
                     + $signed({{1{be[EXP-1]}}, be})
                     - norm_shift + rovf_pos - rovf_neg;

            if (exp_wide > MAX_EXP_VAL) begin
                oe = AMB_EXP;
            end else if (exp_wide < MIN_EXP_VAL) begin
                of_ = {of_[FRAC-1], of_[FRAC-1:1]};
                oe = AMB_EXP;
            end else begin
                oe = exp_wide[EXP-1:0];
            end
        end
    endtask

    integer af, bf, ae, be, pass, fail;
    reg signed [FRAC-1:0] gf; reg signed [EXP-1:0] ge;

    initial begin
        pass=0; fail=0;
        for(ae=-(1<<(EXP-1));ae<(1<<(EXP-1));ae=ae+1)
        for(be=-(1<<(EXP-1));be<(1<<(EXP-1));be=be+1)
        for(af=-(1<<(FRAC-1));af<(1<<(FRAC-1));af=af+1)
        for(bf=-(1<<(FRAC-1));bf<(1<<(FRAC-1));bf=bf+1) begin
            if(ae==AMB_EXP || be==AMB_EXP || !is_n1(af) || !is_n1(bf)) begin
                // skip
            end else begin
                a_frac=af; a_exp=ae; b_frac=bf; b_exp=be; #2;
                gold_multiply(af,bf,ae,be,gf,ge);
                if(ge==AMB_EXP) begin
                    if(r_exp===ge) pass=pass+1;
                    else begin
                        if(fail<10) $display("FAIL(amb) a=(%0d,%0d) b=(%0d,%0d): got_exp=%0d want_exp=%0d",
                            af,ae,bf,be,r_exp,ge);
                        fail=fail+1;
                    end
                end else begin
                    if(r_frac===gf && r_exp===ge) pass=pass+1;
                    else begin
                        if(fail<10) $display("FAIL a=(%0d,%0d) b=(%0d,%0d): got(%0d,%0d) want(%0d,%0d)",
                            af,ae,bf,be,r_frac,r_exp,gf,ge);
                        fail=fail+1;
                    end
                end
            end
        end
        $display("Done: %0d pass %0d fail out of %0d", pass, fail, pass+fail);
        $finish;
    end
endmodule
