// Exhaustive testbench for spirix_modulo (F5E4).
// Gold model: independently compute divide → floor → multiply → subtract
// using the same algorithms as the RTL modules.

`timescale 1ns/1ps
module tb;
    parameter FRAC = 5;
    parameter EXP  = 4;
    localparam PROD_BITS = 2 * FRAC - 1;
    localparam WIDE = 2 * FRAC;
    localparam INT_BITS = FRAC + 2;

    reg signed [FRAC-1:0] a_frac, b_frac;
    reg signed [EXP-1:0]  a_exp,  b_exp;
    wire signed [FRAC-1:0] r_frac;
    wire signed [EXP-1:0]  r_exp;

    spirix_modulo #(.FRAC_BITS(FRAC),.EXP_BITS(EXP)) dut(
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

    // ---- Gold: Divide (matches spirix_divide.v) ----
    task gold_divide;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg rsign, a_no, b_no;
        reg [FRAC-2:0] aa, ab;
        reg signed [EXP:0] aea, bea;
        reg [WIDE-1:0] wa, wq, wr;
        reg [FRAC:0] q, qn;
        reg qovf;
        reg [FRAC-1:0] fpr, fr;
        reg g, os, rs, s, l, ru, rovf;
        reg signed [EXP:0] ew;
        reg [FRAC-1:0] pf;
        reg signed [EXP-1:0] pe, ne;
        reg signed [FRAC-1:0] nf;
        reg niph;
        begin
            rsign = af[FRAC-1] ^ bf[FRAC-1];
            a_no = (af == NEG_ONE); b_no = (bf == NEG_ONE);
            aa = a_no ? POS_HALF[FRAC-2:0] : (af[FRAC-1] ? (~af[FRAC-2:0]+1) : af[FRAC-2:0]);
            ab = b_no ? POS_HALF[FRAC-2:0] : (bf[FRAC-1] ? (~bf[FRAC-2:0]+1) : bf[FRAC-2:0]);
            aea = $signed({{1{ae[EXP-1]}},ae}) + a_no;
            bea = $signed({{1{be[EXP-1]}},be}) + b_no;
            wa = {{(FRAC+1){1'b0}}, aa} << FRAC;
            wq = wa / {{(FRAC+1){1'b0}}, ab};
            wr = wa - wq * {{(FRAC+1){1'b0}}, ab};
            q = wq[FRAC:0]; qovf = q[FRAC];
            qn = qovf ? (q >> 1) : q;
            fpr = qn[FRAC:1]; g = qn[0];
            os = qovf & q[0]; rs = |wr; s = os | rs;
            l = fpr[0]; ru = g & (s | l);
            fr = fpr + ru;
            rovf = !fpr[FRAC-1] & fr[FRAC-1];
            pf = rovf ? POS_HALF : fr;
            ew = aea - bea + qovf + rovf;
            pe = ew[EXP-1:0];
            niph = (pf == POS_HALF);
            nf = niph ? NEG_ONE : (~pf+1);
            ne = niph ? (pe-1) : pe;
            of_ = rsign ? nf : pf; oe = rsign ? ne : pe;
            if (ew > MAX_EXP_VAL) oe = AMB_EXP;
            else if (ew < MIN_EXP_VAL) begin of_ = {of_[FRAC-1], of_[FRAC-1:1]}; oe = AMB_EXP; end
        end
    endtask

    // ---- Gold: Floor ----
    task gold_floor;
        input signed [FRAC-1:0] qf;
        input signed [EXP-1:0]  qe;
        output signed [FRAC-1:0] ff;
        output signed [EXP-1:0]  fe;
        reg signed [EXP:0] fc;
        reg [FRAC-1:0] lm, km, unit;
        reg signed [FRAC-1:0] tf, fadj, fmid;
        reg hfb, nfa;
        reg [$clog2(FRAC):0] ls;
        integer fi;
        reg fmid_n1, fzero;
        begin
            if (qe == AMB_EXP) begin
                if (qf[FRAC-1]) begin ff = NEG_ONE; fe = 0; end
                else begin ff = qf; fe = qe; end
            end
            else begin
                fc = (FRAC-1) - $signed({{1{qe[EXP-1]}},qe});
                if (fc <= 0) begin ff = qf; fe = qe; end // all integer
                else if (fc >= FRAC) begin
                    // all fractional
                    if (qf[FRAC-1]) begin ff = NEG_ONE; fe = 0; end // floor(-x) = -1
                    else begin ff = 0; fe = AMB_EXP; end // floor(+x) = 0
                end else begin
                    lm = (1 << fc[2:0]) - 1;
                    km = ~lm;
                    tf = qf & km;
                    hfb = |(qf & lm);
                    nfa = qf[FRAC-1] & hfb;
                    unit = 1 << fc[2:0];
                    fadj = tf - unit;
                    fmid = nfa ? fadj : tf;
                    fzero = (fmid == 0);
                    fmid_n1 = (fmid[FRAC-1] != fmid[FRAC-2]);

                    if (fzero) begin ff = 0; fe = AMB_EXP; end
                    else if (fmid_n1) begin ff = fmid; fe = qe; end
                    else begin
                        // Renormalize
                        ls = FRAC;
                        for (fi = FRAC-2; fi >= 0; fi = fi - 1)
                            if (fmid[fi] != fmid[FRAC-1] && ls == FRAC) ls = FRAC-1-fi;
                        ff = fmid <<< (ls - 1);
                        fe = qe - (ls - 1);
                    end
                end
            end
        end
    endtask

    // ---- Gold: Multiply (matches spirix_multiply.v) ----
    task gold_multiply;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg signed [PROD_BITS-1:0] prod, norm;
        reg signed [FRAC-1:0] fraw, frnd;
        reg ns, g, s, l, ru, rp, rn;
        reg signed [EXP:0] ew;
        begin
            prod = af * bf;
            if (prod[PROD_BITS-1] != prod[PROD_BITS-2]) begin ns = 0; norm = prod; end
            else begin ns = 1; norm = prod <<< 1; end
            fraw = norm[PROD_BITS-1 -: FRAC];
            g = norm[PROD_BITS-1-FRAC];
            s = (PROD_BITS-2-FRAC >= 0) ? |norm[PROD_BITS-2-FRAC:0] : 0;
            l = fraw[0]; ru = g & (s | l);
            frnd = fraw + ru;
            rp = !fraw[FRAC-1] & frnd[FRAC-1];
            rn = frnd[FRAC-1] & frnd[FRAC-2];
            if (rp) of_ = POS_HALF;
            else if (rn) of_ = NEG_ONE;
            else of_ = frnd;
            ew = $signed({{1{ae[EXP-1]}},ae}) + $signed({{1{be[EXP-1]}},be}) - ns + rp - rn;
            if (ew > MAX_EXP_VAL) oe = AMB_EXP;
            else if (ew < MIN_EXP_VAL) begin of_ = {of_[FRAC-1], of_[FRAC-1:1]}; oe = AMB_EXP; end
            else oe = ew[EXP-1:0];
        end
    endtask

    // ---- Gold: Add (matches spirix_add.v — single-path for gold) ----
    function [$clog2(INT_BITS):0] gl;
        input signed [INT_BITS-1:0] v;
        integer k; reg f;
        begin gl=INT_BITS; f=0;
            for(k=INT_BITS-2;k>=0;k=k-1)
                if(!f && v[k]!=v[INT_BITS-1]) begin gl=INT_BITS-1-k; f=1; end
        end
    endfunction

    task gold_add;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg signed [INT_BITS-1:0] big_ext, small_ext, small_shifted, sum, normalized;
        reg [EXP-1:0] ad;
        reg [$clog2(INT_BITS):0] ld, nsft;
        reg signed [EXP:0] ow;
        reg signed [EXP-1:0] ofs, big_exp, ofs_m1;
        reg signed [FRAC-1:0] of_raw;
        reg guard_g, lsb_g, round_up_g, alignment_sticky, extraction_sticky;
        integer si;
        begin
            if (ae >= be) begin
                big_exp = ae; big_ext = $signed(af) <<< 1;
                small_ext = $signed(bf) <<< 1; ad = ae - be;
            end else begin
                big_exp = be; big_ext = $signed(bf) <<< 1;
                small_ext = $signed(af) <<< 1; ad = be - ae;
            end
            if (ad >= FRAC) begin of_ = (ae >= be) ? af : bf; oe = big_exp; end
            else begin
                alignment_sticky = 0;
                for (si = 0; si < ad; si = si + 1) alignment_sticky = alignment_sticky | small_ext[si];
                small_shifted = small_ext >>> ad;
                sum = big_ext + small_shifted;
                if (sum == 0) begin of_ = 0; oe = AMB_EXP; end
                else begin
                    ld = gl(sum); nsft = ld - 1;
                    normalized = $unsigned(sum) << nsft;
                    of_raw = normalized[INT_BITS-1 -: FRAC];
                    guard_g = normalized[INT_BITS-1-FRAC];
                    extraction_sticky = (INT_BITS-2-FRAC >= 0) ? |normalized[INT_BITS-2-FRAC:0] : 0;
                    lsb_g = of_raw[0];
                    round_up_g = guard_g & ((alignment_sticky | extraction_sticky) | lsb_g);
                    of_ = of_raw + round_up_g;
                    ow = $signed({{1{big_exp[EXP-1]}},big_exp}) + 2 - $signed({{1'b0},ld});
                    ofs = ow[EXP-1:0]; oe = ofs;
                    ofs_m1 = ofs - 1;
                    if (big_exp[EXP-1] && !ofs_m1[EXP-1]) begin
                        of_ = {normalized[INT_BITS-1], normalized[INT_BITS-1 -: FRAC-1]};
                        oe = AMB_EXP;
                    end else begin
                        if (round_up_g && of_raw == {1'b0, {(FRAC-1){1'b1}}}) begin
                            of_ = POS_HALF; oe = oe + 1;
                        end else if (round_up_g && of_raw == {1'b1, 1'b0, {(FRAC-2){1'b1}}}) begin
                            of_ = NEG_ONE; oe = oe - 1;
                        end
                    end
                end
            end
        end
    endtask

    task gold_subtract;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg signed [FRAC-1:0] nbf;
        reg signed [EXP-1:0] nbe;
        begin
            if (bf == POS_HALF) begin nbf = NEG_ONE; nbe = be - 1; end
            else if (bf == NEG_ONE) begin nbf = POS_HALF; nbe = be + 1; end
            else begin nbf = -bf; nbe = be; end
            gold_add(af, nbf, ae, nbe, of_, oe);
        end
    endtask

    // ---- Gold: Modulo = a - floor(a/b) * b ----
    task gold_modulo;
        input signed [FRAC-1:0] af, bf;
        input signed [EXP-1:0]  ae, be;
        output signed [FRAC-1:0] of_;
        output signed [EXP-1:0]  oe;
        reg signed [FRAC-1:0] qf, ff, pf;
        reg signed [EXP-1:0]  qe, fe, pe;
        begin
            gold_divide(af, bf, ae, be, qf, qe);
            gold_floor(qf, qe, ff, fe);
            // Match DUT floor_zero: zero frac OR positive-frac with AMB_EXP
            if (ff == 0 || (fe == AMB_EXP && !ff[FRAC-1])) begin
                // floor is effectively zero, result = a itself
                of_ = af; oe = ae;
            end else begin
                gold_multiply(ff, bf, fe, be, pf, pe);
                gold_subtract(af, pf, ae, pe, of_, oe);
            end
        end
    endtask

    integer af_i, bf_i, ae_i, be_i, pass, fail, skip;
    reg signed [FRAC-1:0] gf; reg signed [EXP-1:0] ge;

    initial begin
        pass=0; fail=0; skip=0;
        for(ae_i=-(1<<(EXP-1));ae_i<(1<<(EXP-1));ae_i=ae_i+1)
        for(be_i=-(1<<(EXP-1));be_i<(1<<(EXP-1));be_i=be_i+1)
        for(af_i=-(1<<(FRAC-1));af_i<(1<<(FRAC-1));af_i=af_i+1)
        for(bf_i=-(1<<(FRAC-1));bf_i<(1<<(FRAC-1));bf_i=bf_i+1) begin
            if(ae_i==AMB_EXP || be_i==AMB_EXP || !is_n1(af_i) || !is_n1(bf_i)) begin
                skip=skip+1;
            end else begin
                a_frac=af_i; a_exp=ae_i; b_frac=bf_i; b_exp=be_i; #2;
                gold_modulo(af_i, bf_i, ae_i, be_i, gf, ge);
                if(ge==AMB_EXP) begin
                    if(r_exp===ge) pass=pass+1;
                    else begin
                        if(fail<10) $display("FAIL(amb) a=(%0d,%0d) b=(%0d,%0d): got(%0d,%0d) want_exp=%0d",
                            af_i,ae_i,bf_i,be_i,r_frac,r_exp,ge);
                        fail=fail+1;
                    end
                end else begin
                    if(r_frac===gf && r_exp===ge) pass=pass+1;
                    else begin
                        if(fail<10) $display("FAIL a=(%0d,%0d) b=(%0d,%0d): got(%0d,%0d) want(%0d,%0d)",
                            af_i,ae_i,bf_i,be_i,r_frac,r_exp,gf,ge);
                        fail=fail+1;
                    end
                end
            end
        end
        $display("Done: %0d pass %0d fail out of %0d (skipped %0d)", pass, fail, pass+fail, skip);
        $finish;
    end
endmodule
