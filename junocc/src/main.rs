use std::hash::{ DefaultHasher, Hash, Hasher };
use std::path::Path;
use std::process::Command;

use libjuno::inkwell::OptimizationLevel;
use libjuno::inkwell::targets::{
    CodeModel,
    InitializationConfig,
    RelocMode,
    Target,
    TargetMachine,
};
use libjuno::{ compile_file, inkwell::module::Module };
use clap::Parser;

mod optimizer;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    files: Vec<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    bc: bool,
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
            "juno" =>
                JunoObject {
                    module: compile_file(Path::new(&file)),
                    filename: file,
                },
            _ => panic!("Unknown input filetyp: {}", ext),
        };
        optimizer::optimize(&mut o.module);
        objects.push(o);
    }
    if args.bc {
        for o in &objects {
            let mut s = DefaultHasher::new();

            o.module.to_string().hash(&mut s);
            let hash = s.finish();

            o.module.write_bitcode_to_path(
                Path::new(&format!("./{}-{:x}.bc", o.filename, hash).to_string())
            );
        }
    }
    match out_ext {
        "junoc" => {}
        "junobj" => {}
        "elf" => {
            let mut object_paths: Vec<String> = vec![];
            for o in &objects {
                let mut s = DefaultHasher::new();

                o.module.to_string().hash(&mut s);
                let hash = s.finish();
                let path = &format!("./{}-{:x}.o", o.filename, hash).to_string();
                target_machine.write_to_file(
                    &o.module,
                    libjuno::inkwell::targets::FileType::Object,
                    Path::new(path)
                );
                object_paths.push(path.clone());
            }
            let mut linker_args: Vec<String> = vec!["-o".to_string(), "program".to_string(), "-no-pie".to_string()];
            object_paths.extend(linker_args);
            let status = Command::new(&linker)
                .args(&object_paths)
                .status().unwrap();
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
            CodeModel::Default
        )
        .unwrap()
}
