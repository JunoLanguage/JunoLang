//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use inkwell::types::StructType;

use super::*;
use crate::{ast::JunoSpan, metair::*};

impl<'ctx> LLVMBackend<'ctx> {
    pub fn struct_type(
        &self,
        s: &MetaStruct,
        _span: &JunoSpan,
    ) -> Result<StructType<'ctx>, LLVMError> {
        let mut fields = Vec::with_capacity(s.fields.len());

        for field in &s.fields {
            fields.push(self.lower_type(&field.ty, &field.span)?);
        }

        Ok(self.context.struct_type(&fields, false))
    }

    pub fn lower_struct(&mut self, s: &MetaStruct, span: &JunoSpan) -> Result<(), LLVMError> {
        let ty = self.struct_type(s, span)?;
        self.add_struct(s.name.clone(), &ty)?;
        Ok(())
    }

    pub fn add_struct(&mut self, id: SymbolId, ty: &StructType<'ctx>) -> Result<(), LLVMError> {
        self.structs.insert(id, *ty);
        Ok(())
    }

    pub fn get_struct(&self, target: &str) -> Result<StructType<'ctx>, LLVMError> {
        if target.split('.').count() != 1 {
            return Err(LLVMError::Message(
                "qualified struct lookup is not implemented".into(),
            ));
        }

        self.structs
            .get(target)
            .copied()
            .ok_or_else(|| LLVMError::Message(format!("unknown struct '{}'", target)))
    }
}
