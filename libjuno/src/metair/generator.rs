use std::collections::HashMap;

use crate::ast::{JunoSpan, Program};
use crate::metair::metair::*;

/// Generator state used while lowering the AST into MetaIR.
///
/// The lowering logic itself is split across the other modules.
pub struct MetaIRGen<'a> {
    pub program: &'a Program,
    pub source_code: String,
    pub source_file_name: String,
    /// struct_id -> (field_name -> field_index)
    pub struct_fields: HashMap<SymbolId, HashMap<String, u32>>,

    /// String interning.
    pub strings: HashMap<String, StringId>,

    /// All visible declarations (functions + extern declarations).
    pub declarations: HashMap<String, MetaDeclaration>,

    /// Interned symbol table.
    pub symbol_list: Vec<String>,

    /// Interned string table.
    pub string_list: Vec<String>,

    /// Local scopes.
    ///
    /// The last element is always the current scope.
    pub locals: Vec<HashMap<SymbolId, MetaType>>,

    /// Struct definitions by name.
    pub structs: HashMap<String, MetaStruct>,

    pub(crate) next_string: u32,
    pub(crate) next_struct_field: u32,
    pub(crate) counter: u32,
}

impl<'a> MetaIRGen<'a> {
    pub fn new(program: &'a Program, source_code: String, source_file_name: String) -> Self {
        Self {
            program,
            source_code,
            source_file_name,
            struct_fields: HashMap::new(),
            strings: HashMap::new(),
            declarations: HashMap::new(),
            symbol_list: Vec::new(),
            string_list: Vec::new(),
            locals: Vec::new(),
            structs: HashMap::new(),

            next_string: 0,
            next_struct_field: 0,
            counter: 0,
        }
    }

    pub fn make_span_error(&self, msg: &str, span: JunoSpan) -> miette::Error {
        miette::Error::from(span.err_to_report(msg, self.source_code.clone(), &self.source_file_name))
    }
}

impl MetaProgram {
    pub fn get_struct(&self, name: SymbolId) -> Option<&MetaStruct> {
        self.structs.get(&name)
    }
}