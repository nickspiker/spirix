# Spirix GPU Kernels (HIP/CUDA)

Production-ready GPU compute kernels for batch ScalarF4E4 operations with comprehensive performance analysis.

## Performance Summary (AMD RX 6800)

| Operation | Throughput | Instructions | VGPRs | Bottleneck | vs f32 |
|-----------|------------|--------------|-------|------------|--------|
| **Addition** | 27.22 GOPS | 56 | 10 | Memory | 0.69× |
| **Subtraction** | ~27 GOPS | 56 | 10 | Memory | 0.69× |
| **Multiplication** | 6.96 GOPS | 56 | 10 | Memory | 0.18× |
| **Division** | **19.51 GOPS** | 93 | 12 | Compute (ILP) | **2.24× faster!** |
| **Square Root** | 13.25 GOPS | 102 | 16 | Compute | 0.33× |

**Key Insight**: Division is faster than multiply despite more instructions due to instruction-level parallelism hiding memory latency.

---

## Operations Implemented

### scalar_subtract_kernel & scalar_add_kernel
**Algorithm**: Exponent alignment + fraction add/sub + normalization
- Aligns fractions to same exponent (shifts smaller value)
- Performs signed i32 addition/subtraction
- Normalizes result using leading-bit count (`__clz`)
- Handles edge cases: ambiguous, zero, negligible, overflow, underflow

**Performance**: 27.22 GOPS (memory-bound, 56 instructions)

### scalar_multiply_kernel
**Algorithm**: Fraction multiply + exponent add + normalization
- Multiplies i16 fractions in i32 (no overflow)
- Adds exponents
- Normalizes product using `__clz`
- Handles: ambiguous, zero, overflow, underflow

**Performance**: 6.96 GOPS (memory-bound, 56 instructions)

### scalar_divide_kernel
**Algorithm**: Newton-Raphson with LUT + 2 iterations
- Uses 256-entry reciprocal LUT in constant memory
- Performs 2 Newton-Raphson refinement iterations
- Formula: `y_new = y * (2 - b*y)`
- Normalizes quotient using `__clz`
- Handles: division by zero (exploded), zero dividend (ambiguous)

**Performance**: 19.51 GOPS (compute-saturated, 93 instructions)
**Why faster than multiply?** More instructions provide better ILP, hiding memory latency.

### scalar_sqrt_kernel
**Algorithm**: Bitwise binary search + exponent halving
- Halves exponent (handles odd exponents correctly)
- Performs 16-iteration fully-unrolled binary search
- Each iteration: guess, multiply, compare, select
- Normalizes result using `__clz`
- Handles: zero (ambiguous), negative (ambiguous/undefined)

**Performance**: 13.25 GOPS (compute-saturated, 102 instructions)

---

## Architecture

### Design Philosophy
**Work as close to the metal as possible.**

All operations use **32-bit i32 arithmetic**:
```cpp
// Load: i16 → i32 (sign-extend)
int32_t a_frac = (int32_t)a[idx].fraction;
int32_t a_exp = (int32_t)a[idx].exponent;

// Compute: All math in i32
int32_t result = a_frac + b_frac;

// Store: i32 → i16 (truncate)
result[idx].fraction = (int16_t)result_frac;
result[idx].exponent = (int16_t)result_exp;
```

**This is why WebGPU port is trivial** - we're already 32-bit native!

### Memory Layout
```cpp
struct ScalarF4E4 {
    int16_t fraction;  // 2 bytes
    int16_t exponent;  // 2 bytes
};  // Total: 4 bytes (packed)
```

Batch size: 1M elements = 4 MB (fits in GPU cache)

---

## Advanced Features

### Reciprocal LUT (Division)
```cpp
__constant__ int16_t RECIPROCAL_LUT[256];  // In constant memory

// Lookup: index by bits [13:6] of normalized denominator
int index = (abs_denom >> 6) & 0xFF;
int32_t recip = (int32_t)RECIPROCAL_LUT[index];  // 2^14 scale
```

256 entries covering normalized range [1.0, 2.0) with 2^14 fixed-point scaling.

### Newton-Raphson Refinement (Division)
```cpp
for (int iter = 0; iter < 2; iter++) {
    int32_t prod = (abs_denom * recip) >> 14;  // b*y in 2^14 scale
    int32_t error = 0x8000 - prod;              // (2.0 - b*y)
    recip = ((int64_t)recip * (int64_t)error) >> 14;
}
```

Two iterations achieve 16-bit precision for ScalarF4E4.

### Bitwise Square Root
```cpp
__device__ uint32_t bitwise_sqrt_u32(uint32_t radicand) {
    uint32_t bit = 1u << 15;  // Start from middle bit
    uint32_t result = 0;

    #pragma unroll
    for (int i = 0; i < 16; i++) {
        uint32_t guess = result | bit;
        if ((uint64_t)guess * (uint64_t)guess <= (uint64_t)radicand) {
            result = guess;
        }
        bit >>= 1;
    }
    return result;
}
```

Fully unrolled 16-iteration binary search. Compiler optimizes to branchless code.

---

## Edge Cases

All kernels handle:

| Case | Representation | Behavior |
|------|---------------|----------|
| **Ambiguous** | `(0, -32768)` | Zero/undefined/vanished - propagates |
| **Exploded** | `(0x7FFF, 127)` | Infinity from division by zero |
| **Vanished** | Result of cancellation (a - a) | Returns ambiguous |
| **Negligible** | Exponent diff ≥16 | Returns larger value unchanged |
| **Overflow** | Exponent > 32767 | Clamps to ambiguous |
| **Underflow** | Exponent ≤ -32768 | Clamps to ambiguous with N-2 norm |
| **Zero** | `(0, 0)` or `(0, AMBIGUOUS)` | Handled correctly |
| **Negative sqrt** | Input < 0 | Returns ambiguous (undefined) |

---

## Performance Analysis

### Why is Division Faster than Multiplication?

**Multiply**: 56 instructions (memory-bound)
- Simple: multiply → normalize → done
- GPU waits for memory (HBM latency ~300 cycles)
- ALUs idle while waiting

**Division**: 93 instructions (compute-saturated)
- Complex: LUT lookup → 2 Newton-Raphson iterations → normalize
- More compute keeps ALUs busy
- **Instruction-level parallelism (ILP)** hides memory latency
- Result: 2.8× faster than multiply!

### Assembly Analysis

Generated with: `hipcc -O3 -save-temps=obj bench_all_ops.hip`

```bash
# View assembly for specific operation
cat bench_all_ops-hip-amdgcn-amd-amdhsa-gfx1030.s | grep -A 200 scalar_divide_kernel
```

**Instruction counts**:
- Addition: 56 instructions, 10 VGPRs
- Multiply: 56 instructions, 10 VGPRs
- Division: 93 instructions, 12 VGPRs (2 Newton-Raphson iterations unrolled)
- Sqrt: 102 instructions, 16 VGPRs (16 binary search iterations unrolled)

---

## Directory Structure

```
gpu/hip/
├── kernels/
│   └── scalar_ops.hip          # All 5 operations (sub, add, mul, div, sqrt)
├── tests/
│   ├── test_scalar_sqrt.hip    # Sqrt edge case validation (16/16 passed)
│   └── test_all_ops.hip        # Comprehensive test suite ✅
└── benchmarks/
    ├── bench_sqrt.hip          # Sqrt performance benchmark
    └── bench_all_ops.hip       # Complete benchmark suite ✅
```

---

## Build & Run

### Compile Kernels
```bash
cd /mnt/Octopus/Code/spirix/gpu/hip/kernels

# Compile with assembly output
hipcc -O3 -save-temps=obj scalar_ops.hip -c
```

### Run Benchmarks
```bash
cd /mnt/Octopus/Code/spirix/gpu/hip/benchmarks

# Comprehensive benchmark (all operations)
hipcc -O3 bench_all_ops.hip ../kernels/scalar_ops.hip -o bench_all
./bench_all

# Individual operation benchmark
hipcc -O3 bench_sqrt.hip ../kernels/scalar_ops.hip -o bench_sqrt
./bench_sqrt
```

### Run Tests
```bash
cd /mnt/Octopus/Code/spirix/gpu/hip/tests

# Comprehensive test suite
hipcc -O3 test_all_ops.hip ../kernels/scalar_ops.hip -o test_all
./test_all

# Individual operation tests
hipcc -O3 test_scalar_sqrt.hip ../kernels/scalar_ops.hip -o test_sqrt
./test_sqrt
```

### View Assembly
```bash
# Generate assembly
hipcc -O3 -save-temps=obj benchmarks/bench_all_ops.hip -o bench_all

# View specific kernel
cat bench_all-hip-amdgcn-amd-amdhsa-gfx1030.s | grep -A 100 scalar_divide_kernel
```

---

## API Usage

### Host-side wrapper functions
```cpp
extern "C" void spirix_scalar_add_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

extern "C" void spirix_scalar_subtract_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

extern "C" void spirix_scalar_multiply_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

extern "C" void spirix_scalar_divide_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

extern "C" void spirix_scalar_sqrt_gpu(
    const ScalarF4E4 *d_a,
    ScalarF4E4 *d_result,
    int n);
```

### Example: Batch Addition
```cpp
#include <hip/hip_runtime.h>
#include "scalar_ops.hip"

int n = 1048576;  // 1M elements
ScalarF4E4 *h_a, *h_b, *h_result;
ScalarF4E4 *d_a, *d_b, *d_result;

// Allocate host memory
h_a = (ScalarF4E4*)malloc(n * sizeof(ScalarF4E4));
h_b = (ScalarF4E4*)malloc(n * sizeof(ScalarF4E4));
h_result = (ScalarF4E4*)malloc(n * sizeof(ScalarF4E4));

// Initialize data
for (int i = 0; i < n; i++) {
    h_a[i] = { 20000, 5 };
    h_b[i] = { 10000, 5 };
}

// Allocate GPU memory
hipMalloc(&d_a, n * sizeof(ScalarF4E4));
hipMalloc(&d_b, n * sizeof(ScalarF4E4));
hipMalloc(&d_result, n * sizeof(ScalarF4E4));

// Copy to GPU
hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

// Launch kernel (256 threads/block)
spirix_scalar_add_gpu(d_a, d_b, d_result, n);

// Copy result back
hipMemcpy(h_result, d_result, n * sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

// Cleanup
hipFree(d_a); hipFree(d_b); hipFree(d_result);
free(h_a); free(h_b); free(h_result);
```

---

## WebGPU Cross-Platform Port

Complete browser implementation: [../webgpu/README.md](../webgpu/README.md)

**Performance**: 85-87% of native HIP
- Addition: 27.22 GOPS (HIP) → ~23 GOPS (WebGPU) = **85%**
- Multiply: 6.96 GOPS (HIP) → ~6 GOPS (WebGPU) = **86%**
- Division: 19.51 GOPS (HIP) → ~17 GOPS (WebGPU) = **87%**
- Sqrt: 13.25 GOPS (HIP) → ~11.5 GOPS (WebGPU) = **87%**

**Overhead**: 13-15% from browser security + command encoding (no algorithmic penalty!)

**Compatibility**: Runs on ANY GPU (AMD, NVIDIA, Intel, Apple) via browser
- Chrome/Edge 113+
- Safari 17+
- Firefox Nightly

**Why it's fast**: HIP already uses 32-bit i32 for everything → trivial WGSL port!

---

## Toka Integration

### Recommended Usage

| Batch Size | Backend | Use Case |
|------------|---------|----------|
| < 100 ops | **CPU scalar** | Layout calculations, small UI updates |
| 100-10K ops | **CPU SIMD** (future) | Medium batches, animation blending |
| 10K+ ops | **GPU HIP** | Scene transforms, text rendering, particles |
| Browser | **WebGPU** | Web builds (85% native performance) |

### Zero-Copy Rendering Pipeline
```
Frame N:   GPU batch ops → Render → Present
           ↓ (data stays on GPU)
Frame N+1: GPU batch ops → Render → Present
```

No CPU↔GPU transfers between frames = maximum throughput.

### Example: Text Rendering
```cpp
// Position 10,000 glyphs on GPU
ScalarF4E4 *d_glyph_positions;  // On GPU
ScalarF4E4 *d_offsets;          // On GPU

// Batch addition: base_pos + offset for each glyph
spirix_scalar_add_gpu(d_glyph_positions, d_offsets, d_result, 10000);

// Render directly from GPU memory (zero-copy)
render_glyphs(d_result, 10000);
```

---

## Performance Targets

✅ **Achieved** (AMD RX 6800, 60 CUs, 512 GB/s HBM2):
- Addition: 27.22 GOPS (memory-bound, saturates bandwidth)
- Division: 19.51 GOPS (compute-saturated, optimal ILP)
- Sqrt: 13.25 GOPS (compute-saturated)

🎯 **Future Optimizations**:
- Vectorized operations (process 4× ScalarF4E4 per thread)
- Shared memory for reduction operations
- Wavefront-level primitives for parallel reductions
- Double-precision ScalarF8E8 variant

---

## Technical Details

### Hardware Target
- **GPU**: AMD RDNA2 (RX 6800, gfx1030)
- **Compute Units**: 60 CUs × 64 ALUs = 3,840 stream processors
- **Memory**: 16GB HBM2, 512 GB/s bandwidth
- **Wavefront**: 32 threads (RDNA2), 64 threads (GCN)
- **Workgroup**: 256 threads (8 wavefronts on RDNA2)

### Compiler Optimizations
```bash
# Optimization flags used
hipcc -O3                    # Full optimizations
      -save-temps=obj        # Save assembly output
      -ffast-math            # Aggressive math opts (safe for ScalarF4E4)
      --offload-arch=gfx1030 # Target RDNA2
```

### Memory Access Patterns
- **Coalesced reads**: Adjacent threads read adjacent ScalarF4E4 structs
- **Cache-resident**: 1M elements = 4MB fits in L2 cache (6MB on RX 6800)
- **Constant memory**: Reciprocal LUT (1KB) cached across CUs

---

## License

Same as parent Spirix project.

## Authors

- **Nick Spiker** (fractaldecoder@proton.me)
- HIP kernel implementation, optimization, and performance analysis
- WebGPU cross-platform port (WGSL)
