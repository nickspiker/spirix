; circle_divide.ll  -  Spirix CircleF5E4 complex division, hand-written LLVM IR
;
; Mirrors Circle::circle_divide_circle in src/implementations/division/circle_circle.rs for the concrete instantiation Circle<i32, i16> (CircleF5E4: 32-bit components, 16-bit shared exponent).
;
; Verified by ports/test/ : every value below is differentially tested against the Rust implementation over randomized and edge-case inputs. If you change the encoding in Rust, that harness will tell you.
;
; ---------------------------------------------------------------------
; CONTRACT
; ---------------------------------------------------------------------
; This is the NORMAL x NORMAL fast path only. Spirix's escape classes (undefined / zero / infinity / exploded / vanished operands) all live at the AMBIG exponent, and their cross-product is a large table that the Rust version handles after this path declines. So the function returns i1:
;
;   1 = result written to %out, bit-identical to Rust
;   0 = operand in an escape class, NOTHING written, caller must fall back to the full implementation
;
; The caller's own `is_normal()` is exactly the check done here, so a caller that has already tested both operands can ignore the flag.
;
; ---------------------------------------------------------------------
; READING GUIDE
; ---------------------------------------------------------------------
;   %name   = local SSA value, assigned exactly once, infinite supply
;   @name   = global (function or constant)
;   iN      = N-bit integer. NO signedness in the type. Signedness lives in the INSTRUCTION: ashr vs lshr, sdiv vs udiv, icmp slt vs ult, sext vs zext. Exactly like your ASM, unlike C/Swift/Rust.
;   mul/add/sub/shl wrap by default (two's complement). Wrapping is the NATIVE behavior; Rust/Swift/C add checks or UB on top of it. This is why Rust's w_add / w_mul / w_sub (all wrapping_*) map to bare add / mul / sub here with nothing bolted on.
;   Every basic block ends in exactly one terminator (br / ret).

; Field order and offsets match Rust's actual layout for this type (size 12, align 4, offsets 0 / 4 / 8), but note that Rust's `Circle` is NOT #[repr(C)], so that layout is not guaranteed by the language. Callers should go through a #[repr(C)] mirror struct, as ports/test/ does, rather than transmuting Circle directly.
%Circle = type { i32, i32, i16 }      ; real, imaginary, exponent

; ---------------------------------------------------------------------
; CONSTANTS
; ---------------------------------------------------------------------
; Spirix encodes the exponent as a CYCLIC UNSIGNED value in Z/2^16 Z, with AMBIG at the all-zeros pattern. Two consequences drive everything below:
;
;   1. Widening an exponent for arithmetic is a ZERO-extend (`zext`), not a sign-extend. That is Rust's `cycle_widen()`. Using `sext` here silently breaks every exponent at or above 0x8000, which is every value >= 1.0.
;   2. The in-range test is against the widened CYCLE POSITIONS [1, 65535], not against i16's signed range. Overflow past the top and underflow past the bottom both land on 0 = AMBIG from opposite directions, which is what makes one compare per side sufficient.

; Circle::ambiguous_exponent() = E::zero().
@AMBIG_EXPONENT = internal constant i16 0

; Circle::binade_origin() = E::min_value() = -32768 = 0x8000, the bit-pattern offset of the +1.0 binade from AMBIG. The division uses it already widened: binade_origin().cycle_widen() = zext(0x8000) = 32768.
@BINADE_ORIGIN = internal constant i32 32768

; Circle::max_exponent() = -E::one() = 0xFFFF, widened = 65535. Circle::min_exponent() =  E::one() = 0x0001, widened = 1.
@MAX_EXPONENT_POS = internal constant i32 65535
@MIN_EXPONENT_POS = internal constant i32 1

; Circle::ZERO and Circle::INFINITY. Both sit at AMBIG (exponent 0); the components carry the class. Zero is all-zeros so that a memset(0) value reads as Spirix Zero; Infinity is all-ones, the deliberate mirror image. Infinity is singular here - unlike IEEE-754 there is no signed infinity, because the direction is indeterminate.
@CIRCLE_ZERO     = internal constant %Circle { i32  0, i32  0, i16 0 }
@CIRCLE_INFINITY = internal constant %Circle { i32 -1, i32 -1, i16 0 }

; 2^62 = Rust's `scale`: F::one().sign_extend().w_shl(2*FRAC - 2), with FRAC = 32. The reciprocal is computed in this fixed-point scale.
@RECIP_SCALE = internal constant i64 4611686018427387904

; Intrinsics: target-independent ops each backend lowers to the best instruction it has (lzcnt on x86 with BMI, clz on ARM64). The i1 flag = "is result for input 0 poison?" false => returns bit width.
declare i32 @llvm.ctlz.i32(i32, i1)
declare i64 @llvm.ctlz.i64(i64, i1)
declare i32 @llvm.umax.i32(i32, i32)
declare i32 @llvm.umin.i32(i32, i32)
declare i64 @llvm.umax.i64(i64, i64)
declare i64 @llvm.umin.i64(i64, i64)

; Rust's WideOps::leading_same = leading_ones().max(leading_zeros()), i.e. how many leading bits agree with the sign bit, counting the sign bit itself. Your ASM lines 034-039, one-for-one: lzcnt, not, lzcnt, cmov-max. Result is in [1, 32] for any input: ctlz(0) is 32 and ctlz(~0) is 0, so the max is never 0.
define internal i32 @leading_same32(i32 %x) {
  %lz  = call i32 @llvm.ctlz.i32(i32 %x, i1 false)
  %inv = xor i32 %x, -1                            ; `not` = xor with all-ones
  %lo  = call i32 @llvm.ctlz.i32(i32 %inv, i1 false)
  %m   = call i32 @llvm.umax.i32(i32 %lz, i32 %lo)
  ret i32 %m
}

; Same for the 64-bit wide values. Result is in [1, 64].
define internal i64 @leading_same64(i64 %x) {
  %lz  = call i64 @llvm.ctlz.i64(i64 %x, i1 false)
  %inv = xor i64 %x, -1
  %lo  = call i64 @llvm.ctlz.i64(i64 %inv, i1 false)
  %m   = call i64 @llvm.umax.i64(i64 %lz, i64 %lo)
  ret i64 %m
}

; Circle::canonical_n1_pair. Normalizes a component pair to canonical N1 form (the dominant component's top two bits are 01 or 10) and reports the left-shift applied, so the caller can pay it back on the exponent.
;
; Why it exists: Circle's fields are pub, so a caller can build a de-normalized pair by hand. Feeding one to the reciprocal below would zero the divisor and destroy the product's precision.
;
; Returns an anonymous struct { r, i, s }. First-class aggregates are built with insertvalue and taken apart with extractvalue. s = -1 signals "both components zero", which the caller turns into Zero (numerator) or Infinity (denominator).
define internal { i32, i32, i32 } @canonical_n1_pair(i32 %r, i32 %i) {
entry:
  %or   = or i32 %r, %i
  %zero = icmp eq i32 %or, 0
  br i1 %zero, label %is_zero, label %normalize

is_zero:
  ret { i32, i32, i32 } { i32 0, i32 0, i32 -1 }

normalize:
  %lr   = call i32 @leading_same32(i32 %r)
  %li   = call i32 @leading_same32(i32 %i)
  %lead = call i32 @llvm.umin.i32(i32 %lr, i32 %li)
  ; leading_same is >= 1, so this shift is >= 0 and never poisons `shl`. Rust early-returns when it is 0; shifting by 0 is the same thing.
  %s    = sub i32 %lead, 1
  %nr   = shl i32 %r, %s                 ; no `cl` rule. The backend handles it.
  %ni   = shl i32 %i, %s
  %t0   = insertvalue { i32, i32, i32 } undef, i32 %nr, 0
  %t1   = insertvalue { i32, i32, i32 } %t0,   i32 %ni, 1
  %t2   = insertvalue { i32, i32, i32 } %t1,   i32 %s,  2
  ret { i32, i32, i32 } %t2
}

; i1 circle_divide_f5e4(Circle *out, const Circle *num, const Circle *den) Pointer shape matches your ASM (result ptr, dividend ptr, divisor ptr); the i1 return is the escape-class flag described under CONTRACT above.
define i1 @circle_divide_f5e4(ptr %out, ptr %num, ptr %den) {
entry:
  ; getelementptr (GEP) = address arithmetic from the TYPE layout. Your [rsi+4] / [rsi+8], but the offsets come from %Circle, not your head.
  %num_i_p = getelementptr inbounds %Circle, ptr %num, i32 0, i32 1
  %num_e_p = getelementptr inbounds %Circle, ptr %num, i32 0, i32 2
  %den_i_p = getelementptr inbounds %Circle, ptr %den, i32 0, i32 1
  %den_e_p = getelementptr inbounds %Circle, ptr %den, i32 0, i32 2
  %xr = load i32, ptr %num                  ; field 0 lives at the base ptr
  %xi = load i32, ptr %num_i_p
  %xe = load i16, ptr %num_e_p
  %yr = load i32, ptr %den
  %yi = load i32, ptr %den_i_p
  %ye = load i16, ptr %den_e_p

  ; Circle::is_normal() is exactly `exponent != ambiguous_exponent()`. Everything else is an escape class this fast path does not implement.
  %ambig  = load i16, ptr @AMBIG_EXPONENT
  %xn     = icmp ne i16 %xe, %ambig
  %yn     = icmp ne i16 %ye, %ambig
  %normal = and i1 %xn, %yn
  br i1 %normal, label %canonicalize, label %ret_escape

canonicalize:
  %pn    = call { i32, i32, i32 } @canonical_n1_pair(i32 %xr, i32 %xi)
  %nr    = extractvalue { i32, i32, i32 } %pn, 0
  %ni    = extractvalue { i32, i32, i32 } %pn, 1
  %s_num = extractvalue { i32, i32, i32 } %pn, 2
  %pd    = call { i32, i32, i32 } @canonical_n1_pair(i32 %yr, i32 %yi)
  %dr    = extractvalue { i32, i32, i32 } %pd, 0
  %di    = extractvalue { i32, i32, i32 } %pd, 1
  %s_den = extractvalue { i32, i32, i32 } %pd, 2

  ; A normal-exponent value whose components are both zero is a degenerate normal, reachable because the fields are pub. 0-valued / x = 0, and x / 0-valued = infinity.
  %num_zero = icmp slt i32 %s_num, 0
  br i1 %num_zero, label %ret_zero, label %check_den

check_den:
  %den_zero = icmp slt i32 %s_den, 0
  br i1 %den_zero, label %ret_inf, label %compute

compute:
  ; Two's-complement boundary guard. |F::MIN| exceeds every positive magnitude by one, and the reciprocal-multiply is exactly one bit short for it - (-1)/i used to reach +2^63 and wrap the imaginary sign. Pre-halve any operand carrying a boundary component; the exponent bump below preserves the value, and the leading_same normalization absorbs the shift.
  ;
  ; This also keeps c*c + d*d below 2^63: with both |c|,|d| <= 2^31 - 1 the sum is at most 2^63 - 2^33 + 2, so mag_sq stays positive and the lshr/udiv below read it as the unsigned value it is.
  ;
  ; `select` = branchless ternary, lowers to cmov / csel.
  %nr_min = icmp eq i32 %nr, -2147483648
  %ni_min = icmp eq i32 %ni, -2147483648
  %a_bump = or i1 %nr_min, %ni_min
  %nr_h   = ashr i32 %nr, 1
  %ni_h   = ashr i32 %ni, 1
  %ar     = select i1 %a_bump, i32 %nr_h, i32 %nr
  %ai     = select i1 %a_bump, i32 %ni_h, i32 %ni

  %dr_min = icmp eq i32 %dr, -2147483648
  %di_min = icmp eq i32 %di, -2147483648
  %c_bump = or i1 %dr_min, %di_min
  %dr_h   = ashr i32 %dr, 1
  %di_h   = ashr i32 %di, 1
  %cr     = select i1 %c_bump, i32 %dr_h, i32 %dr
  %ci     = select i1 %c_bump, i32 %di_h, i32 %di

  ; sign_extend(): your movsx. The fraction IS signed, so this one is a sext - contrast with the exponent's zext further down.
  %a = sext i32 %ar to i64
  %b = sext i32 %ai to i64
  %c = sext i32 %cr to i64
  %d = sext i32 %ci to i64

  ; (a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c^2 + d^2). The divisor is computed once as a reciprocal in 2^62 fixed point, then multiplied into both numerators - one udiv instead of two.
  %cc         = mul i64 %c, %c
  %dd         = mul i64 %d, %d
  %mag_sq     = add i64 %cc, %dd
  %mag_hi     = lshr i64 %mag_sq, 32                        ; w_shr_logical(FRAC)
  %scale      = load i64, ptr @RECIP_SCALE
  ; Canonical N1 guarantees |c| or |d| >= 2^30, so mag_sq >= 2^60 and mag_hi >= 2^28. It is never 0, which matters because udiv by 0 is immediate UB in LLVM (Rust would panic instead).
  %reciprocal = udiv i64 %scale, %mag_hi                    ; w_div_unsigned

  %ac        = mul i64 %a, %c
  %bd        = mul i64 %b, %d
  %bc        = mul i64 %b, %c
  %ad        = mul i64 %a, %d
  %real_num  = add i64 %ac, %bd
  %imag_num  = sub i64 %bc, %ad
  %rn_s      = ashr i64 %real_num, 32                       ; w_shr (arithmetic)
  %in_s      = ashr i64 %imag_num, 32
  %real_wide = mul i64 %rn_s, %reciprocal
  %imag_wide = mul i64 %in_s, %reciprocal

  ; Renormalize the wide result back to canonical N1 and remember the shift, which the exponent pays back one binade per bit.
  %lr    = call i64 @leading_same64(i64 %real_wide)
  %li    = call i64 @leading_same64(i64 %imag_wide)
  %lead  = call i64 @llvm.umin.i64(i64 %lr, i64 %li)
  %shift = sub i64 %lead, 1
  ; leading_same64 is in [1, 64], so %shift is in [0, 63] and the mask is a no-op today. It stays because shl by >= bit width is POISON in LLVM, and Rust's w_shl instead saturates to 0 - the mask keeps a future change from turning that difference into silent garbage.
  %sh    = and i64 %shift, 63

  %rw_l = shl i64 %real_wide, %sh
  %rw_r = ashr i64 %rw_l, 32
  %real = trunc i64 %rw_r to i32                            ; deflate()
  %iw_l = shl i64 %imag_wide, %sh
  %iw_r = ashr i64 %iw_l, 32
  %imag = trunc i64 %iw_r to i32

  ; Exponent bookkeeping, all on widened CYCLE POSITIONS in i32. cycle_widen() is a ZERO-extend: the exponent is an unsigned position on a 2^16 cycle, not a signed number. See CONSTANTS above.
  %xe32    = zext i16 %xe to i32                            ; cycle_widen
  %ye32    = zext i16 %ye to i32                            ; cycle_widen
  ; Canonicalization multiplied each pair by 2^s, so the exponent owes -s; the boundary halving divided by 2, so it owes +1.
  %a_inc   = zext i1 %a_bump to i32                         ; true -> 1
  %c_inc   = zext i1 %c_bump to i32
  %pa0     = sub i32 %xe32, %s_num
  %pa      = add i32 %pa0, %a_inc
  %pb0     = sub i32 %ye32, %s_den
  %pb      = add i32 %pb0, %c_inc
  %bo      = load i32, ptr @BINADE_ORIGIN
  ; Rust narrows the shift to E and sign-extends it (shift_e.sign_extend()), so mirror that rather than truncating straight to i32. Same answer for the [0, 63] range reachable here, same shape if E ever changes.
  %shift16 = trunc i64 %shift to i16
  %shift32 = sext i16 %shift16 to i32
  ; stored_pos = pa - pb + binade_origin - shift
  %e0      = sub i32 %pa, %pb
  %e1      = add i32 %e0, %bo
  %stored  = sub i32 %e1, %shift32

  ; Saturation, against widened cycle positions. Overflow keeps the fraction and drops the exponent to AMBIG (an exploded value); underflow halves the fraction on the way to AMBIG (vanished).
  %max_pos = load i32, ptr @MAX_EXPONENT_POS
  %over    = icmp sgt i32 %stored, %max_pos
  br i1 %over, label %store, label %check_under

check_under:
  %min_pos = load i32, ptr @MIN_EXPONENT_POS
  %under   = icmp slt i32 %stored, %min_pos
  br i1 %under, label %vanished, label %in_range

vanished:
  %real_v = ashr i32 %real, 1
  %imag_v = ashr i32 %imag, 1
  br label %store

in_range:
  %exp_n = trunc i32 %stored to i16                         ; deflate()
  br label %store

; ONE exit block for three paths. phi picks each value based on which block we arrived from. This is how SSA says "this variable has different values depending on the path" without ever assigning anything twice. Note the predecessors are compute / vanished / in_range - check_under always falls through to one of the latter two, so it never reaches here.
store:
  %fr = phi i32 [ %real,  %compute ], [ %real_v, %vanished ], [ %real,  %in_range ]
  %fi = phi i32 [ %imag,  %compute ], [ %imag_v, %vanished ], [ %imag,  %in_range ]
  %fe = phi i16 [ 0,      %compute ], [ 0,       %vanished ], [ %exp_n, %in_range ]
  %out_i_p = getelementptr inbounds %Circle, ptr %out, i32 0, i32 1
  %out_e_p = getelementptr inbounds %Circle, ptr %out, i32 0, i32 2
  store i32 %fr, ptr %out
  store i32 %fi, ptr %out_i_p
  store i16 %fe, ptr %out_e_p
  ret i1 true

; Field-wise stores rather than one aggregate store, so the two padding bytes at offset 10 are left exactly as the caller had them.
ret_zero:
  %z_r = load i32, ptr getelementptr inbounds (%Circle, ptr @CIRCLE_ZERO, i32 0, i32 0)
  %z_i = load i32, ptr getelementptr inbounds (%Circle, ptr @CIRCLE_ZERO, i32 0, i32 1)
  %z_e = load i16, ptr getelementptr inbounds (%Circle, ptr @CIRCLE_ZERO, i32 0, i32 2)
  %z_i_p = getelementptr inbounds %Circle, ptr %out, i32 0, i32 1
  %z_e_p = getelementptr inbounds %Circle, ptr %out, i32 0, i32 2
  store i32 %z_r, ptr %out
  store i32 %z_i, ptr %z_i_p
  store i16 %z_e, ptr %z_e_p
  ret i1 true

ret_inf:
  %f_r = load i32, ptr getelementptr inbounds (%Circle, ptr @CIRCLE_INFINITY, i32 0, i32 0)
  %f_i = load i32, ptr getelementptr inbounds (%Circle, ptr @CIRCLE_INFINITY, i32 0, i32 1)
  %f_e = load i16, ptr getelementptr inbounds (%Circle, ptr @CIRCLE_INFINITY, i32 0, i32 2)
  %f_i_p = getelementptr inbounds %Circle, ptr %out, i32 0, i32 1
  %f_e_p = getelementptr inbounds %Circle, ptr %out, i32 0, i32 2
  store i32 %f_r, ptr %out
  store i32 %f_i, ptr %f_i_p
  store i16 %f_e, ptr %f_e_p
  ret i1 true

; Escape class: nothing written, caller falls back to the full path.
ret_escape:
  ret i1 false
}
