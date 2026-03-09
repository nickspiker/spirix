// spirix_floor — combinational floor (largest integer ≤ value)
//
// Edge cases:
//   vanished positive → ZERO
//   vanished negative → NEG_ONE
//   exploded/undef/inf/zero → passthrough
//   normal, exp ≤ 0 → ZERO (positive) or NEG_ONE (negative)
//   normal, exp ≥ frac_bits-1 → passthrough (already integer)
//   normal, otherwise → mask off fractional bits
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.

module spirix_floor #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    output wire signed [MAX_FRAC-1:0] result_frac,
    output wire signed [MAX_EXP-1:0]  result_exp
);

    // ================================================================
    //  CONSTANTS
    // ================================================================
    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {2'b01, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    wire [6:0] frac_bits = (frac_width == 2'd0) ? 7'd8  :
                            (frac_width == 2'd1) ? 7'd16 :
                            (frac_width == 2'd2) ? 7'd32 : 7'd64;

    // ================================================================
    //  STATE DETECTION
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire a_is_normal = ~a_is_ambig;
    wire a_neg = a_frac[MAX_FRAC-1];

    wire a_n1 = (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-2]);
    wire a_n2 = ~a_n1 & (a_frac[MAX_FRAC-1] != a_frac[MAX_FRAC-3]);
    wire a_vanished = a_n2;

    // ================================================================
    //  FLOOR MASK (zero out bits below integer boundary)
    //  For MSB-aligned fraction with exponent e:
    //    integer bits = top (e+1) bits, fractional = rest
    //    mask = ~((1 << (frac_bits - 1 - e)) - 1) shifted to MSB position
    // ================================================================
    // frac_bits_m1 = active fraction bits - 1 (= integer capacity)
    wire [6:0] frac_bits_m1 = frac_bits - 7'd1;

    // How many low bits to zero out (in the 64-bit field):
    // sub_bits = (64 - frac_bits) + (frac_bits - 1 - exp) = 63 - exp
    // But we need to account for MSB alignment: actual zero count = 64 - (exp + 1) = 63 - exp
    // Clamp exp to valid range [1, frac_bits_m1-1]
    wire signed [MAX_EXP-1:0] exp_clamped = a_exp;
    wire [6:0] exp_small = exp_clamped[6:0];  // safe — we only use when exp is in range

    // Number of bits to keep (from MSB): exp + 1 (sign bit + integer bits)
    // Number of bits to zero (from LSB): 64 - (exp + 1) = 63 - exp
    // But fraction is MSB-aligned in 64 bits, so mask depends on exp value
    wire [6:0] zero_count = 7'd63 - exp_small;

    // Generate mask: all ones in top (64 - zero_count) bits, zeros below
    // We do this with a barrel-style shift of all-ones
    wire [MAX_FRAC-1:0] ones = {MAX_FRAC{1'b1}};
    wire [MAX_FRAC-1:0] floor_mask = ones << zero_count;

    // Exp is within maskable range? (0 < exp < frac_bits - 1)
    wire exp_in_range = a_is_normal & ~a_exp[MAX_EXP-1] & (a_exp > 0)
                        & ($signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}) > a_exp);

    // Exp ≤ 0 but normal (magnitude < 1)
    wire exp_sub_one = a_is_normal & (a_exp <= $signed({{(MAX_EXP-1){1'b0}}, 1'b0}));

    // Exp ≥ frac_bits-1 (already an integer, no fractional bits)
    wire exp_big = a_is_normal & (a_exp >= $signed({{(MAX_EXP-7){1'b0}}, frac_bits_m1}));

    // ================================================================
    //  OUTPUT MUX
    // ================================================================
    reg signed [MAX_FRAC-1:0] r_frac;
    reg signed [MAX_EXP-1:0]  r_exp;

    always @(*) begin
        if (~a_is_normal) begin
            if (a_vanished) begin
                if (a_neg) begin
                    // Vanished negative → NEG_ONE (-1)
                    r_frac = NEG_ONE;
                    r_exp  = {MAX_EXP{1'b0}};  // exp = 0 for -1
                end else begin
                    // Vanished positive → ZERO
                    r_frac = {MAX_FRAC{1'b0}};
                    r_exp  = AMBIG_EXP;
                end
            end else begin
                // Zero, infinity, exploded, undefined → passthrough
                r_frac = a_frac;
                r_exp  = a_exp;
            end
        end else if (exp_sub_one) begin
            // Normal, exp ≤ 0: magnitude < 1
            if (a_neg) begin
                r_frac = NEG_ONE;
                r_exp  = {MAX_EXP{1'b0}};  // exp = 0
            end else begin
                r_frac = {MAX_FRAC{1'b0}};
                r_exp  = AMBIG_EXP;
            end
        end else if (exp_big) begin
            // Already an integer → passthrough
            r_frac = a_frac;
            r_exp  = a_exp;
        end else begin
            // Maskable range: zero out fractional bits
            r_frac = a_frac & floor_mask;
            r_exp  = a_exp;
        end
    end

    assign result_frac = r_frac;
    assign result_exp  = r_exp;

endmodule
