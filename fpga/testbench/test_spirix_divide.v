// Exhaustive testbench for spirix_divide (parameterized).
// Uses F5E4 (5-bit frac, 4-bit exp) for full enumeration.
// Gold model: unsigned absolute-value division at DUT precision,
// banker's rounding, N1 corner case handling.

`timescale 1ns/1ps
module tb;
    parameter FRAC = 5;
    parameter EXP  = 4;

    reg signed [FRAC-1:0] a_frac, b_frac;
    reg signed [EXP-1:0]  a_exp,  b_exp;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;

    spirix_divide #(.FRAC_BITS(FRAC),.EXP_BITS(EXP)) dut(
        .a_frac(a_frac),.a_exp(a_exp),.b_frac(b_frac),.b_exp(b_exp),
        .result_frac(r_frac),.result_exp(r_exp));

    localparam AMB_EXP = -(1<<(EXP-1));
    localparam signed [FRAC-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC-2){1'b0}}};
    localparam signed [FRAC-1:0] NEG_ONE  = {1'b1, {(FRAC-1){1'b0}}};
    localparam WIDE = 2 * FRAC;
    localparam MAX_EXP_VAL = (1 << (EXP-1)) - 1;
    localparam MIN_EXP_VAL = -(1 << (EXP-1)) + 1;

    function is_n1;
        input signed [FRAC-1:0] frac;
        begin
            is_n1 = (frac[FRAC-1] != frac[FRAC-2]);
        end
    endfunction

    // Gold model
    task gold_divide;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;

        reg result_sign;
        reg a_neg_one, b_neg_one;
        reg [FRAC-2:0] abs_a, abs_b;
        reg signed [EXP:0] a_exp_adj, b_exp_adj;
        reg [WIDE-1:0] wide_a, quotient_full, remainder_val;
        reg [FRAC:0] q, q_norm;
        reg q_overflow;
        reg [FRAC-1:0] frac_pos_raw, frac_rounded, pos_frac;
        reg guard_bit, ovf_sticky, rem_sticky, sticky_bit, lsb_bit, round_up_bit;
        reg round_ovf;
        reg signed [EXP:0] exp_wide;
        reg signed [EXP-1:0] pos_exp, neg_exp;
        reg signed [FRAC-1:0] neg_frac;
        reg neg_is_ph;
        begin
            // Step 1: Sign and absolute values
            result_sign = af[FRAC-1] ^ bf[FRAC-1];

            a_neg_one = (af == NEG_ONE);
            b_neg_one = (bf == NEG_ONE);

            if (a_neg_one)
                abs_a = POS_HALF[FRAC-2:0];
            else if (af[FRAC-1])
                abs_a = (~af[FRAC-2:0]) + 1'b1;
            else
                abs_a = af[FRAC-2:0];

            if (b_neg_one)
                abs_b = POS_HALF[FRAC-2:0];
            else if (bf[FRAC-1])
                abs_b = (~bf[FRAC-2:0]) + 1'b1;
            else
                abs_b = bf[FRAC-2:0];

            a_exp_adj = $signed({{1{ae[EXP-1]}}, ae}) + a_neg_one;
            b_exp_adj = $signed({{1{be[EXP-1]}}, be}) + b_neg_one;

            // Step 2: Fixed-point division
            wide_a = {{(FRAC+1){1'b0}}, abs_a} << FRAC;
            quotient_full = wide_a / {{(FRAC+1){1'b0}}, abs_b};
            remainder_val = wide_a - quotient_full * {{(FRAC+1){1'b0}}, abs_b};

            q = quotient_full[FRAC:0];

            // Step 3: Bounded normalization
            q_overflow = q[FRAC];
            q_norm = q_overflow ? (q >> 1) : q;

            frac_pos_raw = q_norm[FRAC:1];

            // Step 4: Banker's rounding
            guard_bit = q_norm[0];
            ovf_sticky = q_overflow & q[0];
            rem_sticky = |remainder_val;
            sticky_bit = ovf_sticky | rem_sticky;
            lsb_bit = frac_pos_raw[0];
            round_up_bit = guard_bit & (sticky_bit | lsb_bit);

            frac_rounded = frac_pos_raw + round_up_bit;

            round_ovf = !frac_pos_raw[FRAC-1] & frac_rounded[FRAC-1];
            pos_frac = round_ovf ? POS_HALF : frac_rounded;

            // Step 5: Exponent
            exp_wide = a_exp_adj - b_exp_adj + q_overflow + round_ovf;

            pos_exp = exp_wide[EXP-1:0];

            // Step 6: Apply sign
            neg_is_ph = (pos_frac == POS_HALF);
            if (neg_is_ph) begin
                neg_frac = NEG_ONE;
                neg_exp = pos_exp - 1;
            end else begin
                neg_frac = ~pos_frac + 1'b1;
                neg_exp = pos_exp;
            end

            if (result_sign) begin
                of_ = neg_frac;
                oe  = neg_exp;
            end else begin
                of_ = pos_frac;
                oe  = pos_exp;
            end

            // Step 7: Overflow/underflow
            if (exp_wide > MAX_EXP_VAL) begin
                oe = AMB_EXP;
            end else if (exp_wide < MIN_EXP_VAL) begin
                of_ = {of_[FRAC-1], of_[FRAC-1:1]};
                oe = AMB_EXP;
            end
        end
    endtask

    integer af, bf, ae, be, pass, fail, skip;
    reg signed [FRAC-1:0] gf; reg signed [EXP-1:0] ge;

    initial begin
        pass=0; fail=0; skip=0;
        for(ae=-(1<<(EXP-1));ae<(1<<(EXP-1));ae=ae+1)
        for(be=-(1<<(EXP-1));be<(1<<(EXP-1));be=be+1)
        for(af=-(1<<(FRAC-1));af<(1<<(FRAC-1));af=af+1)
        for(bf=-(1<<(FRAC-1));bf<(1<<(FRAC-1));bf=bf+1) begin
            // Skip invalid inputs: ambiguous exp, non-N1, or zero divisor
            if(ae==AMB_EXP || be==AMB_EXP || !is_n1(af) || !is_n1(bf)) begin
                skip=skip+1;
            end else begin
                a_frac=af; a_exp=ae; b_frac=bf; b_exp=be; #2;
                gold_divide(af,bf,ae,be,gf,ge);
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
        $display("Done: %0d pass %0d fail out of %0d (skipped %0d)", pass, fail, pass+fail, skip);
        $finish;
    end
endmodule
