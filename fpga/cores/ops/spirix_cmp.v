// spirix_cmp — combinational compare
//
// Outputs: lt, eq, gt, unord (one-hot, exactly one is high)
//
// Ordering: [-↑] < [-#] < [-↓] < [0] < [+↓] < [+#] < [+↑]
// Unordered: undefined, infinity, same-sign escaped pairs
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.

module spirix_cmp #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output wire                       cmp_lt,
    output wire                       cmp_eq,
    output wire                       cmp_gt,
    output wire                       cmp_unord
);

    // ================================================================
    //  CONSTANTS
    // ================================================================
    localparam signed [MAX_EXP-1:0] AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // frac_neg1_val: all-ones in active width, zeros below (infinity fraction)
    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                               {64'hFFFFFFFFFFFFFFFF};

    // ================================================================
    //  STATE DETECTION
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire b_is_ambig = (b_exp == AMBIG_EXP);

    wire a_frac_zero = (a_frac == {MAX_FRAC{1'b0}});
    wire b_frac_zero = (b_frac == {MAX_FRAC{1'b0}});
    wire a_frac_neg1 = (a_frac == frac_neg1_val);
    wire b_frac_neg1 = (b_frac == frac_neg1_val);

    wire a_n1 = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire b_n1 = (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-2]);
    wire a_n0 = a_frac_zero | a_frac_neg1;
    wire b_n0 = b_frac_zero | b_frac_neg1;
    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire b_top3 = (b_frac[MAX_FRAC-1] == b_frac[MAX_FRAC-2]) &
                  (b_frac[MAX_FRAC-2] == b_frac[MAX_FRAC-3]);
    wire a_n2 = ~a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire b_n2 = ~b_n1 & (b_frac[MAX_FRAC-1] != b_frac[MAX_FRAC-3]);

    wire a_is_normal  = ~a_is_ambig;
    wire b_is_normal  = ~b_is_ambig;
    wire a_is_zero    = a_is_ambig & a_frac_zero;
    wire b_is_zero    = b_is_ambig & b_frac_zero;
    wire a_is_inf     = a_is_ambig & a_frac_neg1;
    wire b_is_inf     = b_is_ambig & b_frac_neg1;
    wire a_undef      = ~a_n0 & a_top3;
    wire b_undef      = ~b_n0 & b_top3;
    wire a_exploded   = a_is_ambig & a_n1;
    wire b_exploded   = b_is_ambig & b_n1;
    wire a_vanished   = a_n2;
    wire b_vanished   = b_n2;

    wire a_neg = a_frac[MAX_FRAC-1];
    wire b_neg = b_frac[MAX_FRAC-1];
    wire same_sign = (a_neg == b_neg);

    // ================================================================
    //  COMPARE LOGIC (matches Rust compare() exactly)
    // ================================================================
    reg r_lt, r_eq, r_gt, r_unord;

    always @(*) begin
        r_lt = 1'b0; r_eq = 1'b0; r_gt = 1'b0; r_unord = 1'b0;

        if (a_is_normal & b_is_normal) begin
            // Both normal
            if (same_sign) begin
                if (a_exp != b_exp) begin
                    // Different exponents: for positive, bigger exp = bigger value
                    // For negative, bigger exp = more negative = smaller value
                    if (a_neg) begin
                        // Negative: reverse — bigger exp = more negative = smaller
                        // Rust: other.exp.cmp(&self.exp) → b_exp > a_exp means a > b
                        if ($signed(b_exp) > $signed(a_exp)) r_gt = 1'b1;
                        else if ($signed(b_exp) < $signed(a_exp)) r_lt = 1'b1;
                        else r_eq = 1'b1;
                    end else begin
                        if ($signed(a_exp) > $signed(b_exp)) r_gt = 1'b1;
                        else if ($signed(a_exp) < $signed(b_exp)) r_lt = 1'b1;
                        else r_eq = 1'b1;
                    end
                end else begin
                    // Same exponent: compare fractions directly (signed)
                    if (a_frac > b_frac) r_gt = 1'b1;
                    else if (a_frac < b_frac) r_lt = 1'b1;
                    else r_eq = 1'b1;
                end
            end else begin
                // Different signs
                if (a_neg) r_lt = 1'b1;
                else       r_gt = 1'b1;
            end
        end
        else if (a_undef | a_is_inf | b_undef | b_is_inf) begin
            r_unord = 1'b1;
        end
        else if (a_is_zero & b_is_zero) begin
            r_eq = 1'b1;
        end
        else if (a_is_zero) begin
            if (b_neg) r_gt = 1'b1;
            else       r_lt = 1'b1;
        end
        else if (b_is_zero) begin
            if (a_neg) r_lt = 1'b1;
            else       r_gt = 1'b1;
        end
        else if ((a_exploded & b_exploded) | (a_vanished & b_vanished)) begin
            if (same_sign) r_unord = 1'b1;
            else if (a_neg) r_lt = 1'b1;
            else            r_gt = 1'b1;
        end
        else if (a_vanished) begin
            if (b_neg) r_gt = 1'b1;
            else       r_lt = 1'b1;
        end
        else if (b_vanished) begin
            if (a_neg) r_lt = 1'b1;
            else       r_gt = 1'b1;
        end
        else if (a_exploded) begin
            if (a_neg) r_lt = 1'b1;
            else       r_gt = 1'b1;
        end
        else begin
            // other.exploded case (b is exploded, a is normal)
            if (b_neg) r_gt = 1'b1;
            else       r_lt = 1'b1;
        end
    end

    assign cmp_lt   = r_lt;
    assign cmp_eq   = r_eq;
    assign cmp_gt   = r_gt;
    assign cmp_unord = r_unord;

endmodule
