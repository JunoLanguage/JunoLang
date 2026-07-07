use std::{ path::Path, process::exit };
use inkwell::module::Module;
use pest::Parser;

use crate::*;

pub fn compile_file(p: &Path) -> Module<'static> {
    let input = match std::fs::read_to_string(p) {
        Err(e) => panic!("Error while reading file {} (ERR: {})", p.to_str().unwrap(), e),
        Ok(s) => s,
    };

    let pairs = match JunoParser::parse(Rule::program, &input.as_str()) {
        Ok(pairs) => pairs,
        Err(e) => {
            panic!("{e}");
        }
    };
    let expr = parse_program(pairs.into_iter().next().unwrap());
    let mut metairgen = MetaIRGen::new(&expr);
    let metair = Box::leak(Box::new(metairgen.lower_program(&expr)));
    let context = Box::leak(Box::new(inkwell::context::Context::create()));
    let mut irgen = LLVMBackend::new(context, metair, "main");
    if let Err(e) = irgen.compile() {
        eprintln!("{:#?}", e);
        exit(1);
    }

    return irgen.module;
}
