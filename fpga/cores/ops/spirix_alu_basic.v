// spirix_alu_basic — combinational multi-op ALU for trivial operations
//
// Ops (3-bit select):
//   0: NEG(a)      — negate
//   1: ABS(a)      — absolute value
//   2: SIGN(a)     — sign function: +1, -1, or undefined
//   3: SHL(a, b)   — shift left (a.exp += b.exp as integer shift)
//   4: SHR(a, b)   — shift right (a.exp -= b.exp as integer shift)
//
// Runtime-selectable precision:
//   frac_width[1:0]: 00=8 01=16 10=32 11=64
//   exp_width[1:0]:  00=8 01=16 10=32 11=64
//
// Fractions MSB-aligned (Spirix sa() convention).
// Exponents LSB-aligned (plain integer). Full edge case handling.

module spirix_alu_basic #(
    parameter MAX_FRAC = 64,
    parameter MAX_EXP  = 64
)(
    input  wire [2:0]                 op,
    input  wire [1:0]                 frac_width,
    input  wire [1:0]                 exp_width,
    input  wire signed [MAX_FRAC-1:0] a_frac,
    input  wire signed [MAX_EXP-1:0]  a_exp,
    input  wire signed [MAX_FRAC-1:0] b_frac,
    input  wire signed [MAX_EXP-1:0]  b_exp,
    output wire signed [MAX_FRAC-1:0] result_frac,
    output wire signed [MAX_EXP-1:0]  result_exp
);

    // ================================================================
    //  CONSTANTS
    // ================================================================
    localparam signed [MAX_FRAC-1:0] NEG_ONE   = {1'b1, {(MAX_FRAC-1){1'b0}}};
    localparam signed [MAX_FRAC-1:0] POS_HALF  = {2'b01, {(MAX_FRAC-2){1'b0}}};
    localparam signed [MAX_EXP-1:0]  AMBIG_EXP = {1'b1, {(MAX_EXP-1){1'b0}}};

    // Undefined prefix constants (MSB-aligned from 8-bit patterns)
    // SIGN_INDETERMINATE = 0xE4 = 0b11100100
    localparam signed [MAX_FRAC-1:0] UNDEF_SIGN = {8'hE4, {(MAX_FRAC-8){1'b0}}};

    // ================================================================
    //  NEG — instantiate spirix_neg
    // ================================================================
    wire signed [MAX_FRAC-1:0] neg_frac;
    wire signed [MAX_EXP-1:0]  neg_exp;

    spirix_neg #(.MAX_FRAC(MAX_FRAC), .MAX_EXP(MAX_EXP)) u_neg (
        .frac_width(frac_width), .exp_width(exp_width),
        .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(neg_frac), .result_exp(neg_exp)
    );

    // ================================================================
    //  ABS = neg if negative, else passthrough
    // ================================================================
    wire signed [MAX_FRAC-1:0] abs_frac = a_frac[MAX_FRAC-1] ? neg_frac : a_frac;
    wire signed [MAX_EXP-1:0]  abs_exp  = a_frac[MAX_FRAC-1] ? neg_exp  : a_exp;

    // ================================================================
    //  SIGN detection (for SIGN op)
    // ================================================================
    wire a_is_ambig = (a_exp == AMBIG_EXP);
    wire a_frac_zero = (a_frac == {MAX_FRAC{1'b0}});

    // frac_neg1_val: all-ones in active width
    wire signed [MAX_FRAC-1:0] frac_neg1_val =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                               {64'hFFFFFFFFFFFFFFFF};
    wire a_frac_neg1 = (a_frac == frac_neg1_val);
    wire a_n0 = a_frac_zero | a_frac_neg1;

    // Undefined detection (top 3 bits uniform, not n0)
    wire a_top3 = (a_frac[MAX_FRAC-1] == a_frac[MAX_FRAC-2]) &
                  (a_frac[MAX_FRAC-2] == a_frac[MAX_FRAC-3]);
    wire a_undef = ~a_n0 & a_top3;

    wire a_neg = a_frac[MAX_FRAC-1];

    // SIGN result
    reg signed [MAX_FRAC-1:0] sign_frac;
    reg signed [MAX_EXP-1:0]  sign_exp;

    always @(*) begin
        if (a_undef & a_is_ambig) begin
            // Undefined → passthrough
            sign_frac = a_frac;
            sign_exp  = a_exp;
        end else if (a_n0 & a_is_ambig) begin
            // Zero or infinity → SIGN_INDETERMINATE
            sign_frac = UNDEF_SIGN;
            sign_exp  = AMBIG_EXP;
        end else if (a_neg) begin
            // Negative (normal, vanished, or exploded) → NEG_ONE @ exp=0
            sign_frac = NEG_ONE;
            sign_exp  = {MAX_EXP{1'b0}};
        end else begin
            // Positive (normal, vanished, or exploded) → POS_HALF @ exp=1
            sign_frac = POS_HALF;
            sign_exp  = {{(MAX_EXP-1){1'b0}}, 1'b1};
        end
    end

    // ================================================================
    //  SHL / SHR — exponent shift with overflow detection
    // ================================================================
    // SHL: new_exp = a_exp + b_exp
    // SHR: new_exp = a_exp - b_exp
    // Non-normal a → passthrough. Overflow → explode/vanish.

    wire a_is_normal = ~a_is_ambig;

    wire signed [MAX_EXP-1:0] shl_exp = a_exp + b_exp;
    wire signed [MAX_EXP-1:0] shr_exp = a_exp - b_exp;

    // Active sign bits of operands and results at each exp width
    // exp_width: 0=8, 1=16, 2=32, 3=64
    wire a_active_sign = (exp_width == 2'd0) ? a_exp[7]  :
                         (exp_width == 2'd1) ? a_exp[15] :
                         (exp_width == 2'd2) ? a_exp[31] : a_exp[63];
    wire b_active_sign = (exp_width == 2'd0) ? b_exp[7]  :
                         (exp_width == 2'd1) ? b_exp[15] :
                         (exp_width == 2'd2) ? b_exp[31] : b_exp[63];

    // Check if result fits in active exp width (sign extension from active sign bit)
    // For N-bit: bits [63:N-1] must all match bit [N-1]
    wire shl_fits_e3 = &(shl_exp[63:7]  ^~ {57{shl_exp[7]}});
    wire shl_fits_e4 = &(shl_exp[63:15] ^~ {49{shl_exp[15]}});
    wire shl_fits_e5 = &(shl_exp[63:31] ^~ {33{shl_exp[31]}});
    // 64-bit: standard signed addition overflow (both same sign, result flips)
    wire shl_ovf_e6 = (a_exp[63] == b_exp[63]) & (a_exp[63] != shl_exp[63]);
    wire shl_fits = (exp_width == 2'd0) ? shl_fits_e3 :
                    (exp_width == 2'd1) ? shl_fits_e4 :
                    (exp_width == 2'd2) ? shl_fits_e5 : ~shl_ovf_e6;

    wire shr_fits_e3 = &(shr_exp[63:7]  ^~ {57{shr_exp[7]}});
    wire shr_fits_e4 = &(shr_exp[63:15] ^~ {49{shr_exp[15]}});
    wire shr_fits_e5 = &(shr_exp[63:31] ^~ {33{shr_exp[31]}});
    // 64-bit: standard signed subtraction overflow (different signs, result flips from a)
    wire shr_ovf_e6 = (a_exp[63] != b_exp[63]) & (a_exp[63] != shr_exp[63]);
    wire shr_fits = (exp_width == 2'd0) ? shr_fits_e3 :
                    (exp_width == 2'd1) ? shr_fits_e4 :
                    (exp_width == 2'd2) ? shr_fits_e5 : ~shr_ovf_e6;

    // Check if result IS AMBIG at active width
    wire shl_is_ambig_e3 = shl_exp[7]  & ~|shl_exp[6:0]   & &shl_exp[63:8];
    wire shl_is_ambig_e4 = shl_exp[15] & ~|shl_exp[14:0]  & &shl_exp[63:16];
    wire shl_is_ambig_e5 = shl_exp[31] & ~|shl_exp[30:0]  & &shl_exp[63:32];
    wire shl_is_ambig = (exp_width == 2'd0) ? shl_is_ambig_e3 :
                        (exp_width == 2'd1) ? shl_is_ambig_e4 :
                        (exp_width == 2'd2) ? shl_is_ambig_e5 :
                                              (shl_exp == AMBIG_EXP);

    wire shr_is_ambig_e3 = shr_exp[7]  & ~|shr_exp[6:0]   & &shr_exp[63:8];
    wire shr_is_ambig_e4 = shr_exp[15] & ~|shr_exp[14:0]  & &shr_exp[63:16];
    wire shr_is_ambig_e5 = shr_exp[31] & ~|shr_exp[30:0]  & &shr_exp[63:32];
    wire shr_is_ambig = (exp_width == 2'd0) ? shr_is_ambig_e3 :
                        (exp_width == 2'd1) ? shr_is_ambig_e4 :
                        (exp_width == 2'd2) ? shr_is_ambig_e5 :
                                              (shr_exp == AMBIG_EXP);

    // Overflow = doesn't fit OR hits AMBIG
    wire shl_overflow = ~shl_fits | shl_is_ambig;
    wire shr_overflow = ~shr_fits | shr_is_ambig;

    // Direction: both operands' active signs determine explode vs vanish
    // Positive overflow (both non-neg) → explode: {frac, AMBIG}
    // Negative overflow (both neg) → vanish: {frac >> 1, AMBIG}
    // For SHL: shift = b_exp. Positive ovf = a>=0 && b>=0. Negative ovf = a<0 && b<0.
    // For SHR: shift = -b_exp effectively. Positive ovf = a>=0 && b<0. Negative ovf = a<0 && b>=0.
    wire shl_pos_ovf = shl_overflow & ~a_active_sign & ~b_active_sign;
    wire shl_neg_ovf = shl_overflow &  a_active_sign &  b_active_sign;
    // For SHR, new_exp = a_exp - b_exp, so overflow direction flips on b's sign
    wire shr_pos_ovf = shr_overflow & ~a_active_sign &  b_active_sign;
    wire shr_neg_ovf = shr_overflow &  a_active_sign & ~b_active_sign;

    // Arithmetic right shift by 1 (for vanish case)
    // Manual ASR avoids Verilog signedness context issues with >>>.
    // Must mask to active frac width — 64-bit shift leaks bits into padding.
    wire [MAX_FRAC-1:0] frac_active_mask =
        (frac_width == 2'd0) ? {8'hFF,  56'b0} :
        (frac_width == 2'd1) ? {16'hFFFF, 48'b0} :
        (frac_width == 2'd2) ? {32'hFFFFFFFF, 32'b0} :
                               {64'hFFFFFFFFFFFFFFFF};
    wire signed [MAX_FRAC-1:0] a_frac_shr1 =
        {a_frac[MAX_FRAC-1], a_frac[MAX_FRAC-1:1]} & frac_active_mask;

    // SHL result
    reg signed [MAX_FRAC-1:0] shl_frac;
    reg signed [MAX_EXP-1:0]  shl_result_exp;

    always @(*) begin
        if (~a_is_normal) begin
            shl_frac       = a_frac;
            shl_result_exp = a_exp;
        end else if (shl_pos_ovf) begin
            // Explode: fraction unchanged, exp = AMBIG
            shl_frac       = a_frac;
            shl_result_exp = AMBIG_EXP;
        end else if (shl_neg_ovf) begin
            // Vanish: fraction >> 1, exp = AMBIG
            shl_frac       = a_frac_shr1;
            shl_result_exp = AMBIG_EXP;
        end else begin
            shl_frac       = a_frac;
            shl_result_exp = shl_exp;
        end
    end

    // SHR result
    reg signed [MAX_FRAC-1:0] shr_frac;
    reg signed [MAX_EXP-1:0]  shr_result_exp;

    always @(*) begin
        if (~a_is_normal) begin
            shr_frac       = a_frac;
            shr_result_exp = a_exp;
        end else if (shr_pos_ovf) begin
            shr_frac       = a_frac;
            shr_result_exp = AMBIG_EXP;
        end else if (shr_neg_ovf) begin
            shr_frac       = a_frac_shr1;
            shr_result_exp = AMBIG_EXP;
        end else begin
            shr_frac       = a_frac;
            shr_result_exp = shr_exp;
        end
    end

    // ================================================================
    //  OUTPUT MUX
    // ================================================================
    reg signed [MAX_FRAC-1:0] r_frac;
    reg signed [MAX_EXP-1:0]  r_exp;

    always @(*) begin
        case (op)
            3'd0: begin r_frac = neg_frac;  r_exp = neg_exp;        end  // NEG
            3'd1: begin r_frac = abs_frac;  r_exp = abs_exp;        end  // ABS
            3'd2: begin r_frac = sign_frac; r_exp = sign_exp;       end  // SIGN
            3'd3: begin r_frac = shl_frac;  r_exp = shl_result_exp; end  // SHL
            3'd4: begin r_frac = shr_frac;  r_exp = shr_result_exp; end  // SHR
            default: begin r_frac = a_frac; r_exp = a_exp;          end
        endcase
    end

    assign result_frac = r_frac;
    assign result_exp  = r_exp;

endmodule
