// circle_divide.m  -  Spirix CircleF5E4 complex division, Objective-C port
//
// Mirrors Circle::circle_divide_circle in src/implementations/division/circle_circle.rs for Circle<i32, i16>. Same contract as circle_divide.ll: normal x normal only, false for escape-class operands. Verified against Rust by ports/test/.
//
// C structs, no Foundation, no ObjC runtime - this is what you would ship. For the cost of doing it with NSObject instead, see nsobject_overhead.m.
//
// The C-language trap, and it is worse than Swift's: SIGNED OVERFLOW IS UB. Not wrapping, not trapping - undefined, and LLVM will delete code around it. Rust says wrapping_mul, Swift says &*, and C says nothing at all, so every wrapping multiply below launders through uint64_t.

#include <stdint.h>
#include <stdbool.h>

typedef struct {
    int32_t real;
    int32_t imaginary;
    int16_t exponent;
} SpxCircleF5E4;

static const int32_t kMaxExpPos    = 65535;   // max_exponent().cycle_widen()
static const int32_t kMinExpPos    = 1;       // min_exponent().cycle_widen()
static const int32_t kBinadeOrigin = 32768;   // binade_origin().cycle_widen()
static const int16_t kAmbigExp     = 0;

// __builtin_clz is UNDEFINED for 0, unlike Swift's leadingZeroBitCount and unlike llvm.ctlz with the false flag. Guard it or lose the zero case.
static inline int leading_same32(int32_t x) {
    uint32_t u = (uint32_t)x;
    int lz = (u == 0) ? 32 : __builtin_clz(u);
    uint32_t n = ~u;
    int lo = (n == 0) ? 32 : __builtin_clz(n);
    return lz > lo ? lz : lo;
}

static inline int leading_same64(int64_t x) {
    uint64_t u = (uint64_t)x;
    int lz = (u == 0) ? 64 : __builtin_clzll(u);
    uint64_t n = ~u;
    int lo = (n == 0) ? 64 : __builtin_clzll(n);
    return lz > lo ? lz : lo;
}

static inline int32_t canonical_n1_pair(int32_t r, int32_t i,
                                        int32_t *out_r, int32_t *out_i) {
    if ((r | i) == 0) { *out_r = 0; *out_i = 0; return -1; }
    int lr = leading_same32(r), li = leading_same32(i);
    int32_t s = (int32_t)((lr < li ? lr : li) - 1);
    // Left-shifting a negative int32_t is UB in C too. Launder through unsigned.
    *out_r = (int32_t)((uint32_t)r << s);
    *out_i = (int32_t)((uint32_t)i << s);
    return s;
}

// Returns true when handled, matching the i1 flag in circle_divide.ll.
bool circle_divide_struct(SpxCircleF5E4 *out,
                          const SpxCircleF5E4 *num,
                          const SpxCircleF5E4 *den) {
    if (num->exponent == kAmbigExp || den->exponent == kAmbigExp) return false;

    int32_t nr, ni, dr, di;
    int32_t s_num = canonical_n1_pair(num->real, num->imaginary, &nr, &ni);
    int32_t s_den = canonical_n1_pair(den->real, den->imaginary, &dr, &di);

    if (s_num < 0) { out->real = 0;  out->imaginary = 0;  out->exponent = 0; return true; }
    if (s_den < 0) { out->real = -1; out->imaginary = -1; out->exponent = 0; return true; }

    bool a_bump = (nr == INT32_MIN) || (ni == INT32_MIN);
    bool c_bump = (dr == INT32_MIN) || (di == INT32_MIN);
    int64_t a = a_bump ? (nr >> 1) : nr;
    int64_t b = a_bump ? (ni >> 1) : ni;
    int64_t c = c_bump ? (dr >> 1) : dr;
    int64_t d = c_bump ? (di >> 1) : di;

    uint64_t ua = (uint64_t)a, ub = (uint64_t)b, uc = (uint64_t)c, ud = (uint64_t)d;
    uint64_t mag_sq = ua * 0 + (uc * uc) + (ud * ud);   // wraps by definition
    uint64_t reciprocal = (UINT64_C(1) << 62) / (mag_sq >> 32);

    int64_t real_num = (int64_t)((ua * uc) + (ub * ud));
    int64_t imag_num = (int64_t)((ub * uc) - (ua * ud));
    int64_t real_wide = (int64_t)((uint64_t)(real_num >> 32) * reciprocal);
    int64_t imag_wide = (int64_t)((uint64_t)(imag_num >> 32) * reciprocal);

    int lr = leading_same64(real_wide), li = leading_same64(imag_wide);
    int64_t shift = (int64_t)((lr < li ? lr : li) - 1);

    int32_t real = (int32_t)((int64_t)((uint64_t)real_wide << shift) >> 32);
    int32_t imag = (int32_t)((int64_t)((uint64_t)imag_wide << shift) >> 32);

    // cycle_widen: zero-extend through the unsigned type.
    int32_t xe = (int32_t)(uint16_t)num->exponent;
    int32_t ye = (int32_t)(uint16_t)den->exponent;
    int32_t pa = xe - s_num + (a_bump ? 1 : 0);
    int32_t pb = ye - s_den + (c_bump ? 1 : 0);
    int32_t stored = pa - pb + kBinadeOrigin - (int32_t)shift;

    if (stored > kMaxExpPos) {
        out->real = real; out->imaginary = imag; out->exponent = kAmbigExp;
    } else if (stored < kMinExpPos) {
        out->real = real >> 1; out->imaginary = imag >> 1; out->exponent = kAmbigExp;
    } else {
        out->real = real; out->imaginary = imag; out->exponent = (int16_t)stored;
    }
    return true;
}

