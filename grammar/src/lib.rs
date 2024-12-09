pub use grammar::{Expr, Generic, Literal, Variable};

#[cfg(feature = "discriminants")]
pub use grammar::ExprDiscriminants;

use rust_sitter::errors::ParseError;
use std::{
    boxed::Box,
    error::Error,
    hash::{Hash, Hasher},
};

#[rust_sitter::grammar("fol")]
mod grammar {
    #[cfg(feature = "discriminants")]
    use strum::{Display, EnumDiscriminants, EnumMessage};

    #[derive(Debug, Clone, Eq)]
    #[cfg_attr(
        feature = "discriminants",
        derive(EnumDiscriminants, EnumMessage),
        strum_discriminants(derive(Display))
    )]
    #[rust_sitter::language]
    ///Representation of a logic expression
    pub enum Expr {
        #[cfg_attr(feature = "discriminants", strum(message = "P"))]
        Literal(Literal),

        #[cfg_attr(feature = "discriminants", strum(message = "x"))]
        Variable(Variable),

        #[cfg_attr(feature = "discriminants", strum(message = "a"))]
        Generic(Generic),

        #[cfg_attr(feature = "discriminants", strum(message = "⊤"))]
        Tautology(#[rust_sitter::leaf(pattern = r"⊤|1")] ()),

        #[cfg_attr(feature = "discriminants", strum(message = "⊥"))]
        Contradiction(#[rust_sitter::leaf(pattern = r"⊥|0")] ()),

        Group {
            #[rust_sitter::leaf(pattern = r"\(")]
            _open_token: (),
            expr: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"\)")]
            _close_token: (),
        },

        Group2 {
            #[rust_sitter::leaf(pattern = r"\[")]
            _open_token: (),
            expr: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"\]")]
            _close_token: (),
        },

        Group3 {
            #[rust_sitter::leaf(pattern = r"\{")]
            _open_token: (),
            expr: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"\}")]
            _close_token: (),
        },

        #[cfg_attr(feature = "discriminants", strum(message = "p(Q)"))]
        #[rust_sitter::prec(10)]
        Function {
            func: Generic,
            #[rust_sitter::leaf(pattern = r"\(")]
            _open_token: (),
            #[rust_sitter::repeat(non_empty = true)]
            #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
            args: Vec<Expr>,
            #[rust_sitter::leaf(pattern = r"\)")]
            _close_token: (),
        },

        #[cfg_attr(feature = "discriminants", strum(message = "P(Q)"))]
        #[rust_sitter::prec(10)]
        Predicate {
            pred: Literal,
            #[rust_sitter::leaf(pattern = r"\(")]
            _open_token: (),
            #[rust_sitter::repeat(non_empty = true)]
            #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
            args: Vec<Expr>,
            #[rust_sitter::leaf(pattern = r"\)")]
            _close_token: (),
        },

        #[cfg_attr(feature = "discriminants", strum(message = "¬P"))]
        #[rust_sitter::prec(9)]
        Not {
            #[rust_sitter::leaf(pattern = r"¬|~|!")]
            _token: (),
            expr: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "P ∧ Q"))]
        #[rust_sitter::prec_left(8)]
        And {
            left: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"∧|\*|&")]
            _token: (),
            right: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "P ∨ Q"))]
        #[rust_sitter::prec_left(8)]
        Or {
            left: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"∨|\+|\|")]
            _token: (),
            right: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "P ⊕ Q"))]
        #[rust_sitter::prec_left(8)]
        Xor {
            left: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"⊕|(!∨)|(!\+)|(!\|)")]
            _token: (),
            right: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "P → Q"))]
        #[rust_sitter::prec_left(7)]
        Conditional {
            left: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"→|(->)")]
            _token: (),
            right: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "P ↔ Q"))]
        #[rust_sitter::prec_left(7)]
        Biconditional {
            left: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"↔|(<->)")]
            _token: (),
            right: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "∀x(...)"))]
        #[rust_sitter::prec(6)]
        Universal {
            #[rust_sitter::leaf(pattern = r"∀|@")]
            _token: (),
            iter: Variable,
            expr: Box<Expr>,
        },

        #[cfg_attr(feature = "discriminants", strum(message = "∃x(...)"))]
        #[rust_sitter::prec(6)]
        Existential {
            #[rust_sitter::leaf(pattern = r"∃|/")]
            _token: (),
            iter: Variable,
            expr: Box<Expr>,
        },

        #[rust_sitter::prec_left(5)]
        UnknownOperator {
            left: Box<Expr>,
            #[rust_sitter::leaf(pattern = r"[^\w\s⊤⊥\(\)\[\]\{\}¬→↔∀@∃]+", transform=|v| v.to_string())]
            operator: String,
            right: Box<Expr>,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Literal {
        #[rust_sitter::leaf(pattern = r"[A-Z][a-zA-Z0-9]*", transform = |x| x.to_string())]
        pub lit: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Generic {
        #[rust_sitter::leaf(pattern = r"[a-s][a-zA-Z0-9]*", transform = |x| x.to_string())]
        pub lit: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Variable {
        #[rust_sitter::leaf(pattern = r"[t-z][a-zA-Z0-9]*", transform = |x| x.to_string())]
        pub var: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

impl Expr {
    pub fn simplify(&self) -> &Self {
        match self {
            Self::Group { expr, .. } | Self::Group2 { expr, .. } | Self::Group3 { expr, .. } => {
                expr.simplify()
            }
            _ => self,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self.simplify(), other.simplify()) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (Self::Variable(l0), Self::Variable(r0)) => l0 == r0,
            (Self::Generic(l0), Self::Generic(r0)) => l0 == r0,
            (Self::Tautology(l0), Self::Tautology(r0))
            | (Self::Contradiction(l0), Self::Contradiction(r0)) => l0 == r0,
            (
                Self::Function {
                    func: l_func,

                    args: l_args,
                    ..
                },
                Self::Function {
                    func: r_func,

                    args: r_args,
                    ..
                },
            ) => l_func == r_func && l_args == r_args,
            (
                Self::Predicate {
                    pred: l_pred,
                    args: l_args,
                    ..
                },
                Self::Predicate {
                    pred: r_pred,
                    args: r_args,
                    ..
                },
            ) => l_pred == r_pred && l_args == r_args,
            (Self::Not { expr: l_expr, .. }, Self::Not { expr: r_expr, .. }) => l_expr == r_expr,
            (
                Self::And {
                    left: l_left,
                    right: l_right,
                    ..
                },
                Self::And {
                    left: r_left,
                    right: r_right,
                    ..
                },
            )
            | (
                Self::Or {
                    left: l_left,
                    right: l_right,
                    ..
                },
                Self::Or {
                    left: r_left,
                    right: r_right,
                    ..
                },
            )
            | (
                Self::Xor {
                    left: l_left,
                    right: l_right,
                    ..
                },
                Self::Xor {
                    left: r_left,
                    right: r_right,
                    ..
                },
            )
            | (
                Self::Conditional {
                    left: l_left,
                    right: l_right,
                    ..
                },
                Self::Conditional {
                    left: r_left,
                    right: r_right,
                    ..
                },
            )
            | (
                Self::Biconditional {
                    left: l_left,
                    right: l_right,
                    ..
                },
                Self::Biconditional {
                    left: r_left,
                    right: r_right,
                    ..
                },
            ) => l_left == r_left && l_right == r_right,
            (
                Self::Universal {
                    iter: l_iter,
                    expr: l_expr,
                    ..
                },
                Self::Universal {
                    iter: r_iter,
                    expr: r_expr,
                    ..
                },
            )
            | (
                Self::Existential {
                    iter: l_iter,
                    expr: l_expr,
                    ..
                },
                Self::Existential {
                    iter: r_iter,
                    expr: r_expr,
                    ..
                },
            ) => l_iter == r_iter && l_expr == r_expr,
            (
                Self::UnknownOperator {
                    left: l_left,
                    operator: l_operator,
                    right: l_right,
                },
                Self::UnknownOperator {
                    left: r_left,
                    operator: r_operator,
                    right: r_right,
                },
            ) => l_left == r_left && l_operator == r_operator && l_right == r_right,
            _ => false,
        }
    }
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let thing = self.simplify();
        core::mem::discriminant(thing).hash(state);
        match thing.simplify() {
            Expr::Literal(literal) => literal.hash(state),
            Expr::Variable(variable) => variable.hash(state),
            Expr::Generic(generic) => generic.hash(state),
            Expr::Tautology(_) | Expr::Contradiction(_) => (),
            Expr::Group { .. } | Expr::Group2 { .. } | Expr::Group3 { .. } => {
                unreachable!()
            }
            Expr::Function { func, args, .. } => {
                func.hash(state);
                args.hash(state);
            }
            Expr::Predicate { pred, args, .. } => {
                pred.hash(state);
                args.hash(state);
            }
            Expr::Not { expr, .. } => {
                expr.hash(state);
            }
            Expr::And { left, right, .. }
            | Expr::Or { left, right, .. }
            | Expr::Xor { left, right, .. }
            | Expr::Conditional { left, right, .. }
            | Expr::Biconditional { left, right, .. } => {
                left.hash(state);
                right.hash(state);
            }
            Expr::Universal { iter, expr, .. } | Expr::Existential { iter, expr, .. } => {
                iter.hash(state);
                expr.hash(state);
            }
            Expr::UnknownOperator {
                left,
                operator,
                right,
            } => {
                left.hash(state);
                operator.hash(state);
                right.hash(state);
            }
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, Vec<ParseError>> {
    Ok(grammar::parse(input)?)
}
