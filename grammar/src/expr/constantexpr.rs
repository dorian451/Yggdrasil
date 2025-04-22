use super::variable::Variable;

#[cfg(feature = "discriminants")]
use strum::{Display, EnumDiscriminants, EnumMessage};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "discriminants",
    derive(EnumDiscriminants, EnumMessage),
    strum_discriminants(derive(Display))
)]
pub enum ConstantExpr {
    #[cfg_attr(feature = "discriminants", strum(message = "a"))]
    Constant(Constant),

    #[cfg_attr(feature = "discriminants", strum(message = "x"))]
    Variable(Variable),

    #[cfg_attr(feature = "discriminants", strum(message = "0"))]
    Number(isize),

    #[cfg_attr(feature = "discriminants", strum(message = "a(b)"))]
    Function {
        func: Constant,
        args: Vec<ConstantExpr>,
    },

    #[cfg_attr(feature = "discriminants", strum(message = "a + b"))]
    Operator(String, Box<ConstantExpr>, Box<ConstantExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constant(pub String);
