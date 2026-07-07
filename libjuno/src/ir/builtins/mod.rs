use inkwell::{ types::FunctionType, values::FunctionValue };
use inkwell::module::Linkage;

use crate::{ LLVMBackend, get_builtin_id };

pub mod writing;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn declare_builtins(&mut self) {
        self.add_printf();
        self.add_puts();
        self.add_putchar();
        self.add_getchar();
    }
}

impl<'ctx> LLVMBackend<'ctx> {
    pub(in crate::ir::builtins) fn declare_builtin(
        &mut self,
        name: &'static str,
        ty: FunctionType<'ctx>
    ) -> FunctionValue<'ctx> {
        let function = self.module.add_function(name, ty, Some(Linkage::External));

        self.add_function(get_builtin_id(name), &function).expect("builtin already registered");

        self.builtins.insert(name, function);

        function
    }
}
