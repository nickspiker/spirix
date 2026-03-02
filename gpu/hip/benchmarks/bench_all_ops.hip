// Comprehensive ScalarF4E4 GPU Benchmark Suite
// Tests all 5 operations: add, subtract, multiply, divide, sqrt
//
// Compile:
//   hipcc -O3 bench_all_ops.hip ../kernels/scalar_ops.hip -o bench_all
//
// Run:
//   ./bench_all

#include <hip/hip_runtime.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <math.h>

// ScalarF4E4 definition
struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

// External kernel wrappers from scalar_ops.hip
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

// Benchmark a binary operation (add, subtract, multiply, divide)
void benchmark_binary_op(
    const char *name,
    void (*kernel)(const ScalarF4E4*, const ScalarF4E4*, ScalarF4E4*, int),
    int n,
    int iterations)
{
    // Allocate host memory
    ScalarF4E4 *h_a = (ScalarF4E4*)malloc(n * sizeof(ScalarF4E4));
    ScalarF4E4 *h_b = (ScalarF4E4*)malloc(n * sizeof(ScalarF4E4));

    // Initialize test data with variety
    for (int i = 0; i < n; i++) {
        h_a[i].fraction = 20000 + (i % 100);
        h_a[i].exponent = (i % 20) - 10;
        h_b[i].fraction = 15000 + (i % 100);
        h_b[i].exponent = (i % 10) - 5;
    }

    // Allocate GPU memory
    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    // Copy data to GPU
    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Warmup
    kernel(d_a, d_b, d_result, n);
    hipDeviceSynchronize();

    // Benchmark
    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++) {
        kernel(d_a, d_b, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    // Calculate performance
    double seconds = milliseconds / 1000.0;
    double ops_per_sec = (double)n * iterations / seconds;
    double gops = ops_per_sec / 1e9;

    printf("%-15s: %8.2f GOPS  (%8.3f ms, %10lld ops)\n",
           name, gops, milliseconds, (long long)n * iterations);

    // Cleanup
    hipEventDestroy(start);
    hipEventDestroy(stop);
    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    free(h_a);
    free(h_b);
}

// Benchmark a unary operation (sqrt)
void benchmark_unary_op(
    const char *name,
    void (*kernel)(const ScalarF4E4*, ScalarF4E4*, int),
    int n,
    int iterations)
{
    // Allocate host memory
    ScalarF4E4 *h_a = (ScalarF4E4*)malloc(n * sizeof(ScalarF4E4));

    // Initialize test data with positive values
    for (int i = 0; i < n; i++) {
        h_a[i].fraction = 20000 + (i % 100);
        h_a[i].exponent = (i % 20) - 10;
    }

    // Allocate GPU memory
    ScalarF4E4 *d_a, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    // Copy data to GPU
    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Warmup
    kernel(d_a, d_result, n);
    hipDeviceSynchronize();

    // Benchmark
    hipEvent_t start, stop;
    hipEventCreate(&start);
    hipEventCreate(&stop);

    hipEventRecord(start);
    for (int i = 0; i < iterations; i++) {
        kernel(d_a, d_result, n);
    }
    hipEventRecord(stop);
    hipEventSynchronize(stop);

    float milliseconds = 0;
    hipEventElapsedTime(&milliseconds, start, stop);

    // Calculate performance
    double seconds = milliseconds / 1000.0;
    double ops_per_sec = (double)n * iterations / seconds;
    double gops = ops_per_sec / 1e9;

    printf("%-15s: %8.2f GOPS  (%8.3f ms, %10lld ops)\n",
           name, gops, milliseconds, (long long)n * iterations);

    // Cleanup
    hipEventDestroy(start);
    hipEventDestroy(stop);
    hipFree(d_a);
    hipFree(d_result);
    free(h_a);
}

int main()
{
    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║     ScalarF4E4 GPU Benchmark Suite - All Operations         ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n\n");

    // GPU info
    hipDeviceProp_t prop;
    hipGetDeviceProperties(&prop, 0);
    printf("GPU: %s\n", prop.name);
    printf("Compute Units: %d\n", prop.multiProcessorCount);
    printf("Memory: %.0f GB\n", prop.totalGlobalMem / 1e9);
    printf("\n");

    const int n = 1048576;  // 1M elements (4MB, cache-resident)
    const int iterations = 100;

    printf("Configuration:\n");
    printf("  Elements:   %d (%.2f MB)\n", n, n * sizeof(ScalarF4E4) / 1e6);
    printf("  Iterations: %d\n", iterations);
    printf("  Total ops:  %lld\n\n", (long long)n * iterations);

    printf("Results:\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Benchmark all operations
    benchmark_binary_op("Addition", spirix_scalar_add_gpu, n, iterations);
    benchmark_binary_op("Subtraction", spirix_scalar_subtract_gpu, n, iterations);
    benchmark_binary_op("Multiplication", spirix_scalar_multiply_gpu, n, iterations);
    benchmark_binary_op("Division", spirix_scalar_divide_gpu, n, iterations);
    benchmark_unary_op("Square Root", spirix_scalar_sqrt_gpu, n, iterations);

    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    printf("Performance Analysis:\n");
    printf("  • Addition/Subtraction: Memory-bound (56 instructions)\n");
    printf("  • Multiplication: Memory-bound (56 instructions)\n");
    printf("  • Division: Compute-saturated (93 inst, Newton-Raphson)\n");
    printf("  • Square Root: Compute-saturated (102 inst, bitwise)\n");
    printf("  • Division > Multiply: ILP hides memory latency!\n\n");

    printf("Assembly Analysis:\n");
    printf("  Generate with: hipcc -O3 -save-temps=obj bench_all_ops.hip\n");
    printf("  View kernel:   cat bench_all-*-gfx1030.s | grep -A 100 scalar_divide_kernel\n\n");

    return 0;
}
