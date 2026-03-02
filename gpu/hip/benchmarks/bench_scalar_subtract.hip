// Spirix GPU Benchmark - ScalarF4E4 vs IEEE 754 f32
// Author: Nick Spiker fractaldecoder@proton.me
//
// Comprehensive benchmark testing:
// - Random valid ScalarF4E4 operations (normalized)
// - Edge cases: infinity, NaN, zero, huge differences
// - Single vs chained vs batch operations
// - Comparison with IEEE 754 f32 subtraction
// - Memory bandwidth vs compute throughput

#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <random>

// ScalarF4E4 definition
struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

#define AMBIGUOUS_EXPONENT ((int16_t)0x8000) // -32768
#define FRACTION_BITS 16

// External kernel
extern "C" void spirix_scalar_subtract_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

// ============================================================================
// IEEE 754 f32 Subtraction Kernel (for comparison)
// ============================================================================

__global__ void f32_subtract_kernel(
    const float *a,
    const float *b,
    float *result,
    int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        result[idx] = a[idx] - b[idx];
    }
}

void f32_subtract_gpu(const float *d_a, const float *d_b, float *d_result, int n)
{
    int threads = 256;
    int blocks = (n + threads - 1) / threads;
    hipLaunchKernelGGL(f32_subtract_kernel, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_result, n);
}

// ============================================================================
// Chained Dependency Kernel (tests latency, not throughput)
// ============================================================================

__global__ void scalar_subtract_chained_kernel(
    const ScalarF4E4 *a,
    const ScalarF4E4 *b,
    ScalarF4E4 *result,
    int n,
    int chains)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    ScalarF4E4 acc = a[idx];

    // Chain multiple subtractions: acc = acc - b - b - b ...
    for (int i = 0; i < chains; i++)
    {
        // Inline subtraction to create dependency
        // This forces sequential execution, measuring latency
        int32_t a_frac = (int32_t)acc.fraction;
        int32_t b_frac = (int32_t)b[idx].fraction;
        int32_t a_exp = (int32_t)acc.exponent;
        int32_t b_exp = (int32_t)b[idx].exponent;

        // Quick check for ambiguous
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
            // Negligible difference
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

        // Simplified subtraction (good enough for latency test)
        int32_t result_frac;
        int32_t result_exp;

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

        if (result_frac == 0)
        {
            acc.fraction = 0;
            acc.exponent = AMBIGUOUS_EXPONENT;
            continue;
        }

        // Normalize (simplified)
        int lz = __clz(result_frac);
        int lo = __clz(~result_frac);
        int leading = (lz > lo) ? lz : lo;

        result_frac = (result_frac << (leading - 1)) >> FRACTION_BITS;
        result_exp = result_exp + 17 - leading;

        acc.fraction = (int16_t)result_frac;
        acc.exponent = (int16_t)result_exp;
    }

    result[idx] = acc;
}

// ============================================================================
// Random ScalarF4E4 Generation (on CPU, then normalize)
// ============================================================================

ScalarF4E4 normalize_scalar(int16_t frac, int16_t exp)
{
    if (frac == 0 || exp == AMBIGUOUS_EXPONENT)
    {
        return {0, AMBIGUOUS_EXPONENT};
    }

    int32_t f = (int32_t)frac;
    int32_t e = (int32_t)exp;

    // Count leading same bits
    int lz = __builtin_clz((uint32_t)f);
    int lo = __builtin_clz((uint32_t)(~f));
    int leading = (lz > lo) ? lz : lo;

    // Normalize to N-1 position
    f = (f << (leading - 1)) >> FRACTION_BITS;
    e = e + 17 - leading;

    if (e <= AMBIGUOUS_EXPONENT)
    {
        return {0, AMBIGUOUS_EXPONENT};
    }

    return {(int16_t)f, (int16_t)e};
}

void generate_random_scalars(ScalarF4E4 *data, int n, unsigned int seed)
{
    std::mt19937 gen(seed);
    std::uniform_int_distribution<int16_t> frac_dist(-32768, 32767);
    std::uniform_int_distribution<int16_t> exp_dist(-30000, 30000);

    for (int i = 0; i < n; i++)
    {
        int16_t frac = frac_dist(gen);
        int16_t exp = exp_dist(gen);

        // Normalize to ensure valid ScalarF4E4
        data[i] = normalize_scalar(frac, exp);
    }
}

void generate_edge_case_scalars(ScalarF4E4 *data, int n)
{
    int idx = 0;

    // Fill with edge cases
    while (idx < n)
    {
        if (idx < n) data[idx++] = {0, AMBIGUOUS_EXPONENT};              // Ambiguous
        if (idx < n) data[idx++] = {16384, 0};                           // Normal
        if (idx < n) data[idx++] = {-16384, 0};                          // Normal negative
        if (idx < n) data[idx++] = {1, 30000};                           // Huge exponent
        if (idx < n) data[idx++] = {1, -30000};                          // Tiny exponent
        if (idx < n) data[idx++] = {32767, 100};                         // Max fraction
        if (idx < n) data[idx++] = {-32768, 100};                        // Min fraction
        if (idx < n) data[idx++] = {1, AMBIGUOUS_EXPONENT + 100};        // Near underflow
    }
}

// ============================================================================
// Benchmark Harness
// ============================================================================

struct BenchResult
{
    const char *name;
    double time_ms;
    double ops_per_sec;
    double bandwidth_gbps;
};

void print_result(const BenchResult &r)
{
    printf("%-40s | %8.3f ms | %8.2f MOPS | %6.2f GB/s\n",
           r.name, r.time_ms, r.ops_per_sec / 1e6, r.bandwidth_gbps);
}

BenchResult bench_scalar_subtract(const ScalarF4E4 *h_a, const ScalarF4E4 *h_b,
                                   int n, int iterations, const char *name)
{
    ScalarF4E4 *d_a, *d_b, *d_result;
    ScalarF4E4 *h_result = new ScalarF4E4[n];

    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Warmup
    spirix_scalar_subtract_gpu(d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    // Benchmark
    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        spirix_scalar_subtract_gpu(d_a, d_b, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    hipMemcpy(h_result, d_result, n * sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

    double total_ops = (double)n * iterations;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 3 * sizeof(ScalarF4E4); // read a, read b, write result
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    delete[] h_result;

    return {name, milliseconds, ops_per_sec, bandwidth_gbps};
}

BenchResult bench_f32_subtract(int n, int iterations)
{
    float *h_a = new float[n];
    float *h_b = new float[n];
    float *d_a, *d_b, *d_result;

    // Generate random f32 values
    std::mt19937 gen(42);
    std::uniform_real_distribution<float> dist(-1e6f, 1e6f);

    for (int i = 0; i < n; i++)
    {
        h_a[i] = dist(gen);
        h_b[i] = dist(gen);
    }

    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_b, n * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(float), hipMemcpyHostToDevice);

    // Warmup
    f32_subtract_gpu(d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    // Benchmark
    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        f32_subtract_gpu(d_a, d_b, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 3 * sizeof(float);
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;

    return {"IEEE 754 f32 subtraction", milliseconds, ops_per_sec, bandwidth_gbps};
}

// ============================================================================
// Main Benchmark Suite
// ============================================================================

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║        Spirix ScalarF4E4 GPU Benchmark - Subtraction Performance             ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n\n");

    const int sizes[] = {1024, 10000, 100000, 1000000, 10000000};
    const int iterations = 100;

    for (int size : sizes)
    {
        printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        printf("  Batch Size: %d operations × %d iterations\n", size, iterations);
        printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        printf("%-40s | %-10s | %-11s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
        printf("────────────────────────────────────────┼────────────┼─────────────┼────────────\n");

        // Generate test data
        ScalarF4E4 *h_a_random = new ScalarF4E4[size];
        ScalarF4E4 *h_b_random = new ScalarF4E4[size];
        ScalarF4E4 *h_a_edge = new ScalarF4E4[size];
        ScalarF4E4 *h_b_edge = new ScalarF4E4[size];

        generate_random_scalars(h_a_random, size, 12345);
        generate_random_scalars(h_b_random, size, 67890);
        generate_edge_case_scalars(h_a_edge, size);
        generate_edge_case_scalars(h_b_edge, size);

        // Run benchmarks
        print_result(bench_scalar_subtract(h_a_random, h_b_random, size, iterations, "ScalarF4E4 - Random normalized"));
        print_result(bench_scalar_subtract(h_a_edge, h_b_edge, size, iterations, "ScalarF4E4 - Edge cases (∞, 0, huge)"));
        print_result(bench_f32_subtract(size, iterations));

        delete[] h_a_random;
        delete[] h_b_random;
        delete[] h_a_edge;
        delete[] h_b_edge;
    }

    printf("\n╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                              Benchmark Complete                               ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n");

    return 0;
}
