// Spirix GPU Test - Denormal Flush-to-Zero Detection
// Author: Nick Spiker fractaldecoder@proton.me
//
// Tests whether AMD GPU is flushing IEEE 754 denormals to zero
// Compares GPU f32 subtraction vs CPU f32 subtraction on denormal-producing inputs

#include <hip/hip_runtime.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <cfloat>

// ============================================================================
// IEEE 754 f32 Subtraction Kernel (same as benchmark)
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
// CPU Reference (standard f32 arithmetic)
// ============================================================================

void f32_subtract_cpu(const float *a, const float *b, float *result, int n)
{
    for (int i = 0; i < n; i++)
    {
        result[i] = a[i] - b[i];
    }
}

// ============================================================================
// Denormal Test Cases
// ============================================================================

void print_float_bits(const char *name, float value)
{
    uint32_t bits = *(uint32_t*)&value;
    uint32_t sign = (bits >> 31) & 0x1;
    uint32_t exponent = (bits >> 23) & 0xFF;
    uint32_t mantissa = bits & 0x7FFFFF;

    printf("%s = %.17e\n", name, value);
    printf("  bits: %08X (sign=%u, exp=%03u, mantissa=%06X)\n",
           bits, sign, exponent, mantissa);

    if (exponent == 0 && mantissa != 0)
    {
        printf("  ⚠️  DENORMAL (subnormal) number\n");
    }
    else if (exponent == 0 && mantissa == 0)
    {
        printf("  ✓ Zero\n");
    }
    else
    {
        printf("  ✓ Normal number\n");
    }
}

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════╗\n");
    printf("║     AMD GPU Denormal Flush-to-Zero (FTZ) Detection Test      ║\n");
    printf("╚═══════════════════════════════════════════════════════════════╝\n\n");

    // IEEE 754 single-precision limits
    printf("IEEE 754 f32 Constants:\n");
    printf("  FLT_MIN (smallest normal):    %.17e\n", FLT_MIN);
    printf("  FLT_TRUE_MIN (smallest denorm): %.17e\n", FLT_TRUE_MIN);
    printf("  Denormal range: [%.3e, %.3e]\n\n", FLT_TRUE_MIN, FLT_MIN);

    // Test cases designed to produce denormal results
    const int n = 6;
    float h_a[n] = {
        2.0f * FLT_MIN,        // Normal - Normal = Denormal
        1.0f * FLT_MIN,        // Normal - Normal = Denormal
        5.0e-39f,              // Denormal - Denormal = Denormal
        1.0e-40f,              // Denormal - Denormal = Denormal
        FLT_TRUE_MIN * 100.0f, // Denormal - Denormal = Denormal
        1.0e-45f               // Tiny denormal - Tiny denormal
    };

    float h_b[n] = {
        1.9f * FLT_MIN,        // Should produce denormal: 0.1 * FLT_MIN
        0.9f * FLT_MIN,        // Should produce denormal: 0.1 * FLT_MIN
        4.5e-39f,              // Should produce denormal
        0.5e-40f,              // Should produce denormal
        FLT_TRUE_MIN * 95.0f,  // Should produce denormal
        0.5e-45f               // Should produce denormal
    };

    float h_cpu_result[n];
    float h_gpu_result[n];

    // Compute CPU reference
    printf("Computing CPU f32 reference (with denormal support)...\n");
    f32_subtract_cpu(h_a, h_b, h_cpu_result, n);

    // Compute GPU results
    printf("Computing GPU f32 results (checking for FTZ)...\n");
    float *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(float));
    hipMalloc(&d_b, n * sizeof(float));
    hipMalloc(&d_result, n * sizeof(float));

    hipMemcpy(d_a, h_a, n * sizeof(float), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(float), hipMemcpyHostToDevice);

    f32_subtract_gpu(d_a, d_b, d_result, n);

    hipMemcpy(h_gpu_result, d_result, n * sizeof(float), hipMemcpyDeviceToHost);

    // Compare results
    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("Test Results:\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    int denormals_detected = 0;
    int ftz_detected = 0;

    for (int i = 0; i < n; i++)
    {
        printf("Test %d:\n", i + 1);
        printf("  a = %.17e\n", h_a[i]);
        printf("  b = %.17e\n", h_b[i]);
        printf("  CPU result = %.17e\n", h_cpu_result[i]);
        printf("  GPU result = %.17e\n", h_gpu_result[i]);

        // Check if CPU result is denormal
        uint32_t cpu_bits = *(uint32_t*)&h_cpu_result[i];
        uint32_t cpu_exp = (cpu_bits >> 23) & 0xFF;
        uint32_t cpu_mantissa = cpu_bits & 0x7FFFFF;
        bool cpu_is_denormal = (cpu_exp == 0 && cpu_mantissa != 0);

        // Check if GPU flushed to zero
        bool gpu_is_zero = (h_gpu_result[i] == 0.0f);

        if (cpu_is_denormal)
        {
            denormals_detected++;
            printf("  → CPU produced DENORMAL\n");

            if (gpu_is_zero)
            {
                ftz_detected++;
                printf("  → GPU FLUSHED TO ZERO! ⚠️  FTZ MODE DETECTED\n");
            }
            else if (h_gpu_result[i] != h_cpu_result[i])
            {
                printf("  → GPU produced different value (not exact FTZ)\n");
            }
            else
            {
                printf("  → GPU preserved denormal (FTZ disabled)\n");
            }
        }
        else if (h_cpu_result[i] != h_gpu_result[i])
        {
            printf("  → Mismatch but not denormal-related\n");
        }
        else
        {
            printf("  → Normal result, GPU matches CPU ✓\n");
        }

        printf("\n");
    }

    // Summary
    printf("╔═══════════════════════════════════════════════════════════════╗\n");
    printf("║                          Summary                              ║\n");
    printf("╠═══════════════════════════════════════════════════════════════╣\n");
    printf("║ Tests that produced denormals (CPU):  %2d                     ║\n", denormals_detected);
    printf("║ Tests where GPU flushed to zero:      %2d                     ║\n", ftz_detected);
    printf("╚═══════════════════════════════════════════════════════════════╝\n\n");

    if (ftz_detected > 0)
    {
        printf("🔥 FLUSH-TO-ZERO (FTZ) MODE CONFIRMED!\n\n");
        printf("AMD GPU is flushing denormal f32 values to zero for performance.\n");
        printf("This is the default behavior on RDNA2 GPUs.\n\n");
        printf("Implications:\n");
        printf("  ✓ f32 subtraction is FAST (no denormal handling overhead)\n");
        printf("  ✗ f32 subtraction LOSES PRECISION near underflow\n");
        printf("  ✓ ScalarF4E4 preserves fraction bits during underflow\n");
        printf("  ✓ ScalarF4E4 provides better dynamic range control\n\n");
    }
    else
    {
        printf("✓ GPU preserved denormals (FTZ mode disabled)\n");
        printf("This is unusual for AMD GPUs - may indicate special driver settings.\n");
    }

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);

    return 0;
}
