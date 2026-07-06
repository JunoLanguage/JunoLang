use std::fmt;

#[derive(Debug)]
pub enum LLVMError {
    UnknownVariable(u32),
    UnknownFunction(u32),
    UnknownType(u32),

    InvalidExpression,

    Message(String),
}

impl fmt::Display for LLVMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLVMError::UnknownVariable(id) => { write!(f, "unknown variable {id}") }

            LLVMError::UnknownFunction(id) => { write!(f, "unknown function {id}") }

            LLVMError::UnknownType(id) => { write!(f, "unknown type {id}") }

            LLVMError::InvalidExpression => { write!(f, "invalid expression") }

            LLVMError::Message(msg) => { write!(f, "{msg}") }
        }
    }
}

impl std::error::Error for LLVMError {}
