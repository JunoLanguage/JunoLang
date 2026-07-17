use super::LLVMBackend;
use crate::*;
use inkwell::AddressSpace;

impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_malloc(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[self.context.i64_type().into()], false);

        self.declare_builtin("malloc", ty);
    }

    pub(super) fn add_calloc(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );

        self.declare_builtin("calloc", ty);
    }

    pub(super) fn add_realloc(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), self.context.i64_type().into()], false);

        self.declare_builtin("realloc", ty);
    }

    pub(super) fn add_free(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.void_type().fn_type(&[ptr.into()], false);

        self.declare_builtin("free", ty);
    }
}
