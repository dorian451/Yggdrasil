pub mod constantexpr;
pub mod literal;
pub mod variable;

use constantexpr::ConstantExpr;
use literal::Literal;
use variable::Variable;

#[cfg(feature = "discriminants")]
use strum::{Display, EnumDiscriminants, EnumMessage};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "discriminants",
    derive(EnumDiscriminants, EnumMessage),
    strum_discriminants(derive(Display))
)]
///Representation of a logic expression
pub enum Expr {
    #[cfg_attr(feature = "discriminants", strum(message = "P"))]
    Literal(Literal),

    #[cfg_attr(feature = "discriminants", strum(message = "x"))]
    Variable(Variable),

    #[cfg_attr(feature = "discriminants", strum(message = "⊤"))]
    Tautology,

    #[cfg_attr(feature = "discriminants", strum(message = "⊥"))]
    Contradiction,

    #[cfg_attr(feature = "discriminants", strum(message = "P(Q)"))]
    Predicate {
        pred: Literal,
        args: Vec<ConstantExpr>,
    },

    #[cfg_attr(feature = "discriminants", strum(message = "¬P"))]
    Not(Box<Expr>),

    #[cfg_attr(feature = "discriminants", strum(message = "P ∧ Q"))]
    And(Box<Expr>, Box<Expr>),

    #[cfg_attr(feature = "discriminants", strum(message = "P ∨ Q"))]
    Or(Box<Expr>, Box<Expr>),

    #[cfg_attr(feature = "discriminants", strum(message = "P ⊕ Q"))]
    Xor(Box<Expr>, Box<Expr>),

    #[cfg_attr(feature = "discriminants", strum(message = "P → Q"))]
    Conditional(Box<Expr>, Box<Expr>),

    #[cfg_attr(feature = "discriminants", strum(message = "P ↔ Q"))]
    Biconditional(Box<Expr>, Box<Expr>),

    #[cfg_attr(feature = "discriminants", strum(message = "∀x(...)"))]
    Universal {
        iter: Variable,
        expr: Box<Expr>,
    },

    #[cfg_attr(feature = "discriminants", strum(message = "∃x(...)"))]
    Existential {
        iter: Variable,
        expr: Box<Expr>,
    },

    #[cfg_attr(feature = "discriminants", strum(message = "a = b"))]
    ConstantValue(ConstantExpr),

    UnknownOperator {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },

    Invalid,
}
