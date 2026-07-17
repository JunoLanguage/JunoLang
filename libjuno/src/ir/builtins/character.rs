use super::LLVMBackend;
use crate::*;

impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_isalnum(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("isalnum", ty);
    }

    pub(super) fn add_isalpha(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("isalpha", ty);
    }

    pub(super) fn add_isdigit(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("isdigit", ty);
    }

    pub(super) fn add_islower(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("islower", ty);
    }

    pub(super) fn add_isupper(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("isupper", ty);
    }

    pub(super) fn add_isspace(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("isspace", ty);
    }

    pub(super) fn add_isxdigit(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("isxdigit", ty);
    }

    pub(super) fn add_tolower(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("tolower", ty);
    }

    pub(super) fn add_toupper(&mut self) {
        let ty = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);

        self.declare_builtin("toupper", ty);
    }
}
