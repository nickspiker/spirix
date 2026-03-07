// spirix_nr_divsqrt — Shared Newton-Raphson divide/sqrt for Spirix scalars
//
// Floor-only (no rounding). Single shared multiplier, FSM-sequenced.
// Parameterized: FRAC_BITS in {12..128}, EXP_BITS in {2..128}.
//
// Architecture: one (FRAC+1)×(FRAC+1) multiplier serves both divide and sqrt.
// NR iteration count auto-scales: 1 iter for F≤18, 2 for F≤38, 3 for F≤78.
//
// Divide algorithm (2*NR+4 cycles):
//   Setup: abs values, reciprocal LUT seed → x₀ ≈ 1/|b|
//   NR iterations: E = 2^F − B×X, X_new = X×E >> (F−1)
//   Final: Q_raw = A×X >> shift, remainder check Q×B, ±4 correction
//   Finalize: normalize, floor extract, sign, exponent
//
// Sqrt algorithm (3*NR+4 cycles):
//   Setup: radicand S, inverse-sqrt LUT seed → x₀ ≈ 1/√S
//   NR iterations: H=X², P=S×H, E=3·2^(F−1)−P, X_new=X×E>>(F)
//   Final: raw = S×X >> shift, raw² for ±4 correction
//   Finalize: normalize, floor extract, exponent
//
// Interface:
//   start: pulse high for 1 cycle. Inputs sampled on this edge.
//   mode:  0 = divide (a/b), 1 = sqrt(a). b_frac/b_exp ignored for sqrt.
//   busy:  high while computing. Do not assert start while busy.
//   done:  pulses high for 1 cycle when result is valid.
//
// Multiplier port (directly usable by external multiply op when idle):
//   mul_a, mul_b: operand outputs. mul_prod: product input.
//   When !busy, mul_a/mul_b hold the values from the last operation
//   and can be overridden externally for a standalone multiply.

module spirix_nr_divsqrt #(
    parameter FRAC_BITS = 32,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire start,
    input  wire mode,           // 0 = divide, 1 = sqrt
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    input  wire signed [FRAC_BITS-1:0] b_frac,
    input  wire signed [EXP_BITS-1:0]  b_exp,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp,
    output wire busy,
    output reg  done = 0
);

    // =========================================================================
    // Parameters
    // =========================================================================

    localparam AMBIG_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam MAG = FRAC_BITS - 1;

    // NR iteration count: LUT gives ~10 bits, each iteration doubles.
    // Sqrt has 3 truncation points per iter (vs 2 for divide), so needs
    // more conservative threshold. Use 1 iter only for F≤12.
    localparam NR_ITERS = (FRAC_BITS <= 12) ? 1 :
                          (FRAC_BITS <= 26) ? 2 :
                          (FRAC_BITS <= 54) ? 3 : 4;

    // Cycle counts
    localparam DIV_STEPS  = 2 * NR_ITERS + 4;
    localparam SQRT_STEPS = 3 * NR_ITERS + 4;
    localparam MAX_STEPS  = SQRT_STEPS;  // sqrt is always >= div
    localparam STEP_BITS  = $clog2(MAX_STEPS + 1);

    // Multiplier width
    localparam MW = FRAC_BITS + 1;
    localparam PW = 2 * MW;

    // Exponent calc width
    localparam ECW = EXP_BITS + 2;

    // Wide product for remainder check
    localparam WIDE = 2 * FRAC_BITS;

    // =========================================================================
    // State
    // =========================================================================

    reg running = 0;
    assign busy = running;

    reg [STEP_BITS-1:0] step;
    reg                  mode_r;

    // Operand registers
    reg [MAG-1:0]            abs_a_r, abs_b_r;
    reg                      sign_r;
    reg                      a_neg_r;     // for Euclidean adjustment
    reg                      b_zero_r;
    reg                      invalid_r;   // sqrt: negative or ambig input
    reg signed [EXP_BITS:0]  a_exp_adj_r, b_exp_adj_r;

    // NR working registers
    reg [FRAC_BITS-1:0]      x_reg;      // current NR approximation
    reg [FRAC_BITS-1:0]      S_reg;      // radicand (sqrt mode)
    reg [FRAC_BITS:0]        q_reg;      // quotient/sqrt raw result

    // Sqrt exponent
    reg signed [EXP_BITS:0]  sqrt_exp_r;

    // =========================================================================
    // Multiplier (single shared, combinational + registered output)
    // =========================================================================

    reg  [MW-1:0]  mul_a, mul_b;
    wire [PW-1:0]  mul_prod_comb = mul_a * mul_b;

    // =========================================================================
    // Reciprocal LUT (divide): 256 × 10 bits
    // Index: top 8 magnitude bits of |b| (excluding leading 1).
    // Output: lut ≈ 1024/((128+i)/128) = 131072/(128+i)
    // =========================================================================

    wire [7:0] recip_idx = abs_b_comb[MAG-2 : MAG-9];
    reg  [9:0] recip_lut;

    wire [MAG-1:0] abs_a_comb, abs_b_comb;
    wire a_is_neg_one = (a_frac == NEG_ONE);
    wire b_is_neg_one = (b_frac == NEG_ONE);

    assign abs_a_comb = a_is_neg_one ? POS_HALF[MAG-1:0] :
                         (a_frac[FRAC_BITS-1] ? (~a_frac[MAG-1:0] + 1'b1) :
                                                 a_frac[MAG-1:0]);
    assign abs_b_comb = b_is_neg_one ? POS_HALF[MAG-1:0] :
                         (b_frac[FRAC_BITS-1] ? (~b_frac[MAG-1:0] + 1'b1) :
                                                 b_frac[MAG-1:0]);

    always @(*) begin
        case (recip_idx)
            8'd0: recip_lut = 10'd1022; 8'd1: recip_lut = 10'd1018;
            8'd2: recip_lut = 10'd1014; 8'd3: recip_lut = 10'd1010;
            8'd4: recip_lut = 10'd1006; 8'd5: recip_lut = 10'd1002;
            8'd6: recip_lut = 10'd998;  8'd7: recip_lut = 10'd994;
            8'd8: recip_lut = 10'd991;  8'd9: recip_lut = 10'd987;
            8'd10: recip_lut = 10'd983; 8'd11: recip_lut = 10'd979;
            8'd12: recip_lut = 10'd976; 8'd13: recip_lut = 10'd972;
            8'd14: recip_lut = 10'd969; 8'd15: recip_lut = 10'd965;
            8'd16: recip_lut = 10'd961; 8'd17: recip_lut = 10'd958;
            8'd18: recip_lut = 10'd954; 8'd19: recip_lut = 10'd951;
            8'd20: recip_lut = 10'd948; 8'd21: recip_lut = 10'd944;
            8'd22: recip_lut = 10'd941; 8'd23: recip_lut = 10'd937;
            8'd24: recip_lut = 10'd934; 8'd25: recip_lut = 10'd931;
            8'd26: recip_lut = 10'd927; 8'd27: recip_lut = 10'd924;
            8'd28: recip_lut = 10'd921; 8'd29: recip_lut = 10'd918;
            8'd30: recip_lut = 10'd914; 8'd31: recip_lut = 10'd911;
            8'd32: recip_lut = 10'd908; 8'd33: recip_lut = 10'd905;
            8'd34: recip_lut = 10'd902; 8'd35: recip_lut = 10'd899;
            8'd36: recip_lut = 10'd896; 8'd37: recip_lut = 10'd893;
            8'd38: recip_lut = 10'd890; 8'd39: recip_lut = 10'd887;
            8'd40: recip_lut = 10'd884; 8'd41: recip_lut = 10'd881;
            8'd42: recip_lut = 10'd878; 8'd43: recip_lut = 10'd875;
            8'd44: recip_lut = 10'd872; 8'd45: recip_lut = 10'd869;
            8'd46: recip_lut = 10'd866; 8'd47: recip_lut = 10'd863;
            8'd48: recip_lut = 10'd860; 8'd49: recip_lut = 10'd858;
            8'd50: recip_lut = 10'd855; 8'd51: recip_lut = 10'd852;
            8'd52: recip_lut = 10'd849; 8'd53: recip_lut = 10'd846;
            8'd54: recip_lut = 10'd844; 8'd55: recip_lut = 10'd841;
            8'd56: recip_lut = 10'd838; 8'd57: recip_lut = 10'd836;
            8'd58: recip_lut = 10'd833; 8'd59: recip_lut = 10'd830;
            8'd60: recip_lut = 10'd828; 8'd61: recip_lut = 10'd825;
            8'd62: recip_lut = 10'd823; 8'd63: recip_lut = 10'd820;
            8'd64: recip_lut = 10'd817; 8'd65: recip_lut = 10'd815;
            8'd66: recip_lut = 10'd812; 8'd67: recip_lut = 10'd810;
            8'd68: recip_lut = 10'd807; 8'd69: recip_lut = 10'd805;
            8'd70: recip_lut = 10'd802; 8'd71: recip_lut = 10'd800;
            8'd72: recip_lut = 10'd798; 8'd73: recip_lut = 10'd795;
            8'd74: recip_lut = 10'd793; 8'd75: recip_lut = 10'd790;
            8'd76: recip_lut = 10'd788; 8'd77: recip_lut = 10'd786;
            8'd78: recip_lut = 10'd783; 8'd79: recip_lut = 10'd781;
            8'd80: recip_lut = 10'd779; 8'd81: recip_lut = 10'd776;
            8'd82: recip_lut = 10'd774; 8'd83: recip_lut = 10'd772;
            8'd84: recip_lut = 10'd769; 8'd85: recip_lut = 10'd767;
            8'd86: recip_lut = 10'd765; 8'd87: recip_lut = 10'd763;
            8'd88: recip_lut = 10'd760; 8'd89: recip_lut = 10'd758;
            8'd90: recip_lut = 10'd756; 8'd91: recip_lut = 10'd754;
            8'd92: recip_lut = 10'd752; 8'd93: recip_lut = 10'd750;
            8'd94: recip_lut = 10'd747; 8'd95: recip_lut = 10'd745;
            8'd96: recip_lut = 10'd743; 8'd97: recip_lut = 10'd741;
            8'd98: recip_lut = 10'd739; 8'd99: recip_lut = 10'd737;
            8'd100: recip_lut = 10'd735; 8'd101: recip_lut = 10'd733;
            8'd102: recip_lut = 10'd731; 8'd103: recip_lut = 10'd729;
            8'd104: recip_lut = 10'd727; 8'd105: recip_lut = 10'd725;
            8'd106: recip_lut = 10'd723; 8'd107: recip_lut = 10'd721;
            8'd108: recip_lut = 10'd719; 8'd109: recip_lut = 10'd717;
            8'd110: recip_lut = 10'd715; 8'd111: recip_lut = 10'd713;
            8'd112: recip_lut = 10'd711; 8'd113: recip_lut = 10'd709;
            8'd114: recip_lut = 10'd707; 8'd115: recip_lut = 10'd705;
            8'd116: recip_lut = 10'd703; 8'd117: recip_lut = 10'd701;
            8'd118: recip_lut = 10'd699; 8'd119: recip_lut = 10'd698;
            8'd120: recip_lut = 10'd696; 8'd121: recip_lut = 10'd694;
            8'd122: recip_lut = 10'd692; 8'd123: recip_lut = 10'd690;
            8'd124: recip_lut = 10'd688; 8'd125: recip_lut = 10'd687;
            8'd126: recip_lut = 10'd685; 8'd127: recip_lut = 10'd683;
            8'd128: recip_lut = 10'd681; 8'd129: recip_lut = 10'd680;
            8'd130: recip_lut = 10'd678; 8'd131: recip_lut = 10'd676;
            8'd132: recip_lut = 10'd674; 8'd133: recip_lut = 10'd673;
            8'd134: recip_lut = 10'd671; 8'd135: recip_lut = 10'd669;
            8'd136: recip_lut = 10'd667; 8'd137: recip_lut = 10'd666;
            8'd138: recip_lut = 10'd664; 8'd139: recip_lut = 10'd662;
            8'd140: recip_lut = 10'd661; 8'd141: recip_lut = 10'd659;
            8'd142: recip_lut = 10'd657; 8'd143: recip_lut = 10'd656;
            8'd144: recip_lut = 10'd654; 8'd145: recip_lut = 10'd652;
            8'd146: recip_lut = 10'd651; 8'd147: recip_lut = 10'd649;
            8'd148: recip_lut = 10'd648; 8'd149: recip_lut = 10'd646;
            8'd150: recip_lut = 10'd644; 8'd151: recip_lut = 10'd643;
            8'd152: recip_lut = 10'd641; 8'd153: recip_lut = 10'd640;
            8'd154: recip_lut = 10'd638; 8'd155: recip_lut = 10'd637;
            8'd156: recip_lut = 10'd635; 8'd157: recip_lut = 10'd633;
            8'd158: recip_lut = 10'd632; 8'd159: recip_lut = 10'd630;
            8'd160: recip_lut = 10'd629; 8'd161: recip_lut = 10'd627;
            8'd162: recip_lut = 10'd626; 8'd163: recip_lut = 10'd624;
            8'd164: recip_lut = 10'd623; 8'd165: recip_lut = 10'd621;
            8'd166: recip_lut = 10'd620; 8'd167: recip_lut = 10'd618;
            8'd168: recip_lut = 10'd617; 8'd169: recip_lut = 10'd616;
            8'd170: recip_lut = 10'd614; 8'd171: recip_lut = 10'd613;
            8'd172: recip_lut = 10'd611; 8'd173: recip_lut = 10'd610;
            8'd174: recip_lut = 10'd608; 8'd175: recip_lut = 10'd607;
            8'd176: recip_lut = 10'd606; 8'd177: recip_lut = 10'd604;
            8'd178: recip_lut = 10'd603; 8'd179: recip_lut = 10'd601;
            8'd180: recip_lut = 10'd600; 8'd181: recip_lut = 10'd599;
            8'd182: recip_lut = 10'd597; 8'd183: recip_lut = 10'd596;
            8'd184: recip_lut = 10'd595; 8'd185: recip_lut = 10'd593;
            8'd186: recip_lut = 10'd592; 8'd187: recip_lut = 10'd591;
            8'd188: recip_lut = 10'd589; 8'd189: recip_lut = 10'd588;
            8'd190: recip_lut = 10'd587; 8'd191: recip_lut = 10'd585;
            8'd192: recip_lut = 10'd584; 8'd193: recip_lut = 10'd583;
            8'd194: recip_lut = 10'd581; 8'd195: recip_lut = 10'd580;
            8'd196: recip_lut = 10'd579; 8'd197: recip_lut = 10'd578;
            8'd198: recip_lut = 10'd576; 8'd199: recip_lut = 10'd575;
            8'd200: recip_lut = 10'd574; 8'd201: recip_lut = 10'd572;
            8'd202: recip_lut = 10'd571; 8'd203: recip_lut = 10'd570;
            8'd204: recip_lut = 10'd569; 8'd205: recip_lut = 10'd568;
            8'd206: recip_lut = 10'd566; 8'd207: recip_lut = 10'd565;
            8'd208: recip_lut = 10'd564; 8'd209: recip_lut = 10'd563;
            8'd210: recip_lut = 10'd561; 8'd211: recip_lut = 10'd560;
            8'd212: recip_lut = 10'd559; 8'd213: recip_lut = 10'd558;
            8'd214: recip_lut = 10'd557; 8'd215: recip_lut = 10'd555;
            8'd216: recip_lut = 10'd554; 8'd217: recip_lut = 10'd553;
            8'd218: recip_lut = 10'd552; 8'd219: recip_lut = 10'd551;
            8'd220: recip_lut = 10'd550; 8'd221: recip_lut = 10'd548;
            8'd222: recip_lut = 10'd547; 8'd223: recip_lut = 10'd546;
            8'd224: recip_lut = 10'd545; 8'd225: recip_lut = 10'd544;
            8'd226: recip_lut = 10'd543; 8'd227: recip_lut = 10'd542;
            8'd228: recip_lut = 10'd541; 8'd229: recip_lut = 10'd539;
            8'd230: recip_lut = 10'd538; 8'd231: recip_lut = 10'd537;
            8'd232: recip_lut = 10'd536; 8'd233: recip_lut = 10'd535;
            8'd234: recip_lut = 10'd534; 8'd235: recip_lut = 10'd533;
            8'd236: recip_lut = 10'd532; 8'd237: recip_lut = 10'd531;
            8'd238: recip_lut = 10'd530; 8'd239: recip_lut = 10'd529;
            8'd240: recip_lut = 10'd527; 8'd241: recip_lut = 10'd526;
            8'd242: recip_lut = 10'd525; 8'd243: recip_lut = 10'd524;
            8'd244: recip_lut = 10'd523; 8'd245: recip_lut = 10'd522;
            8'd246: recip_lut = 10'd521; 8'd247: recip_lut = 10'd520;
            8'd248: recip_lut = 10'd519; 8'd249: recip_lut = 10'd518;
            8'd250: recip_lut = 10'd517; 8'd251: recip_lut = 10'd516;
            8'd252: recip_lut = 10'd515; 8'd253: recip_lut = 10'd514;
            8'd254: recip_lut = 10'd513; 8'd255: recip_lut = 10'd512;
        endcase
    end

    wire [FRAC_BITS-1:0] x0_div = {1'b0, recip_lut, {(FRAC_BITS-11){1'b0}}};

    // =========================================================================
    // Inverse-sqrt LUT (sqrt): 256 × 11 bits
    // Index: top 8 bits of radicand S.
    // S[F-1:F-8] ∈ [64,127] for even exp (s∈[0.5,1)), [128,255] for odd (s∈[1,2))
    // =========================================================================

    wire s_exp_odd = a_exp[0];
    wire [FRAC_BITS-1:0] S_comb = s_exp_odd ? {abs_a_comb, 1'b0} :
                                                {1'b0, abs_a_comb};
    wire [7:0] isqrt_idx = S_comb[FRAC_BITS-1 : FRAC_BITS-8];
    reg  [10:0] isqrt_lut;

    always @(*) begin
        case (isqrt_idx)
            8'd64: isqrt_lut = 11'd1443; 8'd65: isqrt_lut = 11'd1431;
            8'd66: isqrt_lut = 11'd1421; 8'd67: isqrt_lut = 11'd1410;
            8'd68: isqrt_lut = 11'd1400; 8'd69: isqrt_lut = 11'd1390;
            8'd70: isqrt_lut = 11'd1380; 8'd71: isqrt_lut = 11'd1370;
            8'd72: isqrt_lut = 11'd1361; 8'd73: isqrt_lut = 11'd1351;
            8'd74: isqrt_lut = 11'd1342; 8'd75: isqrt_lut = 11'd1333;
            8'd76: isqrt_lut = 11'd1325; 8'd77: isqrt_lut = 11'd1316;
            8'd78: isqrt_lut = 11'd1308; 8'd79: isqrt_lut = 11'd1299;
            8'd80: isqrt_lut = 11'd1291; 8'd81: isqrt_lut = 11'd1283;
            8'd82: isqrt_lut = 11'd1275; 8'd83: isqrt_lut = 11'd1268;
            8'd84: isqrt_lut = 11'd1260; 8'd85: isqrt_lut = 11'd1253;
            8'd86: isqrt_lut = 11'd1246; 8'd87: isqrt_lut = 11'd1239;
            8'd88: isqrt_lut = 11'd1231; 8'd89: isqrt_lut = 11'd1225;
            8'd90: isqrt_lut = 11'd1218; 8'd91: isqrt_lut = 11'd1211;
            8'd92: isqrt_lut = 11'd1205; 8'd93: isqrt_lut = 11'd1198;
            8'd94: isqrt_lut = 11'd1192; 8'd95: isqrt_lut = 11'd1186;
            8'd96: isqrt_lut = 11'd1179; 8'd97: isqrt_lut = 11'd1173;
            8'd98: isqrt_lut = 11'd1167; 8'd99: isqrt_lut = 11'd1161;
            8'd100: isqrt_lut = 11'd1156; 8'd101: isqrt_lut = 11'd1150;
            8'd102: isqrt_lut = 11'd1144; 8'd103: isqrt_lut = 11'd1139;
            8'd104: isqrt_lut = 11'd1133; 8'd105: isqrt_lut = 11'd1128;
            8'd106: isqrt_lut = 11'd1123; 8'd107: isqrt_lut = 11'd1117;
            8'd108: isqrt_lut = 11'd1112; 8'd109: isqrt_lut = 11'd1107;
            8'd110: isqrt_lut = 11'd1102; 8'd111: isqrt_lut = 11'd1097;
            8'd112: isqrt_lut = 11'd1092; 8'd113: isqrt_lut = 11'd1087;
            8'd114: isqrt_lut = 11'd1083; 8'd115: isqrt_lut = 11'd1078;
            8'd116: isqrt_lut = 11'd1073; 8'd117: isqrt_lut = 11'd1069;
            8'd118: isqrt_lut = 11'd1064; 8'd119: isqrt_lut = 11'd1060;
            8'd120: isqrt_lut = 11'd1055; 8'd121: isqrt_lut = 11'd1051;
            8'd122: isqrt_lut = 11'd1047; 8'd123: isqrt_lut = 11'd1042;
            8'd124: isqrt_lut = 11'd1038; 8'd125: isqrt_lut = 11'd1034;
            8'd126: isqrt_lut = 11'd1030; 8'd127: isqrt_lut = 11'd1026;
            8'd128: isqrt_lut = 11'd1022; 8'd129: isqrt_lut = 11'd1018;
            8'd130: isqrt_lut = 11'd1014; 8'd131: isqrt_lut = 11'd1010;
            8'd132: isqrt_lut = 11'd1006; 8'd133: isqrt_lut = 11'd1003;
            8'd134: isqrt_lut = 11'd999;  8'd135: isqrt_lut = 11'd995;
            8'd136: isqrt_lut = 11'd992;  8'd137: isqrt_lut = 11'd988;
            8'd138: isqrt_lut = 11'd984;  8'd139: isqrt_lut = 11'd981;
            8'd140: isqrt_lut = 11'd977;  8'd141: isqrt_lut = 11'd974;
            8'd142: isqrt_lut = 11'd971;  8'd143: isqrt_lut = 11'd967;
            8'd144: isqrt_lut = 11'd964;  8'd145: isqrt_lut = 11'd960;
            8'd146: isqrt_lut = 11'd957;  8'd147: isqrt_lut = 11'd954;
            8'd148: isqrt_lut = 11'd951;  8'd149: isqrt_lut = 11'd948;
            8'd150: isqrt_lut = 11'd944;  8'd151: isqrt_lut = 11'd941;
            8'd152: isqrt_lut = 11'd938;  8'd153: isqrt_lut = 11'd935;
            8'd154: isqrt_lut = 11'd932;  8'd155: isqrt_lut = 11'd929;
            8'd156: isqrt_lut = 11'd926;  8'd157: isqrt_lut = 11'd923;
            8'd158: isqrt_lut = 11'd920;  8'd159: isqrt_lut = 11'd917;
            8'd160: isqrt_lut = 11'd914;  8'd161: isqrt_lut = 11'd912;
            8'd162: isqrt_lut = 11'd909;  8'd163: isqrt_lut = 11'd906;
            8'd164: isqrt_lut = 11'd903;  8'd165: isqrt_lut = 11'd901;
            8'd166: isqrt_lut = 11'd898;  8'd167: isqrt_lut = 11'd895;
            8'd168: isqrt_lut = 11'd892;  8'd169: isqrt_lut = 11'd890;
            8'd170: isqrt_lut = 11'd887;  8'd171: isqrt_lut = 11'd885;
            8'd172: isqrt_lut = 11'd882;  8'd173: isqrt_lut = 11'd880;
            8'd174: isqrt_lut = 11'd877;  8'd175: isqrt_lut = 11'd875;
            8'd176: isqrt_lut = 11'd872;  8'd177: isqrt_lut = 11'd870;
            8'd178: isqrt_lut = 11'd867;  8'd179: isqrt_lut = 11'd865;
            8'd180: isqrt_lut = 11'd862;  8'd181: isqrt_lut = 11'd860;
            8'd182: isqrt_lut = 11'd858;  8'd183: isqrt_lut = 11'd855;
            8'd184: isqrt_lut = 11'd853;  8'd185: isqrt_lut = 11'd851;
            8'd186: isqrt_lut = 11'd848;  8'd187: isqrt_lut = 11'd846;
            8'd188: isqrt_lut = 11'd844;  8'd189: isqrt_lut = 11'd842;
            8'd190: isqrt_lut = 11'd839;  8'd191: isqrt_lut = 11'd837;
            8'd192: isqrt_lut = 11'd835;  8'd193: isqrt_lut = 11'd833;
            8'd194: isqrt_lut = 11'd831;  8'd195: isqrt_lut = 11'd829;
            8'd196: isqrt_lut = 11'd826;  8'd197: isqrt_lut = 11'd824;
            8'd198: isqrt_lut = 11'd822;  8'd199: isqrt_lut = 11'd820;
            8'd200: isqrt_lut = 11'd818;  8'd201: isqrt_lut = 11'd816;
            8'd202: isqrt_lut = 11'd814;  8'd203: isqrt_lut = 11'd812;
            8'd204: isqrt_lut = 11'd810;  8'd205: isqrt_lut = 11'd808;
            8'd206: isqrt_lut = 11'd806;  8'd207: isqrt_lut = 11'd804;
            8'd208: isqrt_lut = 11'd802;  8'd209: isqrt_lut = 11'd800;
            8'd210: isqrt_lut = 11'd799;  8'd211: isqrt_lut = 11'd797;
            8'd212: isqrt_lut = 11'd795;  8'd213: isqrt_lut = 11'd793;
            8'd214: isqrt_lut = 11'd791;  8'd215: isqrt_lut = 11'd789;
            8'd216: isqrt_lut = 11'd787;  8'd217: isqrt_lut = 11'd786;
            8'd218: isqrt_lut = 11'd784;  8'd219: isqrt_lut = 11'd782;
            8'd220: isqrt_lut = 11'd780;  8'd221: isqrt_lut = 11'd778;
            8'd222: isqrt_lut = 11'd777;  8'd223: isqrt_lut = 11'd775;
            8'd224: isqrt_lut = 11'd773;  8'd225: isqrt_lut = 11'd771;
            8'd226: isqrt_lut = 11'd770;  8'd227: isqrt_lut = 11'd768;
            8'd228: isqrt_lut = 11'd766;  8'd229: isqrt_lut = 11'd765;
            8'd230: isqrt_lut = 11'd763;  8'd231: isqrt_lut = 11'd761;
            8'd232: isqrt_lut = 11'd760;  8'd233: isqrt_lut = 11'd758;
            8'd234: isqrt_lut = 11'd757;  8'd235: isqrt_lut = 11'd755;
            8'd236: isqrt_lut = 11'd753;  8'd237: isqrt_lut = 11'd752;
            8'd238: isqrt_lut = 11'd750;  8'd239: isqrt_lut = 11'd749;
            8'd240: isqrt_lut = 11'd747;  8'd241: isqrt_lut = 11'd745;
            8'd242: isqrt_lut = 11'd744;  8'd243: isqrt_lut = 11'd742;
            8'd244: isqrt_lut = 11'd741;  8'd245: isqrt_lut = 11'd739;
            8'd246: isqrt_lut = 11'd738;  8'd247: isqrt_lut = 11'd736;
            8'd248: isqrt_lut = 11'd735;  8'd249: isqrt_lut = 11'd733;
            8'd250: isqrt_lut = 11'd732;  8'd251: isqrt_lut = 11'd731;
            8'd252: isqrt_lut = 11'd729;  8'd253: isqrt_lut = 11'd728;
            8'd254: isqrt_lut = 11'd726;  8'd255: isqrt_lut = 11'd725;
            default: isqrt_lut = 11'd1024;
        endcase
    end

    wire [FRAC_BITS-1:0] x0_sqrt = {isqrt_lut, {(FRAC_BITS-11){1'b0}}};

    // =========================================================================
    // Product: combinational (mul_a/mul_b registered, product instant)
    //
    // Setup sets mul_a/mul_b via NB. Next cycle they're active regs.
    // mul_prod_comb = mul_a * mul_b is valid in the same cycle mul_a/mul_b
    // are registered, so the FSM can read the product at step N from the
    // multiply issued at step N-1 (or setup).
    // =========================================================================

    wire [PW-1:0] prod_r = mul_prod_comb;

    // =========================================================================
    // Product extraction helpers
    // =========================================================================

    // P = (B×X) >> (F-2): top F+1 bits of (F-1)×F product
    wire [FRAC_BITS:0] extract_P = prod_r[2*FRAC_BITS-2 : FRAC_BITS-2];

    // X_new = (X×E) >> (F-1): top F bits of F×(F+1) product
    wire [FRAC_BITS-1:0] extract_X = prod_r[2*FRAC_BITS-2 : FRAC_BITS-1];

    // Q_raw = (A×X) >> (F-3): top F+1 bits of (F-1)×F product
    wire [FRAC_BITS:0] extract_Q = prod_r[2*FRAC_BITS-3 : FRAC_BITS-3];

    // Sqrt: H = X² >> (F-1): top F bits of F×F product (also P extraction)
    wire [FRAC_BITS-1:0] extract_H = prod_r[2*FRAC_BITS-2 : FRAC_BITS-1];

    // Sqrt: X_new = (X×E) >> F (extra /2 vs divide for inverse-sqrt NR)
    wire [FRAC_BITS-1:0] extract_X_sqrt = prod_r[2*FRAC_BITS-1 : FRAC_BITS];

    // Sqrt: raw = (S×X) >> (F-2): top F+1 bits
    wire [FRAC_BITS:0] extract_raw = prod_r[2*FRAC_BITS-2 : FRAC_BITS-2];

    // =========================================================================
    // Divide: Remainder correction (combinational, used at correction step)
    // At correction step: prod_r = Q_raw × B, q_reg = Q_raw
    // =========================================================================

    wire [WIDE-1:0] d_a_shifted = {{(FRAC_BITS+1){1'b0}}, abs_a_r} << FRAC_BITS;
    wire [WIDE-1:0] d_qb = prod_r[WIDE-1:0];
    wire signed [WIDE:0] d_rem = $signed({1'b0, d_a_shifted}) - $signed({1'b0, d_qb});

    wire [WIDE-1:0] d_1b = {{(FRAC_BITS+1){1'b0}}, abs_b_r};
    wire [WIDE-1:0] d_2b = {{FRAC_BITS{1'b0}}, abs_b_r, 1'b0};
    wire [WIDE-1:0] d_3b = d_1b + d_2b;
    wire [WIDE-1:0] d_4b = {{(FRAC_BITS-1){1'b0}}, abs_b_r, 2'b0};

    wire d_rem_neg = d_rem[WIDE];
    wire signed [WIDE:0] d_rm1 = d_rem - $signed({1'b0, d_1b});
    wire signed [WIDE:0] d_rm2 = d_rem - $signed({1'b0, d_2b});
    wire signed [WIDE:0] d_rm3 = d_rem - $signed({1'b0, d_3b});
    wire signed [WIDE:0] d_rp1 = d_rem + $signed({1'b0, d_1b});
    wire signed [WIDE:0] d_rp2 = d_rem + $signed({1'b0, d_2b});
    wire signed [WIDE:0] d_rp3 = d_rem + $signed({1'b0, d_3b});
    wire signed [WIDE:0] d_rp4 = d_rem + $signed({1'b0, d_4b});

    wire signed [3:0] d_delta =
        d_rem_neg ? (d_rp3[WIDE]  ? -4'sd4 :
                     d_rp2[WIDE]  ? -4'sd3 :
                     d_rp1[WIDE]  ? -4'sd2 : -4'sd1) :
        !d_rm1[WIDE] ? (!d_rm2[WIDE] ? (!d_rm3[WIDE] ?
            (!d_rem[WIDE] && (d_rem >= $signed({1'b0, d_4b})) ? 4'sd4 : 4'sd3)
            : 4'sd2) : 4'sd1) : 4'sd0;

    wire [FRAC_BITS:0] d_q_corrected = $unsigned($signed({1'b0, q_reg}) + d_delta);

    wire signed [WIDE:0] d_rem_exact =
        (d_delta == -4'sd4) ? d_rp4 :
        (d_delta == -4'sd3) ? d_rp3 :
        (d_delta == -4'sd2) ? d_rp2 :
        (d_delta == -4'sd1) ? d_rp1 :
        (d_delta ==  4'sd4) ? (d_rem - $signed({1'b0, d_4b})) :
        (d_delta ==  4'sd3) ? d_rm3 :
        (d_delta ==  4'sd2) ? d_rm2 :
        (d_delta ==  4'sd1) ? d_rm1 : d_rem;

    wire d_has_remainder = |d_rem_exact[WIDE-1:0];
    wire d_euclid_adj = a_neg_r & d_has_remainder;
    wire [FRAC_BITS:0] d_q_euclid = d_q_corrected + {{FRAC_BITS{1'b0}}, d_euclid_adj};

    // =========================================================================
    // Divide: Finalization (floor extraction from q_reg, already corrected)
    // At finalize step: q_reg = corrected quotient (stored at correction step)
    // =========================================================================

    wire fin_d_norm_shift = q_reg[FRAC_BITS];
    wire [FRAC_BITS:0] fin_d_q_norm = fin_d_norm_shift ? (q_reg >> 1) : q_reg;

    wire [FRAC_BITS-1:0] fin_d_frac_pos = fin_d_q_norm[FRAC_BITS:1];
    wire fin_d_trunc_bits = fin_d_q_norm[0] | (fin_d_norm_shift & q_reg[0]);
    wire fin_d_neg_floor_adj = sign_r & fin_d_trunc_bits;
    wire [FRAC_BITS-1:0] fin_d_frac_adj = fin_d_frac_pos + {{(FRAC_BITS-1){1'b0}}, fin_d_neg_floor_adj};

    wire fin_d_floor_ovf = fin_d_neg_floor_adj & (&fin_d_frac_pos[FRAC_BITS-2:0]);

    wire fin_d_mag_is_half = (fin_d_frac_adj == POS_HALF);
    wire signed [FRAC_BITS-1:0] fin_d_neg_frac = fin_d_mag_is_half ? NEG_ONE :
                                                   (~fin_d_frac_adj + 1'b1);
    wire signed [FRAC_BITS-1:0] fin_d_final_frac = sign_r ? fin_d_neg_frac : $signed(fin_d_frac_adj);

    wire fin_d_neg_half_adj = sign_r & fin_d_mag_is_half;

    localparam signed [ECW-1:0] MAX_EXP_W = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [ECW-1:0] MIN_EXP_W = -(1 <<< (EXP_BITS - 1)) + 1;

    wire signed [ECW-1:0] fin_d_exp_calc =
        $signed({{(ECW-EXP_BITS-1){a_exp_adj_r[EXP_BITS]}}, a_exp_adj_r})
        - $signed({{(ECW-EXP_BITS-1){b_exp_adj_r[EXP_BITS]}}, b_exp_adj_r})
        + {{(ECW-1){1'b0}}, fin_d_norm_shift}
        + {{(ECW-1){1'b0}}, fin_d_floor_ovf}
        - {{(ECW-1){1'b0}}, fin_d_neg_half_adj};

    wire fin_d_overflow  = (fin_d_exp_calc > MAX_EXP_W);
    wire fin_d_underflow = (fin_d_exp_calc < MIN_EXP_W);

    wire signed [FRAC_BITS-1:0] fin_d_vanished_frac = {fin_d_final_frac[FRAC_BITS-1],
                                                         fin_d_final_frac[FRAC_BITS-1:1]};

    wire signed [FRAC_BITS-1:0] d_out_frac =
        b_zero_r       ? {FRAC_BITS{1'b0}} :
        fin_d_overflow  ? fin_d_final_frac :
        fin_d_underflow ? fin_d_vanished_frac :
                          fin_d_final_frac;

    wire signed [EXP_BITS-1:0] d_out_exp =
        (b_zero_r | fin_d_overflow | fin_d_underflow) ? AMBIG_EXP[EXP_BITS-1:0] :
                                                         fin_d_exp_calc[EXP_BITS-1:0];

    // =========================================================================
    // Sqrt: ±4 correction (combinational, used at correction step)
    // At correction step: prod_r = raw², q_reg = raw
    // =========================================================================

    localparam CW = 2 * (FRAC_BITS + 1);
    wire [CW-1:0] sq_raw_sq = prod_r[CW-1:0];
    wire [CW-1:0] sq_S_scaled = {1'b0, S_reg, {(FRAC_BITS+1){1'b0}}};

    wire sq_is_big = (sq_raw_sq > sq_S_scaled);
    wire [CW-1:0] sq_diff = sq_is_big ? (sq_raw_sq - sq_S_scaled) :
                                          (sq_S_scaled - sq_raw_sq);

    wire [CW-1:0] sq_2r = {{(CW-FRAC_BITS-2){1'b0}}, q_reg, 1'b0};
    wire [CW-1:0] sq_4r = {{(CW-FRAC_BITS-3){1'b0}}, q_reg, 2'b0};
    wire [CW-1:0] sq_6r = sq_4r + sq_2r;
    wire [CW-1:0] sq_8r = {{(CW-FRAC_BITS-4){1'b0}}, q_reg, 3'b0};

    wire [CW-1:0] sq_tm1 = sq_2r - 1;
    wire [CW-1:0] sq_tm2 = sq_4r - 4;
    wire [CW-1:0] sq_tm3 = sq_6r - 9;
    wire [CW-1:0] sq_tm4 = sq_8r - 16;
    wire [CW-1:0] sq_tp1 = sq_2r + 1;
    wire [CW-1:0] sq_tp2 = sq_4r + 4;
    wire [CW-1:0] sq_tp3 = sq_6r + 9;
    wire [CW-1:0] sq_tp4 = sq_8r + 16;

    reg [FRAC_BITS:0] sq_corrected;
    always @(*) begin
        if (sq_is_big) begin
            if (sq_diff <= sq_tm1)      sq_corrected = q_reg - 1;
            else if (sq_diff <= sq_tm2) sq_corrected = q_reg - 2;
            else if (sq_diff <= sq_tm3) sq_corrected = q_reg - 3;
            else                        sq_corrected = q_reg - 4;
        end else begin
            if (sq_diff >= sq_tp4)      sq_corrected = q_reg + 4;
            else if (sq_diff >= sq_tp3) sq_corrected = q_reg + 3;
            else if (sq_diff >= sq_tp2) sq_corrected = q_reg + 2;
            else if (sq_diff >= sq_tp1) sq_corrected = q_reg + 1;
            else                        sq_corrected = q_reg;
        end
    end

    // =========================================================================
    // Sqrt: Finalization (floor extraction from q_reg, already corrected)
    // At finalize step: q_reg = corrected sqrt (stored at correction step)
    // =========================================================================

    wire fin_sq_norm_shift = q_reg[FRAC_BITS];
    wire [FRAC_BITS:0] fin_sq_q_norm = fin_sq_norm_shift ? (q_reg >> 1) : q_reg;
    wire [FRAC_BITS-1:0] fin_sq_frac_pos = fin_sq_q_norm[FRAC_BITS:1];

    wire signed [FRAC_BITS-1:0] sq_out_frac = invalid_r ? {FRAC_BITS{1'b0}} :
                                                $signed(fin_sq_frac_pos);

    wire signed [EXP_BITS:0] sq_exp_out = sqrt_exp_r
                                          + {{EXP_BITS{1'b0}}, fin_sq_norm_shift};
    wire signed [EXP_BITS-1:0] sq_out_exp = invalid_r ? AMBIG_EXP[EXP_BITS-1:0] :
                                              sq_exp_out[EXP_BITS-1:0];

    // =========================================================================
    // FSM
    //
    // Multiply is combinational: prod_r = mul_a * mul_b (instant from regs).
    // Setup sets mul_a/mul_b. At step 0, prod_r has the setup multiply result.
    //
    // Divide running steps (NR_ITERS=2, 2*NR+3=7 running steps, +1 setup=8):
    //   step 0: read P=B×x₀, compute E₁, issue x₀×E₁
    //   step 1: read X₁, issue B×X₁
    //   step 2: read P=B×X₁, compute E₂, issue X₁×E₂
    //   step 3: read X₂, issue A×X₂
    //   step 4: read Q_raw=A×X₂, issue Q×B
    //   step 5: correction from Q×B
    //   step 6: finalize, done
    //
    // Sqrt running steps (NR_ITERS=2, 3*NR+4=10 running steps, +1 setup=11):
    //   step 0: read H₀=x₀², issue S×H₀
    //   step 1: read P=S×H₀, compute E₁, issue x₀×E₁
    //   step 2: read X₁, issue X₁²
    //   step 3: read H₁=X₁², issue S×H₁
    //   step 4: read P=S×H₁, compute E₂, issue X₁×E₂
    //   step 5: read X₂, issue S×X₂
    //   step 6: read raw=S×X₂, issue raw²
    //   step 7: correction from raw²
    //   step 8: finalize, done
    // =========================================================================

    // NR error term: E = 2^F − P (divide) or E = 3·2^(F−1) − P (sqrt)
    wire [FRAC_BITS:0] nr_E_div  = {1'b1, {FRAC_BITS{1'b0}}} - extract_P;
    wire [FRAC_BITS:0] nr_E_sqrt = {2'b11, {(FRAC_BITS-1){1'b0}}} - {1'b0, extract_H};

    always @(posedge clk) begin
        done <= 1'b0;

        if (!running && start) begin
            // ============= SETUP =============
            running <= 1'b1;
            step    <= 0;
            mode_r  <= mode;

            // Register operands
            abs_a_r <= abs_a_comb;
            abs_b_r <= abs_b_comb;
            sign_r  <= a_frac[FRAC_BITS-1] ^ b_frac[FRAC_BITS-1];
            a_neg_r <= a_frac[FRAC_BITS-1];
            b_zero_r <= (b_frac == 0);
            invalid_r <= a_frac[FRAC_BITS-1] | (a_frac == 0) |
                         (a_exp == AMBIG_EXP[EXP_BITS-1:0]);

            a_exp_adj_r <= $signed({a_exp[EXP_BITS-1], a_exp})
                          + {{EXP_BITS{1'b0}}, a_is_neg_one};
            b_exp_adj_r <= $signed({b_exp[EXP_BITS-1], b_exp})
                          + {{EXP_BITS{1'b0}}, b_is_neg_one};

            // Sqrt exponent: (a_exp >>> 1). The (a_exp & 1) correction is
            // handled by norm_shift: odd exp always has raw >= 2^F.
            sqrt_exp_r <= ($signed({a_exp[EXP_BITS-1], a_exp}) >>> 1);

            // Sqrt radicand
            S_reg <= S_comb;

            // Initial NR seed + issue first multiply
            if (mode == 0) begin
                x_reg <= x0_div;
                mul_a <= {2'b0, abs_b_comb};
                mul_b <= {1'b0, x0_div};
            end else begin
                x_reg <= x0_sqrt;
                mul_a <= {1'b0, x0_sqrt};
                mul_b <= {1'b0, x0_sqrt};
            end

        end else if (running) begin
            step <= step + 1;

            if (mode_r == 0) begin
                // ============= DIVIDE FSM =============
                // NR iteration: even steps read P, odd steps read X_new
                if (step < 2 * NR_ITERS && step[0] == 1'b0) begin
                    // Even step (0,2,4...): read P=B×X, compute E, issue X×E
                    mul_a <= {1'b0, x_reg};
                    mul_b <= nr_E_div;
                end else if (step < 2 * NR_ITERS && step[0] == 1'b1) begin
                    // Odd step (1,3,5...): read X_new, issue B×X or A×X
                    x_reg <= extract_X;
                    if (step < 2 * NR_ITERS - 1) begin
                        // More NR iterations: issue B×X_new
                        mul_a <= {2'b0, abs_b_r};
                        mul_b <= {1'b0, extract_X};
                    end else begin
                        // Last NR iter: issue A×X_final
                        mul_a <= {2'b0, abs_a_r};
                        mul_b <= {1'b0, extract_X};
                    end
                end else if (step == 2 * NR_ITERS) begin
                    // Read Q_raw = A×X, issue Q×B for correction
                    q_reg <= extract_Q;
                    mul_a <= extract_Q;
                    mul_b <= {2'b0, abs_b_r};
                end else if (step == 2 * NR_ITERS + 1) begin
                    // Correction: prod_r = Q×B, compute delta + Euclidean adj
                    q_reg <= d_q_euclid;
                end else begin
                    // Finalize from q_reg (already corrected)
                    result_frac <= d_out_frac;
                    result_exp  <= d_out_exp;
                    done <= 1'b1;
                    running <= 1'b0;
                end

            end else begin
                // ============= SQRT FSM =============
                // NR iterations: groups of 3 steps (phase = step % 3)
                // Phase 0: read H (X²), issue S×H
                // Phase 1: read P (S×H), compute E, issue X×E
                // Phase 2: read X_new, issue X² (or S×X if last iter)

                if (step < 3 * NR_ITERS) begin
                    case (step % 3)
                        0: begin
                            // Phase 0: read H, issue S×H
                            mul_a <= {1'b0, S_reg};
                            mul_b <= {1'b0, extract_H};
                        end
                        1: begin
                            // Phase 1: read P = S×H, compute E, issue X×E
                            mul_a <= {1'b0, x_reg};
                            mul_b <= nr_E_sqrt;
                        end
                        2: begin
                            // Phase 2: read X_new (extra >>1 for sqrt /2)
                            x_reg <= extract_X_sqrt;
                            if (step < 3 * NR_ITERS - 1) begin
                                // More iterations: issue X_new²
                                mul_a <= {1'b0, extract_X_sqrt};
                                mul_b <= {1'b0, extract_X_sqrt};
                            end else begin
                                // Last iter: issue S×X_final
                                mul_a <= {1'b0, S_reg};
                                mul_b <= {1'b0, extract_X_sqrt};
                            end
                        end
                        default: ;
                    endcase
                end else if (step == 3 * NR_ITERS) begin
                    // Read raw = S×X_final, issue raw²
                    q_reg <= extract_raw;
                    mul_a <= extract_raw;
                    mul_b <= extract_raw;
                end else if (step == 3 * NR_ITERS + 1) begin
                    // Correction: prod_r = raw²
                    q_reg <= sq_corrected;
                end else begin
                    // Finalize from q_reg (already corrected)
                    result_frac <= sq_out_frac;
                    result_exp  <= sq_out_exp;
                    done <= 1'b1;
                    running <= 1'b0;
                end
            end
        end
    end

endmodule
