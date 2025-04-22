use std::collections::HashSet;

use crate::{error::ValidationError, EngineResult};
use strum::{Display, EnumIter, EnumMessage, EnumString};
use yggdrasil_grammar::expr::{Expr, ExprDiscriminants};

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

type ExprSet = HashSet<Box<Expr>>;

impl BranchRule {
    /// Decomposes an [Expr] into two branches
    pub fn decompose(&self, expr: &Expr) -> EngineResult<(ExprSet, ExprSet)> {
        match self {
            Self::Or => {
                if let Expr::Or(left, right) = expr {
                    Ok((
                        HashSet::from([left.clone()]),
                        HashSet::from([right.clone()]),
                    ))
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Or,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
            Self::Nand => {
                if let Expr::Not(expr) = expr {
                    if let Expr::And(left, right) = expr.as_ref() {
                        Ok((
                            HashSet::from([left.clone()]),
                            HashSet::from([right.clone()]),
                        ))
                    } else {
                        Err(ValidationError::InvalidStatementType(
                            ExprDiscriminants::And,
                            ExprDiscriminants::from(expr.as_ref()),
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
                if let Expr::Conditional(left, right) = expr {
                    Ok((
                        HashSet::from([Box::new(Expr::Not(left.clone()))]),
                        HashSet::from([right.clone()]),
                    ))
                } else {
                    Err(ValidationError::InvalidStatementType(
                        ExprDiscriminants::Conditional,
                        ExprDiscriminants::from(expr),
                    ))?
                }
            }
            Self::Biconditional => {
                if let Expr::Biconditional(left, right) = expr {
                    Ok((
                        HashSet::from([left.clone(), right.clone()]),
                        HashSet::from([
                            Box::new(Expr::Not(left.clone())),
                            Box::new(Expr::Not(right.clone())),
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
                if let Expr::Not(expr) = expr {
                    if let Expr::Biconditional(left, right) = expr.as_ref() {
                        Ok((
                            HashSet::from([left.clone(), Box::new(Expr::Not(right.clone()))]),
                            HashSet::from([Box::new(Expr::Not(left.clone())), right.clone()]),
                        ))
                    } else {
                        Err(ValidationError::InvalidStatementType(
                            ExprDiscriminants::Biconditional,
                            ExprDiscriminants::from(expr.as_ref()),
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
