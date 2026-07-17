use crate::{ast::JunoSpan, *};
use std::fmt;
#[derive(Debug)]
pub enum LLVMError {
    UnknownVariable(SymbolId),
    UnknownFunction(SymbolId),
    UnknownType(SymbolId),

    InvalidExpression,

    Message(String),

    SpanMessage(String, JunoSpan),
}

impl fmt::Display for LLVMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLVMError::UnknownVariable(id) => {
                write!(f, "unknown variable {id}")
            }

            LLVMError::UnknownFunction(id) => {
                write!(f, "unknown function {id}")
            }

            LLVMError::UnknownType(id) => {
                write!(f, "unknown type {id}")
            }

            LLVMError::InvalidExpression => {
                write!(f, "invalid expression")
            }

            LLVMError::Message(msg) => {
                write!(f, "{msg}")
            }
            LLVMError::SpanMessage(msg, span) => {
                write!(f, "{:?}", span.err_to_report(msg))
            }
        }
    }
}

impl std::error::Error for LLVMError {}

impl From<inkwell::Error> for LLVMError {
    fn from(value: inkwell::Error) -> Self {
        Self::Message(format!("{:?}", value))
    }
}
impl From<inkwell::builder::BuilderError> for LLVMError {
    fn from(value: inkwell::builder::BuilderError) -> Self {
        Self::Message(format!("{:?}", value))
    }
}
