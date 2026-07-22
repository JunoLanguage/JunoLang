//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use inkwell::AddressSpace;

use crate::*;

use super::LLVMBackend;

impl<'ctx> LLVMBackend<'ctx> {
    pub(super) fn add_scanf(&mut self) {
        let i8ptr = self.context.ptr_type(AddressSpace::default());

        let ty = self.context.i32_type().fn_type(&[i8ptr.into()], true);

        self.declare_builtin("scanf", ty);
    }
}
