//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::Arc;

use libjuno::{JunoParser, LLVMBackend, MetaIRGen, Rule, parse_program};
use pest::Parser as _;

pub fn test_file(input: &str, file_name: &str) {
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
        Arc::from("testing_file"),
    )
    .unwrap();
    let expr = Box::leak(Box::new(expr_owned));
    let metairgen = Box::leak(Box::new(MetaIRGen::new(
        expr,
        input.to_string(),
        file_name.to_string(),
    )));
    let metair = Box::leak(Box::new(metairgen.lower_program(expr)));
    let context = Box::leak(Box::new(inkwell::context::Context::create()));
    let irgen = LLVMBackend::new(
        context,
        metair,
        "main",
        input.to_string(),
        file_name.to_string(),
    );

    match irgen.compile() {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
}
