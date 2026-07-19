//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::process::Command;

use clap::Parser;
use libjuno::inkwell::OptimizationLevel;
use libjuno::inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine,
};
use libjuno::{compile_file, inkwell::module::Module};

mod optimizer;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    files: Vec<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    bc: bool,

    #[arg(long = "emit-ir")]
    emit_llvm_ir: bool,
}

struct JunoObject<'a> {
    module: Module<'a>,
    filename: String,
}

fn main() {
    let args = Cli::parse();
    let output = args.output.unwrap_or("out.junoc".to_string());
    let linker = std::env::var("JUNO_LD").unwrap_or("clang".to_string());
    let _out_ext: Vec<&str> = output.split(".").collect();
    let mut out_ext = *_out_ext.last().unwrap();
    if _out_ext.len() < 2 {
        out_ext = "elf";
    }
    let mut objects: Vec<JunoObject> = vec![];
    let target_machine = get_target_machine();
    for file in args.files {
        let ext = file.split(".").last().unwrap();

        let mut o = match ext {
            "juno" => JunoObject {
                module: compile_file(Path::new(&file), None),
                filename: file,
            },
            _ => panic!("Unknown input filetyp: {}", ext),
        };
        optimizer::optimize(&mut o.module);
        objects.push(o);
    }
    if args.emit_llvm_ir {
        for o in &objects {
            o.module
                .print_to_file(format!("{}.ll", o.filename))
                .unwrap();
        }
    }
    if args.bc {
        for o in &objects {
            dbg!(&format!("./{}.bc", o.filename).to_string());
            o.module
                .write_bitcode_to_path(Path::new(&format!("./{}.bc", o.filename).to_string()));
        }
    }
    match out_ext {
        "junoc" => {}
        "junobj" => {}
        "elf" => {
            let mut object_paths: Vec<String> = vec![];
            for o in &objects {
                let path = &format!("./{}.o", o.filename).to_string();
                let _ = target_machine.write_to_file(
                    &o.module,
                    libjuno::inkwell::targets::FileType::Object,
                    Path::new(path),
                );
                object_paths.push(path.clone());
            }
            let linker_args: Vec<String> = vec!["-o".to_string(), output, "-no-pie".to_string()];
            object_paths.extend(linker_args);
            let _status = Command::new(&linker).args(&object_paths).status().unwrap();
        }
        "lib" => {}
        "bc" => {}
        _ => panic!("Unknown output filetyp: {}", out_ext),
    }
}

pub fn get_target_machine() -> TargetMachine {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&triple).unwrap();

    target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap()
}
