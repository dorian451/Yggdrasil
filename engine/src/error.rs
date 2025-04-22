use thiserror::Error;
use yggdrasil_grammar::expr::ExprDiscriminants;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    #[error("Statement is not well formed: {0}")]
    ValidationError(ValidationError),

    #[error("This feature (\"{0}\") isn't supported yet")]
    NotSupported(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error(
        "Variable {0} not declared in an enclosing universal (∀) or existential (∃) statement"
    )]
    InvalidVariable(String),

    #[error("Expected statement of type {0}, found {1}")]
    InvalidStatementType(ExprDiscriminants, ExprDiscriminants),
}

impl From<ValidationError> for EngineError {
    fn from(value: ValidationError) -> Self {
        EngineError::ValidationError(value)
    }
}
