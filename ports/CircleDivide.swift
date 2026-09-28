// CircleDivide.swift  -  Spirix CircleF5E4 complex division, Swift port
//
// Mirrors Circle::circle_divide_circle in src/implementations/division/circle_circle.rs for Circle<i32, i16>. Same contract as circle_divide.ll: normal x normal only, nil for escape-class operands. Verified against Rust by ports/test/.
//
// Swift specifics worth knowing here:
//   * `+ - * <<` TRAP on overflow, release builds too. The `&+ &- &* &<< &>>` family is the wrapping version: Rust's wrapping_add etc. as operators.
//   * `&<<` / `&>>` also mask the shift amount, like Rust's wrapping_shl.
//   * Widening conversions (Int64(someInt32)) cannot fail. Narrowing needs `truncatingIfNeeded:` or it traps if the value does not fit.
//   * The exponent is a cyclic UNSIGNED position, so widening it goes through UInt16 first. Int32(someInt16) sign-extends and silently breaks every exponent >= 0x8000, which is every value >= 1.0.

public struct CircleF5E4: Equatable {   // struct = value type, no ARC traffic
    public var real: Int32
    public var imaginary: Int32
    public var exponent: Int16

    public init(real: Int32, imaginary: Int32, exponent: Int16) {
        self.real = real; self.imaginary = imaginary; self.exponent = exponent
    }

    static let fractionBits: Int64 = 32
    // Widened CYCLE POSITIONS, not i16's signed range: max_exponent()=0xFFFF, min_exponent()=1.
    static let maxExpPos: Int32 = 65535
    static let minExpPos: Int32 = 1
    // binade_origin().cycle_widen() = zext(0x8000)
    static let binadeOrigin: Int32 = 32768
    static let ambigExp: Int16 = 0

    static let zero     = CircleF5E4(real:  0, imaginary:  0, exponent: 0)
    static let infinity = CircleF5E4(real: -1, imaginary: -1, exponent: 0)

    public var isNormal: Bool { exponent != CircleF5E4.ambigExp }
}

// Protocol extension on the stdlib's FixedWidthInteger: one generic leadingSame for Int32 AND Int64.
extension FixedWidthInteger {
    @inline(__always)
    var leadingSame: Int {
        // leadingZeroBitCount is well-defined for 0 (returns bitWidth). No UB.
        Swift.max(leadingZeroBitCount, (~self).leadingZeroBitCount)
    }
}

extension CircleF5E4 {
    @inline(__always)
    static func canonicalN1Pair(_ r: Int32, _ i: Int32) -> (Int32, Int32, Int32) {
        if r | i == 0 { return (0, 0, -1) }
        let s = Int32(Swift.min(r.leadingSame, i.leadingSame) - 1)
        return (r &<< s, i &<< s, s)
    }

    // Normal x normal fast path. Returns nil for escape-class operands, matching the i1 flag in circle_divide.ll.
    public static func divideNormal(_ lhs: CircleF5E4, _ rhs: CircleF5E4) -> CircleF5E4? {
        guard lhs.isNormal && rhs.isNormal else { return nil }

        let (nr, ni, sNum) = canonicalN1Pair(lhs.real, lhs.imaginary)
        let (dr, di, sDen) = canonicalN1Pair(rhs.real, rhs.imaginary)
        if sNum < 0 { return .zero }
        if sDen < 0 { return .infinity }

        let aBump = nr == .min || ni == .min
        let cBump = dr == .min || di == .min
        let a = Int64(aBump ? nr >> 1 : nr)    // Int64(Int32) = sign_extend
        let b = Int64(aBump ? ni >> 1 : ni)
        let c = Int64(cBump ? dr >> 1 : dr)
        let d = Int64(cBump ? di >> 1 : di)
        let fb = fractionBits

        // c*c + d*d can exceed Int64.max, so wrap, then reinterpret the bits as unsigned for the logical shift and unsigned divide.
        let magSq = UInt64(bitPattern: c &* c &+ d &* d)
        let scale: UInt64 = 1 << (2 * 32 - 2)
        let reciprocal = Int64(bitPattern: scale / (magSq >> UInt64(fb)))

        let realNum = a &* c &+ b &* d
        let imagNum = b &* c &- a &* d
        let realWide = (realNum >> fb) &* reciprocal   // >> on Int64 = arithmetic
        let imagWide = (imagNum >> fb) &* reciprocal

        let shift = Int64(Swift.min(realWide.leadingSame, imagWide.leadingSame) - 1)
        // Int32(x) would TRAP if x didn't fit. Rust's `as` / deflate() truncates; in Swift you must say so by name.
        let real = Int32(truncatingIfNeeded: (realWide &<< shift) >> fb)
        let imag = Int32(truncatingIfNeeded: (imagWide &<< shift) >> fb)

        // cycle_widen: the exponent is an unsigned cycle position, so widen through the UNSIGNED type. Int32(lhs.exponent) would sign-extend and break every exponent >= 0x8000, i.e. every value >= 1.0.
        let xe = Int32(UInt16(bitPattern: lhs.exponent))
        let ye = Int32(UInt16(bitPattern: rhs.exponent))
        let pa = xe &- sNum &+ (aBump ? 1 : 0)
        let pb = ye &- sDen &+ (cBump ? 1 : 0)
        let stored = pa &- pb &+ binadeOrigin &- Int32(shift)

        switch stored {
        case let s where s > maxExpPos:
            return CircleF5E4(real: real, imaginary: imag, exponent: ambigExp)
        case let s where s < minExpPos:
            return CircleF5E4(real: real >> 1, imaginary: imag >> 1, exponent: ambigExp)
        default:
            return CircleF5E4(real: real, imaginary: imag, exponent: Int16(truncatingIfNeeded: stored))
        }
    }
}

// C-callable entry point so the Rust differential harness can reach it. @_cdecl demands ObjC-representable parameter types and a Swift struct is not one, so the boundary is raw pointers with an explicit load/store.
@_cdecl("circle_divide_swift")
public func circle_divide_swift(_ out: UnsafeMutableRawPointer,
                                _ num: UnsafeRawPointer,
                                _ den: UnsafeRawPointer) -> Bool {
    let n = num.loadUnaligned(as: CircleF5E4.self)
    let d = den.loadUnaligned(as: CircleF5E4.self)
    guard let r = CircleF5E4.divideNormal(n, d) else { return false }
    out.storeBytes(of: r.real,      toByteOffset: 0, as: Int32.self)
    out.storeBytes(of: r.imaginary, toByteOffset: 4, as: Int32.self)
    out.storeBytes(of: r.exponent,  toByteOffset: 8, as: Int16.self)
    return true
}
