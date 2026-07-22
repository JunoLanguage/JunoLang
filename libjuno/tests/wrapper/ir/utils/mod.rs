//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use inkwell::context::Context;
use libjuno::ast::JunoSpan;
use libjuno::ir::{LLVMBackend, LLVMError};
use libjuno::{JunoParser, Rule, metair::*, parse_program};
use pest::Parser;
use std::collections::HashMap;
use std::sync::Arc;

/// Adjust this to match your real JunoSpan constructor.
pub fn dummy_span() -> JunoSpan {
    // Example placeholder:
    JunoSpan { start: 0, end: 0 }
}

pub fn test_program() -> MetaProgram {
    let input = include_str!("../../test_files/test0.juno");
    let pairs = match JunoParser::parse(Rule::program, &input) {
        Ok(pairs) => pairs,
        Err(e) => {
            panic!("{e}");
        }
    };
    let expr_owned = parse_program(
        pairs.into_iter().next().unwrap(),
        "test".to_string(),
        input.into(),
        Arc::from("test file"),
    )
    .unwrap();
    let expr = Box::leak(Box::new(expr_owned));
    let metairgen = Box::leak(Box::new(MetaIRGen::new(
        expr,
        input.to_string(),
        "test file".to_string(),
    )));
    metairgen.lower_program(expr)
}

/// Leaks program/context so lifetimes are easy in tests.
pub fn make_backend(program: &'static MetaProgram) -> (LLVMBackend<'static>, &'static Context) {
    let context = Box::leak(Box::new(Context::create()));
    let backend = LLVMBackend::new(
        context,
        program,
        "test_mod",
        "// test\n".into(),
        "test.juno".into(),
    );
    (backend, context)
}

pub fn dummy_meta_function() -> MetaFunction {
    MetaFunction {
        name: "dummy".into(),
        params: Vec::new(),
        ret: Some(MetaType::Named("void".into(), dummy_span())),
        body: Vec::new(),
        span: dummy_span(),
        locals: HashMap::new(),
    }
}
