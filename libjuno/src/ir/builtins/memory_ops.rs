use super::LLVMBackend;
use crate::*;
use inkwell::AddressSpace;

impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_memcpy(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[ptr.into(), ptr.into(), self.context.i64_type().into()],
            false,
        );

        self.declare_builtin("memcpy", ty);
    }

    pub(super) fn add_memmove(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[ptr.into(), ptr.into(), self.context.i64_type().into()],
            false,
        );

        self.declare_builtin("memmove", ty);
    }

    pub(super) fn add_memset(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[
                ptr.into(),
                self.context.i32_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );

        self.declare_builtin("memset", ty);
    }

    pub(super) fn add_memcmp(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.i32_type().fn_type(
            &[ptr.into(), ptr.into(), self.context.i64_type().into()],
            false,
        );

        self.declare_builtin("memcmp", ty);
    }

    pub(super) fn add_memchr(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[
                ptr.into(),
                self.context.i32_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );

        self.declare_builtin("memchr", ty);
    }
}
