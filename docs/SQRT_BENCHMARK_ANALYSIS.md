# Square Root Algorithm Benchmark Analysis

## Summary

Compared Newton-Raphson (with division) vs Non-Restoring Bit-by-Bit sqrt implementations.

**Unexpected Result**: Newton-Raphson is **1.8-6.3× FASTER** despite using division!

## Benchmark Results

| Bit Width | Newton (ns) | Bitwise (ns) | Speedup   |
|-----------|-------------|--------------|-----------|
| 8-bit     | 5.6         | 10.2         | 1.8× slower |
| 16-bit    | 7.1         | 21.9         | 3.1× slower |
| 32-bit    | 19.0        | 40.3         | 2.1× slower |
| 64-bit    | 28.8        | 181.4        | 6.3× slower |

## Assembly Analysis

### Newton-Raphson Assembly Characteristics
- **Total lines**: 571 (including edge cases)
- **Core loop**: Tight 10-15 instruction loop at `.LBB9_9`
- **Division**: Single `divl` instruction per iteration
- **Typical iterations**: 3-5 for convergence
- **Code size**: Small, fits in I-cache easily
- **Branch prediction**: Single predictable loop branch

Key loop section:
```asm
.LBB9_9:
    movl    %eax, %esi
    cmpl    %edi, %eax
    ja      .LBB9_12
    testl   %esi, %esi
    je      .LBB9_14
    movl    %edi, %eax
    xorl    %edx, %edx
    divl    %esi          # Newton step: x/y
    addl    %esi, %eax
    shrl    %eax          # (y + x/y) / 2
    cmpl    %esi, %eax
    jb      .LBB9_9       # Loop back if not converged
```

### Bit-by-Bit Non-Restoring Assembly Characteristics
- **Total lines**: 463
- **Core structure**: Fully unrolled 16 iterations
- **Per iteration**: ~10 instructions (shift, add, compare, conditional move)
- **Total instructions**: ~160 in straight line
- **Code size**: Large, may exceed I-cache line
- **Branch prediction**: 16 conditional moves, no loops

Key pattern (repeated 16 times):
```asm
    movl    %r9d, %r10d
    shll    $14, %r10d
    orl     $67108864, %r10d
    xorl    %r9d, %r9d
    cmpq    %r10, %rcx
    setae   %r9b          # Set if remainder >= delta
    cmovbq  %rax, %r10    # Conditional move
    subq    %r10, %rcx    # Update remainder
    shll    $13, %r9d
    orl     %r8d, %r9d
```

## Why Newton-Raphson Wins

### 1. Modern CPU Division is Fast
- Modern x86 CPUs have pipelined dividers
- For 32-bit division: ~10-20 cycles latency, but throughput is good
- Newton only needs 3-5 divisions total

### 2. Loop Overhead is Minimal
- Small loop body fits in µOP cache
- Branch predictor learns the pattern quickly
- Loop-stream detector may kick in

### 3. Bitwise Unrolling Hurts
- **I-cache pressure**: 160+ instructions don't fit in a cache line (64 bytes)
- **Register pressure**: Needs many registers (r8, r9, r10, rcx, etc.)
- **Dependency chains**: Each iteration depends on previous remainder value
- **No SIMD**: Can't vectorize, stuck with scalar ops

### 4. Conditional Moves Aren't Free
- 16 `cmovbq` instructions create long dependency chains
- Each depends on the previous `cmpq` and `subq`
- CPU can't execute them in parallel

## Theoretical vs Actual Performance

**Theory** (why we expected bitwise to win):
- No division: ✓
- Simple operations: ✓
- Predictable behavior: ✓

**Reality** (why Newton won):
- Division is optimized in hardware
- Small loop > large unrolled code
- Modern CPUs prefer loops they can optimize

## Conclusions

1. **Keep Newton-Raphson**: It's faster and simpler
2. **Historical context**: Bit-by-bit was faster on:
   - CPUs without hardware dividers (8-bit microcontrollers)
   - Software floating-point libraries
   - Pre-Pentium era CPUs

3. **Modern CPUs optimize differently**:
   - Hardware division is fast
   - Small loops are well-optimized
   - Large unrolled code hurts I-cache

## Test Coverage

✅ All algorithms produce bit-identical results
✅ Comprehensive test suite: 5 tests across all bit widths
✅ 100% match rate: Newton vs Bitwise on 3000+ test cases

## Recommendation

**Use Newton-Raphson (`sqrt()`) for all bit widths**. The bit-by-bit non-restoring implementation (`sqrt_bb()`) is mathematically correct but slower on modern hardware.

Keep `sqrt_bb()` as:
- Educational reference
- Verification tool
- Potential optimization for specific embedded targets without hardware division
