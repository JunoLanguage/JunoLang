use super::LLVMBackend;
use crate::*;
use inkwell::AddressSpace;

impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_strlen(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.i64_type().fn_type(&[ptr.into()], false);

        self.declare_builtin("strlen", ty);
    }

    pub(super) fn add_strcmp(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self
            .context
            .i32_type()
            .fn_type(&[ptr.into(), ptr.into()], false);

        self.declare_builtin("strcmp", ty);
    }

    pub(super) fn add_strncmp(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.i32_type().fn_type(
            &[ptr.into(), ptr.into(), self.context.i64_type().into()],
            false,
        );

        self.declare_builtin("strncmp", ty);
    }

    pub(super) fn add_strcpy(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), ptr.into()], false);

        self.declare_builtin("strcpy", ty);
    }

    pub(super) fn add_strncpy(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[ptr.into(), ptr.into(), self.context.i64_type().into()],
            false,
        );

        self.declare_builtin("strncpy", ty);
    }

    pub(super) fn add_strcat(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), ptr.into()], false);

        self.declare_builtin("strcat", ty);
    }

    pub(super) fn add_strncat(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(
            &[ptr.into(), ptr.into(), self.context.i64_type().into()],
            false,
        );

        self.declare_builtin("strncat", ty);
    }

    pub(super) fn add_strchr(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), self.context.i32_type().into()], false);

        self.declare_builtin("strchr", ty);
    }

    pub(super) fn add_strrchr(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), self.context.i32_type().into()], false);

        self.declare_builtin("strrchr", ty);
    }

    pub(super) fn add_strstr(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), ptr.into()], false);

        self.declare_builtin("strstr", ty);
    }

    pub(super) fn add_strtok(&mut self) {
        let ptr = self.context.ptr_type(AddressSpace::default());

        let ty = ptr.fn_type(&[ptr.into(), ptr.into()], false);

        self.declare_builtin("strtok", ty);
    }
}
