//! Tensor operations
//!
//! Matrix multiply, transpose, activations, etc. All operations are clean - no IEEE-754 edge cases.

use super::tensor::Tensor;
use alloc::vec::Vec;
use core::ops::{Add, Mul};

/// Matrix multiplication (A × B)
///
/// A: (m, n)
/// B: (n, p)
/// Result: (m, p)
///
/// `zero` parameter is used for initialization (e.g., ScalarF4E4::ZERO)
pub fn matmul<T>(a: &Tensor<T>, b: &Tensor<T>, zero: T) -> Tensor<T>
where
    T: Add<Output = T> + Mul<Output = T> + Clone,
{
    assert!(a.is_matrix(), "First argument must be a matrix");
    assert!(b.is_matrix(), "Second argument must be a matrix");

    let (m, n) = a.matrix_dims();
    let (n2, p) = b.matrix_dims();

    assert_eq!(
        n, n2,
        "Matrix dimensions don't match: ({}, {}) × ({}, {})",
        m, n, n2, p
    );

    // Allocate result matrix filled with zeros
    let mut result = vec![zero.clone(); m * p];

    // Standard matrix multiply: C[i,j] = sum(A[i,k] * B[k,j])
    for i in 0..m {
        for j in 0..p {
            let mut sum = zero.clone();
            for k in 0..n {
                let a_val = a.data[i * n + k].clone();
                let b_val = b.data[k * p + j].clone();
                sum = sum + (a_val * b_val);
            }
            result[i * p + j] = sum;
        }
    }

    Tensor::new(result, vec![m, p])
}

/// Transpose a matrix
pub fn transpose<T: Clone>(tensor: &Tensor<T>) -> Tensor<T> {
    assert!(tensor.is_matrix(), "Can only transpose 2D tensors");

    let (rows, cols) = tensor.matrix_dims();
    let mut result = Vec::with_capacity(rows * cols);

    // Transpose: result[j,i] = input[i,j]
    for j in 0..cols {
        for i in 0..rows {
            result.push(tensor.data[i * cols + j].clone());
        }
    }

    Tensor::new(result, vec![cols, rows])
}

/// ReLU activation: max(0, x)
///
/// `zero` parameter is the zero value for the type (e.g., ScalarF4E4::ZERO)
pub fn relu<T>(tensor: &Tensor<T>, zero: T) -> Tensor<T>
where
    T: Clone + PartialOrd,
{
    let data: Vec<T> = tensor
        .data
        .iter()
        .map(|x| if x > &zero { x.clone() } else { zero.clone() })
        .collect();

    Tensor::new(data, tensor.shape.clone())
}

/// Scalar multiplication (multiply all elements by scalar)
pub fn scale<T>(tensor: &Tensor<T>, scalar: T) -> Tensor<T>
where
    T: Mul<Output = T> + Clone,
{
    let data: Vec<T> = tensor
        .data
        .iter()
        .map(|x| x.clone() * scalar.clone())
        .collect();

    Tensor::new(data, tensor.shape.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScalarF4E4;

    #[test]
    fn test_matmul_2x2() {
        // A = [[1, 2], [3, 4]]
        let a = Tensor::new(
            vec![
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(3.0),
                ScalarF4E4::from(4.0),
            ],
            vec![2, 2],
        );

        // B = [[5, 6], [7, 8]]
        let b = Tensor::new(
            vec![
                ScalarF4E4::from(5.0),
                ScalarF4E4::from(6.0),
                ScalarF4E4::from(7.0),
                ScalarF4E4::from(8.0),
            ],
            vec![2, 2],
        );

        // C = A × B = [[19, 22], [43, 50]]
        let c = matmul(&a, &b, ScalarF4E4::ZERO);

        assert_eq!(c.data[0].to_f64(), 19.0); // 1*5 + 2*7
        assert_eq!(c.data[1].to_f64(), 22.0); // 1*6 + 2*8
        assert_eq!(c.data[2].to_f64(), 43.0); // 3*5 + 4*7
        assert_eq!(c.data[3].to_f64(), 50.0); // 3*6 + 4*8
    }

    #[test]
    fn test_transpose() {
        // A = [[1, 2, 3], [4, 5, 6]]
        let a = Tensor::new(
            vec![
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(3.0),
                ScalarF4E4::from(4.0),
                ScalarF4E4::from(5.0),
                ScalarF4E4::from(6.0),
            ],
            vec![2, 3],
        );

        // A^T = [[1, 4], [2, 5], [3, 6]]
        let at = transpose(&a);

        assert_eq!(at.shape, vec![3, 2]);
        assert_eq!(at.data[0].to_f64(), 1.0);
        assert_eq!(at.data[1].to_f64(), 4.0);
        assert_eq!(at.data[2].to_f64(), 2.0);
        assert_eq!(at.data[3].to_f64(), 5.0);
    }

    #[test]
    fn test_relu() {
        let a = Tensor::new(
            vec![
                ScalarF4E4::from(-1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(-3.0),
                ScalarF4E4::from(4.0),
            ],
            vec![4],
        );

        let r = relu(&a, ScalarF4E4::ZERO);

        assert_eq!(r.data[0].to_f64(), 0.0); // max(0, -1)
        assert_eq!(r.data[1].to_f64(), 2.0); // max(0, 2)
        assert_eq!(r.data[2].to_f64(), 0.0); // max(0, -3)
        assert_eq!(r.data[3].to_f64(), 4.0); // max(0, 4)
    }
}
