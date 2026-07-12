use crate::metair::*;
use inkwell::types::BasicType;

use super::*;

use inkwell::types::BasicTypeEnum;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_type(&mut self, ty: &MetaType) -> Result<BasicTypeEnum<'ctx>, LLVMError> {
        match ty {
            MetaType::Pointer(inner) => {
                Ok(self
                    .lower_type(inner)?
                    .ptr_type(inkwell::AddressSpace::default())
                    .into()) // TODO ptr_type is deprecated, searching for alternative
            }

            MetaType::Reference(inner) => {
                Ok(self
                    .lower_type(inner)?
                    .ptr_type(inkwell::AddressSpace::default())
                    .into()) // TODO
            }
            MetaType::Array { elem, size } => Ok(self.lower_type(elem)?.array_type(*size).into()),

            MetaType::Named(id) => match id.as_str() {
                "bool" => Ok(self.context.bool_type().into()),

                "char" => Ok(self.context.i8_type().into()),
                "str" => Ok(self.context.i32_type().into()),
                "i8" => Ok(self.context.i8_type().into()),
                "i16" => Ok(self.context.i16_type().into()),
                "i32" => Ok(self.context.i32_type().into()),
                "i64" => Ok(self.context.i64_type().into()),

                "u8" => Ok(self.context.i8_type().into()),
                "u16" => Ok(self.context.i16_type().into()),
                "u32" => Ok(self.context.i32_type().into()),
                "u64" => Ok(self.context.i64_type().into()),

                "f32" => Ok(self.context.f32_type().into()),
                "f64" => Ok(self.context.f64_type().into()),

                _ => {
                    if self.program.structs.get(id).is_some() {
                        if !self.structs.contains_key(id) {
                            self.lower_struct(&self.program.structs[&id.clone()])?;
                        }
                        return match self.get_struct(id.clone()) {
                            Err(e) => Err(e),
                            Ok(s) => Ok(s.into()),
                        };
                    }
                    Err(LLVMError::UnknownType(id.clone()))
                }
            },
            MetaType::Unit => todo!(), //e => Err(LLVMError::Message(format!("type not implemented: {:#?}", e).into())),
        }
    }
}
