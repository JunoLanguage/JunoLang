use inkwell::module::Module;
use pest::Parser;
use std::{
    path::{Component, Path},
    process::exit,
};

use crate::*;

pub fn compile_file(p: &Path, pkg_name: Option<String>) -> Module<'static> {
    let namespace = path_to_namespace(p, pkg_name);
    let input = match std::fs::read_to_string(p) {
        Err(e) => panic!(
            "Error while reading file {} (ERR: {})",
            p.to_str().unwrap(),
            e
        ),
        Ok(s) => s,
    };

    let pairs = match JunoParser::parse(Rule::program, &input.as_str()) {
        Ok(pairs) => pairs,
        Err(e) => {
            panic!("{e}");
        }
    };
    let expr_owned = parse_program(pairs.into_iter().next().unwrap(), namespace).unwrap();
    let expr = Box::leak(Box::new(expr_owned));
    let metairgen = Box::leak(Box::new(MetaIRGen::new(expr)));
    let metair = Box::leak(Box::new(metairgen.lower_program(expr)));
    let context = Box::leak(Box::new(inkwell::context::Context::create()));
    let mut irgen = LLVMBackend::new(context, metair, "main");

    if let Err(e) = irgen.compile() {
        eprintln!("{:#?}", e);
        exit(1);
    }
    irgen.dump_ir();
    return irgen.module;
}

pub fn path_to_namespace(p: &Path, pkg_name: Option<String>) -> String {
    let pkg_name = match pkg_name {
        None => "__main".to_string(),
        Some(n) => n,
    };
    dbg!(p);
    if p.is_relative() {
        let juno_root = p.parent().unwrap().parent().unwrap(); // ./src/path/to/file.juno -> .
        if juno_root.join("juno.toml").exists() {
            // Juno Package exists

            let mut components = p.components(); // ./src/path/to/file.juno -> src, path, to, file.juno
            components.next(); // src, path, to, file.juno -> path, to, file.juno
            return format!(
                "j::{}::{}",
                pkg_name,
                components
                    .collect::<Vec<Component>>()
                    .iter()
                    .map(|s: &Component| s.as_os_str().to_str().unwrap().to_string())
                    .collect::<Vec<String>>()
                    .join("::")
            );
        } else {
            println!("Warning: juno modules does not work without a juno package")
        }
    } else {
        println!(
            "Warning: juno modules does not work with absolute path, juno need a package for working with multiple files"
        );
    }
    return p.file_prefix().unwrap().to_str().unwrap().to_string();
}
