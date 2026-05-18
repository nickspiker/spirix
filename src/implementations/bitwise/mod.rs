mod circle;
mod scalar;

#[derive(Clone, Copy)]
pub(crate) enum BitwiseOp {
    And,
    Or,
    Xor,
}
