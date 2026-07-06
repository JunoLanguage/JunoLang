use libjuno::inkwell::{
    targets::{ InitializationConfig, Target, TargetMachine, RelocMode, CodeModel },
    OptimizationLevel,
    passes::PassBuilderOptions,
    module::Module
    
};

use crate::get_target_machine;

pub fn optimize(module: &mut Module) {
    let target_machine = get_target_machine();
    module.run_passes("default<O3>", &target_machine, PassBuilderOptions::create()).unwrap();
    
}
