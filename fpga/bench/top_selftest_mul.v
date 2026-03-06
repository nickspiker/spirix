// Self-test top for spirix_multiply_pipe2 on Colorlight 5A-75B (ECP5-25F)
//
// Consensus test: 7 identical multiply_pipe2 instances.
// Input chain: m[i] gets inputs delayed by i cycles vs m[0].
// Reference chain: m[0]'s output delayed by i cycles compared to m[i].
// Any disagreement = fail.
//
// LED: off=no lock, fast blink=all agree, slow blink=mismatch.
// 7 copies × 4 DSP18 = 28 DSP18 (100% of ECP5-25F)

module top_selftest (
    input  wire clk,    // 25 MHz board clock (P6)
    output reg  led     // status LED (T6)
);

    // =========================================================================
    // Clock: use PLL if defines are set, otherwise raw 25 MHz
    // =========================================================================
    wire sys_clk, pll_lock;

`ifdef PLL_CLKFB_DIV
    (* ICP_CURRENT="12" *) (* LPF_RESISTOR="8" *)
    (* MFG_ENABLE_FILTEROPAMP="1" *) (* MFG_GMCREF_SEL="2" *)
    EHXPLLL #(
        .PLLRST_ENA       ("DISABLED"),
        .INTFB_WAKE       ("DISABLED"),
        .STDBY_ENABLE     ("DISABLED"),
        .DPHASE_SOURCE    ("DISABLED"),
        .OUTDIVIDER_MUXA  ("DIVA"),
        .OUTDIVIDER_MUXB  ("DIVB"),
        .OUTDIVIDER_MUXC  ("DIVC"),
        .OUTDIVIDER_MUXD  ("DIVD"),
        .CLKI_DIV         (`PLL_CLKI_DIV),
        .CLKFB_DIV        (`PLL_CLKFB_DIV),
        .CLKOP_ENABLE     ("ENABLED"),
        .CLKOP_DIV        (`PLL_CLKOP_DIV),
        .CLKOP_CPHASE     (`PLL_CLKOP_CPHASE),
        .CLKOP_FPHASE     (0),
        .FEEDBK_PATH      ("CLKOP")
    ) pll (
        .RST          (1'b0),
        .STDBY        (1'b0),
        .CLKI         (clk),
        .CLKOP        (sys_clk),
        .CLKFB        (sys_clk),
        .CLKINTFB     (),
        .PHASESEL0    (1'b0),
        .PHASESEL1    (1'b0),
        .PHASEDIR     (1'b1),
        .PHASESTEP    (1'b1),
        .PHASELOADREG (1'b1),
        .PLLWAKESYNC  (1'b0),
        .ENCLKOP      (1'b0),
        .LOCK         (pll_lock)
    );
`else
    assign sys_clk  = clk;
    assign pll_lock = 1'b1;
`endif

    // =========================================================================
    // PRNG: two xorshift32 instances
    // =========================================================================
    reg [31:0] rng_a = 32'hCAFE_BABE;
    reg [31:0] rng_b = 32'hDEAD_BEEF;

    wire [31:0] xa1 = rng_a ^ (rng_a << 13);
    wire [31:0] xa2 = xa1  ^ (xa1  >> 17);
    wire [31:0] next_a = xa2 ^ (xa2 << 5);

    wire [31:0] xb1 = rng_b ^ (rng_b << 13);
    wire [31:0] xb2 = xb1  ^ (xb1  >> 17);
    wire [31:0] next_b = xb2 ^ (xb2 << 5);

    always @(posedge sys_clk) begin
        rng_a <= next_a;
        rng_b <= next_b;
    end

    // =========================================================================
    // Generate N1-normalized test vectors
    // =========================================================================
    localparam FRAC = 25;
    localparam EXP  = 8;
    localparam N    = 7;

    wire signed [FRAC-1:0] a_frac = rng_a[0]
        ? {2'b10, rng_a[FRAC-2:1]}
        : {2'b01, rng_a[FRAC-2:1]};
    wire signed [EXP-1:0] raw_a_exp = rng_a[FRAC+EXP-1:FRAC];
    wire signed [EXP-1:0] a_exp = (raw_a_exp == -8'sd128) ? -8'sd127 : raw_a_exp;

    wire signed [FRAC-1:0] b_frac = rng_b[0]
        ? {2'b10, rng_b[FRAC-2:1]}
        : {2'b01, rng_b[FRAC-2:1]};
    wire signed [EXP-1:0] raw_b_exp = rng_b[FRAC+EXP-1:FRAC];
    wire signed [EXP-1:0] b_exp = (raw_b_exp == -8'sd128) ? -8'sd127 : raw_b_exp;

    // =========================================================================
    // Input delay chain: m[i] gets inputs delayed by i cycles
    // =========================================================================
    reg signed [FRAC-1:0] in_af [0:N-1];
    reg signed [EXP-1:0]  in_ae [0:N-1];
    reg signed [FRAC-1:0] in_bf [0:N-1];
    reg signed [EXP-1:0]  in_be [0:N-1];

    integer j;
    always @(posedge sys_clk) begin
        in_af[0] <= a_frac;  in_ae[0] <= a_exp;
        in_bf[0] <= b_frac;  in_be[0] <= b_exp;
        for (j = 1; j < N; j = j + 1) begin
            in_af[j] <= in_af[j-1];  in_ae[j] <= in_ae[j-1];
            in_bf[j] <= in_bf[j-1];  in_be[j] <= in_be[j-1];
        end
    end

    // =========================================================================
    // 7 multiply_pipe2 instances, each with staggered inputs
    // =========================================================================
    wire signed [FRAC-1:0] out_f [0:N-1];
    wire signed [EXP-1:0]  out_e [0:N-1];

    genvar gi;
    generate
        for (gi = 0; gi < N; gi = gi + 1) begin : mul
            spirix_multiply_pipe2 #(.FRAC_BITS(FRAC), .EXP_BITS(EXP)) inst (
                .clk(sys_clk),
                .a_frac(in_af[gi]), .a_exp(in_ae[gi]),
                .b_frac(in_bf[gi]), .b_exp(in_be[gi]),
                .negate(1'b0),
                .result_frac(out_f[gi]), .result_exp(out_e[gi])
            );
        end
    endgenerate

    // =========================================================================
    // Reference delay chain: m[0] output delayed by 1..6 cycles
    // m[i] output at time T should == m[0] output at time T-i
    // =========================================================================
    reg signed [FRAC-1:0] ref_f [1:N-1];
    reg signed [EXP-1:0]  ref_e [1:N-1];

    always @(posedge sys_clk) begin
        ref_f[1] <= out_f[0];  ref_e[1] <= out_e[0];
        for (j = 2; j < N; j = j + 1) begin
            ref_f[j] <= ref_f[j-1];  ref_e[j] <= ref_e[j-1];
        end
    end

    // =========================================================================
    // Comparator: m[i] output must match m[0] output delayed by i cycles
    // =========================================================================
    reg [3:0] warmup = 0;
    reg fail = 0;

    wire mismatch = (out_f[1] != ref_f[1]) || (out_e[1] != ref_e[1])
                  || (out_f[2] != ref_f[2]) || (out_e[2] != ref_e[2])
                  || (out_f[3] != ref_f[3]) || (out_e[3] != ref_e[3])
                  || (out_f[4] != ref_f[4]) || (out_e[4] != ref_e[4])
                  || (out_f[5] != ref_f[5]) || (out_e[5] != ref_e[5])
                  || (out_f[6] != ref_f[6]) || (out_e[6] != ref_e[6]);

    always @(posedge sys_clk) begin
        if (!pll_lock)
            warmup <= 0;
        else if (warmup < 15)
            warmup <= warmup + 1;
        else if (mismatch)
            fail <= 1;
    end

    // =========================================================================
    // LED: off=no lock, ~20Hz blink=pass, ~1Hz blink=fail
    // =========================================================================
    reg [29:0] blink_ctr = 0;
    always @(posedge sys_clk) blink_ctr <= blink_ctr + 1;

    always @(posedge sys_clk) begin
        if (!pll_lock)
            led <= 1;                              // off (active-low)
        else if (fail)
            led <= (blink_ctr[29:27] != 3'b000);   // ~1 Hz, 1/8 duty
        else
            led <= blink_ctr[27];                  // ~2.6 Hz @700M, 50% duty
    end

endmodule
