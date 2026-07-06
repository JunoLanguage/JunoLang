use inkwell::AddressSpace;
use crate::*;
use super::LLVMBackend;
impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_printf(&mut self) {
        let i8ptr = self.context.ptr_type(AddressSpace::default());

        let printf_ty = self.context
            .i32_type()
            .fn_type(&[i8ptr.into()], true);

        let printf = self.module.add_function("printf", printf_ty, None);
        self.add_function(get_builtin_id("printf"), &printf).unwrap();
        self.builtins.insert("printf", printf);
    }
}