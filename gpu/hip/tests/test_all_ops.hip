// Comprehensive ScalarF4E4 GPU Test Suite
// Tests all 5 operations with edge cases
//
// Compile:
//   hipcc -O3 test_all_ops.hip ../kernels/scalar_ops.hip -o test_all
//
// Run:
//   ./test_all

#include <hip/hip_runtime.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <math.h>

#define AMBIGUOUS_EXPONENT ((int16_t)-32768)

// ScalarF4E4 definition
struct ScalarF4E4
{
    int16_t fraction;
    int16_t exponent;
};

// External kernel wrappers
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

// Test runner
typedef struct {
    const char *name;
    ScalarF4E4 a;
    ScalarF4E4 b;
    ScalarF4E4 expected;
} BinaryTestCase;

typedef struct {
    const char *name;
    ScalarF4E4 a;
    ScalarF4E4 expected;
} UnaryTestCase;

void run_binary_tests(
    const char *op_name,
    void (*kernel)(const ScalarF4E4*, const ScalarF4E4*, ScalarF4E4*, int),
    BinaryTestCase *tests,
    int num_tests)
{
    printf("\n%s Tests:\n", op_name);
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    ScalarF4E4 *d_a, *d_b, *d_result;
    hipMalloc(&d_a, sizeof(ScalarF4E4));
    hipMalloc(&d_b, sizeof(ScalarF4E4));
    hipMalloc(&d_result, sizeof(ScalarF4E4));

    int passed = 0;
    for (int i = 0; i < num_tests; i++) {
        BinaryTestCase *test = &tests[i];

        // Copy to GPU
        hipMemcpy(d_a, &test->a, sizeof(ScalarF4E4), hipMemcpyHostToDevice);
        hipMemcpy(d_b, &test->b, sizeof(ScalarF4E4), hipMemcpyHostToDevice);

        // Run kernel
        kernel(d_a, d_b, d_result, 1);

        // Copy result
        ScalarF4E4 result;
        hipMemcpy(&result, d_result, sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

        // Check result
        bool pass = (result.fraction == test->expected.fraction &&
                     result.exponent == test->expected.exponent);

        if (pass) {
            printf("  ✓ %s\n", test->name);
            passed++;
        } else {
            printf("  ✗ %s\n", test->name);
            printf("    Input:    a=(%d, %d) b=(%d, %d)\n",
                   test->a.fraction, test->a.exponent,
                   test->b.fraction, test->b.exponent);
            printf("    Expected: (%d, %d)\n",
                   test->expected.fraction, test->expected.exponent);
            printf("    Got:      (%d, %d)\n",
                   result.fraction, result.exponent);
        }
    }

    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("Passed: %d/%d\n", passed, num_tests);

    hipFree(d_a);
    hipFree(d_b);
    hipFree(d_result);
}

void run_unary_tests(
    const char *op_name,
    void (*kernel)(const ScalarF4E4*, ScalarF4E4*, int),
    UnaryTestCase *tests,
    int num_tests)
{
    printf("\n%s Tests:\n", op_name);
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    ScalarF4E4 *d_a, *d_result;
    hipMalloc(&d_a, sizeof(ScalarF4E4));
    hipMalloc(&d_result, sizeof(ScalarF4E4));

    int passed = 0;
    for (int i = 0; i < num_tests; i++) {
        UnaryTestCase *test = &tests[i];

        // Copy to GPU
        hipMemcpy(d_a, &test->a, sizeof(ScalarF4E4), hipMemcpyHostToDevice);

        // Run kernel
        kernel(d_a, d_result, 1);

        // Copy result
        ScalarF4E4 result;
        hipMemcpy(&result, d_result, sizeof(ScalarF4E4), hipMemcpyDeviceToHost);

        // Check result
        bool pass = (result.fraction == test->expected.fraction &&
                     result.exponent == test->expected.exponent);

        if (pass) {
            printf("  ✓ %s\n", test->name);
            passed++;
        } else {
            printf("  ✗ %s\n", test->name);
            printf("    Input:    (%d, %d)\n",
                   test->a.fraction, test->a.exponent);
            printf("    Expected: (%d, %d)\n",
                   test->expected.fraction, test->expected.exponent);
            printf("    Got:      (%d, %d)\n",
                   result.fraction, result.exponent);
        }
    }

    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    printf("Passed: %d/%d\n", passed, num_tests);

    hipFree(d_a);
    hipFree(d_result);
}

int main()
{
    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║     ScalarF4E4 GPU Test Suite - All Operations              ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n");

    // Addition tests
    BinaryTestCase add_tests[] = {
        { "Same exponent", {20000, 5}, {10000, 5}, {30000, 5} },
        { "Zero result", {16384, 5}, {-16384, 5}, {0, AMBIGUOUS_EXPONENT} },
        { "Ambiguous input", {0, AMBIGUOUS_EXPONENT}, {10000, 5}, {0, AMBIGUOUS_EXPONENT} },
    };
    run_binary_tests("Addition", spirix_scalar_add_gpu, add_tests, 3);

    // Subtraction tests
    BinaryTestCase sub_tests[] = {
        { "Same exponent", {20000, 5}, {10000, 5}, {10000, 5} },
        { "Zero result", {16384, 5}, {16384, 5}, {0, AMBIGUOUS_EXPONENT} },
        { "Ambiguous input", {0, AMBIGUOUS_EXPONENT}, {10000, 5}, {0, AMBIGUOUS_EXPONENT} },
    };
    run_binary_tests("Subtraction", spirix_scalar_subtract_gpu, sub_tests, 3);

    // Multiplication tests
    BinaryTestCase mul_tests[] = {
        { "Normal multiply", {16384, 2}, {16384, 3}, {16384, 5} },
        { "Zero multiply", {0, 5}, {10000, 5}, {0, AMBIGUOUS_EXPONENT} },
        { "Ambiguous input", {0, AMBIGUOUS_EXPONENT}, {10000, 5}, {0, AMBIGUOUS_EXPONENT} },
    };
    run_binary_tests("Multiplication", spirix_scalar_multiply_gpu, mul_tests, 3);

    // Division tests
    BinaryTestCase div_tests[] = {
        { "Normal divide", {16384, 10}, {16384, 5}, {16384, 5} },
        { "Division by zero", {16384, 5}, {0, 5}, {0x7FFF, 127} },
        { "Zero dividend", {0, 5}, {16384, 5}, {0, AMBIGUOUS_EXPONENT} },
    };
    run_binary_tests("Division", spirix_scalar_divide_gpu, div_tests, 3);

    // Square root tests
    UnaryTestCase sqrt_tests[] = {
        { "Perfect square", {16384, 10}, {16384, 5} },
        { "Zero input", {0, 5}, {0, AMBIGUOUS_EXPONENT} },
        { "Negative input", {-16384, 5}, {0, AMBIGUOUS_EXPONENT} },
    };
    run_unary_tests("Square Root", spirix_scalar_sqrt_gpu, sqrt_tests, 3);

    printf("\n╔══════════════════════════════════════════════════════════════╗\n");
    printf("║                     All Tests Complete                       ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n\n");

    return 0;
}
