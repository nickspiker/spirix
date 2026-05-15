// divide_step_chain — PARALLEL trial-subtract steps for restoring division.
//
// Extracted from spirix_divide.v as a self-contained module so the trial-sub
// chain can be optimized in place (alternative algorithms, placement hints,
// etc.) without touching the surrounding FSM.
//
// Algorithm (restoring, radix-2):
//   step k:  trial = step_in[k] - d
//            q_bit = (trial >= 0)
//            step_out[k] = q_bit ? trial : step_in[k]   ← output mux
//   step k+1 input = step_out[k] << 1
//
// The output mux is the per-stage tail. A non-restoring variant
// (divide_step_chain_nr) eliminates it; see that module for details.

(* keep_hierarchy = "no" *)
module divide_step_chain #(
    parameter COMPUTE_FRAC = 25,
    parameter PARALLEL     = 4
)(
    input  wire                          starting,   // first cycle: r_in unused, abs_a_q used
    input  wire [COMPUTE_FRAC-2:0]       abs_a_q,    // |numerator| (Q form, 24-bit)
    input  wire [COMPUTE_FRAC-2:0]       d,          // |denominator| (Q form, 24-bit)
    input  wire [COMPUTE_FRAC-1:0]       r_in,       // remainder from prior cycle (25-bit)
    output wire [COMPUTE_FRAC-1:0]       r_next,     // remainder for next cycle
    output wire [PARALLEL-1:0]           q_chunk     // PARALLEL quotient bits (MSB first)
);
    wire [COMPUTE_FRAC-1:0] step_in  [0:PARALLEL-1];
    wire [COMPUTE_FRAC-1:0] step_out [0:PARALLEL-1];
    wire [PARALLEL-1:0]     step_ge;

    assign step_in[0]  = starting ? {1'b0, abs_a_q}
                                  : {r_in[COMPUTE_FRAC-2:0], 1'b0};
    wire [COMPUTE_FRAC:0] trial0 = {1'b0, step_in[0]} - {2'b00, d};
    assign step_ge[0]  = !trial0[COMPUTE_FRAC];
    assign step_out[0] = step_ge[0] ? trial0[COMPUTE_FRAC-1:0] : step_in[0];

    genvar k;
    generate
        for (k = 1; k < PARALLEL; k = k + 1) begin : ts_chain
            assign step_in[k] = {step_out[k-1][COMPUTE_FRAC-2:0], 1'b0};
            wire [COMPUTE_FRAC:0] trial_k = {1'b0, step_in[k]} - {2'b00, d};
            assign step_ge[k] = !trial_k[COMPUTE_FRAC];
            assign step_out[k] = step_ge[k] ? trial_k[COMPUTE_FRAC-1:0] : step_in[k];
        end
    endgenerate

    assign r_next = step_out[PARALLEL-1];

    generate
        for (k = 0; k < PARALLEL; k = k + 1) begin : qpack
            assign q_chunk[PARALLEL-1-k] = step_ge[k];
        end
    endgenerate

endmodule
