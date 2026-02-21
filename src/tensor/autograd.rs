//! Simple autograd for backpropagation
//!
//! Manual gradient computation for our small network.
//! For production, we'd build a full computation graph,
//! but for now, manual gradients are clean and explicit.

use super::ops::{matmul, transpose};
use super::tensor::Tensor;
use std::ops::{Add, Mul, Sub};

/// Compute gradient of loss with respect to weights
///
/// For a linear layer: y = W·x + b
/// Gradient of loss L wrt W: dL/dW = dL/dy · x^T
/// Gradient of loss wrt b: dL/db = dL/dy
/// Gradient of loss wrt x: dL/dx = W^T · dL/dy
pub struct LinearGradients<T> {
    pub weight_grad: Tensor<T>,
    pub bias_grad: Tensor<T>,
    pub input_grad: Tensor<T>,
}

/// Compute gradients for linear layer
///
/// Given:
/// - output_grad: gradient flowing back (dL/dy)
/// - input: input that was fed forward
/// - weights: weight matrix
/// - zero: zero value (e.g., ScalarF4E4::ZERO)
///
/// Returns: gradients for weights, bias, and input
pub fn linear_backward<T>(
    output_grad: &Tensor<T>,
    input: &Tensor<T>,
    weights: &Tensor<T>,
    zero: T,
) -> LinearGradients<T>
where
    T: Add<Output = T> + Mul<Output = T> + Clone,
{
    // dL/dW = dL/dy · x^T
    let input_t = transpose(input);
    let weight_grad = matmul(output_grad, &input_t, zero.clone());

    // dL/db = sum of dL/dy over batch (for now, just copy output_grad)
    let bias_grad = output_grad.clone();

    // dL/dx = W^T · dL/dy
    let weights_t = transpose(weights);
    let input_grad = matmul(&weights_t, output_grad, zero);

    LinearGradients {
        weight_grad,
        bias_grad,
        input_grad,
    }
}

/// Compute gradient for ReLU activation
///
/// ReLU: y = max(0, x)
/// Gradient: dy/dx = 1 if x > 0, else 0
///
/// Given:
/// - output_grad: gradient flowing back (dL/dy)
/// - input: input that was fed forward (x)
/// - zero: zero value
///
/// Returns: gradient wrt input (dL/dx = dL/dy · dy/dx)
pub fn relu_backward<T>(output_grad: &Tensor<T>, input: &Tensor<T>, zero: T) -> Tensor<T>
where
    T: Mul<Output = T> + Clone + PartialOrd,
{
    assert_eq!(
        output_grad.shape, input.shape,
        "Shape mismatch in ReLU backward"
    );

    let grad_data: Vec<T> = output_grad
        .data
        .iter()
        .zip(input.data.iter())
        .map(|(g, x)| {
            if x > &zero {
                g.clone() // Gradient passes through
            } else {
                zero.clone() // Gradient is zero
            }
        })
        .collect();

    Tensor::new(grad_data, output_grad.shape.clone())
}

/// Mean Squared Error loss
///
/// MSE = (1/n) * sum((predicted - target)^2)
pub fn mse_loss<T>(predicted: &Tensor<T>, target: &Tensor<T>) -> T
where
    T: Sub<Output = T> + Mul<Output = T> + Add<Output = T> + Clone,
{
    assert_eq!(predicted.shape, target.shape, "Shape mismatch in MSE loss");

    let mut sum = target.data[0].clone() - target.data[0].clone(); // Clever way to get zero

    for (p, t) in predicted.data.iter().zip(target.data.iter()) {
        let diff = p.clone() - t.clone();
        sum = sum + (diff.clone() * diff);
    }

    sum
}

/// Gradient of MSE loss
///
/// dL/dy = (2/n) * (predicted - target)
pub fn mse_loss_grad<T>(predicted: &Tensor<T>, target: &Tensor<T>) -> Tensor<T>
where
    T: Sub<Output = T> + Mul<Output = T> + Clone,
{
    assert_eq!(predicted.shape, target.shape, "Shape mismatch in MSE grad");

    // For simplicity, we're not dividing by n (or multiplying by 2/n)
    // The learning rate will absorb this constant
    let grad_data: Vec<T> = predicted
        .data
        .iter()
        .zip(target.data.iter())
        .map(|(p, t)| p.clone() - t.clone())
        .collect();

    Tensor::new(grad_data, predicted.shape.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScalarF4E4;

    #[test]
    fn test_linear_backward() {
        // Simple 2×2 weight matrix
        let weights = Tensor::new(
            vec![
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(3.0),
                ScalarF4E4::from(4.0),
            ],
            vec![2, 2],
        );

        // Input: [1, 2]
        let input = Tensor::new(
            vec![ScalarF4E4::from(1.0), ScalarF4E4::from(2.0)],
            vec![2, 1],
        );

        // Output gradient: [1, 1]
        let output_grad = Tensor::new(
            vec![ScalarF4E4::from(1.0), ScalarF4E4::from(1.0)],
            vec![2, 1],
        );

        let grads = linear_backward(&output_grad, &input, &weights, ScalarF4E4::ZERO);

        // Check shapes
        assert_eq!(grads.weight_grad.shape, weights.shape);
        assert_eq!(grads.input_grad.shape, input.shape);
    }

    #[test]
    fn test_relu_backward() {
        // Input: [-1, 2, -3, 4]
        let input = Tensor::new(
            vec![
                ScalarF4E4::from(-1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(-3.0),
                ScalarF4E4::from(4.0),
            ],
            vec![4],
        );

        // Output grad: [1, 1, 1, 1]
        let output_grad = Tensor::new(
            vec![
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(1.0),
            ],
            vec![4],
        );

        let input_grad = relu_backward(&output_grad, &input, ScalarF4E4::ZERO);

        // Expected: [0, 1, 0, 1] (gradient only passes where input > 0)
        assert_eq!(input_grad.data[0].to_f64(), 0.0);
        assert_eq!(input_grad.data[1].to_f64(), 1.0);
        assert_eq!(input_grad.data[2].to_f64(), 0.0);
        assert_eq!(input_grad.data[3].to_f64(), 1.0);
    }

    #[test]
    fn test_mse_loss_grad() {
        // Predicted: [2, 3]
        let predicted = Tensor::new(vec![ScalarF4E4::from(2.0), ScalarF4E4::from(3.0)], vec![2]);

        // Target: [1, 2]
        let target = Tensor::new(vec![ScalarF4E4::from(1.0), ScalarF4E4::from(2.0)], vec![2]);

        let grad = mse_loss_grad(&predicted, &target);

        // Expected: [2-1, 3-2] = [1, 1]
        assert_eq!(grad.data[0].to_f64(), 1.0);
        assert_eq!(grad.data[1].to_f64(), 1.0);
    }
}
