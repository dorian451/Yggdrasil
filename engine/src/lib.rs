use error::EngineError;

pub mod error;
pub mod rules;
pub mod syntax;
pub mod util;

pub type EngineResult<T = ()> = Result<T, EngineError>;
