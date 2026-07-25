//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::Path;

use pest::Parser;

use crate::*;

#[derive(Clone, Debug)]
pub enum SymbolDecl {
    Function {
        name: String,
        params: Box<[MetaParam]>,
        ret: Option<MetaType>,
    },
    Struct {
        name: String,
        fields: Box<[MetaField]>,
    },
}
#[derive(Clone, Debug)]
pub struct SymbolDeclTable {
    pub symbols: Box<[SymbolDecl]>,
    pub program: MetaProgram,
}

pub fn get_symbols_from_file(p: &Path, pkg_name: String) -> SymbolDeclTable {
    let input = match std::fs::read_to_string(p) {
        Err(e) => panic!(
            "Error while reading file {} (ERR: {})",
            p.to_str().unwrap(),
            e
        ),
        Ok(s) => s,
    };
    let namespace = path_to_namespace(p, Some(pkg_name));
    get_symbols(
        input,
        namespace,
        Some(p.to_str().unwrap_or("<unknown>").to_string()),
    )
}

pub fn get_symbols(input: String, namespace: String, filename: Option<String>) -> SymbolDeclTable {
    let filename = filename.unwrap_or("<unknown>".to_string());
    let pairs = match JunoParser::parse(Rule::program, input.as_str()) {
        Ok(pairs) => pairs,
        Err(e) => {
            panic!("{e}");
        }
    };
    let expr_owned = parse_program(
        pairs.into_iter().next().unwrap(),
        namespace,
        input.clone().into(),
        filename.clone().into(),
    )
    .unwrap();
    let expr = Box::leak(Box::new(expr_owned));
    let metairgen = Box::leak(Box::new(MetaIRGen::new(expr, input, filename)));
    let metair = Box::leak(Box::new(metairgen.lower_program(expr)));
    let mut symbols = vec![];
    for f in metair.functions.values() {
        symbols.push(SymbolDecl::Function {
            name: f.name.clone(),
            params: f.params.clone(),
            ret: f.ret.clone(),
        });
    }
    for s in metair.structs.values() {
        symbols.push(SymbolDecl::Struct {
            name: s.name.clone(),
            fields: s.fields.clone(),
        });
    }
    SymbolDeclTable {
        symbols: symbols.into_boxed_slice(),
        program: metair.clone(),
    }
}
