use inkwell::{
    types::{ AsTypeRef, BasicTypeEnum },
    values::{ ArrayValue, AsValueRef, BasicValueEnum },
};

use crate::metair::*;

use super::*;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_expr(&mut self, expr: &MetaExpr) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        match &expr.kind {
            MetaExprKind::Unary { op, expr } => { self.lower_unary(op, expr) }

            MetaExprKind::Call { target, args } => {
                self.lower_call(target, args)?.ok_or_else(||
                    LLVMError::Message("void function used as expression".into())
                )
            }

            MetaExprKind::String(id) => self.lower_string(*id),

            MetaExprKind::Array(inner) => {
                let mut items = Vec::new();

                for expr in inner {
                    items.push(self.lower_expr(expr)?);
                }

                let (elem_ty, expected_size) = match &expr.ty {
                    MetaType::Array { elem, size } => { (self.lower_type(elem)?, *size as usize) }

                    _ => {
                        return Err(LLVMError::Message("array literal has non-array type".into()));
                    }
                };

                while items.len() < expected_size {
                    items.push(elem_ty.const_zero());
                }

                if items.len() != expected_size {
                    return Err(LLVMError::Message("array literal size mismatch".into()));
                }

                let raw: Vec<_> = items
                    .iter()
                    .map(|v| v.as_value_ref())
                    .collect();

                Ok(
                    BasicValueEnum::ArrayValue(unsafe {
                        ArrayValue::new_raw_const_array(elem_ty.as_type_ref(), &raw)
                    })
                )
            }

            MetaExprKind::Binary { op, lhs, rhs } => { self.lower_binary(op, lhs, rhs) }

            MetaExprKind::Const(MetaConst::Int(value)) => {
                match self.lower_type(&expr.ty)? {
                    BasicTypeEnum::IntType(i) => { Ok(i.const_int(*value as u64, true).into()) }

                    _ => Err(LLVMError::Message("integer constant has non-integer type".into())),
                }
            }

            MetaExprKind::Const(MetaConst::Bool(value)) => {
                match self.lower_type(&expr.ty)? {
                    BasicTypeEnum::IntType(i) => { Ok(i.const_int(*value as u64, false).into()) }

                    _ => Err(LLVMError::Message("bool constant has non-bool type".into())),
                }
            }

            MetaExprKind::Const(MetaConst::Char(value)) => {
                match self.lower_type(&expr.ty)? {
                    BasicTypeEnum::IntType(i) => { Ok(i.const_int(*value as u64, false).into()) }

                    _ => Err(LLVMError::Message("char constant has non-char type".into())),
                }
            }

            MetaExprKind::Var(id) => {
                let var = self.get_variable(*id)?;
                Ok(
                    self.builder
                        .build_load(var.ty, var.ptr, &self.program.symbol_table[*id as usize])
                        .map_err(|e| LLVMError::Message(e.to_string()))?
                )
            }

            MetaExprKind::Void => Err(LLVMError::Message("void expression used as value".into())),

            other => Err(LLVMError::Message(format!("expression not implemented: {:#?}", other))),
        }
    }

    fn lower_unary(
        &mut self,
        op: &MetaUnOp,
        expr: &MetaExpr
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        match op {
            MetaUnOp::Neg => {
                let value = self.lower_expr(expr)?.into_int_value();

                Ok(
                    self.builder
                        .build_int_neg(value, "negtmp")
                        .map_err(|e| LLVMError::Message(e.to_string()))?
                        .into()
                )
            }

            MetaUnOp::Not => {
                let value = self.lower_expr(expr)?.into_int_value();

                Ok(
                    self.builder
                        .build_not(value, "nottmp")
                        .map_err(|e| LLVMError::Message(e.to_string()))?
                        .into()
                )
            }

            MetaUnOp::Ref =>
                match &expr.kind {
                    MetaExprKind::Var(id) => Ok(self.get_variable(*id)?.ptr.into()),

                    _ => Err(LLVMError::Message("reference requires a variable".into())),
                }

            MetaUnOp::Deref => {
                let ptr = self.lower_expr(expr)?.into_pointer_value();

                let pointee = match &expr.ty {
                    MetaType::Pointer(inner) | MetaType::Reference(inner) => inner,
                    _ => {
                        return Err(LLVMError::Message("cannot dereference non-pointer".into()));
                    }
                };

                let llvm_ty = self.lower_type(pointee)?;

                Ok(
                    self.builder
                        .build_load(llvm_ty, ptr, "deref")
                        .map_err(|e| LLVMError::Message(e.to_string()))?
                )
            }
        }
    }

    fn lower_string(&mut self, id: StringId) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        let string = self.program.string_table
            .get(id as usize)
            .ok_or_else(|| { LLVMError::Message(format!("unknown string id {}", id)) })?;

        let ptr = self.builder
            .build_global_string_ptr(string, &format!("str.{}", id))
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        Ok(ptr.as_pointer_value().into())
    }

    pub fn lower_call(
        &mut self,
        target: &[SymbolId],
        args: &[MetaArg]
    ) -> Result<Option<BasicValueEnum<'ctx>>, LLVMError> {
        let function = self.get_function(target)?;

        let mut llvm_args = Vec::new();

        for arg in args {
            let value = match arg {
                MetaArg::Pos(expr) => self.lower_expr(expr)?,
                MetaArg::Named(_, expr) => self.lower_expr(expr)?,
            };

            llvm_args.push(value.into());
        }

        let call = self.builder
            .build_call(function, &llvm_args, "calltmp")
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Ok(Some(value)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }

    fn lower_binary(
        &mut self,
        op: &MetaBinOp,
        lhs: &MetaExpr,
        rhs: &MetaExpr
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        let lhs = self.lower_expr(lhs)?.into_int_value();
        let rhs = self.lower_expr(rhs)?.into_int_value();

        let value = (
            match op {
                MetaBinOp::Add => self.builder.build_int_add(lhs, rhs, "addtmp"),
                MetaBinOp::Sub => self.builder.build_int_sub(lhs, rhs, "subtmp"),
                MetaBinOp::Mul => self.builder.build_int_mul(lhs, rhs, "multmp"),
                MetaBinOp::Div => self.builder.build_int_signed_div(lhs, rhs, "divtmp"),
                MetaBinOp::Mod => self.builder.build_int_signed_rem(lhs, rhs, "modtmp"),

                MetaBinOp::Eq =>
                    self.builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "eqtmp"),

                MetaBinOp::Neq =>
                    self.builder.build_int_compare(inkwell::IntPredicate::NE, lhs, rhs, "netmp"),

                MetaBinOp::Lt =>
                    self.builder.build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, "lttmp"),

                MetaBinOp::Lte =>
                    self.builder.build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "ltetmp"),

                MetaBinOp::Gt =>
                    self.builder.build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gttmp"),

                MetaBinOp::Gte =>
                    self.builder.build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gtetmp"),

                MetaBinOp::And => self.builder.build_and(lhs, rhs, "andtmp"),

                MetaBinOp::Or => self.builder.build_or(lhs, rhs, "ortmp"),
            }
        ).map_err(|e| LLVMError::Message(e.to_string()))?;

        Ok(value.into())
    }
}
