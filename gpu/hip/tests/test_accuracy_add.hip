// Spirix GPU Accuracy Test - Addition vs Rust CPU Reference
// Author: Nick Spiker fractaldecoder@proton.me
//
// Validates GPU kernel produces identical results to Rust reference implementation

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

extern "C" void spirix_scalar_add_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

// ============================================================================
// Rust Reference Implementation (ported to C++)
// ============================================================================

ScalarF4E4 scalar_add_cpu_reference(ScalarF4E4 a, ScalarF4E4 b)
{
    int32_t a_frac = (int32_t)a.fraction;
    int32_t b_frac = (int32_t)b.fraction;
    int32_t a_exp = (int32_t)a.exponent;
    int32_t b_exp = (int32_t)b.exponent;

    // Check for ambiguous
    if (a_exp == AMBIGUOUS_EXPONENT || b_exp == AMBIGUOUS_EXPONENT)
    {
        return {0, AMBIGUOUS_EXPONENT};
    }

    int32_t exp_diff = a_exp - b_exp;
    int32_t abs_diff = (exp_diff < 0) ? -exp_diff : exp_diff;

    // Check negligible
    if (abs_diff >= FRACTION_BITS)
    {
        if (exp_diff >= 0)
        {
            return {(int16_t)a_frac, (int16_t)a_exp};
        }
        else
        {
            return {(int16_t)b_frac, (int16_t)b_exp};
        }
    }

    // Align and ADD
    int32_t add_result;
    int32_t result_exp;

    if (exp_diff >= 0)
    {
        add_result = (a_frac << abs_diff) + b_frac;
        result_exp = b_exp;
    }
    else
    {
        add_result = a_frac + (b_frac << abs_diff);
        result_exp = a_exp;
    }

    // Sign-extend
    add_result = (add_result << 16) >> 16;

    // Check zero
    if (add_result == 0)
    {
        return {0, AMBIGUOUS_EXPONENT};
    }

    // Count leading same bits
    int lz = __builtin_clz((uint32_t)add_result);
    int lo = __builtin_clz((uint32_t)(~add_result));
    int leading = (lz > lo) ? lz : lo;

    // Calculate exponent
    int32_t temp_exp = result_exp + 17 - leading;

    // Check underflow
    if (temp_exp <= AMBIGUOUS_EXPONENT)
    {
        int32_t norm_frac = (add_result << (leading - 2)) >> 16;
        return {(int16_t)norm_frac, AMBIGUOUS_EXPONENT};
    }

    // Normalize
    int32_t norm_frac = (add_result << (leading - 1)) >> 16;

    return {(int16_t)norm_frac, (int16_t)temp_exp};
}

// ============================================================================
// Test Data Generation
// ============================================================================

ScalarF4E4 normalize_scalar(int16_t frac, int16_t exp)
{
    if (frac == 0 || exp == AMBIGUOUS_EXPONENT)
    {
        return {0, AMBIGUOUS_EXPONENT};
    }

    int32_t f = (int32_t)frac;
    int32_t e = (int32_t)exp;

    int lz = __builtin_clz((uint32_t)f);
    int lo = __builtin_clz((uint32_t)(~f));
    int leading = (lz > lo) ? lz : lo;

    f = (f << (leading - 1)) >> FRACTION_BITS;
    e = e + 17 - leading;

    if (e <= AMBIGUOUS_EXPONENT)
    {
        return {0, AMBIGUOUS_EXPONENT};
    }

    return {(int16_t)f, (int16_t)e};
}

void generate_test_data(ScalarF4E4 *a, ScalarF4E4 *b, int n)
{
    std::mt19937 gen(42);
    std::uniform_int_distribution<int16_t> frac_dist(-32768, 32767);
    std::uniform_int_distribution<int16_t> exp_dist(-30000, 30000);

    for (int i = 0; i < n; i++)
    {
        a[i] = normalize_scalar(frac_dist(gen), exp_dist(gen));
        b[i] = normalize_scalar(frac_dist(gen), exp_dist(gen));
    }

    // Add edge cases
    int idx = 0;
    if (idx < n) { a[idx] = {0, AMBIGUOUS_EXPONENT}; b[idx] = {100, 5}; idx++; }
    if (idx < n) { a[idx] = {100, 5}; b[idx] = {0, AMBIGUOUS_EXPONENT}; idx++; }
    if (idx < n) { a[idx] = {100, 5}; b[idx] = {-100, 5}; idx++; }  // Zero result
    if (idx < n) { a[idx] = {100, 20}; b[idx] = {50, 4}; idx++; }  // Negligible
    if (idx < n) { a[idx] = {16384, 10}; b[idx] = {16384, 10}; idx++; }  // Overflow
}

// ============================================================================
// Accuracy Test
// ============================================================================

int main()
{
    printf("╔═══════════════════════════════════════════════════════════════╗\n");
    printf("║   Spirix GPU Accuracy Test - Addition vs Rust CPU Reference  ║\n");
    printf("╚═══════════════════════════════════════════════════════════════╝\n\n");

    const int n = 100000;

    ScalarF4E4 *h_a = new ScalarF4E4[n];
    ScalarF4E4 *h_b = new ScalarF4E4[n];
    ScalarF4E4 *h_cpu_result = new ScalarF4E4[n];
    ScalarF4E4 *h_gpu_result = new ScalarF4E4[n];

    generate_test_data(h_a, h_b, n);

    // Compute CPU reference
    printf("Computing CPU reference results...\n");
    for (int i = 0; i < n; i++)
    {
        h_cpu_result[i] = scalar_add_cpu_reference(h_a[i], h_b[i]);
    }

    // Compute GPU results
    printf("Computing GPU results...\n");
    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, n * sizeof(ScalarF4E4));
    hipMalloc(&d_b, n * sizeof(ScalarF4E4));
    hipMalloc(&d_result, n * sizeof(ScalarF4E4));

    hipMemcpy(d_a, h_a, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, h_b, n * sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    spirix_scalar_add_gpu(d_a, d_b, d_result, n);

    hipMemcpy(h_gpu_result, d_result, n * sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

    // Compare results
    printf("\nComparing results...\n\n");

    int mismatches = 0;
    int first_mismatch = -1;

    for (int i = 0; i < n; i++)
    {
        if (h_cpu_result[i].fraction != h_gpu_result[i].fraction ||
            h_cpu_result[i].exponent != h_gpu_result[i].exponent)
        {
            if (first_mismatch == -1)
            {
                first_mismatch = i;
            }
            mismatches++;
        }
    }

    // Print results
    printf("╔═══════════════════════════════════════════════════════════════╗\n");
    printf("║                         Test Results                          ║\n");
    printf("╠═══════════════════════════════════════════════════════════════╣\n");
    printf("║ Total tests:      %7d                                   ║\n", n);
    printf("║ Matching:         %7d                                   ║\n", n - mismatches);
    printf("║ Mismatches:       %7d                                   ║\n", mismatches);
    printf("║ Accuracy:         %7.4f%%                                ║\n",
           100.0 * (n - mismatches) / n);
    printf("╚═══════════════════════════════════════════════════════════════╝\n");

    if (mismatches > 0)
    {
        printf("\n⚠️  MISMATCH DETECTED!\n\n");
        printf("First mismatch at index %d:\n", first_mismatch);
        printf("  a:          (%6d, %6d)\n",
               h_a[first_mismatch].fraction, h_a[first_mismatch].exponent);
        printf("  b:          (%6d, %6d)\n",
               h_b[first_mismatch].fraction, h_b[first_mismatch].exponent);
        printf("  CPU result: (%6d, %6d)\n",
               h_cpu_result[first_mismatch].fraction, h_cpu_result[first_mismatch].exponent);
        printf("  GPU result: (%6d, %6d)\n",
               h_gpu_result[first_mismatch].fraction, h_gpu_result[first_mismatch].exponent);

        // Show first few mismatches
        printf("\nShowing first 10 mismatches:\n");
        int shown = 0;
        for (int i = 0; i < n && shown < 10; i++)
        {
            if (h_cpu_result[i].fraction != h_gpu_result[i].fraction ||
                h_cpu_result[i].exponent != h_gpu_result[i].exponent)
            {
                printf("  [%5d] a=(%6d,%6d) b=(%6d,%6d) | CPU=(%6d,%6d) GPU=(%6d,%6d)\n",
                       i,
                       h_a[i].fraction, h_a[i].exponent,
                       h_b[i].fraction, h_b[i].exponent,
                       h_cpu_result[i].fraction, h_cpu_result[i].exponent,
                       h_gpu_result[i].fraction, h_gpu_result[i].exponent);
                shown++;
            }
        }
    }
    else
    {
        printf("\n✅ ALL TESTS PASSED! GPU and CPU results match perfectly.\n");
    }

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
    delete[] h_a;
    delete[] h_b;
    delete[] h_cpu_result;
    delete[] h_gpu_result;

    return (mismatches == 0) ? 0 : 1;
}
