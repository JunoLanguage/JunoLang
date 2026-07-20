use anyhow::Error;

use crate::{ast::JunoSpan, *};
use std::fmt;
#[derive(Debug)]
pub enum LLVMError {
    UnknownVariable(SymbolId),
    UnknownFunction(SymbolId),
    UnknownType(SymbolId),

    InvalidExpression,

    Message(String),

    SpanMessage(String, JunoSpan, String, String),
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
            LLVMError::SpanMessage(msg, span, source, source_file_name) => {
                write!(f, "{:?}", span.err_to_report(&msg, source.clone(), &source_file_name))
            }
        }
    }
}

impl<'ctx> LLVMBackend<'ctx> {
    pub fn make_span_error(&self, msg: String, span: JunoSpan) -> LLVMError {
        LLVMError::SpanMessage(msg, span, self.source_code.clone(), self.source_file_name.clone())
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
