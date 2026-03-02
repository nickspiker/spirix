// Spirix GPU Benchmark - Compute vs Memory Bandwidth Analysis
// Author: Nick Spiker fractaldecoder@proton.me
//
// Isolates compute performance from memory bandwidth to understand bottlenecks:
// 1. Cache-resident (L2): Small working set, multiple iterations
// 2. Compute-intensive: Multiple chained operations per element
// 3. Memory-intensive: Single operation, large dataset
// 4. Hybrid: Various compute/memory ratios

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

extern "C" void spirix_scalar_subtract_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

// ============================================================================
// IEEE 754 f32 Kernels
// ============================================================================

// Variant 1: Single subtraction (baseline - memory bound)
__global__ void f32_subtract_single(
    const float *a, const float *b, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        result[idx] = a[idx] - b[idx];
    }
}

// Variant 2: Chained subtractions (compute intensive)
// Computes: result = a - b - c - d - e (5 dependent operations)
__global__ void f32_subtract_chain5(
    const float *a, const float *b, const float *c,
    const float *d, const float *e, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        float tmp = a[idx];
        tmp = tmp - b[idx];  // Op 1
        tmp = tmp - c[idx];  // Op 2
        tmp = tmp - d[idx];  // Op 3
        tmp = tmp - e[idx];  // Op 4
        result[idx] = tmp;   // Op 5 (implicit)
    }
}

// Variant 3: Independent subtractions (compute intensive, no dependencies)
// Computes 5 independent subtractions per thread
__global__ void f32_subtract_batch5(
    const float *a, const float *b, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        // Unroll 5 independent operations
        float r0 = a[idx] - b[idx];
        float r1 = a[idx] - b[idx];
        float r2 = a[idx] - b[idx];
        float r3 = a[idx] - b[idx];
        float r4 = a[idx] - b[idx];

        // Combine to prevent optimization
        result[idx] = r0 + r1 + r2 + r3 + r4;
    }
}

// ============================================================================
// ScalarF4E4 Compute-Intensive Kernel
// ============================================================================

// Chained ScalarF4E4 subtractions (measures latency + compute density)
__global__ void scalar_subtract_chain5(
    const ScalarF4E4 *a, const ScalarF4E4 *b, const ScalarF4E4 *c,
    const ScalarF4E4 *d, const ScalarF4E4 *e, ScalarF4E4 *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    ScalarF4E4 acc = a[idx];

    // Chain 4 subtractions: acc = a - b - c - d - e
    // This creates data dependencies, measuring compute latency
    for (int op = 0; op < 4; op++)
    {
        const ScalarF4E4 *operand;
        if (op == 0) operand = &b[idx];
        else if (op == 1) operand = &c[idx];
        else if (op == 2) operand = &d[idx];
        else operand = &e[idx];

        // Inline subtraction (same as main kernel)
        int32_t a_frac = (int32_t)acc.fraction;
        int32_t b_frac = (int32_t)operand->fraction;
        int32_t a_exp = (int32_t)acc.exponent;
        int32_t b_exp = (int32_t)operand->exponent;

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
// Benchmark Variants
// ============================================================================

struct BenchResult
{
    const char *name;
    double time_ms;
    double ops_per_sec;
    double effective_ops;  // Actual compute operations (may be > n for chained)
    double bandwidth_gbps;
};

void print_result(const BenchResult &r)
{
    printf("%-50s | %8.3f ms | %10.2f GOPS | %6.2f GB/s\n",
           r.name, r.time_ms, r.ops_per_sec / 1e9, r.bandwidth_gbps);
}

// Test 1: Cache-resident (L2 = 4MB, fit ~1M ScalarF4E4)
// Iterate multiple times over same small dataset
BenchResult bench_cache_resident_scalar(int n, int iterations)
{
    ScalarF4E4 *h_a = new ScalarF4E4[n];
    ScalarF4E4 *h_b = new ScalarF4E4[n];

    generate_random_scalars(h_a, n, 42);
    generate_random_scalars(h_b, n, 43);

    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Warmup
    spirix_scalar_subtract_gpu(d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    // Benchmark: iterate over same data many times (stays in cache)
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

    double total_ops = (double)n * iterations;
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 3 * sizeof(ScalarF4E4);
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;

    return {"ScalarF4E4 - Cache resident (L2)", milliseconds, ops_per_sec, total_ops, bandwidth_gbps};
}

BenchResult bench_cache_resident_f32(int n, int iterations)
{
    float *h_a = new float[n];
    float *h_b = new float[n];

    std::mt19937 gen(42);
    std::uniform_real_distribution<float> dist(-1e6f, 1e6f);
    for (int i = 0; i < n; i++)
    {
        h_a[i] = dist(gen);
        h_b[i] = dist(gen);
    }

    float *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_b, n * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(float), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    hipLaunchKernelGGL(f32_subtract_single, dim3(blocks), dim3(threads), 0, 0, d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        hipLaunchKernelGGL(f32_subtract_single, dim3(blocks), dim3(threads), 0, 0, d_a, d_b, d_result, n);
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

    return {"f32 - Cache resident (L2)", milliseconds, ops_per_sec, total_ops, bandwidth_gbps};
}

// Test 2: Compute-intensive (chained operations)
BenchResult bench_compute_intensive_scalar(int n, int iterations)
{
    ScalarF4E4 *h_a = new ScalarF4E4[n];
    ScalarF4E4 *h_b = new ScalarF4E4[n];
    ScalarF4E4 *h_c = new ScalarF4E4[n];
    ScalarF4E4 *h_d = new ScalarF4E4[n];
    ScalarF4E4 *h_e = new ScalarF4E4[n];

    generate_random_scalars(h_a, n, 42);
    generate_random_scalars(h_b, n, 43);
    generate_random_scalars(h_c, n, 44);
    generate_random_scalars(h_d, n, 45);
    generate_random_scalars(h_e, n, 46);

    ScalarF4E4 *d_a, *d_b, *d_c, *d_d, *d_e, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, n * sizeof(ScalarF4E4));
    hipMalloc(&d_c, n * sizeof(ScalarF4E4));
    hipMalloc(&d_d, n * sizeof(ScalarF4E4));
    hipMalloc(&d_e, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_c, h_c, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_d, h_d, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_e, h_e, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    hipLaunchKernelGGL(scalar_subtract_chain5, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_c, d_d, d_e, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        hipLaunchKernelGGL(scalar_subtract_chain5, dim3(blocks), dim3(threads), 0, 0,
                           d_a, d_b, d_c, d_d, d_e, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations * 4;  // 4 subtractions per element
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 6 * sizeof(ScalarF4E4) / 4;  // 6 reads, 1 write, 4 ops
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_c);
    hipFree(d_d);
    hipFree(d_e);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;
    delete[] h_c;
    delete[] h_d;
    delete[] h_e;

    return {"ScalarF4E4 - Compute intensive (5× chain)", milliseconds, ops_per_sec, total_ops, bandwidth_gbps};
}

BenchResult bench_compute_intensive_f32(int n, int iterations)
{
    float *h_a = new float[n];
    float *h_b = new float[n];
    float *h_c = new float[n];
    float *h_d = new float[n];
    float *h_e = new float[n];

    std::mt19937 gen(42);
    std::uniform_real_distribution<float> dist(-1e6f, 1e6f);
    for (int i = 0; i < n; i++)
    {
        h_a[i] = dist(gen);
        h_b[i] = dist(gen);
        h_c[i] = dist(gen);
        h_d[i] = dist(gen);
        h_e[i] = dist(gen);
    }

    float *d_a, *d_b, *d_c, *d_d, *d_e, *d_result;
    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_b, n * sizeof(float));
    hipMalloc(&d_c, n * sizeof(float));
    hipMalloc(&d_d, n * sizeof(float));
    hipMalloc(&d_e, n * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_c, h_c, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_d, h_d, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_e, h_e, n * sizeof(float), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    hipLaunchKernelGGL(f32_subtract_chain5, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_c, d_d, d_e, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        hipLaunchKernelGGL(f32_subtract_chain5, dim3(blocks), dim3(threads), 0, 0,
                           d_a, d_b, d_c, d_d, d_e, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    double total_ops = (double)n * iterations * 4;  // 4 subtractions
    double time_sec = milliseconds / 1000.0;
    double ops_per_sec = total_ops / time_sec;
    double bytes_per_op = 6 * sizeof(float) / 4;
    double bandwidth_gbps = (total_ops * bytes_per_op) / (time_sec * 1e9);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_c);
    hipFree(d_d);
    hipFree(d_e);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;
    delete[] h_c;
    delete[] h_d;
    delete[] h_e;

    return {"f32 - Compute intensive (5× chain)", milliseconds, ops_per_sec, total_ops, bandwidth_gbps};
}

// ============================================================================
// Main
// ============================================================================

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║              Compute vs Memory Bandwidth Bottleneck Analysis                 ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n\n");

    // Test 1: Cache-resident (128KB working set, fits in L2)
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Test 1: Cache Resident (32K elements × 1000 iterations)\n");
    printf("  Working set: 128KB (fits in L2 cache, minimizes memory bandwidth)\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-50s | %-10s | %-12s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
    printf("──────────────────────────────────────────────────┼────────────┼──────────────┼────────────\n");
    print_result(bench_cache_resident_scalar(32 * 1024, 1000));
    print_result(bench_cache_resident_f32(32 * 1024, 1000));

    // Test 2: Compute-intensive (chained operations)
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Test 2: Compute Intensive (100K elements × 100 iterations, 4 ops/element)\n");
    printf("  Chained dependencies: a - b - c - d - e (measures compute latency)\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-50s | %-10s | %-12s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
    printf("──────────────────────────────────────────────────┼────────────┼──────────────┼────────────\n");
    print_result(bench_compute_intensive_scalar(100000, 100));
    print_result(bench_compute_intensive_f32(100000, 100));

    // Test 3: Vary batch size to find transition point
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Test 3: Batch Size Sweep (find memory→compute transition)\n");
    printf("  Single subtraction per element, varying batch sizes\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-50s | %-10s | %-12s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
    printf("──────────────────────────────────────────────────┼────────────┼──────────────┼────────────\n");

    const int sizes[] = {1024, 4096, 16384, 65536, 262144, 1048576, 4194304};
    for (int size : sizes)
    {
        int iters = 100;
        char name[64];
        snprintf(name, sizeof(name), "ScalarF4E4 - %7d elements", size);

        BenchResult r = bench_cache_resident_scalar(size, iters);
        r.name = name;
        print_result(r);
    }

    printf("\n╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                              Analysis Complete                                ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n");

    return 0;
}
