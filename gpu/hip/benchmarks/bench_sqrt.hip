// Spirix GPU Benchmark - Square Root Throughput
// Compares ScalarF4E4 bitwise sqrt vs f32 hardware sqrt

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

extern "C" void spirix_scalar_sqrt_gpu(
    const ScalarF4E4 *d_a,
    ScalarF4E4 *d_result,
    int n);

// ============================================================================
// f32 comparison kernel
// ============================================================================

__global__ void f32_sqrt_kernel(
    const float *a, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        result[idx] = sqrtf(a[idx]);
    }
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
    std::uniform_int_distribution<int16_t> frac_dist(1, 32767);  // Positive only for sqrt
    std::uniform_int_distribution<int16_t> exp_dist(-100, 100);

    for (int i = 0; i < n; i++)
    {
        data[i] = normalize_scalar(frac_dist(gen), exp_dist(gen));
    }
}

// ============================================================================
// Benchmark Helpers
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
    printf("%-50s | %8.3f ms | %10.2f GOPS | %6.2f GB/s\n",
           r.name, r.time_ms, r.ops_per_sec / 1e9, r.bandwidth_gbps);
}

BenchResult bench_scalar_sqrt(
    const ScalarF4E4 *h_a,
    int n, int iterations, const char *name)
{
    ScalarF4E4 *d_a, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Warmup
    spirix_scalar_sqrt_gpu(d_a, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        spirix_scalar_sqrt_gpu(d_a, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 2 * sizeof(ScalarF4E4);  // Read input + write result
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_result);

    return {name, milliseconds, ops_per_sec, bandwidth_gbps};
}

BenchResult bench_f32_sqrt(
    int n, int iterations, const char *name)
{
    float *h_a = new float[n];

    std::mt19937 gen(42);
    std::uniform_real_distribution<float> dist(0.1f, 1000.0f);

    for (int i = 0; i < n; i++)
    {
        h_a[i] = dist(gen);
    }

    float *d_a, *d_result;
    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    // Warmup
    hipLaunchKernelGGL(f32_sqrt_kernel, dim3(blocks), dim3(threads), 0, 0, d_a, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        hipLaunchKernelGGL(f32_sqrt_kernel, dim3(blocks), dim3(threads), 0, 0, d_a, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 2 * sizeof(float);
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_result);
    delete[] h_a;

    return {name, milliseconds, ops_per_sec, bandwidth_gbps};
}

// ============================================================================
// Main
// ============================================================================

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                   Square Root Throughput Benchmark                            ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n\n");

    const int n = 1048576;  // 1M elements (sweet spot from previous benchmarks)
    const int iterations = 100;

    // Generate test data
    ScalarF4E4 *h_a = new ScalarF4E4[n];
    generate_random_scalars(h_a, n, 42);

    // Square root benchmarks
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Square Root: 1M elements × 100 iterations\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-50s | %-10s | %-12s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
    printf("──────────────────────────────────────────────────┼────────────┼──────────────┼────────────\n");

    BenchResult scalar_result = bench_scalar_sqrt(h_a, n, iterations, "ScalarF4E4 Sqrt (bitwise)");
    BenchResult f32_result = bench_f32_sqrt(n, iterations, "f32 Sqrt (hardware)");

    print_result(scalar_result);
    print_result(f32_result);

    // Comparison
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Performance Comparison\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    double speedup = f32_result.time_ms / scalar_result.time_ms;
    printf("  Speedup: %.2fx %s\n",
           speedup > 1.0 ? speedup : 1.0 / speedup,
           speedup > 1.0 ? "(ScalarF4E4 faster)" : "(f32 faster)");

    printf("  ScalarF4E4 throughput: %.2f GOPS\n", scalar_result.ops_per_sec / 1e9);
    printf("  f32 throughput:        %.2f GOPS\n", f32_result.ops_per_sec / 1e9);

    delete[] h_a;

    printf("\n╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                              Benchmark Complete                               ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n");

    return 0;
}
