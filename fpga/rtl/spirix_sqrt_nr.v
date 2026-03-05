// spirix_sqrt_nr — 10-stage pipelined Newton-Raphson square root for Spirix scalars
//
// Computes sqrt(a) using inverse-sqrt NR approximation:
//   1/sqrt(S) via LUT seed + 2 quadratic-convergence iterations (10→20→40 bits),
//   then multiply by S to get sqrt(S), with ±1 correction for exact results.
//
// Latency: 10 cycles. Throughput: 1 sqrt per clock.
//
//   Stage 1:  Sign/abs, exponent even/odd → radicand S, LUT → x₀, x₀² → H₀
//   Stage 2:  S×H₀ → P₁, E₁ = 3·2²⁴ − P₁
//   Stage 3:  x₀×E₁ >> 25 → X₁
//   Stage 4:  X₁² → H₁
//   Stage 5:  S×H₁ → P₂, E₂ = 3·2²⁴ − P₂
//   Stage 6:  X₁×E₂ >> 25 → X₂  (~30-bit accurate 1/sqrt)
//   Stage 7:  S×X₂ → raw sqrt
//   Stage 8:  raw²
//   Stage 9:  ±4 correction via additive thresholds  (no multiplies)
//   Stage 10: Normalize + Banker's round + exponent + clamp
//
// Negative input or zero returns (0, AMBIGUOUS_EXP).
//
// Scaling convention:
//   S = radicand, 25 bits unsigned, S/2²⁴ = s ∈ [0.5, 2.0)
//   x₀/2²⁴ ≈ 1/sqrt(s)  (NR seed from LUT)
//   Xₙ/2²⁴ ≈ 1/sqrt(s)  (refined by NR iterations)
//   sqrt_raw ≈ sqrt(s) × 2²⁵  (26 bits, from S × X₂ >> 23)
//
// Input format: value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
// Valid parameter range: FRAC_BITS >= 4, EXP_BITS >= 4.

module spirix_sqrt_nr #(
    parameter FRAC_BITS = 25,
    parameter EXP_BITS  = 8
)(
    input  wire clk,
    input  wire ce,
    input  wire signed [FRAC_BITS-1:0] a_frac,
    input  wire signed [EXP_BITS-1:0]  a_exp,
    output reg  signed [FRAC_BITS-1:0] result_frac,
    output reg  signed [EXP_BITS-1:0]  result_exp
);

    localparam AMBIGUOUS_EXP = -(1 <<< (EXP_BITS - 1));
    localparam signed [FRAC_BITS-1:0] POS_HALF = {1'b0, 1'b1, {(FRAC_BITS-2){1'b0}}};
    localparam signed [FRAC_BITS-1:0] NEG_ONE  = {1'b1, {(FRAC_BITS-1){1'b0}}};
    localparam signed [EXP_BITS:0] MAX_EXP = (1 <<< (EXP_BITS - 1)) - 1;
    localparam signed [EXP_BITS:0] MIN_EXP = -(1 <<< (EXP_BITS - 1)) + 1;
    localparam MAG = FRAC_BITS - 1;  // 24 unsigned magnitude bits

    // =========================================================================
    // Stage 1: Sign/abs, exponent even/odd, radicand S, LUT seed, x₀²
    // =========================================================================

    wire s1_is_negative = a_frac[FRAC_BITS-1];
    wire s1_is_ambig = (a_exp == AMBIGUOUS_EXP[EXP_BITS-1:0]);
    wire s1_invalid = s1_is_negative | s1_is_ambig;

    wire s1_is_neg_one = (a_frac == NEG_ONE);

    wire [MAG-1:0] s1_abs_frac = s1_is_neg_one ? POS_HALF[MAG-1:0] :
                                   (a_frac[FRAC_BITS-1] ?
                                     (~a_frac[MAG-1:0] + 1'b1) :
                                     a_frac[MAG-1:0]);

    wire signed [EXP_BITS:0] s1_exp_adj = $signed({a_exp[EXP_BITS-1], a_exp})
                                            + {{EXP_BITS{1'b0}}, s1_is_neg_one};

    wire s1_exp_odd = s1_exp_adj[0];

    // Result exponent base = floor(exp/2) via arithmetic right shift
    wire signed [EXP_BITS:0] s1_result_exp_base = s1_exp_adj >>> 1;

    // Radicand S (25 bits unsigned):
    //   Even exp: S = {0, abs_frac}  → S/2²⁴ ∈ [0.5, 1.0)
    //   Odd  exp: S = {abs_frac, 0}  → S/2²⁴ ∈ [1.0, 2.0)
    wire [FRAC_BITS-1:0] s1_S = s1_exp_odd ? {s1_abs_frac, 1'b0} :
                                               {1'b0, s1_abs_frac};

    // --- LUT: 192 used entries × 11 bits ---
    // Indexed by S[24:17] (top 8 bits of 25-bit radicand).
    // Indices [0,63] unused (S always has leading 01 or 1).
    //   [64,127]: even exp, s ∈ [0.5, 1.0), 1/sqrt(s) ∈ (1.0, 1.414]
    //   [128,255]: odd exp, s ∈ [1.0, 2.0), 1/sqrt(s) ∈ (0.707, 1.0]
    //
    // lut[i] = round(1/sqrt((2i+1)/256) × 1024)
    // Reconstruction: x₀ = {lut, 14'b0} = lut << 14 (25 bits)
    // x₀/2²⁴ = lut/1024 ≈ 1/sqrt(s)
    reg [10:0] s1_isqrt_lut;
    always @(*) begin
        case (s1_S[FRAC_BITS-1 : FRAC_BITS-8])
            8'd64: s1_isqrt_lut = 11'd1443;
            8'd65: s1_isqrt_lut = 11'd1431;
            8'd66: s1_isqrt_lut = 11'd1421;
            8'd67: s1_isqrt_lut = 11'd1410;
            8'd68: s1_isqrt_lut = 11'd1400;
            8'd69: s1_isqrt_lut = 11'd1390;
            8'd70: s1_isqrt_lut = 11'd1380;
            8'd71: s1_isqrt_lut = 11'd1370;
            8'd72: s1_isqrt_lut = 11'd1361;
            8'd73: s1_isqrt_lut = 11'd1351;
            8'd74: s1_isqrt_lut = 11'd1342;
            8'd75: s1_isqrt_lut = 11'd1333;
            8'd76: s1_isqrt_lut = 11'd1325;
            8'd77: s1_isqrt_lut = 11'd1316;
            8'd78: s1_isqrt_lut = 11'd1308;
            8'd79: s1_isqrt_lut = 11'd1299;
            8'd80: s1_isqrt_lut = 11'd1291;
            8'd81: s1_isqrt_lut = 11'd1283;
            8'd82: s1_isqrt_lut = 11'd1275;
            8'd83: s1_isqrt_lut = 11'd1268;
            8'd84: s1_isqrt_lut = 11'd1260;
            8'd85: s1_isqrt_lut = 11'd1253;
            8'd86: s1_isqrt_lut = 11'd1246;
            8'd87: s1_isqrt_lut = 11'd1239;
            8'd88: s1_isqrt_lut = 11'd1231;
            8'd89: s1_isqrt_lut = 11'd1225;
            8'd90: s1_isqrt_lut = 11'd1218;
            8'd91: s1_isqrt_lut = 11'd1211;
            8'd92: s1_isqrt_lut = 11'd1205;
            8'd93: s1_isqrt_lut = 11'd1198;
            8'd94: s1_isqrt_lut = 11'd1192;
            8'd95: s1_isqrt_lut = 11'd1186;
            8'd96: s1_isqrt_lut = 11'd1179;
            8'd97: s1_isqrt_lut = 11'd1173;
            8'd98: s1_isqrt_lut = 11'd1167;
            8'd99: s1_isqrt_lut = 11'd1161;
            8'd100: s1_isqrt_lut = 11'd1156;
            8'd101: s1_isqrt_lut = 11'd1150;
            8'd102: s1_isqrt_lut = 11'd1144;
            8'd103: s1_isqrt_lut = 11'd1139;
            8'd104: s1_isqrt_lut = 11'd1133;
            8'd105: s1_isqrt_lut = 11'd1128;
            8'd106: s1_isqrt_lut = 11'd1123;
            8'd107: s1_isqrt_lut = 11'd1117;
            8'd108: s1_isqrt_lut = 11'd1112;
            8'd109: s1_isqrt_lut = 11'd1107;
            8'd110: s1_isqrt_lut = 11'd1102;
            8'd111: s1_isqrt_lut = 11'd1097;
            8'd112: s1_isqrt_lut = 11'd1092;
            8'd113: s1_isqrt_lut = 11'd1087;
            8'd114: s1_isqrt_lut = 11'd1083;
            8'd115: s1_isqrt_lut = 11'd1078;
            8'd116: s1_isqrt_lut = 11'd1073;
            8'd117: s1_isqrt_lut = 11'd1069;
            8'd118: s1_isqrt_lut = 11'd1064;
            8'd119: s1_isqrt_lut = 11'd1060;
            8'd120: s1_isqrt_lut = 11'd1055;
            8'd121: s1_isqrt_lut = 11'd1051;
            8'd122: s1_isqrt_lut = 11'd1047;
            8'd123: s1_isqrt_lut = 11'd1042;
            8'd124: s1_isqrt_lut = 11'd1038;
            8'd125: s1_isqrt_lut = 11'd1034;
            8'd126: s1_isqrt_lut = 11'd1030;
            8'd127: s1_isqrt_lut = 11'd1026;
            8'd128: s1_isqrt_lut = 11'd1022;
            8'd129: s1_isqrt_lut = 11'd1018;
            8'd130: s1_isqrt_lut = 11'd1014;
            8'd131: s1_isqrt_lut = 11'd1010;
            8'd132: s1_isqrt_lut = 11'd1006;
            8'd133: s1_isqrt_lut = 11'd1003;
            8'd134: s1_isqrt_lut = 11'd999;
            8'd135: s1_isqrt_lut = 11'd995;
            8'd136: s1_isqrt_lut = 11'd992;
            8'd137: s1_isqrt_lut = 11'd988;
            8'd138: s1_isqrt_lut = 11'd984;
            8'd139: s1_isqrt_lut = 11'd981;
            8'd140: s1_isqrt_lut = 11'd977;
            8'd141: s1_isqrt_lut = 11'd974;
            8'd142: s1_isqrt_lut = 11'd971;
            8'd143: s1_isqrt_lut = 11'd967;
            8'd144: s1_isqrt_lut = 11'd964;
            8'd145: s1_isqrt_lut = 11'd960;
            8'd146: s1_isqrt_lut = 11'd957;
            8'd147: s1_isqrt_lut = 11'd954;
            8'd148: s1_isqrt_lut = 11'd951;
            8'd149: s1_isqrt_lut = 11'd948;
            8'd150: s1_isqrt_lut = 11'd944;
            8'd151: s1_isqrt_lut = 11'd941;
            8'd152: s1_isqrt_lut = 11'd938;
            8'd153: s1_isqrt_lut = 11'd935;
            8'd154: s1_isqrt_lut = 11'd932;
            8'd155: s1_isqrt_lut = 11'd929;
            8'd156: s1_isqrt_lut = 11'd926;
            8'd157: s1_isqrt_lut = 11'd923;
            8'd158: s1_isqrt_lut = 11'd920;
            8'd159: s1_isqrt_lut = 11'd917;
            8'd160: s1_isqrt_lut = 11'd914;
            8'd161: s1_isqrt_lut = 11'd912;
            8'd162: s1_isqrt_lut = 11'd909;
            8'd163: s1_isqrt_lut = 11'd906;
            8'd164: s1_isqrt_lut = 11'd903;
            8'd165: s1_isqrt_lut = 11'd901;
            8'd166: s1_isqrt_lut = 11'd898;
            8'd167: s1_isqrt_lut = 11'd895;
            8'd168: s1_isqrt_lut = 11'd892;
            8'd169: s1_isqrt_lut = 11'd890;
            8'd170: s1_isqrt_lut = 11'd887;
            8'd171: s1_isqrt_lut = 11'd885;
            8'd172: s1_isqrt_lut = 11'd882;
            8'd173: s1_isqrt_lut = 11'd880;
            8'd174: s1_isqrt_lut = 11'd877;
            8'd175: s1_isqrt_lut = 11'd875;
            8'd176: s1_isqrt_lut = 11'd872;
            8'd177: s1_isqrt_lut = 11'd870;
            8'd178: s1_isqrt_lut = 11'd867;
            8'd179: s1_isqrt_lut = 11'd865;
            8'd180: s1_isqrt_lut = 11'd862;
            8'd181: s1_isqrt_lut = 11'd860;
            8'd182: s1_isqrt_lut = 11'd858;
            8'd183: s1_isqrt_lut = 11'd855;
            8'd184: s1_isqrt_lut = 11'd853;
            8'd185: s1_isqrt_lut = 11'd851;
            8'd186: s1_isqrt_lut = 11'd848;
            8'd187: s1_isqrt_lut = 11'd846;
            8'd188: s1_isqrt_lut = 11'd844;
            8'd189: s1_isqrt_lut = 11'd842;
            8'd190: s1_isqrt_lut = 11'd839;
            8'd191: s1_isqrt_lut = 11'd837;
            8'd192: s1_isqrt_lut = 11'd835;
            8'd193: s1_isqrt_lut = 11'd833;
            8'd194: s1_isqrt_lut = 11'd831;
            8'd195: s1_isqrt_lut = 11'd829;
            8'd196: s1_isqrt_lut = 11'd826;
            8'd197: s1_isqrt_lut = 11'd824;
            8'd198: s1_isqrt_lut = 11'd822;
            8'd199: s1_isqrt_lut = 11'd820;
            8'd200: s1_isqrt_lut = 11'd818;
            8'd201: s1_isqrt_lut = 11'd816;
            8'd202: s1_isqrt_lut = 11'd814;
            8'd203: s1_isqrt_lut = 11'd812;
            8'd204: s1_isqrt_lut = 11'd810;
            8'd205: s1_isqrt_lut = 11'd808;
            8'd206: s1_isqrt_lut = 11'd806;
            8'd207: s1_isqrt_lut = 11'd804;
            8'd208: s1_isqrt_lut = 11'd802;
            8'd209: s1_isqrt_lut = 11'd800;
            8'd210: s1_isqrt_lut = 11'd799;
            8'd211: s1_isqrt_lut = 11'd797;
            8'd212: s1_isqrt_lut = 11'd795;
            8'd213: s1_isqrt_lut = 11'd793;
            8'd214: s1_isqrt_lut = 11'd791;
            8'd215: s1_isqrt_lut = 11'd789;
            8'd216: s1_isqrt_lut = 11'd787;
            8'd217: s1_isqrt_lut = 11'd786;
            8'd218: s1_isqrt_lut = 11'd784;
            8'd219: s1_isqrt_lut = 11'd782;
            8'd220: s1_isqrt_lut = 11'd780;
            8'd221: s1_isqrt_lut = 11'd778;
            8'd222: s1_isqrt_lut = 11'd777;
            8'd223: s1_isqrt_lut = 11'd775;
            8'd224: s1_isqrt_lut = 11'd773;
            8'd225: s1_isqrt_lut = 11'd771;
            8'd226: s1_isqrt_lut = 11'd770;
            8'd227: s1_isqrt_lut = 11'd768;
            8'd228: s1_isqrt_lut = 11'd766;
            8'd229: s1_isqrt_lut = 11'd765;
            8'd230: s1_isqrt_lut = 11'd763;
            8'd231: s1_isqrt_lut = 11'd761;
            8'd232: s1_isqrt_lut = 11'd760;
            8'd233: s1_isqrt_lut = 11'd758;
            8'd234: s1_isqrt_lut = 11'd757;
            8'd235: s1_isqrt_lut = 11'd755;
            8'd236: s1_isqrt_lut = 11'd753;
            8'd237: s1_isqrt_lut = 11'd752;
            8'd238: s1_isqrt_lut = 11'd750;
            8'd239: s1_isqrt_lut = 11'd749;
            8'd240: s1_isqrt_lut = 11'd747;
            8'd241: s1_isqrt_lut = 11'd745;
            8'd242: s1_isqrt_lut = 11'd744;
            8'd243: s1_isqrt_lut = 11'd742;
            8'd244: s1_isqrt_lut = 11'd741;
            8'd245: s1_isqrt_lut = 11'd739;
            8'd246: s1_isqrt_lut = 11'd738;
            8'd247: s1_isqrt_lut = 11'd736;
            8'd248: s1_isqrt_lut = 11'd735;
            8'd249: s1_isqrt_lut = 11'd733;
            8'd250: s1_isqrt_lut = 11'd732;
            8'd251: s1_isqrt_lut = 11'd731;
            8'd252: s1_isqrt_lut = 11'd729;
            8'd253: s1_isqrt_lut = 11'd728;
            8'd254: s1_isqrt_lut = 11'd726;
            8'd255: s1_isqrt_lut = 11'd725;
            default: s1_isqrt_lut = 11'd1024;
        endcase
    end

    // x₀ = {lut, 14'b0} = lut << 14 (25 bits, MSB may be set)
    // x₀/2²⁴ = lut/1024 ≈ 1/sqrt(s)
    // Range: lut ∈ [725, 1443], x₀ ∈ [11,878,400, 23,642,112]
    wire [FRAC_BITS-1:0] s1_x0 = {s1_isqrt_lut, {(FRAC_BITS-11){1'b0}}};

    // H₀ = x₀² >> 24: represents x₀²/2²⁴ ≈ (1/s) × 2²⁴
    // x₀² is 50 bits [49:0]. Taking [48:24] = 25 bits.
    // Note: bit 49 can be set for s ≈ 0.5, but LUT inaccuracy ensures it's rare.
    // If bit 49 IS set, the truncation causes the NR to self-correct in iteration 1.
    wire [2*FRAC_BITS-1:0] s1_x0_sq_full = s1_x0 * s1_x0;  // 50 bits
    wire [FRAC_BITS-1:0] s1_h0 = s1_x0_sq_full[2*FRAC_BITS-2 : FRAC_BITS-1]; // [48:24] = 25 bits

    reg signed [EXP_BITS:0] s1_exp_base_r;
    reg [FRAC_BITS-1:0]     s1_S_r;
    reg [FRAC_BITS-1:0]     s1_x0_r;
    reg [FRAC_BITS-1:0]     s1_h0_r;
    reg                      s1_invalid_r;

    always @(posedge clk) if (ce) begin
        s1_exp_base_r <= s1_result_exp_base;
        s1_S_r        <= s1_S;
        s1_x0_r       <= s1_x0;
        s1_h0_r       <= s1_h0;
        s1_invalid_r  <= s1_invalid;
    end

    // =========================================================================
    // Stage 2: P₁ = (S×H₀)[48:24], E₁ = 3·2²⁴ − P₁
    //
    // NR iteration 1, first half: one multiply + subtraction.
    // =========================================================================

    wire [2*FRAC_BITS-1:0] s2_sh0_full = s1_S_r * s1_h0_r;
    wire [FRAC_BITS-1:0] s2_p1 = s2_sh0_full[2*FRAC_BITS-2 : FRAC_BITS-1]; // [48:24]

    wire [FRAC_BITS:0] s2_e1 = {2'b11, {(FRAC_BITS-1){1'b0}}} - {1'b0, s2_p1};

    reg signed [EXP_BITS:0] s2_exp_base_r;
    reg [FRAC_BITS-1:0]     s2_S_r;
    reg [FRAC_BITS-1:0]     s2_x0_r;
    reg [FRAC_BITS:0]        s2_e1_r;
    reg                      s2_invalid_r;

    always @(posedge clk) if (ce) begin
        s2_exp_base_r <= s1_exp_base_r;
        s2_S_r        <= s1_S_r;
        s2_x0_r       <= s1_x0_r;
        s2_e1_r       <= s2_e1;
        s2_invalid_r  <= s1_invalid_r;
    end

    // =========================================================================
    // Stage 3: X₁ = (x₀ × E₁) >> 25
    //
    // NR iteration 1, second half: one multiply.
    // =========================================================================

    wire [FRAC_BITS + FRAC_BITS:0] s3_x0e1_full = s2_x0_r * s2_e1_r;  // 51 bits
    wire [FRAC_BITS-1:0] s3_x1 = s3_x0e1_full[2*FRAC_BITS-1 : FRAC_BITS]; // [49:25]

    reg signed [EXP_BITS:0] s3_exp_base_r;
    reg [FRAC_BITS-1:0]     s3_S_r;
    reg [FRAC_BITS-1:0]     s3_x1_r;
    reg                      s3_invalid_r;

    always @(posedge clk) if (ce) begin
        s3_exp_base_r <= s2_exp_base_r;
        s3_S_r        <= s2_S_r;
        s3_x1_r       <= s3_x1;
        s3_invalid_r  <= s2_invalid_r;
    end

    // =========================================================================
    // Stage 4: H₁ = X₁² >> 24
    // =========================================================================

    wire [2*FRAC_BITS-1:0] s4_x1_sq_full = s3_x1_r * s3_x1_r;  // 50 bits
    wire [FRAC_BITS-1:0] s4_h1 = s4_x1_sq_full[2*FRAC_BITS-2 : FRAC_BITS-1]; // [48:24]

    reg signed [EXP_BITS:0] s4_exp_base_r;
    reg [FRAC_BITS-1:0]     s4_S_r;
    reg [FRAC_BITS-1:0]     s4_x1_r;
    reg [FRAC_BITS-1:0]     s4_h1_r;
    reg                      s4_invalid_r;

    always @(posedge clk) if (ce) begin
        s4_exp_base_r <= s3_exp_base_r;
        s4_S_r        <= s3_S_r;
        s4_x1_r       <= s3_x1_r;
        s4_h1_r       <= s4_h1;
        s4_invalid_r  <= s3_invalid_r;
    end

    // =========================================================================
    // Stage 5: P₂ = (S×H₁)[48:24], E₂ = 3·2²⁴ − P₂
    // =========================================================================

    wire [2*FRAC_BITS-1:0] s5_sh1_full = s4_S_r * s4_h1_r;
    wire [FRAC_BITS-1:0] s5_p2 = s5_sh1_full[2*FRAC_BITS-2 : FRAC_BITS-1]; // [48:24]

    wire [FRAC_BITS:0] s5_e2 = {2'b11, {(FRAC_BITS-1){1'b0}}} - {1'b0, s5_p2};

    reg signed [EXP_BITS:0] s5_exp_base_r;
    reg [FRAC_BITS-1:0]     s5_S_r;
    reg [FRAC_BITS-1:0]     s5_x1_r;
    reg [FRAC_BITS:0]        s5_e2_r;
    reg                      s5_invalid_r;

    always @(posedge clk) if (ce) begin
        s5_exp_base_r <= s4_exp_base_r;
        s5_S_r        <= s4_S_r;
        s5_x1_r       <= s4_x1_r;
        s5_e2_r       <= s5_e2;
        s5_invalid_r  <= s4_invalid_r;
    end

    // =========================================================================
    // Stage 6: X₂ = (X₁ × E₂) >> 25   (~40-bit accurate 1/sqrt(S))
    // =========================================================================

    wire [FRAC_BITS + FRAC_BITS:0] s6_x1e2_full = s5_x1_r * s5_e2_r;  // 51 bits
    wire [FRAC_BITS-1:0] s6_x2 = s6_x1e2_full[2*FRAC_BITS-1 : FRAC_BITS]; // [49:25]

    reg signed [EXP_BITS:0] s6_exp_base_r;
    reg [FRAC_BITS-1:0]     s6_S_r;
    reg [FRAC_BITS-1:0]     s6_x2_r;
    reg                      s6_invalid_r;

    always @(posedge clk) if (ce) begin
        s6_exp_base_r <= s5_exp_base_r;
        s6_S_r        <= s5_S_r;
        s6_x2_r       <= s6_x2;
        s6_invalid_r  <= s5_invalid_r;
    end

    // =========================================================================
    // Stage 7: sqrt_raw = (S × X₂)[48:23]  (one multiply)
    // =========================================================================

    wire [2*FRAC_BITS-1:0] s7_sx2_full = s6_S_r * s6_x2_r;  // 50 bits
    wire [FRAC_BITS:0] s7_sqrt_raw = s7_sx2_full[2*FRAC_BITS-2 : FRAC_BITS-2]; // [48:23] = 26 bits

    reg signed [EXP_BITS:0] s7_exp_base_r;
    reg [FRAC_BITS-1:0]     s7_S_r;
    reg [FRAC_BITS:0]        s7_raw_r;
    reg                      s7_invalid_r;

    always @(posedge clk) if (ce) begin
        s7_exp_base_r <= s6_exp_base_r;
        s7_S_r        <= s6_S_r;
        s7_raw_r      <= s7_sqrt_raw;
        s7_invalid_r  <= s6_invalid_r;
    end

    // =========================================================================
    // Stage 8: raw² = sqrt_raw × sqrt_raw  (one multiply)
    // =========================================================================

    localparam CW = 2*(FRAC_BITS+1);  // 52 bits for comparison
    wire [CW-1:0] s8_raw_sq = s7_raw_r * s7_raw_r;

    reg signed [EXP_BITS:0] s8_exp_base_r;
    reg [FRAC_BITS-1:0]     s8_S_r;
    reg [FRAC_BITS:0]        s8_raw_r;
    reg [CW-1:0]             s8_raw_sq_r;
    reg                      s8_invalid_r;

    always @(posedge clk) if (ce) begin
        s8_exp_base_r <= s7_exp_base_r;
        s8_S_r        <= s7_S_r;
        s8_raw_r      <= s7_raw_r;
        s8_raw_sq_r   <= s8_raw_sq;
        s8_invalid_r  <= s7_invalid_r;
    end

    // =========================================================================
    // Stage 9: ±4 correction from registered raw² (no multiplies)
    //
    // Uses additive identity: (raw±k)² = raw² ± 2k·raw + k²
    // Compares against S_scaled = S << 26 to find floor(sqrt(S·2²⁶)).
    // =========================================================================

    wire [CW-1:0] s9_S_scaled = {1'b0, s8_S_r, {(FRAC_BITS+1){1'b0}}};

    wire s9_is_big = (s8_raw_sq_r > s9_S_scaled);
    wire [CW-1:0] s9_diff = s9_is_big ? (s8_raw_sq_r - s9_S_scaled)
                                        : (s9_S_scaled - s8_raw_sq_r);

    // Multiples of raw (shifts + one add for 6×)
    wire [CW-1:0] s9_2r = {{(CW-FRAC_BITS-2){1'b0}}, s8_raw_r, 1'b0};
    wire [CW-1:0] s9_4r = {{(CW-FRAC_BITS-3){1'b0}}, s8_raw_r, 2'b0};
    wire [CW-1:0] s9_6r = s9_4r + s9_2r;
    wire [CW-1:0] s9_8r = {{(CW-FRAC_BITS-4){1'b0}}, s8_raw_r, 3'b0};

    // Too-big thresholds: diff ≤ 2k·raw − k²
    wire [CW-1:0] s9_tm1 = s9_2r - 1;
    wire [CW-1:0] s9_tm2 = s9_4r - 4;
    wire [CW-1:0] s9_tm3 = s9_6r - 9;
    wire [CW-1:0] s9_tm4 = s9_8r - 16;

    // Too-small thresholds: gap ≥ 2k·raw + k²
    wire [CW-1:0] s9_tp1 = s9_2r + 1;
    wire [CW-1:0] s9_tp2 = s9_4r + 4;
    wire [CW-1:0] s9_tp3 = s9_6r + 9;
    wire [CW-1:0] s9_tp4 = s9_8r + 16;

    reg [FRAC_BITS:0] s9_sqrt_corrected;
    reg               s9_rem_sticky;

    always @(*) begin
        if (s9_is_big) begin
            if (s9_diff <= s9_tm1) begin
                s9_sqrt_corrected = s8_raw_r - 1;
                s9_rem_sticky     = (s9_diff != s9_tm1);
            end else if (s9_diff <= s9_tm2) begin
                s9_sqrt_corrected = s8_raw_r - 2;
                s9_rem_sticky     = (s9_diff != s9_tm2);
            end else if (s9_diff <= s9_tm3) begin
                s9_sqrt_corrected = s8_raw_r - 3;
                s9_rem_sticky     = (s9_diff != s9_tm3);
            end else begin
                s9_sqrt_corrected = s8_raw_r - 4;
                s9_rem_sticky     = (s9_diff != s9_tm4);
            end
        end else begin
            if (s9_diff >= s9_tp4) begin
                s9_sqrt_corrected = s8_raw_r + 4;
                s9_rem_sticky     = (s9_diff != s9_tp4);
            end else if (s9_diff >= s9_tp3) begin
                s9_sqrt_corrected = s8_raw_r + 3;
                s9_rem_sticky     = (s9_diff != s9_tp3);
            end else if (s9_diff >= s9_tp2) begin
                s9_sqrt_corrected = s8_raw_r + 2;
                s9_rem_sticky     = (s9_diff != s9_tp2);
            end else if (s9_diff >= s9_tp1) begin
                s9_sqrt_corrected = s8_raw_r + 1;
                s9_rem_sticky     = (s9_diff != s9_tp1);
            end else begin
                s9_sqrt_corrected = s8_raw_r;
                s9_rem_sticky     = (s9_diff != 0);
            end
        end
    end

    reg signed [EXP_BITS:0] s9_exp_base_r;
    reg [FRAC_BITS:0]        s9_sqrt_r;
    reg                      s9_rem_sticky_r;
    reg                      s9_invalid_r;

    always @(posedge clk) if (ce) begin
        s9_exp_base_r   <= s8_exp_base_r;
        s9_sqrt_r       <= s9_sqrt_corrected;
        s9_rem_sticky_r <= s9_rem_sticky;
        s9_invalid_r    <= s8_invalid_r;
    end

    // =========================================================================
    // Stage 10: Normalize + Banker's round + exponent + clamp
    //
    // sqrt_corrected ≈ sqrt(s)×2²⁵, 26 bits.
    // sqrt(s) ∈ [0.707, 1.414], so value ∈ [2²⁴·⁵, 2²⁵·⁵].
    // Bit 25 may or may not be set → normalize by 0 or 1 right shift.
    // =========================================================================

    wire s10_norm_shift = s9_sqrt_r[FRAC_BITS];
    wire [FRAC_BITS:0] s10_q_norm = s10_norm_shift ? (s9_sqrt_r >> 1) : s9_sqrt_r;

    wire [FRAC_BITS-1:0] s10_frac_pos_raw = s10_q_norm[FRAC_BITS:1];

    // Banker's rounding (RNE)
    wire s10_guard = s10_q_norm[0];
    wire s10_norm_sticky = s10_norm_shift & s9_sqrt_r[0];
    wire s10_sticky = s10_norm_sticky | s9_rem_sticky_r;
    wire s10_lsb = s10_frac_pos_raw[0];
    wire s10_round_up = s10_guard & (s10_sticky | s10_lsb);

    wire [FRAC_BITS-1:0] s10_frac_rounded = s10_frac_pos_raw
                                              + {{(FRAC_BITS-1){1'b0}}, s10_round_up};

    wire s10_round_ovf = (&s10_frac_pos_raw[FRAC_BITS-2:0]) & s10_round_up;
    wire [FRAC_BITS-1:0] s10_pos_frac = s10_round_ovf ? POS_HALF : s10_frac_rounded;

    // Exponent: base + norm_shift + round_ovf
    wire signed [EXP_BITS:0] s10_exp_out = s9_exp_base_r
                                          + {{EXP_BITS{1'b0}}, s10_norm_shift}
                                          + {{EXP_BITS{1'b0}}, s10_round_ovf};

    wire s10_exp_too_big   = (s10_exp_out > MAX_EXP);
    wire s10_exp_too_small = (s10_exp_out < MIN_EXP);

    // Output register (sqrt is always non-negative)
    always @(posedge clk) if (ce) begin
        result_frac <= s9_invalid_r     ? {FRAC_BITS{1'b0}} :
                       s10_exp_too_big  ? $signed(s10_pos_frac) :
                       s10_exp_too_small ? $signed({1'b0, s10_pos_frac[FRAC_BITS-1:1]}) :
                                          $signed(s10_pos_frac);

        result_exp  <= (s9_invalid_r | s10_exp_too_big | s10_exp_too_small) ?
                        AMBIGUOUS_EXP[EXP_BITS-1:0] : s10_exp_out[EXP_BITS-1:0];
    end

endmodule
