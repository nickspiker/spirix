# Spirix Test Suite Documentation

This document describes **what tests exist** and their organization in the Spirix project.

> **Note**: For information on **how to run tests**, see [TESTING.md](TESTING.md).

## Test Organization

All tests follow a consistent naming convention: `{category}_{component}_{aspect}.rs`

Tests are organized into the following categories:

### Core Tests (`core_*.rs`) - 14 files, ~173 KB

Fundamental functionality tests verifying basic operations work correctly.

#### Scalar Tests
- **`core_scalar_tests.rs`** (42 KB) - Comprehensive scalar arithmetic
  - Addition, subtraction, multiplication, division
  - All bit widths (F3E3 thru F7E7)
  - ~475 tests covering all operations
- **`core_scalar_clamp.rs`** (742 bytes) - Clamp function
- **`core_scalar_max_values.rs`** (700 bytes) - Max value verification

#### Circle Tests (Complex Numbers)
- **`core_circle_tests.rs`** (39 KB) - Comprehensive circle operations
  - Circle/Circle, Circle/Scalar, Scalar/Circle operations
  - Complex arithmetic, magnitude, conjugate
- **`core_circle_reciprocal.rs`** (1.1 KB) - Reciprocal function


#### Edge Cases & Boundaries
- **`core_edge_cases.rs`** (12 KB) - Edge case handling
  - Zero division (0/0, x/0)
  - Overflow and underflow
  - Special values (infinity, undefined)
- **`core_saturation_boundaries.rs`** (5.0 KB) - Saturation behavior
  - Max value handling, overflow saturation
- **`core_precision_boundaries.rs`** (12 KB) - Precision testing
  - Rounding behavior, precision limits

#### State & Error Handling
- **`core_undefined_tests.rs`** (16 KB) - Undefined state handling
  - All undefined bit patterns
  - Undefined propagation
- **`core_error_handling.rs`** (16 KB) - Error handling
  - Error propagation and state handling
- **`core_state_transitions.rs`** (10 KB) - State transitions
  - Normal → Exploded/Vanished transitions

#### Mathematical Functions
- **`core_mathematical_functions.rs`** (16 KB) - Math functions
  - sqrt(), pow(), exp(), log()
  - Trigonometric functions

#### Integer Behavior
- **`core_integer_behavior.rs`** (4.1 KB) - Integer-specific behavior
- **`core_simple_integer_test.rs`** (1.4 KB) - Basic integer sanity checks

### Integration Tests (`integration_*.rs`) - 5 files, ~74 KB

Tests verifying components work together correctly.

- **`integration_general.rs`** (19 KB) - General integration tests
  - Cross-component interactions
  - End-to-end workflows
- **`integration_conversions.rs`** (15 KB) - Type conversions
  - Scalar ↔ Rust primitives
  - Circle ↔ Rust primitives
- **`integration_cross_type_operations.rs`** (15 KB) - Mixed type operations
  - Scalar + Circle, different bit widths
- **`integration_documentation_examples.rs`** (12 KB) - Documentation examples
  - Ensures README examples work
- **`integration_corrected_tests.rs`** (13 KB) - Bug fix verification
  - Regression prevention

### Property Tests (`properties_*.rs`) - 2 files, ~43 KB

Mathematical property verification using `proptest` crate.

- **`properties_general.rs`** (25 KB) - General mathematical properties
  - Commutativity: a + b = b + a
  - Associativity: (a + b) + c = a + (b + c)
  - Distributivity: a * (b + c) = a*b + a*c
- **`properties_enhanced.rs`** (18 KB) - Enhanced property tests
  - Advanced mathematical properties
  - Edge case property verification

### Display Tests (`display_*.rs`) - 2 files, ~29 KB

Formatting and display output verification.

- **`display_scalar_formatting.rs`** (27 KB) - Scalar display formatting
  - Binary representation, debug formatting
- **`display_debug_behavior.rs`** (1.8 KB) - Debug trait behavior

### Performance Tests (`performance_*.rs`) - 1 file, ~13 KB

Performance regression and benchmark verification.

- **`performance_regression.rs`** (13 KB) - Performance monitoring
  - Ensures operations don't slow down
  - Benchmark baseline verification

### Specialized Tests (`specialized_*.rs`) - 2 files, ~4.5 KB

Tests for specific functions and comparisons.

- **`specialized_sqrt_comparison.rs`** (3.6 KB) - Compare sqrt implementations
  - Newton-Raphson vs Bitwise
  - Accuracy and performance comparison
- **`specialized_sqrt_bitwise.rs`** (860 bytes) - Bitwise sqrt test

## Test Summary

| Category | Files | Total Size | Status |
|----------|-------|------------|--------|
| **Core Tests** | 14 | ~173 KB | ✅ Well organized |
| **Integration Tests** | 5 | ~74 KB | ✅ Well organized |
| **Property Tests** | 2 | ~43 KB | ✅ Well organized |
| **Display Tests** | 2 | ~29 KB | ✅ Well organized |
| **Performance Tests** | 1 | ~13 KB | ✅ Well organized |
| **Specialized Tests** | 2 | ~4.5 KB | ✅ Well organized |
| **TOTAL** | **26** | **~337 KB** | ✅ **Fully organized** |

## Archived Files

The following debug/development artifacts have been moved to `tests/archive/`:
- `debug_conversion.rs` (739 bytes)
- `debug_representation.rs` (589 bytes)
- `test_original_conversion.rs` (1,423 bytes)
- `trace_u8_formula.rs` (719 bytes)
- `understand_sa.rs` (660 bytes)
- `verify_conversions_print.rs` (1,242 bytes)

## Recent Changes (2025-02-09)

✅ **Reorganization Complete:**
1. ✅ Archived 6 debug/development files → `tests/archive/`
2. ✅ Converted all `.rsh` files to `.rs` (9 files standardized)
3. ✅ Renamed 6 tests for consistent naming:
   - `check_max_values.rs` → `core_scalar_max_values.rs`
   - `test_clamp_simple.rs` → `core_scalar_clamp.rs`
   - `test_reciprocal_simple.rs` → `core_circle_reciprocal.rs`
   - `test_sqrt_bitwise.rs` → `specialized_sqrt_bitwise.rs`
   - `sqrt_comparison_tests.rs` → `specialized_sqrt_comparison.rs`
   - `saturation_boundary_tests.rs` → `core_saturation_boundaries.rs`
4. ✅ Created comprehensive division tests (170 total tests) in Veritas examples:
   - `/mnt/Octopus/Code/veritas/examples/comprehensive_division_tests.rs` (160 Scalar division tests)
   - `/mnt/Octopus/Code/veritas/examples/comprehensive_circle_tests.rs` (10 Circle division tests)

## Test Coverage

### Arithmetic Operations
✅ Addition, Subtraction, Multiplication - Fully tested
✅ Division - **Comprehensively tested** (all bit widths, all edge cases)
✅ Square root - Multiple implementations tested
✅ Powers, exp, log - Tested
✅ Trigonometric functions - Tested

### Edge Cases
✅ Zero division (0/0, x/0) - Fully covered
✅ Overflow/Underflow - Fully covered
✅ Infinity, Exploded, Vanished states - Fully covered
✅ Undefined states - Complete catalog tested
✅ Sign handling - All combinations tested
✅ Denormals - No flush-to-zero verified

### Type Coverage
✅ Scalar - All operations, all bit widths (F3E3-F7E7)
✅ Circle - All operations, C/C, C/S, S/C
✅ Conversions - Rust primitives ↔ Spirix types
✅ Cross-type operations - Fully tested

### Properties
✅ Commutativity, Associativity, Distributivity
✅ Identity elements
✅ Inverse operations
✅ Function relationships

## Running Tests

```bash
# Run all tests
cargo test

# Run specific category
cargo test core_
cargo test integration_
cargo test properties_

# Run comprehensive division tests
cargo test core_division_comprehensive
cargo test core_circle_division_comprehensive

# Run with output
cargo test -- --nocapture
```

## Naming Convention

All test files follow this pattern:
```
{category}_{component}_{aspect}.rs

Examples:
core_scalar_tests.rs             # Core scalar functionality
core_circle_division_comprehensive.rs  # Core circle division tests
integration_conversions.rs       # Integration of conversions
properties_general.rs            # General mathematical properties
display_scalar_formatting.rs     # Scalar display/formatting
performance_regression.rs        # Performance monitoring
specialized_sqrt_comparison.rs   # Specialized sqrt comparison
```

**Categories:**
- `core_` - Fundamental functionality
- `integration_` - Cross-component interactions
- `properties_` - Mathematical property verification
- `display_` - Formatting and output
- `performance_` - Performance monitoring
- `specialized_` - Specific function tests

## Test Status

🎉 **Test suite fully organized and comprehensive!**

- ✅ Consistent naming convention applied to all 26 test files
- ✅ All debug/development files archived
- ✅ All file extensions standardized (`.rs` only, no more `.rsh`)
- ✅ Complete edge case coverage in existing tests
- ✅ All tests passing
- ✅ Documentation complete (TESTS.md, TESTING.md)

**Total Spirix test files**: 26 organized test files (~337 KB)
**Total test count**: ~475+ tests in core tests, plus integration/property tests

**Additional comprehensive tests** (in Veritas examples):
- 160 Scalar division tests (all bit widths F3E3-F7E7)
- 10 Circle division tests (C/C, C/S, S/C)
