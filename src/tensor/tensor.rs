//! Tensor data structure
//!
//! Generic over Spirix scalar types (ScalarF4E4, ScalarF6E5, etc.)
//! No IEEE-754, no special cases, just clean math.

use alloc::vec::Vec;
use core::ops::{Add, Mul, Sub};

/// A multi-dimensional array of Spirix scalars
///
/// Shape is stored as Vec<usize> for dynamic dimensions.
/// Data is stored in row-major order (C-style).
#[derive(Debug, Clone)]
pub struct Tensor<T> {
    /// Flattened data in row-major order
    pub data: Vec<T>,
    /// Shape of the tensor (e.g., [2, 3, 4] for 2×3×4 tensor)
    pub shape: Vec<usize>,
}

impl<T: Clone> Tensor<T> {
    /// Create a new tensor with given shape and data
    pub fn new(data: Vec<T>, shape: Vec<usize>) -> Self {
        let expected_size: usize = shape.iter().product();
        assert_eq!(
            data.len(),
            expected_size,
            "Data length {} doesn't match shape {:?} (expected {})",
            data.len(),
            shape,
            expected_size
        );
        Tensor { data, shape }
    }

    /// Create a tensor filled with a single value
    pub fn fill(value: T, shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Tensor {
            data: vec![value; size],
            shape,
        }
    }

    /// Get the number of dimensions
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Get the total number of elements
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get element at index (1D only for now)
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.data.get(idx)
    }

    /// Get mutable element at index (1D only for now)
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.data.get_mut(idx)
    }

    /// Reshape tensor (no data copy, just change shape)
    pub fn reshape(&self, new_shape: Vec<usize>) -> Self {
        let new_size: usize = new_shape.iter().product();
        assert_eq!(
            self.size(),
            new_size,
            "Cannot reshape tensor of size {} to shape {:?} (size {})",
            self.size(),
            new_shape,
            new_size
        );
        Tensor {
            data: self.data.clone(),
            shape: new_shape,
        }
    }

    /// Check if this is a matrix (2D tensor)
    pub fn is_matrix(&self) -> bool {
        self.ndim() == 2
    }

    /// Get matrix dimensions (rows, cols) - panics if not 2D
    pub fn matrix_dims(&self) -> (usize, usize) {
        assert!(self.is_matrix(), "Tensor is not a matrix");
        (self.shape[0], self.shape[1])
    }
}

// Element-wise operations
impl<T> Add for Tensor<T>
where
    T: Add<Output = T> + Clone,
{
    type Output = Tensor<T>;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.shape, rhs.shape,
            "Shape mismatch: {:?} vs {:?}",
            self.shape, rhs.shape
        );

        let data: Vec<T> = self
            .data
            .into_iter()
            .zip(rhs.data.into_iter())
            .map(|(a, b)| a + b)
            .collect();

        Tensor {
            data,
            shape: self.shape,
        }
    }
}

impl<T> Sub for Tensor<T>
where
    T: Sub<Output = T> + Clone,
{
    type Output = Tensor<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.shape, rhs.shape,
            "Shape mismatch: {:?} vs {:?}",
            self.shape, rhs.shape
        );

        let data: Vec<T> = self
            .data
            .into_iter()
            .zip(rhs.data.into_iter())
            .map(|(a, b)| a - b)
            .collect();

        Tensor {
            data,
            shape: self.shape,
        }
    }
}

impl<T> Mul for Tensor<T>
where
    T: Mul<Output = T> + Clone,
{
    type Output = Tensor<T>;

    /// Element-wise multiplication (Hadamard product)
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.shape, rhs.shape,
            "Shape mismatch: {:?} vs {:?}",
            self.shape, rhs.shape
        );

        let data: Vec<T> = self
            .data
            .into_iter()
            .zip(rhs.data.into_iter())
            .map(|(a, b)| a * b)
            .collect();

        Tensor {
            data,
            shape: self.shape,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScalarF4E4;

    #[test]
    fn test_tensor_creation() {
        let data = vec![
            ScalarF4E4::from(1.0),
            ScalarF4E4::from(2.0),
            ScalarF4E4::from(3.0),
            ScalarF4E4::from(4.0),
        ];
        let tensor = Tensor::new(data, vec![2, 2]);

        assert_eq!(tensor.shape, vec![2, 2]);
        assert_eq!(tensor.size(), 4);
        assert!(tensor.is_matrix());
    }

    #[test]
    fn test_element_wise_add() {
        let a = Tensor::new(vec![ScalarF4E4::from(1.0), ScalarF4E4::from(2.0)], vec![2]);
        let b = Tensor::new(vec![ScalarF4E4::from(3.0), ScalarF4E4::from(4.0)], vec![2]);

        let c = a + b;

        assert_eq!(c.data[0].to_f64(), 4.0);
        assert_eq!(c.data[1].to_f64(), 6.0);
    }
}
