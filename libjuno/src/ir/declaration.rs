use crate::ast::JunoSpan;
use crate::metair::*;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;

use super::*;

use inkwell::types::BasicMetadataTypeEnum;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_declaration(
        &mut self,
        declaration: &MetaDeclaration,
        span: &JunoSpan,
    ) -> Result<(), LLVMError> {
        let mut params = Vec::<BasicMetadataTypeEnum>::new();

        for param in &declaration.params {
            params.push(self.lower_type(&param.ty, &param.span)?.into());
        }

        let fn_type = match &declaration.ret {
            Some(ret) => self.lower_type(ret, span)?.fn_type(&params, false),

            None => self.context.void_type().fn_type(&params, false),
        };

        let llvm_declaration = self
            .module
            .add_function(declaration.name.as_str(), fn_type, None);
        self.add_function(declaration.name.clone(), &llvm_declaration)?;
        self.functions
            .insert(declaration.name.clone(), llvm_declaration);
        Ok(())
    }
}
