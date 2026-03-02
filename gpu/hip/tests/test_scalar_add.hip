// Spirix GPU Kernel Tests - ScalarF4E4 Addition Correctness
// Author: Nick Spiker fractaldecoder@proton.me
//
// Validates scalar addition kernel against edge cases and special values

#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

// ScalarF4E4 definition (must match kernel)
struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

#define AMBIGUOUS_EXPONENT ((int16_t)0x8000) // -32768
#define FRACTION_BITS 16

// External kernel declaration
extern "C" void spirix_scalar_add_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

// ============================================================================
// CPU Reference Implementation & Normalization
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

ScalarF4E4 scalar_add_cpu_reference(ScalarF4E4 a, ScalarF4E4 b)
{
    int32_t a_frac = (int32_t)a.fraction;
    int32_t b_frac = (int32_t)b.fraction;
    int32_t a_exp = (int32_t)a.exponent;
    int32_t b_exp = (int32_t)b.exponent;

    // Check for ambiguous
    if (a_exp == AMBIGUOUS_EXPONENT || b_exp == AMBIGUOUS_EXPONENT)
        return {0, AMBIGUOUS_EXPONENT};

    int32_t exp_diff = a_exp - b_exp;
    int32_t abs_diff = (exp_diff < 0) ? -exp_diff : exp_diff;

    // Check negligible
    if (abs_diff >= FRACTION_BITS)
    {
        if (exp_diff >= 0)
            return {(int16_t)a_frac, (int16_t)a_exp};
        else
            return {(int16_t)b_frac, (int16_t)b_exp};
    }

    // Align and ADD (only difference from subtraction!)
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
        return {0, AMBIGUOUS_EXPONENT};

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
// Test Case Helpers
// ============================================================================

struct TestCase
{
    const char *name;
    ScalarF4E4 a;
    ScalarF4E4 b;
    ScalarF4E4 expected;
};

void print_scalar(const char *name, ScalarF4E4 s)
{
    if (s.exponent == AMBIGUOUS_EXPONENT)
    {
        printf("%s = AMBIGUOUS (frac=%d)\n", name, s.fraction);
    }
    else
    {
        printf("%s = (%d, %d) = %d × 2^%d\n",
               name, s.fraction, s.exponent, s.fraction, s.exponent);
    }
}

bool scalars_equal(ScalarF4E4 a, ScalarF4E4 b)
{
    return (a.fraction == b.fraction) && (a.exponent == b.exponent);
}

bool run_test_case(const TestCase &tc)
{
    printf("\n--- Test: %s ---\n", tc.name);
    print_scalar("a", tc.a);
    print_scalar("b", tc.b);
    print_scalar("expected", tc.expected);

    // Allocate device memory
    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, sizeof(ScalarF4E4));
    hipMalloc(&d_b, sizeof(ScalarF4E4));
    hipMalloc(&d_result, sizeof(ScalarF4E4));

    // Copy inputs to device
    hipMemcpy(d_a, &tc.a, sizeof(ScalarF4E4), hipMemcpyHostToDevice);
    hipMemcpy(d_b, &tc.b, sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Run kernel
    spirix_scalar_add_gpu(d_a, d_b, d_result, 1);

    // Copy result back
    ScalarF4E4 result;
    hipMemcpy(&result, d_result, sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

    // Free device memory
    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);

    // Check result
    print_scalar("actual", result);

    bool pass = scalars_equal(result, tc.expected);
    printf("Result: %s\n", pass ? "✓ PASS" : "✗ FAIL");

    return pass;
}

// ============================================================================
// Test Cases
// ============================================================================

int main()
{
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║ Spirix GPU Kernel Correctness Tests - Scalar Addition     ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n");

    int total = 0;
    int passed = 0;

    // Generate normalized test inputs
    ScalarF4E4 a1 = normalize_scalar(100, 5);
    ScalarF4E4 b1 = normalize_scalar(50, 5);

    ScalarF4E4 a2 = normalize_scalar(100, 5);
    ScalarF4E4 b2 = normalize_scalar(-50, 5);

    ScalarF4E4 a3 = normalize_scalar(100, 5);
    ScalarF4E4 b3 = normalize_scalar(50, 3);

    ScalarF4E4 a4 = normalize_scalar(16384, 10);
    ScalarF4E4 b4 = normalize_scalar(16384, 10);

    // Test suite
    TestCase tests[] = {
        // Normal-Normal Cases (using CPU reference for expected values)
        {
            "Normal: Same exponent, same sign",
            a1, b1,
            scalar_add_cpu_reference(a1, b1)
        },
        {
            "Normal: Same exponent, opposite signs",
            a2, b2,
            scalar_add_cpu_reference(a2, b2)
        },
        {
            "Normal: Different exponent (a larger)",
            a3, b3,
            scalar_add_cpu_reference(a3, b3)
        },
        {
            "Normal: Different exponent (b larger)",
            b3, a3,  // Swap a and b
            scalar_add_cpu_reference(b3, a3)
        },

        // Ambiguous Input Cases
        {
            "Ambiguous: a is ambiguous",
            {0, AMBIGUOUS_EXPONENT},
            {100, 5},
            {0, AMBIGUOUS_EXPONENT}
        },
        {
            "Ambiguous: b is ambiguous",
            {100, 5},
            {0, AMBIGUOUS_EXPONENT},
            {0, AMBIGUOUS_EXPONENT}
        },
        {
            "Ambiguous: both ambiguous",
            {0, AMBIGUOUS_EXPONENT},
            {0, AMBIGUOUS_EXPONENT},
            {0, AMBIGUOUS_EXPONENT}
        },

        // Negligible Difference Cases (|exp_diff| >= 16)
        {
            "Negligible: a >> b (exp_diff = 16)",
            {100, 20},   // a = 100 × 2^20
            {50, 4},     // b = 50 × 2^4 (negligible)
            {100, 20}    // result ≈ a (b too small to matter)
        },
        {
            "Negligible: b >> a (exp_diff = -20)",
            {50, 4},     // a = 50 × 2^4 (negligible)
            {100, 24},   // b = 100 × 2^24
            {100, 24}    // result ≈ b (a too small to matter)
        },
        {
            "Negligible: extreme difference (exp_diff = 100)",
            {123, 200},
            {456, 100},
            {123, 200}
        },

        // Zero Result Cases (opposite values cancel)
        {
            "Zero: exact cancellation (opposite signs)",
            {100, 5},
            {-100, 5},
            {0, AMBIGUOUS_EXPONENT}
        },
        {
            "Zero: aligned cancellation",
            {200, 8},   // 200 × 2^8
            {-100, 9},  // -100 × 2^9 = -200 × 2^8 (opposite value)
            {0, AMBIGUOUS_EXPONENT}
        },

        // Overflow Cases (addition makes result larger)
        {
            "Overflow: large positive + large positive",
            a4, b4,
            scalar_add_cpu_reference(a4, b4)
        },

        // Edge Cases
        {
            "Edge: positive + negative (different magnitudes)",
            normalize_scalar(200, 8),
            normalize_scalar(-50, 8),
            scalar_add_cpu_reference(normalize_scalar(200, 8), normalize_scalar(-50, 8))
        },
        {
            "Edge: large negative + large negative",
            normalize_scalar(-16384, 10),
            normalize_scalar(-16384, 10),
            scalar_add_cpu_reference(normalize_scalar(-16384, 10), normalize_scalar(-16384, 10))
        },
        {
            "Edge: near-underflow addition",
            normalize_scalar(2, AMBIGUOUS_EXPONENT + 10),
            normalize_scalar(1, AMBIGUOUS_EXPONENT + 10),
            scalar_add_cpu_reference(normalize_scalar(2, AMBIGUOUS_EXPONENT + 10),
                                    normalize_scalar(1, AMBIGUOUS_EXPONENT + 10))
        },
    };

    // Run all tests
    for (const auto &test : tests)
    {
        total++;
        if (run_test_case(test))
        {
            passed++;
        }
    }

    // Summary
    printf("\n╔════════════════════════════════════════════════════════════╗\n");
    printf("║ Test Summary                                               ║\n");
    printf("╠════════════════════════════════════════════════════════════╣\n");
    printf("║ Total:  %3d                                                ║\n", total);
    printf("║ Passed: %3d                                                ║\n", passed);
    printf("║ Failed: %3d                                                ║\n", total - passed);
    printf("╚════════════════════════════════════════════════════════════╝\n");

    return (passed == total) ? 0 : 1;
}
