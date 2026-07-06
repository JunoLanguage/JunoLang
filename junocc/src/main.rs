use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use libjuno::inkwell::execution_engine::ExecutionEngine;
use libjuno::{ compile_file, inkwell::module::Module };
use libjuno::inkwell::OptimizationLevel;
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
    dbg!(&args);
    let output = args.output.unwrap_or("out.junoc".to_string());
    let out_ext = output.split(".").last().unwrap_or("elf");
    let mut objects: Vec<JunoObject> = vec![];

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
        for o in objects {
            let mut s = DefaultHasher::new();

            o.module.to_string().hash(&mut s);
            let hash = s.finish();

            o.module.write_bitcode_to_path(
                Path::new(&format!("./{}-{:x}.bc", o.filename, hash,).to_string())
            );
        }
    }
    match out_ext {
        "junoc" => {}
        "junobj" => {}
        "elf" => {}
        "lib" => {}
        "bc" => {}
        _ => panic!("Unknown output filetyp: {}", out_ext),
    }
}
