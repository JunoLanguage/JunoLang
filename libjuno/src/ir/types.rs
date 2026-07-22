//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ast::JunoSpan, metair::*};
use inkwell::{AddressSpace, types::BasicType};

use super::*;

use inkwell::types::BasicTypeEnum;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_type(
        &self,
        ty: &MetaType,
        span: &JunoSpan,
    ) -> Result<BasicTypeEnum<'ctx>, LLVMError> {
        match ty {
            MetaType::Pointer(_inner, _span) => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),

            MetaType::Reference(_inner, _span) => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),
            MetaType::Array { elem, size, span } => {
                Ok(self.lower_type(elem, span)?.array_type(*size).into())
            }

            MetaType::Named(id, _named_span) => match id.as_str() {
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
                    if self.program.structs.contains_key(id) {
                        if !self.structs.contains_key(id) {
                            return Err(self.make_span_error(
                                "Struct found but not in irgen, maybe a compiler bug?".to_string(),
                                *span,
                            ));
                        }
                        return match self.get_struct(id) {
                            Err(e) => Err(e),
                            Ok(s) => Ok(s.into()),
                        };
                    }
                    Err(LLVMError::UnknownType(id.clone()))
                }
            },
            MetaType::Unit(_span) => todo!(),
        }
    }

    pub fn get_named_from_type(&self, ty: &MetaType) -> Result<String, LLVMError> {
        match ty {
            MetaType::Named(name, _juno_span) => Ok(name.clone()),
            MetaType::Pointer(meta_type, _juno_span) => self.get_named_from_type(meta_type),
            MetaType::Reference(meta_type, _juno_span) => self.get_named_from_type(meta_type),
            MetaType::Array {
                span: _,
                elem,
                size: _,
            } => self.get_named_from_type(elem),
            MetaType::Unit(juno_span) => {
                Err(self.make_span_error("Unit type found".to_string(), *juno_span))
            }
        }
    }
}
