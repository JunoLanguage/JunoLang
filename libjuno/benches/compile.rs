use anyhow::Result;
use criterion::{Criterion, criterion_group, criterion_main};
use libjuno::{JunoParser, LLVMBackend, MetaIRGen, Rule, parse_program, pest::Parser};
use std::{hint::black_box, path::Path};

const DUMMY0: &str = black_box(include_str!("bench_files/dummy0.juno"));
const DUMMY1: &str = black_box(include_str!("bench_files/dummy1.juno"));

const DUMMY_NAMESPACE: &str = black_box("DUMMY");
const DUMMY_PATH: &str = black_box("dummy.juno");

fn compile_dummy(dummy: &str) -> Result<()> {
    let pairs = match JunoParser::parse(Rule::program, DUMMY0) {
        Ok(pairs) => pairs,
        Err(e) => {
            panic!("{e}");
        }
    };
    let expr_owned = parse_program(
        pairs.into_iter().next().unwrap(),
        DUMMY_NAMESPACE.into(),
        dummy.into(),
        DUMMY_PATH.into(),
    )
    .unwrap();
    let expr = Box::leak(Box::new(expr_owned));
    let metairgen = Box::leak(Box::new(MetaIRGen::new(
        expr,
        dummy.to_string(),
        DUMMY_PATH.to_string(),
    )));
    let metair = Box::leak(Box::new(metairgen.lower_program(expr)));
    let context = Box::leak(Box::new(inkwell::context::Context::create()));
    LLVMBackend::new(
        context,
        metair,
        "main",
        dummy.into(),
        DUMMY_PATH.to_string(),
    )
    .compile()?;
    Ok(())
}
fn compile_benchmark(c: &mut Criterion) {
    c.bench_function("compile dummy0", |b| b.iter(|| compile_dummy(DUMMY0)));
    c.bench_function("compile dummy1", |b| b.iter(|| compile_dummy(DUMMY1)));
}

criterion_group!(benches, compile_benchmark);
criterion_main!(benches);
