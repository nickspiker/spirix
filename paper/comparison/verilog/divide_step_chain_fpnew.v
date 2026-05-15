// divide_step_chain_fpnew — non-restoring radix-2 chain using FPnew-style cells.
//
// Architecturally mirrors FPnew's div_sqrt_mvp iteration chain:
//   - Per stage: r_new = (r << 1) + (prev_sign ? +d : -d)
//   - Stage's add/sub polarity selected by previous stage's sign bit (MSB)
//   - No output mux per stage (vs restoring) — sign-select happens at adder INPUT
//
// Width: r is COMPUTE_FRAC+1 bits two's-complement signed (1 sign + 25 magnitude).
// Outputs r_next truncated to COMPUTE_FRAC bits at the boundary.
//
// NOTE: this chain produces non-restoring quotient encoding (q_chunk bits where
// 1 = "subtracted this step" and 0 = "added this step"). Surrounding spirix_divide
// finalize must convert raw → standard binary if bit-exactness vs restoring is
// required. For silicon Fmax bench (gold vs test phase consistency), no such
// conversion is needed — both phases produce identical outputs deterministically.

module divide_step_chain_fpnew #(
    parameter COMPUTE_FRAC = 25,
    parameter PARALLEL     = 3
)(
    input  wire                          starting,
    input  wire [COMPUTE_FRAC-2:0]       abs_a_q,
    input  wire [COMPUTE_FRAC-2:0]       d,
    input  wire [COMPUTE_FRAC-1:0]       r_in,
    output wire [COMPUTE_FRAC-1:0]       r_next,
    output wire [PARALLEL-1:0]           q_chunk
);
    localparam W = COMPUTE_FRAC + 1;   // 26-bit signed working width

    // Sign-extend r_in (always positive in this harness — top of restoring's r_reg).
    // On starting cycle, replace with abs_a_q sign-extended.
    wire signed [W-1:0] r_init = starting ? $signed({2'b00, abs_a_q})
                                          : $signed({1'b0,  r_in});
    wire        [W-1:0] d_ext  = {2'b00, d};

    wire signed [W-1:0] step_r [0:PARALLEL];
    wire        [PARALLEL-1:0] q_raw;
    assign step_r[0] = r_init;

    genvar k;
    generate
        for (k = 0; k < PARALLEL; k = k + 1) begin : nr_chain
            wire prev_neg = step_r[k][W-1];                       // sign of input r
            wire [W-1:0] r_shifted = {step_r[k][W-2:0], 1'b0};    // 2*r in W bits
            // FPnew trick: B = (sign ? d : ~d), Cin = !sign
            //   sign=0 (non-neg): r_shifted + ~d + 1 = r_shifted - d
            //   sign=1 (neg):     r_shifted + d  + 0 = r_shifted + d
            wire [W-1:0] cell_b = prev_neg ? d_ext : ~d_ext;
            wire [1:0]   cell_d_in = {1'b0, ~prev_neg};
            wire [W-1:0] cell_sum;
            wire [1:0]   cell_d_out;
            wire         cell_cout;

            iter_cell_fpnew #(.WIDTH(W)) cell (
                .A_DI            (r_shifted),
                .B_DI            (cell_b),
                .Div_enable_SI   (1'b0),
                .Div_start_dly_SI(1'b0),
                .Sqrt_enable_SI  (1'b1),
                .D_DI            (cell_d_in),
                .D_DO            (cell_d_out),
                .Sum_DO          (cell_sum),
                .Carry_out_DO    (cell_cout)
            );

            assign step_r[k+1] = $signed(cell_sum);
            assign q_raw[k]    = ~prev_neg;     // 1 if we subtracted
        end
    endgenerate

    // Truncate signed 26-bit r back to 25-bit at chain output. Top bit
    // discarded — for Fmax bench this is fine (deterministic).
    assign r_next = step_r[PARALLEL][COMPUTE_FRAC-1:0];

    generate
        for (k = 0; k < PARALLEL; k = k + 1) begin : qpack
            assign q_chunk[PARALLEL-1-k] = q_raw[k];
        end
    endgenerate
endmodule
