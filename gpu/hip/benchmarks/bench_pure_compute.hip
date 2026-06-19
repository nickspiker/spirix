// Spirix GPU Benchmark - Pure Compute Throughput (1:1 comparison)
// Author: Nick Spiker fractaldecoder@proton.me
//
// Eliminates memory bandwidth bottleneck by:
// - Loading static 'a' value once per thread
// - Rotating thru pre-loaded 'b' array (32 elements)
// - Performing 32 consecutive subtractions: a - b[0] - b[1] - ... - b[31]
// - Storing result once
//
// Compute intensity: 32 ops per (1 load a + 32 load b + 1 store) = high ratio

#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <random>

struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

#define AMBIGUOUS_EXPONENT ((int16_t)0x8000)
#define FRACTION_BITS 16
#define ROTATION_SIZE 32  // Number of b values to rotate thru

// ============================================================================
// Pure Compute Kernels - Rotate thru pre-loaded b values
// ============================================================================

// ScalarF4E4: Load a once, subtract b[0..31] sequentially
__global__ void scalar_subtract_rotate32(
    const ScalarF4E4 *a,           // n elements
    const ScalarF4E4 *b_values,    // ROTATION_SIZE pre-computed values (shared by all)
    ScalarF4E4 *result,
    int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    // Load a once
    ScalarF4E4 acc = a[idx];

    // Rotate thru 32 b values, doing 32 subtractions
    for (int i = 0; i < ROTATION_SIZE; i++)
    {
        ScalarF4E4 b = b_values[i];

        // Inline subtraction (same as main kernel)
        int32_t a_frac = (int32_t)acc.fraction;
        int32_t b_frac = (int32_t)b.fraction;
        int32_t a_exp = (int32_t)acc.exponent;
        int32_t b_exp = (int32_t)b.exponent;

        if (a_exp == AMBIGUOUS_EXPONENT || b_exp == AMBIGUOUS_EXPONENT)
        {
            acc.fraction = 0;
            acc.exponent = AMBIGUOUS_EXPONENT;
            continue;
        }

        int32_t exp_diff = a_exp - b_exp;
        int32_t abs_diff = (exp_diff < 0) ? -exp_diff : exp_diff;

        if (abs_diff >= FRACTION_BITS)
        {
            if (exp_diff >= 0)
            {
                // a >> b, b negligible
                acc.fraction = (int16_t)a_frac;
                acc.exponent = (int16_t)a_exp;
            }
            else
            {
                // b >> a, a negligible
                acc.fraction = (int16_t)(-b_frac);
                acc.exponent = (int16_t)b_exp;
            }
            continue;
        }

        int32_t result_frac, result_exp;
        if (exp_diff >= 0)
        {
            result_frac = (a_frac << abs_diff) - b_frac;
            result_exp = b_exp;
        }
        else
        {
            result_frac = a_frac - (b_frac << abs_diff);
            result_exp = a_exp;
        }

        // Sign-extend
        result_frac = (result_frac << 16) >> 16;

        if (result_frac == 0)
        {
            acc.fraction = 0;
            acc.exponent = AMBIGUOUS_EXPONENT;
            continue;
        }

        // Count leading same bits
        int lz = __clz(result_frac);
        int lo = __clz(~result_frac);
        int leading = (lz > lo) ? lz : lo;

        // Calculate exponent
        result_exp = result_exp + 17 - leading;

        if (result_exp <= AMBIGUOUS_EXPONENT)
        {
            acc.fraction = (int16_t)((result_frac << (leading - 2)) >> 16);
            acc.exponent = AMBIGUOUS_EXPONENT;
            continue;
        }

        // Normalize
        acc.fraction = (int16_t)((result_frac << (leading - 1)) >> 16);
        acc.exponent = (int16_t)result_exp;
    }

    result[idx] = acc;
}

// f32: Load a once, subtract b[0..31] sequentially
__global__ void f32_subtract_rotate32(
    const float *a,
    const float *b_values,
    float *result,
    int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    float acc = a[idx];

    // Rotate thru 32 b values
    for (int i = 0; i < ROTATION_SIZE; i++)
    {
        acc = acc - b_values[i];
    }

    result[idx] = acc;
}

// ============================================================================
// Unrolled versions for even higher compute intensity
// ============================================================================

// ScalarF4E4: 128 operations per element
__global__ void scalar_subtract_rotate128(
    const ScalarF4E4 *a,
    const ScalarF4E4 *b_values,  // 128 pre-computed values
    ScalarF4E4 *result,
    int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    ScalarF4E4 acc = a[idx];

    // Unroll 4x (32 × 4 = 128 operations)
    for (int unroll = 0; unroll < 4; unroll++)
    {
        for (int i = 0; i < 32; i++)
        {
            ScalarF4E4 b = b_values[unroll * 32 + i];

            int32_t a_frac = (int32_t)acc.fraction;
            int32_t b_frac = (int32_t)b.fraction;
            int32_t a_exp = (int32_t)acc.exponent;
            int32_t b_exp = (int32_t)b.exponent;

            if (a_exp == AMBIGUOUS_EXPONENT || b_exp == AMBIGUOUS_EXPONENT)
            {
                acc.fraction = 0;
                acc.exponent = AMBIGUOUS_EXPONENT;
                continue;
            }

            int32_t exp_diff = a_exp - b_exp;
            int32_t abs_diff = (exp_diff < 0) ? -exp_diff : exp_diff;

            if (abs_diff >= FRACTION_BITS)
            {
                if (exp_diff >= 0)
                {
                    acc.fraction = (int16_t)a_frac;
                    acc.exponent = (int16_t)a_exp;
                }
                else
                {
                    acc.fraction = (int16_t)(-b_frac);
                    acc.exponent = (int16_t)b_exp;
                }
                continue;
            }

            int32_t result_frac, result_exp;
            if (exp_diff >= 0)
            {
                result_frac = (a_frac << abs_diff) - b_frac;
                result_exp = b_exp;
            }
            else
            {
                result_frac = a_frac - (b_frac << abs_diff);
                result_exp = a_exp;
            }

            result_frac = (result_frac << 16) >> 16;

            if (result_frac == 0)
            {
                acc.fraction = 0;
                acc.exponent = AMBIGUOUS_EXPONENT;
                continue;
            }

            int lz = __clz(result_frac);
            int lo = __clz(~result_frac);
            int leading = (lz > lo) ? lz : lo;

            result_exp = result_exp + 17 - leading;

            if (result_exp <= AMBIGUOUS_EXPONENT)
            {
                acc.fraction = (int16_t)((result_frac << (leading - 2)) >> 16);
                acc.exponent = AMBIGUOUS_EXPONENT;
                continue;
            }

            acc.fraction = (int16_t)((result_frac << (leading - 1)) >> 16);
            acc.exponent = (int16_t)result_exp;
        }
    }

    result[idx] = acc;
}

// f32: 128 operations per element
__global__ void f32_subtract_rotate128(
    const float *a,
    const float *b_values,
    float *result,
    int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    float acc = a[idx];

    // 128 operations
    for (int i = 0; i < 128; i++)
    {
        acc = acc - b_values[i];
    }

    result[idx] = acc;
}

// ============================================================================
// Test Data Generation
// ============================================================================

ScalarF4E4 normalize_scalar(int16_t frac, int16_t exp)
{
    if (frac == 0 || exp == AMBIGUOUS_EXPONENT)
        return {0, AMBIGUOUS_EXPONENT};

    int32_t f = (int32_t)frac;
    int32_t e = (int32_t)exp;

    int lz = __builtin_clz((uint32_t)f);
    int lo = __builtin_clz((uint32_t)(~f));
    int leading = (lz > lo) ? lz : lo;

    f = (f << (leading - 1)) >> FRACTION_BITS;
    e = e + 17 - leading;

    if (e <= AMBIGUOUS_EXPONENT)
        return {0, AMBIGUOUS_EXPONENT};

    return {(int16_t)f, (int16_t)e};
}

void generate_random_scalars(ScalarF4E4 *data, int n, unsigned int seed)
{
    std::mt19937 gen(seed);
    std::uniform_int_distribution<int16_t> frac_dist(-32768, 32767);
    std::uniform_int_distribution<int16_t> exp_dist(-30000, 30000);

    for (int i = 0; i < n; i++)
    {
        data[i] = normalize_scalar(frac_dist(gen), exp_dist(gen));
    }
}

// ============================================================================
// Benchmark
// ============================================================================

struct BenchResult
{
    const char *name;
    int ops_per_element;
    double time_ms;
    double total_ops;
    double ops_per_sec;
    double bandwidth_gbps;
};

void print_result(const BenchResult &r)
{
    printf("%-40s | %4d ops/elem | %8.3f ms | %10.2f GOPS | %6.2f GB/s\n",
           r.name, r.ops_per_element, r.time_ms, r.ops_per_sec / 1e9, r.bandwidth_gbps);
}

BenchResult bench_scalar_rotate(int n, int iterations, int rotation_size)
{
    ScalarF4E4 *h_a = new ScalarF4E4[n];
    ScalarF4E4 *h_b = new ScalarF4E4[rotation_size];

    generate_random_scalars(h_a, n, 42);
    generate_random_scalars(h_b, rotation_size, 43);

    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, rotation_size * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, rotation_size * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    // Warmup
    if (rotation_size == 32)
        hipLaunchKernelGGL(scalar_subtract_rotate32, dim3(blocks), dim3(threads), 0, 0,
                           d_a, d_b, d_result, n);
    else
        hipLaunchKernelGGL(scalar_subtract_rotate128, dim3(blocks), dim3(threads), 0, 0,
                           d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        if (rotation_size == 32)
            hipLaunchKernelGGL(scalar_subtract_rotate32, dim3(blocks), dim3(threads), 0, 0,
                               d_a, d_b, d_result, n);
        else
            hipLaunchKernelGGL(scalar_subtract_rotate128, dim3(blocks), dim3(threads), 0, 0,
                               d_a, d_b, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations * rotation_size;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;

    // Amortized bandwidth: 1 read a + rotation_size reads b + 1 write result, divided by ops
    double bytes_per_op = (sizeof(ScalarF4E4) * (1 + rotation_size + 1)) / (double)rotation_size;
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;

    char name[64];
    snprintf(name, sizeof(name), "ScalarF4E4 - Rotate %d", rotation_size);
    return {name, rotation_size, milliseconds, total_ops, ops_per_sec, bandwidth_gbps};
}

BenchResult bench_f32_rotate(int n, int iterations, int rotation_size)
{
    float *h_a = new float[n];
    float *h_b = new float[rotation_size];

    std::mt19937 gen(42);
    std::uniform_real_distribution<float> dist(-1e6f, 1e6f);
    for (int i = 0; i < n; i++)
        h_a[i] = dist(gen);
    for (int i = 0; i < rotation_size; i++)
        h_b[i] = dist(gen);

    float *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_b, rotation_size * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, rotation_size * sizeof(float), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    if (rotation_size == 32)
        hipLaunchKernelGGL(f32_subtract_rotate32, dim3(blocks), dim3(threads), 0, 0,
                           d_a, d_b, d_result, n);
    else
        hipLaunchKernelGGL(f32_subtract_rotate128, dim3(blocks), dim3(threads), 0, 0,
                           d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        if (rotation_size == 32)
            hipLaunchKernelGGL(f32_subtract_rotate32, dim3(blocks), dim3(threads), 0, 0,
                               d_a, d_b, d_result, n);
        else
            hipLaunchKernelGGL(f32_subtract_rotate128, dim3(blocks), dim3(threads), 0, 0,
                               d_a, d_b, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations * rotation_size;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = (sizeof(float) * (1 + rotation_size + 1)) / (double)rotation_size;
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;

    char name[64];
    snprintf(name, sizeof(name), "f32 - Rotate %d", rotation_size);
    return {name, rotation_size, milliseconds, total_ops, ops_per_sec, bandwidth_gbps};
}

// ============================================================================
// Main
// ============================================================================

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                    Pure Compute Throughput Benchmark                         ║\n");
    printf("║                  (Minimal memory bandwidth overhead)                         ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n\n");

    printf("Methodology:\n");
    printf("  • Load 'a' once per element\n");
    printf("  • Rotate thru N pre-loaded 'b' values\n");
    printf("  • Perform N consecutive subtractions: a - b[0] - b[1] - ... - b[N-1]\n");
    printf("  • Store result once\n");
    printf("  • High compute intensity = N ops per (1 + N + 1) memory accesses\n\n");

    const int n = 1048576;  // 1M elements (sweet spot from previous test)
    const int iterations = 100;

    // Test 1: 32 operations per element
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Test 1: 32 operations per element (1M elements × 100 iterations)\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-40s | %-12s | %-10s | %-12s | %-11s\n", "Test", "Ops/Elem", "Time", "Throughput", "Bandwidth");
    printf("────────────────────────────────────────┼──────────────┼────────────┼──────────────┼────────────\n");
    print_result(bench_scalar_rotate(n, iterations, 32));
    print_result(bench_f32_rotate(n, iterations, 32));

    // Test 2: 128 operations per element
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Test 2: 128 operations per element (1M elements × 100 iterations)\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-40s | %-12s | %-10s | %-12s | %-11s\n", "Test", "Ops/Elem", "Time", "Throughput", "Bandwidth");
    printf("────────────────────────────────────────┼──────────────┼────────────┼──────────────┼────────────\n");
    print_result(bench_scalar_rotate(n, iterations, 128));
    print_result(bench_f32_rotate(n, iterations, 128));

    // Compute theoretical peak
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  AMD RX 6800 Theoretical Peak (RDNA2 @ 2.1 GHz)\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  60 CUs × 64 ALUs/CU × 2.1 GHz = 8,064 GFLOPS (f32 peak)\n");
    printf("  60 CUs × 32 threads × 16 waves × 2.1 GHz = theoretical max throughput\n\n");

    printf("╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                              Benchmark Complete                               ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n");

    return 0;
}
