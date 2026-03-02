// Spirix GPU Kernel Tests - ScalarF4E4 Subtraction Correctness
// Author: Nick Spiker fractaldecoder@proton.me
//
// Validates scalar subtraction kernel against edge cases and special values

#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

// ScalarF4E4 definition (must match kernel)
struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

#define AMBIGUOUS_EXPONENT ((int16_t)0x8000) // -32768
#define FRACTION_BITS 16

// External kernel declaration
extern "C" void spirix_scalar_subtract_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n);

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
    spirix_scalar_subtract_gpu(d_a, d_b, d_result, 1);

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
    printf("║ Spirix GPU Kernel Correctness Tests - Scalar Subtraction  ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n");

    int total = 0;
    int passed = 0;

    // Test suite
    TestCase tests[] = {
        // Normal-Normal Cases
        {
            "Normal: Same exponent",
            {100, 5},  // a = 100 × 2^5
            {50, 5},   // b = 50 × 2^5
            {50, 5}    // result = 50 × 2^5 (approximately, needs normalization)
        },
        {
            "Normal: Different exponent (a larger)",
            {100, 5},  // a = 100 × 2^5 = 3200
            {50, 3},   // b = 50 × 2^3 = 400
            {88, 5}    // result ≈ 2800 = 88 × 2^5 (approximately)
        },
        {
            "Normal: Different exponent (b larger)",
            {50, 3},   // a = 50 × 2^3 = 400
            {100, 5},  // b = 100 × 2^5 = 3200
            {-88, 5}   // result ≈ -2800 = -88 × 2^5 (approximately)
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
            {-100, 24}   // result ≈ -b (a too small to matter)
        },
        {
            "Negligible: extreme difference (exp_diff = 100)",
            {123, 200},
            {456, 100},
            {123, 200}
        },

        // Zero Result Cases
        {
            "Zero: exact cancellation",
            {100, 5},
            {100, 5},
            {0, AMBIGUOUS_EXPONENT}
        },
        {
            "Zero: aligned cancellation",
            {200, 8},   // 200 × 2^8
            {100, 9},   // 100 × 2^9 = 200 × 2^8 (same value)
            {0, AMBIGUOUS_EXPONENT}
        },

        // Edge Cases
        {
            "Edge: max positive fraction",
            {32767, 10},   // Max i16 fraction
            {1, 10},
            {32766, 10}    // Approximately (needs normalization)
        },
        {
            "Edge: min negative fraction",
            {-32768, 10},  // Min i16 fraction
            {1, 10},
            {32767, 9}     // Result after normalization (approximately)
        },
        {
            "Edge: large negative exponent",
            {100, -30000},
            {50, -30000},
            {50, -30000}
        },
        {
            "Edge: near-underflow",
            {2, AMBIGUOUS_EXPONENT + 10},  // Very small exponent
            {1, AMBIGUOUS_EXPONENT + 10},
            {1, AMBIGUOUS_EXPONENT + 10}   // Or might underflow
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
