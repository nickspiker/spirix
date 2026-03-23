//! Neural network layers
//!
//! Clean implementations using Spirix tensors.
//! No IEEE-754 baggage, just straightforward math.

use super::ops::{matmul, relu};
use super::tensor::Tensor;
use core::ops::{Add, Mul};

/// Linear (fully connected) layer
///
/// y = W·x + b
/// where W is (output_size, input_size) and b is (output_size,)
pub struct Linear<T> {
    /// Weight matrix (output_size × input_size)
    pub weights: Tensor<T>,
    /// Bias vector (output_size,)
    pub bias: Tensor<T>,
}

impl<T> Linear<T>
where
    T: Add<Output = T> + Mul<Output = T> + Clone,
{
    /// Create a new linear layer with given dimensions
    pub fn new(weights: Tensor<T>, bias: Tensor<T>) -> Self {
        assert!(weights.is_matrix(), "Weights must be a matrix");
        assert_eq!(bias.ndim(), 1, "Bias must be a vector");
        assert_eq!(
            weights.shape[0], bias.shape[0],
            "Bias dimension must match output dimension"
        );

        Linear { weights, bias }
    }

    /// Forward pass: y = W·x + b
    ///
    /// `zero` is used for initialization (e.g., ScalarF4E4::ZERO)
    pub fn forward(&self, input: &Tensor<T>, zero: T) -> Tensor<T>
    where
        T: Add<Output = T> + Clone,
    {
        // W·x
        let mut output = matmul(&self.weights, input, zero);

        // Add bias to each element
        for (i, out_val) in output.data.iter_mut().enumerate() {
            let bias_idx = i % self.bias.shape[0];
            *out_val = out_val.clone() + self.bias.data[bias_idx].clone();
        }

        output
    }
}

/// Simple feedforward network
///
/// Input → Linear → ReLU → Linear → Output
pub struct SimpleNet<T> {
    pub layer1: Linear<T>,
    pub layer2: Linear<T>,
}

impl<T> SimpleNet<T>
where
    T: Add<Output = T> + Mul<Output = T> + Clone + PartialOrd,
{
    pub fn new(layer1: Linear<T>, layer2: Linear<T>) -> Self {
        SimpleNet { layer1, layer2 }
    }

    /// Forward pass through the network
    pub fn forward(&self, input: &Tensor<T>, zero: T) -> Tensor<T> {
        // Layer 1
        let h = self.layer1.forward(input, zero.clone());

        // ReLU activation
        let h = relu(&h, zero.clone());

        // Layer 2
        self.layer2.forward(&h, zero)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScalarF4E4;

    #[test]
    fn test_linear_layer() {
        // Weight matrix (2×3): [[1, 2, 3],
        //                       [4, 5, 6]]
        let weights = Tensor::new(
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

        // Bias vector: [0.5, 1.0]
        let bias = Tensor::new(vec![ScalarF4E4::from(0.5), ScalarF4E4::from(1.0)], vec![2]);

        let layer = Linear::new(weights, bias);

        // Input vector: [1, 2, 3]
        let input = Tensor::new(
            vec![
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(3.0),
            ],
            vec![3, 1],
        );

        // Forward pass
        let output = layer.forward(&input, ScalarF4E4::ZERO);

        // Expected: [1*1 + 2*2 + 3*3 + 0.5, 4*1 + 5*2 + 6*3 + 1.0]
        //         = [1 + 4 + 9 + 0.5, 4 + 10 + 18 + 1.0]
        //         = [14.5, 33.0]
        assert!((output.data[0].to_f64() - 14.5).abs() < 0.1);
        assert!((output.data[1].to_f64() - 33.0).abs() < 0.1);
    }

    #[test]
    fn test_simple_net() {
        // Create a tiny network: 2 → 3 → 2

        // Layer 1: 3×2 weights
        let w1 = Tensor::new(
            vec![
                ScalarF4E4::from(0.1),
                ScalarF4E4::from(0.2),
                ScalarF4E4::from(0.3),
                ScalarF4E4::from(0.4),
                ScalarF4E4::from(0.5),
                ScalarF4E4::from(0.6),
            ],
            vec![3, 2],
        );
        let b1 = Tensor::new(
            vec![
                ScalarF4E4::from(0.0),
                ScalarF4E4::from(0.0),
                ScalarF4E4::from(0.0),
            ],
            vec![3],
        );

        // Layer 2: 2×3 weights
        let w2 = Tensor::new(
            vec![
                ScalarF4E4::from(0.1),
                ScalarF4E4::from(0.2),
                ScalarF4E4::from(0.3),
                ScalarF4E4::from(0.4),
                ScalarF4E4::from(0.5),
                ScalarF4E4::from(0.6),
            ],
            vec![2, 3],
        );
        let b2 = Tensor::new(vec![ScalarF4E4::from(0.0), ScalarF4E4::from(0.0)], vec![2]);

        let layer1 = Linear::new(w1, b1);
        let layer2 = Linear::new(w2, b2);
        let net = SimpleNet::new(layer1, layer2);

        // Input: [1.0, 2.0]
        let input = Tensor::new(
            vec![ScalarF4E4::from(1.0), ScalarF4E4::from(2.0)],
            vec![2, 1],
        );

        // Forward pass
        let output = net.forward(&input, ScalarF4E4::ZERO);

        // Just check it runs without panicking and produces output
        assert_eq!(output.shape[0], 2);
    }
}
