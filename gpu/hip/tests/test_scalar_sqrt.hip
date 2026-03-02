// Spirix GPU Kernel Correctness Tests - Scalar Square Root
// Tests sqrt operation with edge cases: zero, negative, perfect squares, normalized values

#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <cmath>

// ScalarF4E4: 16-bit fraction + 16-bit exponent
struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

#define AMBIGUOUS_EXPONENT ((int16_t)0x8000)
#define FRACTION_BITS 16

// External kernel declaration
extern "C" void spirix_scalar_sqrt_gpu(
    const ScalarF4E4 *d_a,
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

uint32_t bitwise_sqrt_u32_cpu(uint32_t radicand)
{
    uint32_t bit = 1u << 15;
    uint32_t result = 0;

    for (int i = 0; i < 16; i++)
    {
        uint32_t guess = result | bit;
        if ((uint64_t)guess * (uint64_t)guess <= (uint64_t)radicand)
        {
            result = guess;
        }
        bit >>= 1;
    }

    return result;
}

ScalarF4E4 scalar_sqrt_cpu_reference(ScalarF4E4 input)
{
    // Handle edge cases
    if (input.fraction == 0)
        return {0, AMBIGUOUS_EXPONENT};
    if (input.fraction < 0)
        return {0, AMBIGUOUS_EXPONENT};  // Negative sqrt is undefined

    // Exponent calculation: divide by 2, handle odd exponents
    int16_t exponent = input.exponent / 2;
    int16_t even = input.exponent & 1;
    exponent = exponent + even;

    // Correct for negative odd exponents (floor division)
    if (input.exponent < 0 && even != 0)
    {
        exponent = exponent - 1;
    }

    // Prepare fraction for sqrt
    uint32_t value = (uint32_t)(uint16_t)input.fraction;
    uint32_t shifted_value = value << (17 - even);

    // Compute sqrt
    uint32_t sqrt_result = bitwise_sqrt_u32_cpu(shifted_value);

    // Normalize
    uint32_t leading_zeros = __builtin_clz(sqrt_result) - 1;
    sqrt_result = sqrt_result << leading_zeros;
    int16_t shift_adjust = (int16_t)leading_zeros - 15;
    exponent = exponent + shift_adjust;

    int16_t fraction = (int16_t)(sqrt_result >> 16);
    return {fraction, exponent};
}

// ============================================================================
// Test Case Helpers
// ============================================================================

struct TestCase
{
    const char *name;
    ScalarF4E4 input;
    ScalarF4E4 expected;
};

void print_scalar(const char *label, ScalarF4E4 s)
{
    printf("%s: fraction=0x%04x (%d), exponent=%d\n",
           label, (uint16_t)s.fraction, s.fraction, s.exponent);
}

bool run_test_case(const TestCase &test)
{
    ScalarF4E4 *d_input, *d_result;
    ScalarF4E4 h_result;

    // Allocate device memory
    hipMalloc(&d_input, sizeof(ScalarF4E4));
    hipMalloc(&d_result, sizeof(ScalarF4E4));

    // Copy input to device
    hipMemcpy(d_input, &test.input, sizeof(ScalarF4E4), hipMemcpyHostToDevice);

    // Run kernel
    spirix_scalar_sqrt_gpu(d_input, d_result, 1);

    // Copy result back
    hipMemcpy(&h_result, d_result, sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

    // Check result
    bool passed = (h_result.fraction == test.expected.fraction) &&
                  (h_result.exponent == test.expected.exponent);

    // Print results
    printf("\n--- Test: %s ---\n", test.name);
    print_scalar("Input   ", test.input);
    print_scalar("Expected", test.expected);
    print_scalar("Got     ", h_result);
    printf("Result: %s\n", passed ? "✓ PASS" : "✗ FAIL");

    // Cleanup
    hipFree(d_input);
    hipFree(d_result);

    return passed;
}

// ============================================================================
// Main Test Runner
// ============================================================================

int main()
{
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║ Spirix GPU Kernel Correctness Tests - Square Root         ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n");

    int total = 0;
    int passed = 0;

    // Test suite
    TestCase tests[] = {
        // Edge Cases
        {
            "Zero input",
            {0, 5},
            {0, AMBIGUOUS_EXPONENT}
        },
        {
            "Negative input (undefined)",
            normalize_scalar(-16384, 10),
            {0, AMBIGUOUS_EXPONENT}
        },

        // Perfect Squares (fraction only)
        {
            "sqrt(16384) = 128, exponent 0",
            normalize_scalar(16384, 0),
            scalar_sqrt_cpu_reference(normalize_scalar(16384, 0))
        },
        {
            "sqrt(4096) with small exponent",
            normalize_scalar(4096, 2),
            scalar_sqrt_cpu_reference(normalize_scalar(4096, 2))
        },

        // Normal values (even exponents)
        {
            "Normal: even exponent, small value",
            normalize_scalar(20000, 4),
            scalar_sqrt_cpu_reference(normalize_scalar(20000, 4))
        },
        {
            "Normal: even exponent, medium value",
            normalize_scalar(25000, 10),
            scalar_sqrt_cpu_reference(normalize_scalar(25000, 10))
        },
        {
            "Normal: even exponent, large value",
            normalize_scalar(30000, 20),
            scalar_sqrt_cpu_reference(normalize_scalar(30000, 20))
        },

        // Odd exponents (more complex case)
        {
            "Normal: odd exponent, positive",
            normalize_scalar(18000, 5),
            scalar_sqrt_cpu_reference(normalize_scalar(18000, 5))
        },
        {
            "Normal: odd exponent, negative",
            normalize_scalar(22000, -7),
            scalar_sqrt_cpu_reference(normalize_scalar(22000, -7))
        },

        // Very small values
        {
            "Small: near-zero value",
            normalize_scalar(100, -100),
            scalar_sqrt_cpu_reference(normalize_scalar(100, -100))
        },
        {
            "Small: tiny fraction",
            normalize_scalar(1000, 0),
            scalar_sqrt_cpu_reference(normalize_scalar(1000, 0))
        },

        // Very large values
        {
            "Large: high exponent",
            normalize_scalar(28000, 100),
            scalar_sqrt_cpu_reference(normalize_scalar(28000, 100))
        },
        {
            "Large: near-max fraction",
            normalize_scalar(32000, 50),
            scalar_sqrt_cpu_reference(normalize_scalar(32000, 50))
        },

        // Negative exponents
        {
            "Negative exponent: even",
            normalize_scalar(20000, -10),
            scalar_sqrt_cpu_reference(normalize_scalar(20000, -10))
        },
        {
            "Negative exponent: odd (floor division)",
            normalize_scalar(20000, -11),
            scalar_sqrt_cpu_reference(normalize_scalar(20000, -11))
        },

        // Values near boundaries
        {
            "Boundary: minimum positive normalized",
            normalize_scalar(16384, AMBIGUOUS_EXPONENT + 2),
            scalar_sqrt_cpu_reference(normalize_scalar(16384, AMBIGUOUS_EXPONENT + 2))
        },
    };

    // Run all tests
    for (const auto &test : tests)
    {
        total++;
        if (run_test_case(test))
            passed++;
    }

    // Summary
    printf("\n╔════════════════════════════════════════════════════════════╗\n");
    printf("║                       Test Summary                         ║\n");
    printf("╠════════════════════════════════════════════════════════════╣\n");
    printf("║  Total tests: %-3d                                          ║\n", total);
    printf("║  Passed:      %-3d                                          ║\n", passed);
    printf("║  Failed:      %-3d                                          ║\n", total - passed);
    printf("╚════════════════════════════════════════════════════════════╝\n");

    return (passed == total) ? 0 : 1;
}
