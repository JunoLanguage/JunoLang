//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::ast::{Function, Item, JunoSpan, Program};
use crate::metair::metair::*;

pub struct MetaIRGen<'a> {
    pub program: &'a Program,
    pub source_code: String,
    pub source_file_name: String,
    pub struct_fields: HashMap<SymbolId, HashMap<String, u32>>,

    pub strings: HashMap<String, StringId>,

    pub declarations: HashMap<String, MetaDeclaration>,

    pub symbol_list: Vec<String>,

    pub(crate) symbol_set: std::collections::HashSet<String>,

    pub string_list: Vec<String>,

    pub(crate) function_index: HashMap<String, &'a Function>,

    pub locals: Vec<HashMap<SymbolId, MetaType>>,

    pub structs: HashMap<String, MetaStruct>,

    pub(crate) next_string: u32,
    pub(crate) next_struct_field: u32,
}

impl<'a> MetaIRGen<'a> {
    pub fn new(program: &'a Program, source_code: String, source_file_name: String) -> Self {
        let function_index = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function, _) => Some((function.name.clone(), function)),
                _ => None,
            })
            .collect();

        Self {
            program,
            source_code,
            source_file_name,
            struct_fields: HashMap::new(),
            strings: HashMap::new(),
            declarations: HashMap::new(),
            symbol_list: Vec::new(),
            symbol_set: std::collections::HashSet::new(),
            string_list: Vec::new(),
            locals: Vec::new(),
            structs: HashMap::new(),
            function_index,

            next_string: 0,
            next_struct_field: 0,
        }
    }

    pub fn make_span_error(&self, msg: &str, span: JunoSpan) -> miette::Error {
        span.err_to_report(msg, self.source_code.clone(), &self.source_file_name)
    }
}

impl MetaProgram {
    pub fn get_struct(&self, name: SymbolId) -> Option<&MetaStruct> {
        self.structs.get(&name)
    }
}
