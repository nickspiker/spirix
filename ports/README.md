# Spirix ports

The same Spirix operation written by hand in other languages, kept as a reference for backend, codegen and binding work — the role `fpga/cores` plays for RTL and `gpu/` plays for HIP and WGSL.

Each port mirrors `Circle::circle_divide_circle` from [`src/implementations/division/circle_circle.rs`](../src/implementations/division/circle_circle.rs) for the concrete instantiation `Circle<i32, i16>` (`CircleF5E4`: 32-bit components, 16-bit shared exponent).

| File | Language | Notes |
|------|----------|-------|
| [`circle_divide.ll`](circle_divide.ll) | LLVM IR | Hand-written, target-independent |
| [`circle_divide.m`](circle_divide.m) | Objective-C | C structs, no Foundation, no ObjC runtime |
| [`CircleDivide.swift`](CircleDivide.swift) | Swift | Value-type struct, generic `leadingSame` |
| [`nsobject_overhead.m`](nsobject_overhead.m) | Objective-C | Not a port: measures what NSObject costs |

## Contract

Every port implements the **normal × normal fast path only** and reports whether it handled the operands — `i1`/`bool` in IR and Objective-C, a `nil` Optional in Swift:

- handled — result written, bit-identical to Rust
- declined — an operand is in an escape class, **nothing written**, the
caller must fall back to the full implementation

Spirix's escape classes (undefined, zero, infinity, exploded, vanished) all live at the AMBIG exponent, and their cross-product is a large table. `Circle::is_normal()` is just `exponent != 0`, so the guard costs one compare and the table stays in Rust.

## Verifying

```sh
cd ports/test && cargo run --release
```

Requires `clang` on `PATH`; it compiles both the `.ll` and the `.m` directly, so a separate LLVM install with `llvm-as` is not needed. The Swift port additionally needs `swiftc` — without it that port is skipped with a warning rather than failing the build.

The harness compares every port against `CircleF5E4 / CircleF5E4` over ~240k cases: randomized normal pairs with arbitrary non-canonical components, values built through the public API, the two's-complement boundary where `i32::MIN` forces the pre-halving guard, exponent saturation in both directions, degenerate zero-component normals, and every escape-class combination. It exits non-zero on mismatch and prints both results in stored and rendered form.

The test crate declares its own `[workspace]`, so `cargo test` at the repo root neither builds it nor needs a C or Swift toolchain.

## What the comparison showed

Measured on an M-series Mac, arm64, `-O2` / `-O`. Instruction counts are for the division function body; timings are the best of five runs over ~153k normal pairs.

| Port | arm64 instructions | ns/divide | Object | Runtime deps |
|------|-------------------:|----------:|-------:|--------------|
| LLVM IR | 121 | 10.5 | — | none |
| Objective-C | 122 | 9.9 | 1.1 KB | none |
| Swift | 160 + thunks | 10.4 | 6.8 KB | 4 metadata symbols |
| Rust (full impl) | — | 12.8 | — | none |

Objective-C lands **one instruction** away from hand-written IR. Its struct path is plain C, clang's optimizer reaches the same fixed point, and it gets there from readable source — hand-writing the IR bought nothing on this operation.

Swift costs ~30% more instructions, mostly register traffic around the `Optional<CircleF5E4>` return (32 `mov` against the IR's 13), plus a specialization thunk. It also emits getter/setter/`modify` accessors for every property, and the bare `Equatable` conformance pulls in Swift runtime metadata, so it will not link freestanding.

`nsobject_overhead.m` covers the other half of the Objective-C answer: the same arithmetic behind an `NSObject` with ARC properties runs **17× slower** (7.3 → 124 ns), because every property read is an `objc_msgSend` and every result an `objc_alloc_init` plus an autorelease. That is a cost of objects on a hot path, not of the language.

## Hand-optimization headroom

`leading_same` — count the leading bits agreeing with the sign — is written in Rust as `leading_ones().max(leading_zeros())`, and every port transcribes that shape. It costs **5 instructions** and runs 6× per divide. There is an identity:

```
leading_same(x) == leading_zeros(x ^ (x >>arithmetic (BITS-1)))
```

The arithmetic shift broadcasts the sign, so the XOR complements a negative value and leaves a positive one alone; either way the leading zero count of the result is the answer. `x = 0` and `x = -1` both map to 0, whose count is the full width, which is the correct all-same answer. Verified exhaustively over all 2³² `i32` values.

On arm64 this is **2 instructions** — `eor w8, w0, w0, asr #31` then `clz`, with the shift riding free in `eor`'s shifted operand. Written this way in Rust, LLVM recognizes it and emits `cls` (count leading sign bits) plus `add`.

Applied to `circle_divide.ll` it takes the function from 121 to 103 instructions, still bit-identical, and 10.5 → 9.6 ns.

The identity is not division-specific: `leading_same` has 49 call sites across nearly every operation module. Changing it in `src/core/integer.rs` passes all 2000 tests and, measured by interleaving two prebuilt binaries ten times each to cancel thermal drift, gives roughly −18% on Circle multiply, −11% on Circle divide, and −4% on the Scalar ops. **That change is not currently applied** — it touches the hottest primitive in the library, so it wants its own review.

A caution on measuring it: a naive "run A then run B" comparison on a busy machine is worthless here. The first attempt showed 8–13% wins and the second reversed on three of four operations. Interleave the binaries and take the minimum.

The remaining time is dominated by the single 64-bit `udiv`. Getting past that needs an algorithmic change, not a codegen one.

## Keeping the ports honest

The encoding constants in each port are transcribed from Rust, so they go stale silently if the encoding moves — and all three inherited the same v0.0.x mistakes before the harness existed: AMBIG written as `E::MIN` instead of `0`, `binade_origin` left at zero, saturation checked against i16's signed range instead of the widened cycle positions `[1, 65535]`, and the exponent sign-extended where `cycle_widen` zero-extends.

That last one is the easiest to get wrong and the hardest to see: the fraction is signed and wants `sext`, the exponent is an unsigned cycle position and wants `zext`, and they sit a few lines apart. Sign-extending the exponent breaks every value ≥ 1.0 and nothing else.

Re-run the harness after any change to the exponent encoding, the class constants, or `canonical_n1_pair`. To confirm it still has teeth, break one constant on purpose; each of these produced five to six figures of failures when checked against the IR port:

| Mutation | Failures |
|----------|---------:|
| exponent `sext` instead of `zext` | 126,795 |
| `BINADE_ORIGIN` left at 0 | 213,876 |
| overflow bound `32767` instead of `65535` | 92,675 |
| AMBIG written as `-32768` instead of `0` | 50,154 |
| underflow bound `-32767` instead of `1` | 25,078 |
| `ZERO` / `INFINITY` placeholder patterns | 404 / 388 |
| `is_normal` guard weakened to `or` | 10 |
