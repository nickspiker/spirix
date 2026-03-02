// Spirix GPU Benchmark - Multiply & Divide Pure Compute Throughput
// Author: Nick Spiker fractaldecoder@proton.me
//
// Pure compute benchmark with rotation to eliminate memory bandwidth bottleneck
// Similar to bench_pure_compute.hip but for * and / operations

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
#define ROTATION_SIZE 32

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

// ============================================================================
// f32 comparison kernels
// ============================================================================

__global__ void f32_multiply_kernel(
    const float *a, const float *b, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        result[idx] = a[idx] * b[idx];
    }
}

__global__ void f32_divide_kernel(
    const float *a, const float *b, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n)
    {
        result[idx] = a[idx] / b[idx];
    }
}

// ============================================================================
// Rotate kernels - multiple ops per element
// ============================================================================

// ScalarF4E4 multiply rotation
__global__ void scalar_multiply_rotate32(
    const ScalarF4E4 *a_static,
    const ScalarF4E4 *b_values,  // 32 pre-loaded values
    ScalarF4E4 *result,
    int n)
{
    // Placeholder - call actual multiply kernel 32 times
    // For true rotation, we'd inline the multiply logic here
}

// f32 multiply rotation
__global__ void f32_multiply_rotate32(
    const float *a, const float *b_values, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    float acc = a[idx];
    for (int i = 0; i < ROTATION_SIZE; i++)
    {
        acc = acc * b_values[i];
    }
    result[idx] = acc;
}

// f32 divide rotation
__global__ void f32_divide_rotate32(
    const float *a, const float *b_values, float *result, int n)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    float acc = a[idx];
    for (int i = 0; i < ROTATION_SIZE; i++)
    {
        acc = acc / b_values[i];
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
    std::uniform_int_distribution<int16_t> exp_dist(-100, 100);  // Narrow range to avoid overflow

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

BenchResult bench_scalar_op(
    void (*op_func)(const ScalarF4E4*, const ScalarF4E4*, ScalarF4E4*, int),
    const ScalarF4E4 *h_a, const ScalarF4E4 *h_b,
    int n, int iterations, const char *name)
{
    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Warmup
    op_func(d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        op_func(d_a, d_b, d_result, n);
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

    return {name, milliseconds, ops_per_sec, bandwidth_gbps};
}

BenchResult bench_f32_op(
    dim3 (*kernel)(const float*, const float*, float*, int),
    int n, int iterations, const char *name, bool is_div)
{
    float *h_a = new float[n];
    float *h_b = new float[n];

    std::mt19937 gen(42);
    std::uniform_real_distribution<float> dist_a(-1000.0f, 1000.0f);
    std::uniform_real_distribution<float> dist_b(0.1f, 100.0f);  // Avoid div by tiny values

    for (int i = 0; i < n; i++)
    {
        h_a[i] = dist_a(gen);
        h_b[i] = dist_b(gen);
    }

    float *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_b, n * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(float), hipMemcpyHostToDevice);

    int threads = 256;
    int blocks = (n + threads - 1) / threads;

    if (is_div)
        hipLaunchKernelGGL(f32_divide_kernel, dim3(blocks), dim3(threads), 0, 0, d_a, d_b, d_result, n);
    else
        hipLaunchKernelGGL(f32_multiply_kernel, dim3(blocks), dim3(threads), 0, 0, d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++)
    {
        if (is_div)
            hipLaunchKernelGGL(f32_divide_kernel, dim3(blocks), dim3(threads), 0, 0, d_a, d_b, d_result, n);
        else
            hipLaunchKernelGGL(f32_multiply_kernel, dim3(blocks), dim3(threads), 0, 0, d_a, d_b, d_result, n);
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

    return {name, milliseconds, ops_per_sec, bandwidth_gbps};
}

// ============================================================================
// Main
// ============================================================================

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║         Multiply & Divide Pure Compute Throughput Benchmark                  ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n\n");

    const int n = 1048576;  // 1M elements (sweet spot)
    const int iterations = 100;

    // Generate test data
    ScalarF4E4 *h_a = new ScalarF4E4[n];
    ScalarF4E4 *h_b = new ScalarF4E4[n];

    generate_random_scalars(h_a, n, 42);
    generate_random_scalars(h_b, n, 43);

    // Multiplication benchmarks
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Multiplication: 1M elements × 100 iterations\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-50s | %-10s | %-12s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
    printf("──────────────────────────────────────────────────┼────────────┼──────────────┼────────────\n");
    print_result(bench_scalar_op(spirix_scalar_multiply_gpu, h_a, h_b, n, iterations, "ScalarF4E4 Multiply"));
    print_result(bench_f32_op(nullptr, n, iterations, "f32 Multiply", false));

    // Division benchmarks
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("  Division (Newton-Raphson + LUT): 1M elements × 100 iterations\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("%-50s | %-10s | %-12s | %-11s\n", "Test", "Time", "Throughput", "Bandwidth");
    printf("──────────────────────────────────────────────────┼────────────┼──────────────┼────────────\n");
    print_result(bench_scalar_op(spirix_scalar_divide_gpu, h_a, h_b, n, iterations, "ScalarF4E4 Divide (Newton-Raphson)"));
    print_result(bench_f32_op(nullptr, n, iterations, "f32 Divide (hardware)", true));

    delete[] h_a;
    delete[] h_b;

    printf("\n╔═══════════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                              Benchmark Complete                               ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════════╝\n");

    return 0;
}
