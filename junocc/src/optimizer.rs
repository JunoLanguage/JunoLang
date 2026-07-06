use libjuno::inkwell::{
    targets::{ InitializationConfig, Target, TargetMachine, RelocMode, CodeModel },
    OptimizationLevel,
    passes::PassBuilderOptions,
    module::Module
    
};

pub fn optimize(module: &mut Module) {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&triple).unwrap();

    let target_machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default
        )
        .unwrap();

    module.run_passes("default<O3>", &target_machine, PassBuilderOptions::create()).unwrap();
    
}
