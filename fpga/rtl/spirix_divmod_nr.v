// spirix_divmod_nr — Pipelined Newton-Raphson divmod for Spirix scalars
//
// Computes a / b (quotient) and optionally a mod b (floored modulo, Python-style)
// using Newton-Raphson reciprocal approximation with remainder correction.
//
// ENABLE_MOD=0: divide only, 6-cycle latency (~600 LUT4, 20 DSP).
// ENABLE_MOD=1: divide + modulo, quotient 6 cycles, modulo 8 cycles.
// Throughput: 1 operation per clock.
//
//   Stage 1: Sign/abs, LUT seed, B*x0, E1 = 2^25 - P1
//   Stage 2: x0*E1 -> X1, B*X1 -> P2
//   Stage 3: E2 = 2^25 - P2, X1*E2 -> X2
//   Stage 4: A*X2 -> Q_raw (approximate quotient)
//   Stage 5: Remainder correction — exact Euclidean quotient + remainder
//   Stage 6: Quotient finalization (normalize+round+sign+exp+clamp)
//            + Modulo: Q_lo*abs_b + R_exact (DSP multiply + add)
//   Stage 7: Modulo: barrel shift + conditional subtract + sign correction
//   Stage 8: Modulo normalization (CLZ+shift+sign+exp+clamp)
//
// Modulo is floored (Python-style): result sign matches divisor (b).
// Exact for exponent difference |a_exp - b_exp| <= FRAC_BITS.
// For larger differences: returns (0, AMBIGUOUS_EXP).
//
// Division/modulo by zero returns (0, AMBIGUOUS_EXP).
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Fraction is signed two's complement, N1-normalized (top two bits differ).
// Exponent is signed two's complement. The minimum exponent value
// (AMBIGUOUS_EXP = -2^(EXP_BITS-1)) encodes zero/overflow/underflow.
//
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_divmod_nr #(
    parameter FRAC_BITS  = 25,
    parameter EXP_BITS   = 8,
    parameter ENABLE_MOD = 0   // 0 = divide only (6 cyc), 1 = divide + modulo (6/8 cyc)
)(
    input  wire clk,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    // Quotient output — 6-cycle latency
    output reg  signed [FRAC_BITS-1:0] q_frac,
    output reg  signed [EXP_BITS-1:0]  q_exp,
    // Modulo output — 8-cycle latency (floored/Python-style)
    output reg  signed [FRAC_BITS-1:0] mod_frac,
    output reg  signed [EXP_BITS-1:0]  mod_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;
    localparam WIDE = 2 * FRAC_BITS;
    localparam MAG = FRAC_BITS - 1;  // magnitude bits = 24

    // =========================================================================
    // Stage 1: Sign extraction, absolute values, LUT seed, first NR multiply
    // =========================================================================

    wire s1_result_sign = a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
    wire s1_a_sign = a_frac[FRAC_BITS-1];
    wire s1_b_is_zero = (b_frac == 0);

    wire s1_a_is_neg_one = (a_frac == NEG_ONE);
    wire s1_b_is_neg_one = (b_frac == NEG_ONE);

    wire [MAG-1:0] s1_abs_a = s1_a_is_neg_one ? POS_HALF[MAG-1:0] :
                                (a_frac[FRAC_BITS-1] ? (~a_frac[MAG-1:0] + 1'b1) :
                                                        a_frac[MAG-1:0]);
    wire [MAG-1:0] s1_abs_b = s1_b_is_neg_one ? POS_HALF[MAG-1:0] :
                                (b_frac[FRAC_BITS-1] ? (~b_frac[MAG-1:0] + 1'b1) :
                                                        b_frac[MAG-1:0]);

    wire signed [EXP_BITS:0] s1_a_exp_adj = $signed({a_exp[EXP_BITS-1], a_exp})
                                             + {{EXP_BITS{1'b0}}, s1_a_is_neg_one};
    wire signed [EXP_BITS:0] s1_b_exp_adj = $signed({b_exp[EXP_BITS-1], b_exp})
                                             + {{EXP_BITS{1'b0}}, s1_b_is_neg_one};

    // --- LUT: 256 entries x 10 bits ---
    reg [9:0] s1_recip_lut;
    always @(*) begin
        case (s1_abs_b[FRAC_BITS-3:FRAC_BITS-10])
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

    wire [FRAC_BITS-1:0] s1_x0 = {1'b0, s1_recip_lut, {(FRAC_BITS-11){1'b0}}};

    wire [2*FRAC_BITS-2:0] s1_bx0_full = s1_abs_b * s1_x0;
    wire [FRAC_BITS:0] s1_p1 = s1_bx0_full[2*FRAC_BITS-2 : FRAC_BITS-2];
    wire [FRAC_BITS:0] s1_e1 = {1'b1, {FRAC_BITS{1'b0}}} - s1_p1;

    // --- Stage 1 registers ---
    reg                      s1_sign_r;
    reg                      s1_a_sign_r;
    reg signed [EXP_BITS:0]  s1_a_exp_adj_r, s1_b_exp_adj_r;
    reg [MAG-1:0]            s1_abs_a_r, s1_abs_b_r;
    reg [FRAC_BITS-1:0]      s1_x0_r;
    reg [FRAC_BITS:0]        s1_e1_r;
    reg                      s1_b_zero_r;

    always @(posedge clk) begin
        s1_sign_r      <= s1_result_sign;
        if (ENABLE_MOD) s1_a_sign_r <= s1_a_sign;
        s1_a_exp_adj_r <= s1_a_exp_adj;
        s1_b_exp_adj_r <= s1_b_exp_adj;
        s1_abs_a_r     <= s1_abs_a;
        s1_abs_b_r     <= s1_abs_b;
        s1_x0_r        <= s1_x0;
        s1_e1_r        <= s1_e1;
        s1_b_zero_r    <= s1_b_is_zero;
    end

    // =========================================================================
    // Stage 2: X1 = (x0 * E1) >> 24, P2 = (abs_b * X1) >> 23
    // =========================================================================

    wire [FRAC_BITS + FRAC_BITS:0] s2_x0e1_full = s1_x0_r * s1_e1_r;
    wire [FRAC_BITS-1:0] s2_x1 = s2_x0e1_full[2*FRAC_BITS-2 : FRAC_BITS-1];

    wire [2*FRAC_BITS-2:0] s2_bx1_full = s1_abs_b_r * s2_x1;
    wire [FRAC_BITS:0] s2_p2 = s2_bx1_full[2*FRAC_BITS-2 : FRAC_BITS-2];

    reg                      s2_sign_r;
    reg                      s2_a_sign_r;
    reg signed [EXP_BITS:0]  s2_a_exp_adj_r, s2_b_exp_adj_r;
    reg [MAG-1:0]            s2_abs_a_r, s2_abs_b_r;
    reg [FRAC_BITS-1:0]      s2_x1_r;
    reg [FRAC_BITS:0]        s2_p2_r;
    reg                      s2_b_zero_r;

    always @(posedge clk) begin
        s2_sign_r      <= s1_sign_r;
        if (ENABLE_MOD) s2_a_sign_r <= s1_a_sign_r;
        s2_a_exp_adj_r <= s1_a_exp_adj_r;
        s2_b_exp_adj_r <= s1_b_exp_adj_r;
        s2_abs_a_r     <= s1_abs_a_r;
        s2_abs_b_r     <= s1_abs_b_r;
        s2_x1_r        <= s2_x1;
        s2_p2_r        <= s2_p2;
        s2_b_zero_r    <= s1_b_zero_r;
    end

    // =========================================================================
    // Stage 3: E2 = 2^25 - P2, X2 = (X1 * E2) >> 24
    // =========================================================================

    wire [FRAC_BITS:0] s3_e2 = {1'b1, {FRAC_BITS{1'b0}}} - s2_p2_r;

    wire [FRAC_BITS + FRAC_BITS:0] s3_x1e2_full = s2_x1_r * s3_e2;
    wire [FRAC_BITS-1:0] s3_x2 = s3_x1e2_full[2*FRAC_BITS-2 : FRAC_BITS-1];

    reg                      s3_sign_r;
    reg                      s3_a_sign_r;
    reg signed [EXP_BITS:0]  s3_a_exp_adj_r, s3_b_exp_adj_r;
    reg [MAG-1:0]            s3_abs_a_r, s3_abs_b_r;
    reg [FRAC_BITS-1:0]      s3_x2_r;
    reg                      s3_b_zero_r;

    always @(posedge clk) begin
        s3_sign_r      <= s2_sign_r;
        if (ENABLE_MOD) s3_a_sign_r <= s2_a_sign_r;
        s3_a_exp_adj_r <= s2_a_exp_adj_r;
        s3_b_exp_adj_r <= s2_b_exp_adj_r;
        s3_abs_a_r     <= s2_abs_a_r;
        s3_abs_b_r     <= s2_abs_b_r;
        s3_x2_r        <= s3_x2;
        s3_b_zero_r    <= s2_b_zero_r;
    end

    // =========================================================================
    // Stage 4: Q_raw = (abs_a * X2) >> 22
    // =========================================================================

    wire [2*FRAC_BITS-2:0] s4_ax2_full = s3_abs_a_r * s3_x2_r;
    wire [FRAC_BITS:0] s4_q_raw = s4_ax2_full[2*FRAC_BITS-3 : FRAC_BITS-3];

    reg                      s4_sign_r;
    reg                      s4_a_sign_r;
    reg signed [EXP_BITS:0]  s4_a_exp_adj_r, s4_b_exp_adj_r;
    reg [FRAC_BITS:0]        s4_q_raw_r;
    reg [MAG-1:0]            s4_abs_a_r, s4_abs_b_r;
    reg                      s4_b_zero_r;

    always @(posedge clk) begin
        s4_sign_r       <= s3_sign_r;
        if (ENABLE_MOD) s4_a_sign_r <= s3_a_sign_r;
        s4_a_exp_adj_r  <= s3_a_exp_adj_r;
        s4_b_exp_adj_r  <= s3_b_exp_adj_r;
        s4_q_raw_r      <= s4_q_raw;
        s4_abs_a_r      <= s3_abs_a_r;
        s4_abs_b_r      <= s3_abs_b_r;
        s4_b_zero_r     <= s3_b_zero_r;
    end

    // =========================================================================
    // Stage 5: Remainder correction — exact Euclidean quotient + remainder
    // =========================================================================

    wire [FRAC_BITS + FRAC_BITS - 1:0] s5_p_check = s4_q_raw_r * s4_abs_b_r;
    wire [WIDE-1:0] s5_a_shifted = {{(FRAC_BITS+1){1'b0}}, s4_abs_a_r} << FRAC_BITS;

    wire signed [WIDE:0] s5_r = $signed({1'b0, s5_a_shifted})
                                - $signed({1'b0, s5_p_check});

    wire [WIDE-1:0] s5_1b = {{(FRAC_BITS+1){1'b0}}, s4_abs_b_r};
    wire [WIDE-1:0] s5_2b = {{FRAC_BITS{1'b0}}, s4_abs_b_r, 1'b0};
    wire [WIDE-1:0] s5_3b = s5_1b + s5_2b;
    wire [WIDE-1:0] s5_4b = {{(FRAC_BITS-1){1'b0}}, s4_abs_b_r, 2'b0};

    wire s5_r_neg = s5_r[WIDE];
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
    wire s5_r_lt_n1b = s5_r_p1b[WIDE];
    wire s5_r_lt_n2b = s5_r_p2b[WIDE];
    wire s5_r_lt_n3b = s5_r_p3b[WIDE];

    wire signed [3:0] s5_delta = s5_r_neg ? (s5_r_lt_n3b ? -4'sd4 :
                                              s5_r_lt_n2b ? -4'sd3 :
                                              s5_r_lt_n1b ? -4'sd2 : -4'sd1) :
                                  s5_r_ge_4b ? 4'sd4 :
                                  s5_r_ge_3b ? 4'sd3 :
                                  s5_r_ge_2b ? 4'sd2 :
                                  s5_r_ge_1b ? 4'sd1 : 4'sd0;

    wire [FRAC_BITS:0] s5_q_exact = $unsigned($signed({1'b0, s4_q_raw_r}) + s5_delta);

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

    // --- Q_lo extraction for modulo (runs parallel to remainder correction) ---
    // These depend only on s4 exponent registers, not on Q_exact, so they run
    // in parallel with the ±4 delta correction and add zero critical path.
    // Q_lo = bottom (F-d) bits of Q_exact — the only gate on Q_exact's path
    // is a single AND, adding ~0.5ns.
    wire signed [EXP_BITS:0] s5_d_comb = s4_a_exp_adj_r - s4_b_exp_adj_r;
    wire s5_d_neg_comb = s5_d_comb[EXP_BITS];
    wire s5_d_gt_F_comb = !s5_d_neg_comb && ($unsigned(s5_d_comb) > FRAC_BITS);
    wire [4:0] s5_f_minus_d_comb = FRAC_BITS[4:0] - s5_d_comb[4:0];
    wire [4:0] s5_neg_d_raw_comb = -s5_d_comb[4:0];
    wire s5_neg_d_large_comb = s5_d_neg_comb &&
        (s5_d_comb < -$signed({{(EXP_BITS-4){1'b0}}, 5'd31}));
    wire [4:0] s5_neg_d_comb = s5_neg_d_large_comb ? 5'd31 : s5_neg_d_raw_comb;
    wire [FRAC_BITS:0] s5_one_shl = ({1'b0, {FRAC_BITS{1'b0}}} | 1'b1) << s5_f_minus_d_comb;
    wire [FRAC_BITS-1:0] s5_q_lo_mask = s5_one_shl[FRAC_BITS-1:0] - 1'b1;
    wire [FRAC_BITS-1:0] s5_q_lo_comb = (s5_f_minus_d_comb == 0) ? {FRAC_BITS{1'b0}} :
                                          (s5_q_exact[FRAC_BITS-1:0] & s5_q_lo_mask);

    // --- Stage 5 registers (quotient + modulo passthrough) ---
    reg                      s5_sign_r;
    reg                      s5_a_sign_r;
    reg signed [EXP_BITS:0]  s5_a_exp_adj_r, s5_b_exp_adj_r;
    reg [FRAC_BITS:0]        s5_q_exact_r;
    reg                      s5_rem_sticky_r;
    reg                      s5_b_zero_r;
    // Modulo-specific registers
    reg [MAG-1:0]            s5_abs_a_r;
    reg [MAG-1:0]            s5_abs_b_r;
    reg [MAG-1:0]            s5_r_exact_r;   // bottom MAG bits of R_exact (< abs_b)
    // Pre-computed modulo control (from stage 5 comb, registered for stage 6)
    reg [FRAC_BITS-1:0]      s5_q_lo_r;
    reg [4:0]                s5_f_minus_d_r;
    reg [4:0]                s5_neg_d_r;
    reg                      s5_d_neg_r;
    reg                      s5_d_gt_F_r;

    always @(posedge clk) begin
        s5_sign_r       <= s4_sign_r;
        s5_a_exp_adj_r  <= s4_a_exp_adj_r;
        s5_b_exp_adj_r  <= s4_b_exp_adj_r;
        s5_q_exact_r    <= s5_q_exact;
        s5_rem_sticky_r <= s5_rem_sticky;
        s5_b_zero_r     <= s4_b_zero_r;
        if (ENABLE_MOD) begin
            s5_a_sign_r     <= s4_a_sign_r;
            s5_abs_a_r      <= s4_abs_a_r;
            s5_abs_b_r      <= s4_abs_b_r;
            s5_r_exact_r    <= s5_r_exact[MAG-1:0];
            s5_q_lo_r       <= s5_q_lo_comb;
            s5_f_minus_d_r  <= s5_f_minus_d_comb;
            s5_neg_d_r      <= s5_neg_d_comb;
            s5_d_neg_r      <= s5_d_neg_comb;
            s5_d_gt_F_r     <= s5_d_gt_F_comb;
        end
    end

    // =========================================================================
    // Stage 6: Quotient finalization + Modulo computation
    //
    // Quotient path: identical to spirix_divide_nr stage 6.
    // Modulo path: compute (abs_a << d) mod abs_b from Q_exact and R_exact.
    // =========================================================================

    // --- Quotient path (unchanged) ---
    wire s6_norm_shift = s5_q_exact_r[FRAC_BITS];
    wire [FRAC_BITS:0] s6_q_norm = s6_norm_shift ? (s5_q_exact_r >> 1) : s5_q_exact_r;

    wire [FRAC_BITS-1:0] s6_frac_pos_raw = s6_q_norm[FRAC_BITS:1];

    wire s6_guard  = s6_q_norm[0];
    wire s6_norm_sticky = s6_norm_shift & s5_q_exact_r[0];
    wire s6_sticky = s6_norm_sticky | s5_rem_sticky_r;
    wire s6_lsb    = s6_frac_pos_raw[0];
    wire s6_round_up = s6_guard & (s6_sticky | s6_lsb);

    wire [FRAC_BITS-1:0] s6_frac_rounded = s6_frac_pos_raw
                                             + {{(FRAC_BITS-1){1'b0}}, s6_round_up};

    wire s6_round_ovf = (&s6_frac_pos_raw[FRAC_BITS-2:0]) & s6_round_up;
    wire [FRAC_BITS-1:0] s6_pos_frac = s6_round_ovf ? POS_HALF : s6_frac_rounded;

    wire s6_neg_is_pos_half = (s6_pos_frac == POS_HALF);
    wire signed [FRAC_BITS-1:0] s6_neg_frac = s6_neg_is_pos_half ? NEG_ONE :
                                                (~s6_pos_frac + 1'b1);
    wire signed [FRAC_BITS-1:0] s6_final_frac = s5_sign_r ? s6_neg_frac :
                                                  $signed(s6_pos_frac);

    wire signed [EXP_BITS:0] s6_exp_base = s5_a_exp_adj_r - s5_b_exp_adj_r
                                           + {{EXP_BITS{1'b0}}, s6_norm_shift}
                                           + {{EXP_BITS{1'b0}}, s6_round_ovf};

    wire signed [EXP_BITS:0] s6_exp_final = (s5_sign_r & s6_neg_is_pos_half) ?
                                              (s6_exp_base - 1) : s6_exp_base;

    wire s6_exp_too_big   = (s6_exp_final > MAX_EXP);
    wire s6_exp_too_small = (s6_exp_final < MIN_EXP);
    wire signed [EXP_BITS-1:0] s6_final_exp = s6_exp_final[EXP_BITS-1:0];

    // --- Quotient output register ---
    always @(posedge clk) begin
        q_frac <= s5_b_zero_r     ? {FRAC_BITS{1'b0}} :
                  s6_exp_too_big   ? s6_final_frac :
                  s6_exp_too_small ? {s6_final_frac[FRAC_BITS-1],
                                       s6_final_frac[FRAC_BITS-1:1]} :
                                      s6_final_frac;

        q_exp  <= (s5_b_zero_r | s6_exp_too_big | s6_exp_too_small) ?
                   AMBIGUOUS_EXP[EXP_BITS-1:0] : s6_final_exp;
    end

    // --- Modulo path (stage 6): DSP multiply + add only ---
    // Q_lo, shift amounts, and flags were pre-computed in stage 5 and registered.
    // Stage 6 computes M = Q_lo * abs_b + R_exact, then registers everything
    // for the barrel shift + correction in stage 7.

    // Sign info for floored modulo (just aliases for registered values)
    wire s6_signs_differ = s5_sign_r;  // a_sign XOR b_sign
    wire s6_b_sign = s5_a_sign_r ^ s5_sign_r;

    // M = Q_lo * abs_b + R_exact (one DSP multiply + add, from registered inputs)
    wire [WIDE-2:0] s6_qlo_x_b = s5_q_lo_r * s5_abs_b_r;  // 25x24 = 49 bits
    wire [WIDE-2:0] s6_M = s6_qlo_x_b + {{FRAC_BITS{1'b0}}, s5_r_exact_r};

    // --- Stage 6 modulo intermediate registers (before barrel shift) ---
    reg [WIDE-2:0]           s6_M_r;
    reg [MAG-1:0]            s6_abs_a_r;
    reg [MAG-1:0]            s6_abs_b_r;
    reg                      s6_d_neg_r;
    reg [4:0]                s6_f_minus_d_r;
    reg [4:0]                s6_neg_d_r;
    reg                      s6_signs_differ_r;
    reg                      s6_b_sign_r;
    reg                      s6_b_zero_mod_r;
    reg                      s6_d_gt_F_mod_r;
    reg signed [EXP_BITS:0]  s6_a_exp_mod_r;
    reg signed [EXP_BITS:0]  s6_b_exp_mod_r;

    always @(posedge clk) begin
        if (ENABLE_MOD) begin
            s6_M_r              <= s6_M;
            s6_abs_a_r          <= s5_abs_a_r;
            s6_abs_b_r          <= s5_abs_b_r;
            s6_d_neg_r          <= s5_d_neg_r;
            s6_f_minus_d_r      <= s5_f_minus_d_r;
            s6_neg_d_r          <= s5_neg_d_r;
            s6_signs_differ_r   <= s6_signs_differ;
            s6_b_sign_r         <= s6_b_sign;
            s6_b_zero_mod_r     <= s5_b_zero_r;
            s6_d_gt_F_mod_r     <= s5_d_gt_F_r;
            s6_a_exp_mod_r      <= s5_a_exp_adj_r;
            s6_b_exp_mod_r      <= s5_b_exp_adj_r;
        end
    end

    // =========================================================================
    // Stage 7: Barrel shift + conditional subtract + sign correction
    //
    // All inputs come from s6 registers. Critical path: barrel shifter (5 levels)
    // + conditional subtract + sign correction mux.
    // =========================================================================

    // Barrel shifter input selection
    wire [WIDE-2:0] s7_brl_in = s6_d_neg_r ?
        {{FRAC_BITS{1'b0}}, s6_abs_a_r} : s6_M_r;
    wire [4:0] s7_brl_amt = s6_d_neg_r ? s6_neg_d_r : s6_f_minus_d_r;

    // 5-stage barrel shifter with sticky tracking
    wire [WIDE-2:0] s7_brl0 = s7_brl_amt[0] ? (s7_brl_in >> 1)  : s7_brl_in;
    wire s7_stk0 = s7_brl_amt[0] & s7_brl_in[0];

    wire [WIDE-2:0] s7_brl1 = s7_brl_amt[1] ? (s7_brl0 >> 2)  : s7_brl0;
    wire s7_stk1 = s7_stk0 | (s7_brl_amt[1] & |s7_brl0[1:0]);

    wire [WIDE-2:0] s7_brl2 = s7_brl_amt[2] ? (s7_brl1 >> 4)  : s7_brl1;
    wire s7_stk2 = s7_stk1 | (s7_brl_amt[2] & |s7_brl1[3:0]);

    wire [WIDE-2:0] s7_brl3 = s7_brl_amt[3] ? (s7_brl2 >> 8)  : s7_brl2;
    wire s7_stk3 = s7_stk2 | (s7_brl_amt[3] & |s7_brl2[7:0]);

    wire [WIDE-2:0] s7_brl4 = s7_brl_amt[4] ? (s7_brl3 >> 16) : s7_brl3;
    wire s7_stk4 = s7_stk3 | (s7_brl_amt[4] & |s7_brl3[15:0]);

    wire [MAG-1:0] s7_shifted = s7_brl4[MAG-1:0];  // 24-bit result

    // --- General case (d >= 0): conditional subtraction ---
    wire s7_S_ge_b = ({1'b0, s7_shifted} >= {1'b0, s6_abs_b_r});
    wire [MAG-1:0] s7_euclid_rem = s7_S_ge_b ? (s7_shifted - s6_abs_b_r) : s7_shifted;
    wire s7_euclid_nonzero = |s7_euclid_rem | s7_stk4;

    // Floored-mod sign correction: complement if signs differ and remainder nonzero
    wire s7_complement = s6_signs_differ_r & s7_euclid_nonzero;
    wire [MAG-1:0] s7_mod_general = s7_complement ?
        (s6_abs_b_r - s7_euclid_rem) : s7_euclid_rem;

    // --- Bypass case (d < 0): ---
    wire [MAG-1:0] s7_bypass_diff = s6_abs_b_r - s7_shifted;
    wire s7_bypass_diff_zero = (s7_bypass_diff == 0) && !s7_stk4;
    wire [MAG-1:0] s7_bypass_val = s6_signs_differ_r ? s7_bypass_diff : s6_abs_a_r;

    // --- Final modulo magnitude selection ---
    wire s7_mod_is_zero = s6_b_zero_mod_r | s6_d_gt_F_mod_r |
                           (!s6_d_neg_r && !s7_euclid_nonzero) |
                           (s6_d_neg_r && s6_signs_differ_r && s7_bypass_diff_zero);
    wire s7_mod_ambiguous = s6_b_zero_mod_r | s6_d_gt_F_mod_r;

    wire [MAG-1:0] s7_mod_mag = s6_d_neg_r ? s7_bypass_val : s7_mod_general;

    // Modulo exponent: b's scale for d>=0 and d<0 diff signs, a's scale for d<0 same signs
    wire signed [EXP_BITS:0] s7_mod_exp_raw = (s6_d_neg_r && !s6_signs_differ_r) ?
                                                s6_a_exp_mod_r : s6_b_exp_mod_r;

    // --- Stage 7 modulo registers ---
    reg [MAG-1:0]            s7_mod_mag_r;
    reg signed [EXP_BITS:0]  s7_mod_exp_r;
    reg                      s7_mod_sign_r;
    reg                      s7_mod_zero_r;
    reg                      s7_mod_ambig_r;

    always @(posedge clk) begin
        if (ENABLE_MOD) begin
            s7_mod_mag_r   <= s7_mod_mag;
            s7_mod_exp_r   <= s7_mod_exp_raw;
            s7_mod_sign_r  <= s6_b_sign_r;
            s7_mod_zero_r  <= s7_mod_is_zero;
            s7_mod_ambig_r <= s7_mod_ambiguous;
        end
    end

    // =========================================================================
    // Stage 8: Modulo normalization + sign + exponent + clamp
    //
    // mod_mag is unsigned MAG-bit value in [0, abs_b). May not be N1-normalized.
    // CLZ finds leading 1, barrel shift normalizes, then apply sign and exponent.
    // =========================================================================

    // CLZ on MAG-bit unsigned value (find position of highest set bit)
    // Result: 0 means bit[MAG-1] is set (already normalized), MAG means all zeros.
    reg [$clog2(MAG+1)-1:0] s8_clz;
    integer mi;
    always @(*) begin
        s8_clz = MAG[$clog2(MAG+1)-1:0];
        for (mi = 0; mi <= MAG - 1; mi = mi + 1)
            if (s7_mod_mag_r[mi])
                s8_clz = MAG - 1 - mi;
    end

    // Left barrel shift to normalize: put leading 1 at bit[MAG-1]
    wire [MAG-1:0] s8_mod_norm = s7_mod_mag_r << s8_clz;

    // Positive N1 fraction: {0, normalized_magnitude}
    wire [FRAC_BITS-1:0] s8_pos_frac = {1'b0, s8_mod_norm};

    // Apply sign (b_sign for floored modulo)
    wire s8_neg_is_pos_half = (s8_pos_frac == POS_HALF);

    wire signed [FRAC_BITS-1:0] s8_neg_frac = s8_neg_is_pos_half ? NEG_ONE :
                                                (~s8_pos_frac + 1'b1);

    wire signed [FRAC_BITS-1:0] s8_signed_frac = s7_mod_sign_r ? s8_neg_frac :
                                                   $signed(s8_pos_frac);

    // Exponent: raw_exp - clz, with NEG_ONE adjustment
    wire signed [EXP_BITS:0] s8_exp_adj = s7_mod_exp_r
                                          - {{(EXP_BITS+1-$clog2(MAG+1)){1'b0}}, s8_clz};

    wire signed [EXP_BITS:0] s8_exp_final = (s7_mod_sign_r & s8_neg_is_pos_half) ?
                                              (s8_exp_adj - 1) : s8_exp_adj;

    wire s8_exp_too_big   = (s8_exp_final > MAX_EXP);
    wire s8_exp_too_small = (s8_exp_final < MIN_EXP);

    // --- Modulo output register ---
    always @(posedge clk) begin
        if (ENABLE_MOD) begin
            mod_frac <= (s7_mod_zero_r | s7_mod_ambig_r) ? {FRAC_BITS{1'b0}} :
                        s8_exp_too_big   ? s8_signed_frac :
                        s8_exp_too_small ? {s8_signed_frac[FRAC_BITS-1],
                                             s8_signed_frac[FRAC_BITS-1:1]} :
                                            s8_signed_frac;

            mod_exp  <= (s7_mod_zero_r | s7_mod_ambig_r | s8_exp_too_big | s8_exp_too_small) ?
                        AMBIGUOUS_EXP[EXP_BITS-1:0] : s8_exp_final[EXP_BITS-1:0];
        end else begin
            mod_frac <= {FRAC_BITS{1'b0}};
            mod_exp  <= AMBIGUOUS_EXP[EXP_BITS-1:0];
        end
    end

endmodule
