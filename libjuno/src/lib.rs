#[cfg(feature = "grammar")]
pub mod grammar;
#[cfg(feature = "grammar")]
pub use grammar::JunoParser;
#[cfg(feature = "grammar")]
pub use grammar::JunoParserRule as Rule;
#[cfg(feature = "grammar")]
pub use pest;

#[cfg(feature = "ast")]
pub mod parser;
#[cfg(feature = "ast")]
pub use parser::parse_program;

#[cfg(feature = "metair")]
pub mod metair;
#[cfg(feature = "metair")]
pub use metair::*;
#[cfg(feature = "metair")]
pub mod get_symbols;
#[cfg(feature = "irgen")]
pub mod ir;
#[cfg(feature = "irgen")]
pub use inkwell;
#[cfg(feature = "irgen")]
pub use ir::LLVMBackend;

#[cfg(feature = "compiler")]
pub mod compile;
#[cfg(feature = "compiler")]
pub use compile::*;

#[cfg(feature = "diagnostics")]
pub mod diagnostics;

pub mod builtin_registry;
pub use builtin_registry::*;
pub mod ast;
pub use phf;

// =======================
// IDs
// =======================

pub type SymbolId = String;
pub type StringId = u32;
pub type FunctionId = String;
pub type TypeId = String;
