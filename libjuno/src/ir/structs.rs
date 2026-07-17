use inkwell::types::{BasicTypeEnum, StructType};

use super::*;
use crate::{ast::JunoSpan, metair::*};

impl<'ctx> LLVMBackend<'ctx> {
    fn struct_type(
        &mut self,
        s: &MetaStruct,
        span: &JunoSpan,
    ) -> Result<StructType<'ctx>, LLVMError> {
        let mut fields = Vec::<BasicTypeEnum<'ctx>>::new();

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

    pub fn get_struct(&self, target: SymbolId) -> Result<StructType<'ctx>, LLVMError> {
        if target.len() != 1 {
            return Err(LLVMError::Message(
                "qualified struct lookup is not implemented".into(),
            ));
        }

        self.structs
            .get(&target)
            .copied()
            .ok_or_else(|| LLVMError::Message(format!("unknown struct '{}'", target)))
    }
}
