use std::f64::consts::E;

use inkwell::{
    types::{AsTypeRef, BasicTypeEnum},
    values::{
        ArrayValue, AsValueRef, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue,
    },
};
use pest::Span;

use crate::{ast::JunoSpan, metair::*};

use super::*;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_expr(
        &mut self,
        expr: &MetaExpr,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        match &expr.kind {
            MetaExprKind::Unary { op, expr, span } => self.lower_unary(op, expr, span),

            MetaExprKind::Call { target, args, span } => {
                self.lower_call(target.clone(), args, span)?.ok_or_else(|| {
                    LLVMError::SpanMessage(
                        "void function used as expression".to_string(),
                        span.clone(),
                    )
                })
            }

            MetaExprKind::String(id, span) => self.lower_string(id.clone(), span),

            MetaExprKind::Array(inner, span) => {
                let mut items = Vec::new();

                for expr in inner {
                    items.push(self.lower_expr(expr, &expr.span)?);
                }

                let (elem_ty, expected_size) = match &expr.ty {
                    MetaType::Array { span, elem, size } => {
                        (self.lower_type(elem, span)?, *size as usize)
                    }

                    _ => {
                        return Err(LLVMError::SpanMessage(
                            "array literal has non-array type".to_string(),
                            span.clone(),
                        ));
                    }
                };

                while items.len() < expected_size {
                    items.push(elem_ty.const_zero());
                }

                if items.len() != expected_size {
                    return Err(LLVMError::SpanMessage(
                        "array literal size mismatch".to_string(),
                        span.clone(),
                    ));
                }

                let raw: Vec<_> = items.iter().map(|v| v.as_value_ref()).collect();

                Ok(BasicValueEnum::ArrayValue(unsafe {
                    ArrayValue::new_raw_const_array(elem_ty.as_type_ref(), &raw)
                }))
            }

            MetaExprKind::Binary { op, lhs, rhs, span } => self.lower_binary(op, lhs, rhs, span),

            MetaExprKind::Const(MetaConst::Int(value, _), span) => {
                match self.lower_type(&expr.ty, &expr.span)? {
                    BasicTypeEnum::IntType(i) => Ok(i.const_int(*value as u64, true).into()),

                    _ => Err(LLVMError::SpanMessage(
                        "integer constant has non-integer type".to_string(),
                        span.clone(),
                    )),
                }
            }

            MetaExprKind::Const(MetaConst::Bool(value, _), span) => {
                match self.lower_type(&expr.ty, &expr.span)? {
                    BasicTypeEnum::IntType(i) => Ok(i.const_int(*value as u64, false).into()),

                    _ => Err(LLVMError::SpanMessage(
                        "bool constant has non-bool type".to_string(),
                        span.clone(),
                    )),
                }
            }

            MetaExprKind::Const(MetaConst::Char(value, _), span) => {
                match self.lower_type(&expr.ty, &expr.span)? {
                    BasicTypeEnum::IntType(i) => Ok(i.const_int(*value as u64, false).into()),

                    _ => Err(LLVMError::SpanMessage(
                        "char constant has non-char type".to_string(),
                        span.clone(),
                    )),
                }
            }

            MetaExprKind::Var(id, span) => {
                let var = self.get_variable(id.clone())?;
                Ok(self
                    .builder
                    .build_load(var.ty, var.ptr, id)
                    .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?)
            }
            MetaExprKind::StructInit { span, name, fields } => {
                let s = self.get_struct(name.clone())?;
                let s_ptr = self.builder.build_alloca(s, "tmp").unwrap();

                for (idx, expr) in fields {
                    let gep = self
                        .builder
                        .build_struct_gep(s, s_ptr, idx.clone(), "field")
                        .unwrap();
                    let value = self.lower_expr(expr, &expr.span)?;
                    self.builder.build_store(gep, value).unwrap();
                }

                let value = self.builder.build_load(s, s_ptr, "tmp").unwrap();
                Ok(value)
            }
            MetaExprKind::Void(span) => Err(LLVMError::SpanMessage(
                "void expression used as value".to_string(),
                span.clone(),
            )),
        }
    }

    fn lower_unary(
        &mut self,
        op: &MetaUnOp,
        expr: &MetaExpr,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        match op {
            MetaUnOp::Neg => {
                let value = self.lower_expr(expr, &expr.span)?.into_int_value();

                Ok(self
                    .builder
                    .build_int_neg(value, "negtmp")
                    .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?
                    .into())
            }

            MetaUnOp::Not => {
                let value = self.lower_expr(expr, &expr.span)?.into_int_value();

                Ok(self
                    .builder
                    .build_not(value, "nottmp")
                    .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?
                    .into())
            }

            MetaUnOp::Ref => match &expr.kind {
                MetaExprKind::Var(id, span) => Ok(self.get_variable(id.clone())?.ptr.into()),

                _ => Err(LLVMError::SpanMessage(
                    "reference requires a variable".to_string(),
                    expr.span.clone(),
                )),
            },

            MetaUnOp::Deref => {
                let ptr = self.lower_expr(expr, &expr.span)?.into_pointer_value();

                let pointee = match &expr.ty {
                    MetaType::Pointer(inner, span) | MetaType::Reference(inner, span) => inner,
                    _ => {
                        return Err(LLVMError::SpanMessage(
                            "cannot dereference non-pointer".to_string(),
                            span.clone(),
                        ));
                    }
                };

                let llvm_ty = self.lower_type(pointee, span)?;

                Ok(self
                    .builder
                    .build_load(llvm_ty, ptr, "deref")
                    .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?)
            }
        }
    }

    fn lower_string(
        &mut self,
        id: StringId,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        let string =
            self.program.string_table.get(id as usize).ok_or_else(|| {
                LLVMError::SpanMessage("unknown string id".to_string(), span.clone())
            })?;

        let ptr = self
            .builder
            .build_global_string_ptr(string, &format!("str.{}", id))
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        Ok(ptr.as_pointer_value().into())
    }

    pub fn lower_call(
        &mut self,
        target: SymbolId,
        args: &[MetaArg],
        span: &JunoSpan,
    ) -> Result<Option<BasicValueEnum<'ctx>>, LLVMError> {
        let function = self.get_function(target)?;

        let llvm_args = if function.get_type().is_var_arg() {
            self.lower_variadic_call(function, args)?
        } else {
            self.lower_normal_call(function, args, span)?
        };

        let call = self
            .builder
            .build_call(function, &llvm_args, "calltmp")
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(value) => Ok(Some(value)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }
    fn lower_normal_call(
        &mut self,
        function: FunctionValue<'ctx>,
        args: &[MetaArg],
        span: &JunoSpan,
    ) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, LLVMError> {
        if args.len() != function.count_params() as usize {
            return Err(LLVMError::SpanMessage(
                format!(
                    "expected {} arguments, got {}",
                    function.count_params(),
                    args.len()
                ),
                span.clone(),
            ));
        }

        let mut llvm_args = Vec::with_capacity(args.len());

        for (arg, param) in args.iter().zip(function.get_param_iter()) {
            let value = match arg {
                MetaArg::Pos(expr, _) => self.lower_expr(expr, &expr.span)?,
                MetaArg::Named(_, expr, _) => self.lower_expr(expr, &expr.span)?,
            };

            let value = self.coerce_value(value, param.get_type())?;
            llvm_args.push(value.into());
        }

        Ok(llvm_args)
    }
    fn lower_variadic_call(
        &mut self,
        function: FunctionValue<'ctx>,
        args: &[MetaArg],
    ) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, LLVMError> {
        let params: Vec<_> = function.get_param_iter().collect();

        if args.len() < params.len() {
            return Err(LLVMError::Message(
                format!(
                    "expected at least {} arguments, got {}",
                    params.len(),
                    args.len(),
                )
                .into(),
            ));
        }

        let mut llvm_args = Vec::with_capacity(args.len());

        for (i, arg) in args.iter().enumerate() {
            let value = match arg {
                MetaArg::Pos(expr, _) => self.lower_expr(expr, &expr.span)?,
                MetaArg::Named(_, expr, _) => self.lower_expr(expr, &expr.span)?,
            };

            let value = if i < params.len() {
                self.coerce_value(value, params[i].get_type())?
            } else {
                value
            };

            llvm_args.push(value.into());
        }

        Ok(llvm_args)
    }
    fn lower_binary(
        &mut self,
        op: &MetaBinOp,
        lhs: &MetaExpr,
        rhs: &MetaExpr,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        let lhs = self.lower_expr(lhs, &lhs.span)?;
        let rhs = self.lower_expr(rhs, &rhs.span)?;

        match op {
            MetaBinOp::Eq | MetaBinOp::Neq => self.lower_eq(op, lhs, rhs, span),

            MetaBinOp::Lt | MetaBinOp::Lte | MetaBinOp::Gt | MetaBinOp::Gte => {
                self.lower_cmp(op, lhs, rhs, span)
            }

            MetaBinOp::Add
            | MetaBinOp::Sub
            | MetaBinOp::Mul
            | MetaBinOp::Div
            | MetaBinOp::Mod
            | MetaBinOp::And
            | MetaBinOp::Or => self.lower_int_binary(op, lhs, rhs, span),
        }
    }

    fn lower_int_binary(
        &mut self,
        op: &MetaBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        let lhs = lhs.into_int_value();
        let rhs = rhs.into_int_value();

        let value = (match op {
            MetaBinOp::Add => self.builder.build_int_add(lhs, rhs, "addtmp"),
            MetaBinOp::Sub => self.builder.build_int_sub(lhs, rhs, "subtmp"),
            MetaBinOp::Mul => self.builder.build_int_mul(lhs, rhs, "multmp"),
            MetaBinOp::Div => self.builder.build_int_signed_div(lhs, rhs, "divtmp"),
            MetaBinOp::Mod => self.builder.build_int_signed_rem(lhs, rhs, "modtmp"),

            MetaBinOp::Eq => {
                self.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "eqtmp")
            }

            MetaBinOp::Neq => {
                self.builder
                    .build_int_compare(inkwell::IntPredicate::NE, lhs, rhs, "netmp")
            }

            MetaBinOp::Lt => {
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, "lttmp")
            }

            MetaBinOp::Lte => {
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "ltetmp")
            }

            MetaBinOp::Gt => {
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gttmp")
            }

            MetaBinOp::Gte => {
                self.builder
                    .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gtetmp")
            }

            MetaBinOp::And => self.builder.build_and(lhs, rhs, "andtmp"),

            MetaBinOp::Or => self.builder.build_or(lhs, rhs, "ortmp"),
        })
        .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        Ok(value.into())
    }

    fn lower_eq(
        &mut self,
        op: &MetaBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        use BasicValueEnum::*;

        let pred = match op {
            MetaBinOp::Eq => inkwell::IntPredicate::EQ,
            MetaBinOp::Neq => inkwell::IntPredicate::NE,
            _ => unreachable!(),
        };

        let value = match (lhs, rhs) {
            (IntValue(l), IntValue(r)) => {
                self.builder.build_int_compare(pred, l, r, "eqtmp").unwrap()
            }

            (PointerValue(l), PointerValue(r)) => {
                self.builder.build_int_compare(pred, l, r, "eqtmp").unwrap()
            }

            _ => {
                return Err(LLVMError::SpanMessage(
                    "cannot compare these types".to_string(),
                    span.clone(),
                ));
            }
        };

        Ok(value.into())
    }

    fn lower_cmp(
        &mut self,
        op: &MetaBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        let (lhs, rhs) = match (lhs, rhs) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => (l, r),

            _ => {
                return Err(LLVMError::SpanMessage(
                    "comparison requires integer operands".to_string(),
                    span.clone(),
                ));
            }
        };

        let pred = match op {
            MetaBinOp::Lt => inkwell::IntPredicate::SLT,
            MetaBinOp::Lte => inkwell::IntPredicate::SLE,
            MetaBinOp::Gt => inkwell::IntPredicate::SGT,
            MetaBinOp::Gte => inkwell::IntPredicate::SGE,
            _ => unreachable!(),
        };

        Ok(self
            .builder
            .build_int_compare(pred, lhs, rhs, "cmptmp")
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?
            .into())
    }

    fn lower_logic(
        &mut self,
        op: &MetaBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
        span: &JunoSpan,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        use BasicValueEnum::*;

        let (lhs, rhs) = match (lhs, rhs) {
            (IntValue(l), IntValue(r)) => (l, r),

            _ => {
                return Err(LLVMError::SpanMessage(
                    "logical operators require bool operands".to_string(),
                    span.clone(),
                ));
            }
        };

        let value = match op {
            MetaBinOp::And => self.builder.build_and(lhs, rhs, "andtmp"),
            MetaBinOp::Or => self.builder.build_or(lhs, rhs, "ortmp"),
            _ => unreachable!(),
        }
        .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        Ok(value.into())
    }

    fn coerce_value(
        &self,
        value: BasicValueEnum<'ctx>,
        target: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, LLVMError> {
        match (value, target) {
            (BasicValueEnum::IntValue(v), BasicTypeEnum::IntType(t)) => {
                let src_bits = v.get_type().get_bit_width();
                let dst_bits = t.get_bit_width();

                if src_bits == dst_bits {
                    Ok(v.into())
                } else if src_bits < dst_bits {
                    Ok(self.builder.build_int_z_extend(v, t, "zext")?.into())
                } else {
                    Ok(self.builder.build_int_truncate(v, t, "trunc")?.into())
                }
            }

            (BasicValueEnum::PointerValue(v), BasicTypeEnum::PointerType(_)) => Ok(v.into()),

            (v, _) => Ok(v),
        }
    }
}
