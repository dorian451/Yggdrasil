use std::collections::HashSet;

use crate::{error::ValidationError, EngineResult};
use strum::{Display, EnumIter, EnumMessage, EnumString, IntoEnumIterator};
use yggdrasil_grammar::{Expr, ExprDiscriminants};

#[derive(Debug, Clone, Copy, EnumIter, EnumMessage, Display, EnumString)]
pub enum BranchRule {
    /// p | q
    Or,

    /// ~(p | q)
    Nand,

    /// p -> q
    Conditional,

    /// p <-> q
    Biconditional,

    /// ~(p <-> q)
    NotBiconditional,
}

impl BranchRule {
    /// Decomposes an [Expr] into two branches
    pub fn decompose(&self, expr: &Expr) -> EngineResult<(HashSet<Expr>, HashSet<Expr>)> {
        let expr = expr.simplify();
        match self {
            Self::Or => {
                if let Expr::Or { left, right, .. } = expr {
                    Ok((
                        HashSet::from([left.simplify().clone()]),
                        HashSet::from([right.simplify().clone()]),
                    ))
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Or,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
            Self::Nand => {
                if let Expr::Not { expr, .. } = expr {
                    if let Expr::And { left, right, .. } = expr.simplify() {
                        Ok((
                            HashSet::from([left.simplify().clone()]),
                            HashSet::from([right.simplify().clone()]),
                        ))
                    } else {
                        Err(ValidationError::InvalidStatementType(
                            ExprDiscriminants::And,
                            ExprDiscriminants::from(expr.simplify()),
                        ))?
                    }
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Not,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
            Self::Conditional => {
                if let Expr::Conditional { left, right, .. } = expr {
                    Ok((
                        HashSet::from([Expr::Not {
                            _token: (),
                            expr: Box::new(left.simplify().clone()),
                        }]),
                        HashSet::from([right.simplify().clone()]),
                    ))
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Conditional,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
            Self::Biconditional => {
                if let Expr::Biconditional { left, right, .. } = expr {
                    Ok((
                        HashSet::from([left.simplify().clone(), right.simplify().clone()]),
                        HashSet::from([
                            Expr::Not {
                                _token: (),
                                expr: Box::new(left.simplify().clone()),
                            },
                            Expr::Not {
                                _token: (),
                                expr: Box::new(right.simplify().clone()),
                            },
                        ]),
                    ))
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Biconditional,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
            Self::NotBiconditional => {
                if let Expr::Not { expr, .. } = expr {
                    if let Expr::Biconditional { left, right, .. } = expr.simplify() {
                        Ok((
                            HashSet::from([
                                left.simplify().clone(),
                                Expr::Not {
                                    _token: (),
                                    expr: Box::new(right.simplify().clone()),
                                },
                            ]),
                            HashSet::from([
                                Expr::Not {
                                    _token: (),
                                    expr: Box::new(left.simplify().clone()),
                                },
                                right.simplify().clone(),
                            ]),
                        ))
                    } else {
                        Err(ValidationError::InvalidStatementType(
                            ExprDiscriminants::Biconditional,
                            ExprDiscriminants::from(expr.simplify()),
                        ))?
                    }
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Not,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
        }
    }
}
