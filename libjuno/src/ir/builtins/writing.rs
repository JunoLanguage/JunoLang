use super::LLVMBackend;
use crate::*;
use inkwell::AddressSpace;
impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_printf(&mut self) {
        let i8ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.i32_type().fn_type(&[i8ptr.into()], true);

        self.declare_builtin("printf", ty);
    }

    pub(super) fn add_puts(&mut self) {
        let i8ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.i32_type().fn_type(&[i8ptr.into()], false);

        self.declare_builtin("puts", ty);
    }

    pub(super) fn add_putchar(&mut self) {
        let ty = self
            .context
            .i8_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("putchar", ty);
    }

    pub(super) fn add_getchar(&mut self) {
        let ty = self.context.i32_type().fn_type(&[], false);

        self.declare_builtin("getchar", ty);
    }
}
