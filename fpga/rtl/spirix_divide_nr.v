// spirix_divide_nr — 6-stage pipelined Newton-Raphson divider for Spirix scalars
//
// Computes a / b using Newton-Raphson reciprocal approximation:
//   1/b via LUT seed + 2 quadratic-convergence iterations (10→20→40 bits),
//   then multiply by a, with remainder correction for bit-exact results.
//
// Latency: 6 cycles. Throughput: 1 division per clock.
//
//   Stage 1: Sign/abs, LUT seed, B×x₀, E₁ = 2^25 - P₁
//   Stage 2: x₀×E₁ → X₁, B×X₁ → P₂
//   Stage 3: E₂ = 2^25 - P₂, X₁×E₂ → X₂
//   Stage 4: A×X₂ → Q_raw (approximate quotient)
//   Stage 5: Remainder correction — exact Euclidean quotient + remainder
//   Stage 6: Normalize + round + sign + exponent + clamp
//
// Division by zero returns (0, AMBIGUOUS_EXP).
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_divide_nr #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;

    // =========================================================================
    // Stage 1: Sign extraction, absolute values, LUT seed, first NR multiply
    // =========================================================================

    // --- Combinational: sign, abs, NEG_ONE handling ---
    wire s1_result_sign = a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
    wire s1_b_is_zero = (b_frac == 0);

    wire s1_a_is_neg_one = (a_frac == NEG_ONE);
    wire s1_b_is_neg_one = (b_frac == NEG_ONE);

    wire [FRAC_BITS-2:0] s1_abs_a = s1_a_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                      (a_frac[FRAC_BITS-1] ? (~a_frac[FRAC_BITS-2:0] + 1'b1) :
                                                              a_frac[FRAC_BITS-2:0]);
    wire [FRAC_BITS-2:0] s1_abs_b = s1_b_is_neg_one ? POS_HALF[FRAC_BITS-2:0] :
                                      (b_frac[FRAC_BITS-1] ? (~b_frac[FRAC_BITS-2:0] + 1'b1) :
                                                              b_frac[FRAC_BITS-2:0]);

    wire signed [EXP_BITS:0] s1_a_exp_adj = $signed({a_exp[EXP_BITS-1], a_exp})
                                             + {{EXP_BITS{1'b0}}, s1_a_is_neg_one};
    wire signed [EXP_BITS:0] s1_b_exp_adj = $signed({b_exp[EXP_BITS-1], b_exp})
                                             + {{EXP_BITS{1'b0}}, s1_b_is_neg_one};

    // --- LUT: 256 entries × 10 bits, indexed by abs_b[22:15] ---
    // x₀ = {0, 1, lut[9:0], 13'b0} = 25 unsigned bits
    // Represents reciprocal seed ≈ 1/b_norm with ~10 bits accuracy
    reg [9:0] s1_recip_lut;
    always @(*) begin
        case (s1_abs_b[FRAC_BITS-3:FRAC_BITS-10])  // [22:15] for FRAC=25
            8'd0: s1_recip_lut = 10'd1022;
            8'd1: s1_recip_lut = 10'd1018;
            8'd2: s1_recip_lut = 10'd1014;
            8'd3: s1_recip_lut = 10'd1010;
            8'd4: s1_recip_lut = 10'd1006;
            8'd5: s1_recip_lut = 10'd1002;
            8'd6: s1_recip_lut = 10'd998;
            8'd7: s1_recip_lut = 10'd994;
            8'd8: s1_recip_lut = 10'd991;
            8'd9: s1_recip_lut = 10'd987;
            8'd10: s1_recip_lut = 10'd983;
            8'd11: s1_recip_lut = 10'd979;
            8'd12: s1_recip_lut = 10'd976;
            8'd13: s1_recip_lut = 10'd972;
            8'd14: s1_recip_lut = 10'd969;
            8'd15: s1_recip_lut = 10'd965;
            8'd16: s1_recip_lut = 10'd961;
            8'd17: s1_recip_lut = 10'd958;
            8'd18: s1_recip_lut = 10'd954;
            8'd19: s1_recip_lut = 10'd951;
            8'd20: s1_recip_lut = 10'd948;
            8'd21: s1_recip_lut = 10'd944;
            8'd22: s1_recip_lut = 10'd941;
            8'd23: s1_recip_lut = 10'd937;
            8'd24: s1_recip_lut = 10'd934;
            8'd25: s1_recip_lut = 10'd931;
            8'd26: s1_recip_lut = 10'd927;
            8'd27: s1_recip_lut = 10'd924;
            8'd28: s1_recip_lut = 10'd921;
            8'd29: s1_recip_lut = 10'd918;
            8'd30: s1_recip_lut = 10'd914;
            8'd31: s1_recip_lut = 10'd911;
            8'd32: s1_recip_lut = 10'd908;
            8'd33: s1_recip_lut = 10'd905;
            8'd34: s1_recip_lut = 10'd902;
            8'd35: s1_recip_lut = 10'd899;
            8'd36: s1_recip_lut = 10'd896;
            8'd37: s1_recip_lut = 10'd893;
            8'd38: s1_recip_lut = 10'd890;
            8'd39: s1_recip_lut = 10'd887;
            8'd40: s1_recip_lut = 10'd884;
            8'd41: s1_recip_lut = 10'd881;
            8'd42: s1_recip_lut = 10'd878;
            8'd43: s1_recip_lut = 10'd875;
            8'd44: s1_recip_lut = 10'd872;
            8'd45: s1_recip_lut = 10'd869;
            8'd46: s1_recip_lut = 10'd866;
            8'd47: s1_recip_lut = 10'd863;
            8'd48: s1_recip_lut = 10'd860;
            8'd49: s1_recip_lut = 10'd858;
            8'd50: s1_recip_lut = 10'd855;
            8'd51: s1_recip_lut = 10'd852;
            8'd52: s1_recip_lut = 10'd849;
            8'd53: s1_recip_lut = 10'd846;
            8'd54: s1_recip_lut = 10'd844;
            8'd55: s1_recip_lut = 10'd841;
            8'd56: s1_recip_lut = 10'd838;
            8'd57: s1_recip_lut = 10'd836;
            8'd58: s1_recip_lut = 10'd833;
            8'd59: s1_recip_lut = 10'd830;
            8'd60: s1_recip_lut = 10'd828;
            8'd61: s1_recip_lut = 10'd825;
            8'd62: s1_recip_lut = 10'd823;
            8'd63: s1_recip_lut = 10'd820;
            8'd64: s1_recip_lut = 10'd817;
            8'd65: s1_recip_lut = 10'd815;
            8'd66: s1_recip_lut = 10'd812;
            8'd67: s1_recip_lut = 10'd810;
            8'd68: s1_recip_lut = 10'd807;
            8'd69: s1_recip_lut = 10'd805;
            8'd70: s1_recip_lut = 10'd802;
            8'd71: s1_recip_lut = 10'd800;
            8'd72: s1_recip_lut = 10'd798;
            8'd73: s1_recip_lut = 10'd795;
            8'd74: s1_recip_lut = 10'd793;
            8'd75: s1_recip_lut = 10'd790;
            8'd76: s1_recip_lut = 10'd788;
            8'd77: s1_recip_lut = 10'd786;
            8'd78: s1_recip_lut = 10'd783;
            8'd79: s1_recip_lut = 10'd781;
            8'd80: s1_recip_lut = 10'd779;
            8'd81: s1_recip_lut = 10'd776;
            8'd82: s1_recip_lut = 10'd774;
            8'd83: s1_recip_lut = 10'd772;
            8'd84: s1_recip_lut = 10'd769;
            8'd85: s1_recip_lut = 10'd767;
            8'd86: s1_recip_lut = 10'd765;
            8'd87: s1_recip_lut = 10'd763;
            8'd88: s1_recip_lut = 10'd760;
            8'd89: s1_recip_lut = 10'd758;
            8'd90: s1_recip_lut = 10'd756;
            8'd91: s1_recip_lut = 10'd754;
            8'd92: s1_recip_lut = 10'd752;
            8'd93: s1_recip_lut = 10'd750;
            8'd94: s1_recip_lut = 10'd747;
            8'd95: s1_recip_lut = 10'd745;
            8'd96: s1_recip_lut = 10'd743;
            8'd97: s1_recip_lut = 10'd741;
            8'd98: s1_recip_lut = 10'd739;
            8'd99: s1_recip_lut = 10'd737;
            8'd100: s1_recip_lut = 10'd735;
            8'd101: s1_recip_lut = 10'd733;
            8'd102: s1_recip_lut = 10'd731;
            8'd103: s1_recip_lut = 10'd729;
            8'd104: s1_recip_lut = 10'd727;
            8'd105: s1_recip_lut = 10'd725;
            8'd106: s1_recip_lut = 10'd723;
            8'd107: s1_recip_lut = 10'd721;
            8'd108: s1_recip_lut = 10'd719;
            8'd109: s1_recip_lut = 10'd717;
            8'd110: s1_recip_lut = 10'd715;
            8'd111: s1_recip_lut = 10'd713;
            8'd112: s1_recip_lut = 10'd711;
            8'd113: s1_recip_lut = 10'd709;
            8'd114: s1_recip_lut = 10'd707;
            8'd115: s1_recip_lut = 10'd705;
            8'd116: s1_recip_lut = 10'd703;
            8'd117: s1_recip_lut = 10'd701;
            8'd118: s1_recip_lut = 10'd699;
            8'd119: s1_recip_lut = 10'd698;
            8'd120: s1_recip_lut = 10'd696;
            8'd121: s1_recip_lut = 10'd694;
            8'd122: s1_recip_lut = 10'd692;
            8'd123: s1_recip_lut = 10'd690;
            8'd124: s1_recip_lut = 10'd688;
            8'd125: s1_recip_lut = 10'd687;
            8'd126: s1_recip_lut = 10'd685;
            8'd127: s1_recip_lut = 10'd683;
            8'd128: s1_recip_lut = 10'd681;
            8'd129: s1_recip_lut = 10'd680;
            8'd130: s1_recip_lut = 10'd678;
            8'd131: s1_recip_lut = 10'd676;
            8'd132: s1_recip_lut = 10'd674;
            8'd133: s1_recip_lut = 10'd673;
            8'd134: s1_recip_lut = 10'd671;
            8'd135: s1_recip_lut = 10'd669;
            8'd136: s1_recip_lut = 10'd667;
            8'd137: s1_recip_lut = 10'd666;
            8'd138: s1_recip_lut = 10'd664;
            8'd139: s1_recip_lut = 10'd662;
            8'd140: s1_recip_lut = 10'd661;
            8'd141: s1_recip_lut = 10'd659;
            8'd142: s1_recip_lut = 10'd657;
            8'd143: s1_recip_lut = 10'd656;
            8'd144: s1_recip_lut = 10'd654;
            8'd145: s1_recip_lut = 10'd652;
            8'd146: s1_recip_lut = 10'd651;
            8'd147: s1_recip_lut = 10'd649;
            8'd148: s1_recip_lut = 10'd648;
            8'd149: s1_recip_lut = 10'd646;
            8'd150: s1_recip_lut = 10'd644;
            8'd151: s1_recip_lut = 10'd643;
            8'd152: s1_recip_lut = 10'd641;
            8'd153: s1_recip_lut = 10'd640;
            8'd154: s1_recip_lut = 10'd638;
            8'd155: s1_recip_lut = 10'd637;
            8'd156: s1_recip_lut = 10'd635;
            8'd157: s1_recip_lut = 10'd633;
            8'd158: s1_recip_lut = 10'd632;
            8'd159: s1_recip_lut = 10'd630;
            8'd160: s1_recip_lut = 10'd629;
            8'd161: s1_recip_lut = 10'd627;
            8'd162: s1_recip_lut = 10'd626;
            8'd163: s1_recip_lut = 10'd624;
            8'd164: s1_recip_lut = 10'd623;
            8'd165: s1_recip_lut = 10'd621;
            8'd166: s1_recip_lut = 10'd620;
            8'd167: s1_recip_lut = 10'd618;
            8'd168: s1_recip_lut = 10'd617;
            8'd169: s1_recip_lut = 10'd616;
            8'd170: s1_recip_lut = 10'd614;
            8'd171: s1_recip_lut = 10'd613;
            8'd172: s1_recip_lut = 10'd611;
            8'd173: s1_recip_lut = 10'd610;
            8'd174: s1_recip_lut = 10'd608;
            8'd175: s1_recip_lut = 10'd607;
            8'd176: s1_recip_lut = 10'd606;
            8'd177: s1_recip_lut = 10'd604;
            8'd178: s1_recip_lut = 10'd603;
            8'd179: s1_recip_lut = 10'd601;
            8'd180: s1_recip_lut = 10'd600;
            8'd181: s1_recip_lut = 10'd599;
            8'd182: s1_recip_lut = 10'd597;
            8'd183: s1_recip_lut = 10'd596;
            8'd184: s1_recip_lut = 10'd595;
            8'd185: s1_recip_lut = 10'd593;
            8'd186: s1_recip_lut = 10'd592;
            8'd187: s1_recip_lut = 10'd591;
            8'd188: s1_recip_lut = 10'd589;
            8'd189: s1_recip_lut = 10'd588;
            8'd190: s1_recip_lut = 10'd587;
            8'd191: s1_recip_lut = 10'd585;
            8'd192: s1_recip_lut = 10'd584;
            8'd193: s1_recip_lut = 10'd583;
            8'd194: s1_recip_lut = 10'd581;
            8'd195: s1_recip_lut = 10'd580;
            8'd196: s1_recip_lut = 10'd579;
            8'd197: s1_recip_lut = 10'd578;
            8'd198: s1_recip_lut = 10'd576;
            8'd199: s1_recip_lut = 10'd575;
            8'd200: s1_recip_lut = 10'd574;
            8'd201: s1_recip_lut = 10'd572;
            8'd202: s1_recip_lut = 10'd571;
            8'd203: s1_recip_lut = 10'd570;
            8'd204: s1_recip_lut = 10'd569;
            8'd205: s1_recip_lut = 10'd568;
            8'd206: s1_recip_lut = 10'd566;
            8'd207: s1_recip_lut = 10'd565;
            8'd208: s1_recip_lut = 10'd564;
            8'd209: s1_recip_lut = 10'd563;
            8'd210: s1_recip_lut = 10'd561;
            8'd211: s1_recip_lut = 10'd560;
            8'd212: s1_recip_lut = 10'd559;
            8'd213: s1_recip_lut = 10'd558;
            8'd214: s1_recip_lut = 10'd557;
            8'd215: s1_recip_lut = 10'd555;
            8'd216: s1_recip_lut = 10'd554;
            8'd217: s1_recip_lut = 10'd553;
            8'd218: s1_recip_lut = 10'd552;
            8'd219: s1_recip_lut = 10'd551;
            8'd220: s1_recip_lut = 10'd550;
            8'd221: s1_recip_lut = 10'd548;
            8'd222: s1_recip_lut = 10'd547;
            8'd223: s1_recip_lut = 10'd546;
            8'd224: s1_recip_lut = 10'd545;
            8'd225: s1_recip_lut = 10'd544;
            8'd226: s1_recip_lut = 10'd543;
            8'd227: s1_recip_lut = 10'd542;
            8'd228: s1_recip_lut = 10'd541;
            8'd229: s1_recip_lut = 10'd539;
            8'd230: s1_recip_lut = 10'd538;
            8'd231: s1_recip_lut = 10'd537;
            8'd232: s1_recip_lut = 10'd536;
            8'd233: s1_recip_lut = 10'd535;
            8'd234: s1_recip_lut = 10'd534;
            8'd235: s1_recip_lut = 10'd533;
            8'd236: s1_recip_lut = 10'd532;
            8'd237: s1_recip_lut = 10'd531;
            8'd238: s1_recip_lut = 10'd530;
            8'd239: s1_recip_lut = 10'd529;
            8'd240: s1_recip_lut = 10'd527;
            8'd241: s1_recip_lut = 10'd526;
            8'd242: s1_recip_lut = 10'd525;
            8'd243: s1_recip_lut = 10'd524;
            8'd244: s1_recip_lut = 10'd523;
            8'd245: s1_recip_lut = 10'd522;
            8'd246: s1_recip_lut = 10'd521;
            8'd247: s1_recip_lut = 10'd520;
            8'd248: s1_recip_lut = 10'd519;
            8'd249: s1_recip_lut = 10'd518;
            8'd250: s1_recip_lut = 10'd517;
            8'd251: s1_recip_lut = 10'd516;
            8'd252: s1_recip_lut = 10'd515;
            8'd253: s1_recip_lut = 10'd514;
            8'd254: s1_recip_lut = 10'd513;
            8'd255: s1_recip_lut = 10'd512;
        endcase
    end

    // Reconstruct x₀: {0, lut[9:0], 14'b0} = 25 bits unsigned
    // LUT values range [512,1022], so lut[9] is always 1 (implicit leading 1).
    // x₀_val = x₀ / 2^24 ∈ (0.5, 1.0]
    wire [FRAC_BITS-1:0] s1_x0 = {1'b0, s1_recip_lut, {(FRAC_BITS-11){1'b0}}};

    // First NR multiply: P₁ = (abs_b × x₀) >> 23
    // abs_b is FRAC-1=24 bits, x₀ is FRAC=25 bits → product is 49 bits
    // P₁ = product[48:23] = 26 bits (should be ≈ 2^24 when accurate)
    wire [2*FRAC_BITS-2:0] s1_bx0_full = s1_abs_b * s1_x0;  // 49 bits
    wire [FRAC_BITS:0] s1_p1 = s1_bx0_full[2*FRAC_BITS-2 : FRAC_BITS-2];  // 26 bits

    // E₁ = 2^25 - P₁ (error term, 26 bits)
    wire [FRAC_BITS:0] s1_e1 = {1'b1, {FRAC_BITS{1'b0}}} - s1_p1;  // 2^25 - P1

    // --- Stage 1 registers ---
    reg                      s1_sign_r;
    reg signed [EXP_BITS:0]  s1_a_exp_adj_r, s1_b_exp_adj_r;
    reg [FRAC_BITS-2:0]      s1_abs_a_r, s1_abs_b_r;
    reg [FRAC_BITS-1:0]      s1_x0_r;
    reg [FRAC_BITS:0]        s1_e1_r;
    reg                      s1_b_zero_r;

    always @(posedge clk) begin
        s1_sign_r      <= s1_result_sign;
        s1_a_exp_adj_r <= s1_a_exp_adj;
        s1_b_exp_adj_r <= s1_b_exp_adj;
        s1_abs_a_r     <= s1_abs_a;
        s1_abs_b_r     <= s1_abs_b;
        s1_x0_r        <= s1_x0;
        s1_e1_r        <= s1_e1;
        s1_b_zero_r    <= s1_b_is_zero;
    end

    // =========================================================================
    // Stage 2: X₁ = (x₀ × E₁) >> 24, P₂ = (abs_b × X₁) >> 23
    // =========================================================================

    // x₀ is 25 bits unsigned, E₁ is 26 bits unsigned → product is 51 bits
    // X₁ = product[48:24] = 25 bits (~20 bits accurate)
    wire [FRAC_BITS + FRAC_BITS:0] s2_x0e1_full = s1_x0_r * s1_e1_r;  // 51 bits
    wire [FRAC_BITS-1:0] s2_x1 = s2_x0e1_full[2*FRAC_BITS-2 : FRAC_BITS-1];  // 25 bits

    // P₂ = (abs_b × X₁) >> 23 = 26 bits
    wire [2*FRAC_BITS-2:0] s2_bx1_full = s1_abs_b_r * s2_x1;  // 49 bits
    wire [FRAC_BITS:0] s2_p2 = s2_bx1_full[2*FRAC_BITS-2 : FRAC_BITS-2];  // 26 bits

    // --- Stage 2 registers ---
    reg                      s2_sign_r;
    reg signed [EXP_BITS:0]  s2_a_exp_adj_r, s2_b_exp_adj_r;
    reg [FRAC_BITS-2:0]      s2_abs_a_r, s2_abs_b_r;
    reg [FRAC_BITS-1:0]      s2_x1_r;
    reg [FRAC_BITS:0]        s2_p2_r;
    reg                      s2_b_zero_r;

    always @(posedge clk) begin
        s2_sign_r      <= s1_sign_r;
        s2_a_exp_adj_r <= s1_a_exp_adj_r;
        s2_b_exp_adj_r <= s1_b_exp_adj_r;
        s2_abs_a_r     <= s1_abs_a_r;
        s2_abs_b_r     <= s1_abs_b_r;
        s2_x1_r        <= s2_x1;
        s2_p2_r        <= s2_p2;
        s2_b_zero_r    <= s1_b_zero_r;
    end

    // =========================================================================
    // Stage 3: E₂ = 2^25 - P₂, X₂ = (X₁ × E₂) >> 24
    // =========================================================================

    wire [FRAC_BITS:0] s3_e2 = {1'b1, {FRAC_BITS{1'b0}}} - s2_p2_r;  // 26 bits

    // X₁ is 25 bits, E₂ is 26 bits → product is 51 bits
    // X₂ = product[48:24] = 25 bits (~40 bits accurate)
    wire [FRAC_BITS + FRAC_BITS:0] s3_x1e2_full = s2_x1_r * s3_e2;  // 51 bits
    wire [FRAC_BITS-1:0] s3_x2 = s3_x1e2_full[2*FRAC_BITS-2 : FRAC_BITS-1];  // 25 bits

    // --- Stage 3 registers ---
    reg                      s3_sign_r;
    reg signed [EXP_BITS:0]  s3_a_exp_adj_r, s3_b_exp_adj_r;
    reg [FRAC_BITS-2:0]      s3_abs_a_r, s3_abs_b_r;
    reg [FRAC_BITS-1:0]      s3_x2_r;
    reg                      s3_b_zero_r;

    always @(posedge clk) begin
        s3_sign_r      <= s2_sign_r;
        s3_a_exp_adj_r <= s2_a_exp_adj_r;
        s3_b_exp_adj_r <= s2_b_exp_adj_r;
        s3_abs_a_r     <= s2_abs_a_r;
        s3_abs_b_r     <= s2_abs_b_r;
        s3_x2_r        <= s3_x2;
        s3_b_zero_r    <= s2_b_zero_r;
    end

    // =========================================================================
    // Stage 4: Q_raw = (abs_a × X₂) >> 22
    //
    // abs_a is FRAC-1=24 bits, X₂ is FRAC=25 bits → product is 49 bits
    // Q_raw = product[47:22] = 26 bits (FRAC+1 bits, matching comb divider)
    // sticky_low = |product[21:0]
    // =========================================================================

    wire [2*FRAC_BITS-2:0] s4_ax2_full = s3_abs_a_r * s3_x2_r;  // 49 bits
    wire [FRAC_BITS:0] s4_q_raw = s4_ax2_full[2*FRAC_BITS-3 : FRAC_BITS-3];  // 26 bits
    wire s4_sticky_low = |s4_ax2_full[FRAC_BITS-4:0];  // |product[21:0]

    // --- Stage 4 registers ---
    reg                      s4_sign_r;
    reg signed [EXP_BITS:0]  s4_a_exp_adj_r, s4_b_exp_adj_r;
    reg [FRAC_BITS:0]        s4_q_raw_r;
    reg [FRAC_BITS-2:0]      s4_abs_a_r, s4_abs_b_r;
    reg                      s4_b_zero_r;

    always @(posedge clk) begin
        s4_sign_r       <= s3_sign_r;
        s4_a_exp_adj_r  <= s3_a_exp_adj_r;
        s4_b_exp_adj_r  <= s3_b_exp_adj_r;
        s4_q_raw_r      <= s4_q_raw;
        s4_abs_a_r      <= s3_abs_a_r;
        s4_abs_b_r      <= s3_abs_b_r;
        s4_b_zero_r     <= s3_b_zero_r;
    end

    // =========================================================================
    // Stage 5: Remainder correction — exact Euclidean quotient
    //
    // NR gives Q_raw within ±2 of the true quotient. Compute the exact
    // remainder R = (abs_a << FRAC) - Q_raw * abs_b, then correct Q_raw
    // based on sign and magnitude of R. This makes the result bit-identical
    // to the combinational divider (spirix_divide.v).
    // =========================================================================
    localparam WIDE = 2 * FRAC_BITS;

    // P_check = Q_raw * abs_b (26 × 24 = 50 bits)
    wire [FRAC_BITS + FRAC_BITS - 1:0] s5_p_check = s4_q_raw_r * s4_abs_b_r;

    // A_shifted = abs_a << FRAC_BITS (just wiring, 49 bits)
    wire [WIDE-1:0] s5_a_shifted = {{(FRAC_BITS+1){1'b0}}, s4_abs_a_r} << FRAC_BITS;

    // R = A_shifted - P_check (signed, WIDE+1 bits)
    wire signed [WIDE:0] s5_r = $signed({1'b0, s5_a_shifted})
                                - $signed({1'b0, s5_p_check});

    // abs_b multiples for comparison (all parallel, same scale as R)
    wire [WIDE-1:0] s5_1b = {{(FRAC_BITS+1){1'b0}}, s4_abs_b_r};
    wire [WIDE-1:0] s5_2b = {{FRAC_BITS{1'b0}}, s4_abs_b_r, 1'b0};
    wire [WIDE-1:0] s5_3b = s5_1b + s5_2b;
    wire [WIDE-1:0] s5_4b = {{(FRAC_BITS-1){1'b0}}, s4_abs_b_r, 2'b0};

    // Classify remainder: all comparisons run in parallel
    wire s5_r_neg = s5_r[WIDE];  // R < 0
    wire signed [WIDE:0] s5_r_m1b = s5_r - $signed({1'b0, s5_1b});
    wire signed [WIDE:0] s5_r_m2b = s5_r - $signed({1'b0, s5_2b});
    wire signed [WIDE:0] s5_r_m3b = s5_r - $signed({1'b0, s5_3b});
    wire signed [WIDE:0] s5_r_m4b = s5_r - $signed({1'b0, s5_4b});
    wire s5_r_ge_1b = !s5_r_m1b[WIDE];
    wire s5_r_ge_2b = !s5_r_m2b[WIDE];
    wire s5_r_ge_3b = !s5_r_m3b[WIDE];
    wire s5_r_ge_4b = !s5_r_m4b[WIDE];

    wire signed [WIDE:0] s5_r_p1b = s5_r + $signed({1'b0, s5_1b});
    wire signed [WIDE:0] s5_r_p2b = s5_r + $signed({1'b0, s5_2b});
    wire signed [WIDE:0] s5_r_p3b = s5_r + $signed({1'b0, s5_3b});
    wire signed [WIDE:0] s5_r_p4b = s5_r + $signed({1'b0, s5_4b});
    wire s5_r_lt_n1b = s5_r_p1b[WIDE];  // R < -1b
    wire s5_r_lt_n2b = s5_r_p2b[WIDE];  // R < -2b
    wire s5_r_lt_n3b = s5_r_p3b[WIDE];  // R < -3b

    // Correction delta ∈ {-3, -2, -1, 0, +1, +2, +3, +4}
    // Priority encode: pick the largest |delta| that applies
    wire signed [3:0] s5_delta = s5_r_neg ? (s5_r_lt_n3b ? -4'sd4 :
                                              s5_r_lt_n2b ? -4'sd3 :
                                              s5_r_lt_n1b ? -4'sd2 : -4'sd1) :
                                  s5_r_ge_4b ? 4'sd4 :
                                  s5_r_ge_3b ? 4'sd3 :
                                  s5_r_ge_2b ? 4'sd2 :
                                  s5_r_ge_1b ? 4'sd1 : 4'sd0;

    wire [FRAC_BITS:0] s5_q_exact = $unsigned($signed({1'b0, s4_q_raw_r}) + s5_delta);

    // Exact remainder: R_exact = R - delta * abs_b (via mux, no multiply)
    wire signed [WIDE:0] s5_r_exact = (s5_delta == -4'sd4) ? s5_r_p4b :
                                       (s5_delta == -4'sd3) ? s5_r_p3b :
                                       (s5_delta == -4'sd2) ? s5_r_p2b :
                                       (s5_delta == -4'sd1) ? s5_r_p1b :
                                       (s5_delta ==  4'sd4) ? s5_r_m4b :
                                       (s5_delta ==  4'sd3) ? s5_r_m3b :
                                       (s5_delta ==  4'sd2) ? s5_r_m2b :
                                       (s5_delta ==  4'sd1) ? s5_r_m1b :
                                                               s5_r;
    wire s5_rem_sticky = |s5_r_exact[WIDE-1:0];

    // --- Stage 5 registers ---
    reg                      s5_sign_r;
    reg signed [EXP_BITS:0]  s5_a_exp_adj_r, s5_b_exp_adj_r;
    reg [FRAC_BITS:0]        s5_q_exact_r;
    reg                      s5_rem_sticky_r;
    reg                      s5_b_zero_r;

    always @(posedge clk) begin
        s5_sign_r       <= s4_sign_r;
        s5_a_exp_adj_r  <= s4_a_exp_adj_r;
        s5_b_exp_adj_r  <= s4_b_exp_adj_r;
        s5_q_exact_r    <= s5_q_exact;
        s5_rem_sticky_r <= s5_rem_sticky;
        s5_b_zero_r     <= s4_b_zero_r;
    end

    // =========================================================================
    // Stage 6: Normalize + round + sign + exponent + clamp
    //
    // Same finalization logic as spirix_divide.v steps 3-7.
    // Q_exact is FRAC+1 = 26 unsigned bits in [2^(FRAC-1), 2^(FRAC+1)).
    // =========================================================================

    // --- Bounded normalization: 0 or 1 bit right shift ---
    wire s6_norm_shift = s5_q_exact_r[FRAC_BITS];
    wire [FRAC_BITS:0] s6_q_norm = s6_norm_shift ? (s5_q_exact_r >> 1) : s5_q_exact_r;

    // Extract positive N1 fraction: bits [FRAC:1] = 01xxx (FRAC bits)
    wire [FRAC_BITS-1:0] s6_frac_pos_raw = s6_q_norm[FRAC_BITS:1];

    // --- Banker's rounding ---
    wire s6_guard  = s6_q_norm[0];
    wire s6_norm_sticky = s6_norm_shift & s5_q_exact_r[0];
    wire s6_sticky = s6_norm_sticky | s5_rem_sticky_r;
    wire s6_lsb    = s6_frac_pos_raw[0];
    wire s6_round_up = s6_guard & (s6_sticky | s6_lsb);

    wire [FRAC_BITS-1:0] s6_frac_rounded = s6_frac_pos_raw
                                             + {{(FRAC_BITS-1){1'b0}}, s6_round_up};

    // Early rounding overflow: max positive N1 (0_111...1) + 1 = 1_000...0
    wire s6_round_ovf = (&s6_frac_pos_raw[FRAC_BITS-2:0]) & s6_round_up;
    wire [FRAC_BITS-1:0] s6_pos_frac = s6_round_ovf ? POS_HALF : s6_frac_rounded;

    // --- Apply sign ---
    wire s6_neg_is_pos_half = (s6_pos_frac == POS_HALF);

    wire signed [FRAC_BITS-1:0] s6_neg_frac = s6_neg_is_pos_half ? NEG_ONE :
                                                (~s6_pos_frac + 1'b1);

    wire signed [FRAC_BITS-1:0] s6_final_frac = s5_sign_r ? s6_neg_frac :
                                                  $signed(s6_pos_frac);

    // --- Exponent ---
    wire signed [EXP_BITS:0] s6_exp_base = s5_a_exp_adj_r - s5_b_exp_adj_r
                                           + {{EXP_BITS{1'b0}}, s6_norm_shift}
                                           + {{EXP_BITS{1'b0}}, s6_round_ovf};

    wire signed [EXP_BITS:0] s6_exp_final = (s5_sign_r & s6_neg_is_pos_half) ?
                                              (s6_exp_base - 1) : s6_exp_base;

    wire s6_exp_too_big   = (s6_exp_final > MAX_EXP);
    wire s6_exp_too_small = (s6_exp_final < MIN_EXP);
    wire signed [EXP_BITS-1:0] s6_final_exp = s6_exp_final[EXP_BITS-1:0];

    // --- Output register with overflow/underflow/div-by-zero clamping ---
    always @(posedge clk) begin
        result_frac <= s5_b_zero_r     ? {FRAC_BITS{1'b0}} :
                       s6_exp_too_big   ? s6_final_frac :
                       s6_exp_too_small ? {s6_final_frac[FRAC_BITS-1],
                                            s6_final_frac[FRAC_BITS-1:1]} :
                                           s6_final_frac;

        result_exp  <= (s5_b_zero_r | s6_exp_too_big | s6_exp_too_small) ?
                        AMBIGUOUS_EXP[EXP_BITS-1:0] : s6_final_exp;
    end

endmodule
