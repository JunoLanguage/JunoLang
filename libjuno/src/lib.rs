
#[cfg(feature = "grammar")]
mod grammar;
#[cfg(feature = "grammar")]
pub use grammar::JunoParser;
#[cfg(feature = "grammar")]
pub use pest;
#[cfg(feature = "grammar")]
pub use grammar::JunoParserRule as Rule;


#[cfg(feature = "ast")]
mod parser;
#[cfg(feature = "ast")]
pub use parser::parse_program;

#[cfg(feature = "ast")]
pub mod ast;


#[cfg(feature = "metair")]
mod metair;
#[cfg(feature = "grammar")]
pub use metair::*;


#[cfg(feature = "irgen")]
mod ir;
#[cfg(feature = "irgen")]
pub use inkwell;
#[cfg(feature = "irgen")]
pub use ir::LLVMBackend;


#[cfg(feature = "compiler")]
mod compile;
#[cfg(feature = "compiler")]
pub use compile::*;


mod builtin_registry;
pub use builtin_registry::*;
pub use phf;