use crate::{
    error::{EngineError, ValidationError},
    EngineResult,
};
use std::collections::HashSet;
use yggdrasil_grammar::{Expr, Variable};

pub fn validate_syntax(expr: &Expr) -> EngineResult {
    validate(expr, Default::default())
}

fn validate(expr: &Expr, current_vars: HashSet<Variable>) -> EngineResult {
    match expr.simplify() {
        Expr::Literal(_) => Ok(()),
        Expr::Generic(_) => Err(EngineError::NotSupported("Generics".to_string())),

        Expr::Variable(variable) => {
            if current_vars.contains(variable) {
                Ok(())
            } else {
                Err(ValidationError::InvalidVariable(variable.var.clone()))?
            }
        }

        Expr::Tautology(_) => Ok(()),
        Expr::Contradiction(_) => Ok(()),
        Expr::Group { expr, .. } => validate(expr, current_vars),
        Expr::Group2 { expr, .. } => validate(expr, current_vars),
        Expr::Group3 { expr, .. } => validate(expr, current_vars),

        Expr::Function { args, .. } => args
            .iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Predicate { args, .. } => args
            .iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Not { expr, .. } => validate(expr, current_vars),

        Expr::And { left, right, .. } => [left, right]
            .into_iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Or { left, right, .. } => [left, right]
            .into_iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Xor { left, right, .. } => [left, right]
            .into_iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Conditional { left, right, .. } => [left, right]
            .into_iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Biconditional { left, right, .. } => [left, right]
            .into_iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),

        Expr::Universal { iter, expr, .. } => {
            let mut new_vars = current_vars.clone();
            new_vars.insert(iter.clone());
            validate(expr, new_vars)
        }

        Expr::Existential { iter, expr, .. } => {
            let mut new_vars = current_vars.clone();
            new_vars.insert(iter.clone());
            validate(expr, new_vars)
        }

        Expr::UnknownOperator { left, right, .. } => [left, right]
            .into_iter()
            .try_for_each(|expr| validate(expr, current_vars.clone())),
    }
}
