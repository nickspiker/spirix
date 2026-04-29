//! Optimizers for training neural networks
//!
//! Simple, clean implementations using Spirix.

use super::ops::scale;
use super::tensor::Tensor;
use core::ops::{Mul, Sub};

/// Stochastic Gradient Descent optimizer
///
/// Updates weights: w = w - learning_rate * gradient
pub struct SGD<T> {
    pub learning_rate: T,
}

impl<T: Clone> SGD<T> {
    pub fn new(learning_rate: T) -> Self {
        SGD { learning_rate }
    }

    /// Update parameters in place
    ///
    /// weights = weights - learning_rate * gradients
    pub fn step(&self, weights: &mut Tensor<T>, gradients: &Tensor<T>)
    where
        T: Mul<Output = T> + Sub<Output = T> + Clone,
    {
        assert_eq!(
            weights.shape, gradients.shape,
            "Weight and gradient shapes must match"
        );

        // Scale gradients by learning rate
        let update = scale(gradients, self.learning_rate.clone());

        // Subtract from weights
        for (w, g) in weights.data.iter_mut().zip(update.data.iter()) {
            *w = w.clone() - g.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScalarF4E4;

    #[test]
    fn test_sgd_step() {
        // Initial weights: [1.0, 2.0, 3.0]
        let mut weights = Tensor::new(
            vec![
                ScalarF4E4::from(1.0),
                ScalarF4E4::from(2.0),
                ScalarF4E4::from(3.0),
            ],
            vec![3],
        );

        // Gradients: [0.1, 0.2, 0.3]
        let gradients = Tensor::new(
            vec![
                ScalarF4E4::from(0.1),
                ScalarF4E4::from(0.2),
                ScalarF4E4::from(0.3),
            ],
            vec![3],
        );

        // Learning rate: 0.5
        let optimizer = SGD::new(ScalarF4E4::from(0.5));

        // Update: w = w - lr * g
        // Expected: [1.0 - 0.5*0.1, 2.0 - 0.5*0.2, 3.0 - 0.5*0.3] = [0.95, 1.9, 2.85]
        optimizer.step(&mut weights, &gradients);

        assert!((weights.data[0].to_f64() - 0.95).abs() < 0.01);
        assert!((weights.data[1].to_f64() - 1.9).abs() < 0.01);
        assert!((weights.data[2].to_f64() - 2.85).abs() < 0.01);
    }
}
