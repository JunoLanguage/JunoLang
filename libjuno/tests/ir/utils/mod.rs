use inkwell::context::Context;
use libjuno::ast::JunoSpan;
use libjuno::ir::{LLVMBackend, LLVMError};
use libjuno::metair::*;
use std::collections::HashMap;

/// Adjust this to match your real JunoSpan constructor.
pub fn dummy_span() -> JunoSpan {
    // Example placeholder:
    JunoSpan { start: 0, end: 0 }
}

pub fn empty_program() -> MetaProgram {
    MetaProgram {
        structs: HashMap::new(),
        declarations: HashMap::new(),
        functions: HashMap::new(),
        string_table: Vec::new(),
        struct_fields: HashMap::new(),
        span: dummy_span(),
        symbol_table: Vec::new(),
    }
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
