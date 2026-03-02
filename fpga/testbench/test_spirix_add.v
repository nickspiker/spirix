// Exhaustive testbench for spirix_add (parameterized).
// Uses F5E4 (5-bit frac, 4-bit exp) for full enumeration.
// Gold model at DUT precision (FRAC+2 internal bits): single-path add
// with banker's rounding and rounding overflow correction.

`timescale 1ns/1ps
module tb;
    parameter FRAC = 5;
    parameter EXP  = 4;
    localparam INT_BITS = FRAC + 2; // same as DUT internal width

    reg signed [FRAC-1:0] a_frac, b_frac;
    reg signed [EXP-1:0]  a_exp,  b_exp;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;

    spirix_add #(.FRAC_BITS(FRAC),.EXP_BITS(EXP)) dut(
        .a_frac(a_frac),.a_exp(a_exp),.b_frac(b_frac),.b_exp(b_exp),
        .result_frac(r_frac),.result_exp(r_exp));

    // Leading same count for gold model (on INT_BITS-wide value)
    function [$clog2(INT_BITS):0] gl;
        input signed [INT_BITS-1:0] v;
        integer k; reg f;
        begin gl=INT_BITS; f=0;
            for(k=INT_BITS-2;k>=0;k=k-1)
                if(!f && v[k]!=v[INT_BITS-1]) begin gl=INT_BITS-1-k; f=1; end
        end
    endfunction

    // Gold model: single-path add at DUT precision (FRAC+2 bits)
    // Matches DUT internal width — alignment loss and rounding are identical.
    task gold;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg signed [INT_BITS-1:0] big_ext, small_ext, small_shifted, sum;
        reg signed [INT_BITS-1:0] normalized;
        reg [EXP-1:0] ad;
        reg [$clog2(INT_BITS):0] ld;
        reg [$clog2(INT_BITS):0] norm_shift;
        reg signed [EXP:0] ow;
        reg signed [EXP-1:0] ofs;
        reg signed [EXP-1:0] big_exp;
        reg signed [FRAC-1:0] big_frac;
        reg guard_g, lsb_g, round_up_g;
        reg alignment_sticky;
        reg extraction_sticky;
        reg signed [FRAC-1:0] of_raw;
        reg signed [EXP-1:0] ofs_m1;
        integer si;
        begin
            // Sort by exponent
            if (ae >= be) begin
                big_frac = af; big_exp = ae;
                big_ext = $signed(af) <<< 1;
                small_ext = $signed(bf) <<< 1;
                ad = ae - be;
            end else begin
                big_frac = bf; big_exp = be;
                big_ext = $signed(bf) <<< 1;
                small_ext = $signed(af) <<< 1;
                ad = be - ae;
            end

            if (ad >= FRAC) begin
                // smaller is negligible — return big
                of_ = big_frac;
                oe  = big_exp;
            end else begin
                // Align small: right shift by ad, track sticky from lost bits
                alignment_sticky = 0;
                for (si = 0; si < ad; si = si + 1)
                    alignment_sticky = alignment_sticky | small_ext[si];
                small_shifted = small_ext >>> ad;

                sum = big_ext + small_shifted;

                if (sum == 0) begin
                    of_ = 0;
                    oe  = 1 << (EXP-1); // AMBIGUOUS_EXP
                end else begin
                    ld = gl(sum);
                    norm_shift = ld - 1;
                    normalized = $unsigned(sum) << norm_shift;

                    // Extract top FRAC bits
                    of_raw = normalized[INT_BITS-1 -: FRAC];

                    // Banker's rounding: guard + combined sticky
                    guard_g = normalized[INT_BITS - 1 - FRAC];
                    extraction_sticky = (INT_BITS - 2 - FRAC >= 0) ?
                                        |normalized[INT_BITS - 2 - FRAC:0] : 1'b0;
                    lsb_g = of_raw[0];
                    round_up_g = guard_g & ((alignment_sticky | extraction_sticky) | lsb_g);
                    of_ = of_raw + round_up_g;

                    // Exponent
                    ow = $signed({{1{big_exp[EXP-1]}}, big_exp}) + 2 - $signed({{1'b0}, ld});
                    ofs = ow[EXP-1:0];
                    oe = ofs;

                    // Underflow check
                    ofs_m1 = ofs - 1;
                    if (big_exp[EXP-1] && !ofs_m1[EXP-1]) begin
                        of_ = {normalized[INT_BITS-1],
                               normalized[INT_BITS-1 -: FRAC-1]};
                        oe  = 1 << (EXP-1);
                    end else begin
                        // Rounding overflow renormalization
                        if (round_up_g && of_raw == {1'b0, {(FRAC-1){1'b1}}}) begin
                            of_ = {1'b0, 1'b1, {(FRAC-2){1'b0}}};
                            oe  = oe + 1;
                        end else if (round_up_g && of_raw == {1'b1, 1'b0, {(FRAC-2){1'b1}}}) begin
                            of_ = {1'b1, {(FRAC-1){1'b0}}};
                            oe  = oe - 1;
                        end
                    end
                end
            end
        end
    endtask

    integer af, bf, ae, be, pass, fail;
    reg signed [FRAC-1:0] gf; reg signed [EXP-1:0] ge;

    localparam AMB_EXP = -(1<<(EXP-1));

    function is_n1;
        input signed [FRAC-1:0] frac;
        begin
            is_n1 = (frac[FRAC-1] != frac[FRAC-2]);
        end
    endfunction

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
                gold(af,bf,ae,be,gf,ge);
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
