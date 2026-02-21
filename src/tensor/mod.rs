//! Tensor operations for Spirix scalar types
//!
//! Clean, minimal tensor implementation with ZERO IEEE-754 baggage.
//! All operations use Spirix scalars (no denormals, no NaN, no infinity).

pub mod autograd;
pub mod nn;
pub mod ops;
pub mod optim;
pub mod tensor;

pub use autograd::{linear_backward, mse_loss, mse_loss_grad, relu_backward, LinearGradients};
pub use nn::{Linear, SimpleNet};
pub use ops::*;
pub use optim::SGD;
pub use tensor::Tensor;
