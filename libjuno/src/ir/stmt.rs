use crate::ast::JunoSpan;
use crate::ir::backend::LoopFrame;
use crate::metair::*;

use super::*;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_stmt(&mut self, stmt: &MetaStmt, span: &JunoSpan) -> Result<(), LLVMError> {
        match stmt {
            MetaStmt::Let {
                span,
                name,
                mutable: _,
                ty,
                value,
            } => self.lower_let(name.clone(), ty.as_ref(), value.as_ref(), span),
            MetaStmt::Break(span) => self.lower_break(span),
            MetaStmt::If {
                span,
                cond,
                then_body,
                else_ifs,
                else_body,
            } => self.lower_if(cond, then_body, else_ifs, else_body.as_ref(), span),
            MetaStmt::Continue(span) => self.lower_continue(span),
            MetaStmt::Loop { span, body } => self.lower_loop(body, span),
            MetaStmt::Return(expr, span) => self.lower_return(expr.as_ref(), span),
            MetaStmt::Assign {
                span,
                target,
                value,
            } => self.lower_assign(target.clone(), value, span),
            MetaStmt::Expr(expr) => self.lower_expr_stmt(expr, &expr.span),
        }
    }
    fn lower_expr_stmt(&mut self, expr: &MetaExpr, span: &JunoSpan) -> Result<(), LLVMError> {
        match &expr.kind {
            MetaExprKind::Call { span, target, args } => {
                self.lower_call(target.clone(), &args, span)?;
            }

            _ => {
                self.lower_expr(expr, span)?;
            }
        }

        Ok(())
    }
    fn lower_continue(&mut self, span: &JunoSpan) -> Result<(), LLVMError> {
        let frame = self.loop_stack.last().ok_or_else(|| {
            LLVMError::SpanMessage("continue used outside of a loop".to_string(), span.clone())
        })?;

        self.builder
            .build_unconditional_branch(frame.continue_block)
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        Ok(())
    }
    fn lower_break(&mut self, span: &JunoSpan) -> Result<(), LLVMError> {
        let frame = self.loop_stack.last().ok_or_else(|| {
            LLVMError::SpanMessage("break used outside of a loop".to_string(), span.clone())
        })?;

        self.builder
            .build_unconditional_branch(frame.break_block)
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        Ok(())
    }
    fn lower_loop(&mut self, body: &[MetaStmt], span: &JunoSpan) -> Result<(), LLVMError> {
        let function = self.current_function();

        let header_bb = self.context.append_basic_block(function, "loop.header");

        let body_bb = self.context.append_basic_block(function, "loop.body");

        let exit_bb = self.context.append_basic_block(function, "loop.exit");

        self.builder
            .build_unconditional_branch(header_bb)
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        self.builder.position_at_end(header_bb);

        self.builder
            .build_unconditional_branch(body_bb)
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        self.builder.position_at_end(body_bb);

        self.loop_stack.push(LoopFrame {
            break_block: exit_bb,
            continue_block: header_bb,
        });

        self.push_scope();

        for stmt in body {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                break;
            }

            self.lower_stmt(stmt, span)?;
        }

        self.pop_scope();

        self.loop_stack.pop();

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(header_bb)
                .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
        }

        self.builder.position_at_end(exit_bb);

        Ok(())
    }
    fn lower_if(
        &mut self,
        cond: &MetaExpr,
        then_body: &[MetaStmt],
        else_ifs: &[(MetaExpr, Vec<MetaStmt>)],
        else_body: Option<&Vec<MetaStmt>>,
        span: &JunoSpan,
    ) -> Result<(), LLVMError> {
        let function = self.current_function();

        let merge_bb = self.context.append_basic_block(function, "if.merge");

        let then_bb = self.context.append_basic_block(function, "if.then");

        let first_else_bb = self.context.append_basic_block(function, "if.else");

        let cond = self.lower_expr(cond, &cond.span)?.into_int_value();

        self.builder
            .build_conditional_branch(cond, then_bb, first_else_bb)
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        self.builder.position_at_end(then_bb);

        self.push_scope();

        for stmt in then_body {
            self.lower_stmt(stmt, stmt.span())?;

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                break;
            }
        }

        self.pop_scope();

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
        }

        self.builder.position_at_end(first_else_bb);

        if else_ifs.is_empty() {
            if let Some(body) = else_body {
                self.push_scope();

                for stmt in body {
                    self.lower_stmt(stmt, stmt.span())?;

                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        break;
                    }
                }

                self.pop_scope();
            }
        } else {
            let (cond, body) = &else_ifs[0];

            self.lower_if(cond, body, &else_ifs[1..], else_body, span)?;
        }

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
        }

        self.builder.position_at_end(merge_bb);

        Ok(())
    }
    fn lower_return(&mut self, value: Option<&MetaExpr>, span: &JunoSpan) -> Result<(), LLVMError> {
        match value {
            Some(expr) => {
                let value = self.lower_expr(expr, &expr.span)?;

                self.builder
                    .build_return(Some(&value))
                    .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
            }

            None => {
                self.builder
                    .build_return(None)
                    .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
            }
        }

        Ok(())
    }
    fn lower_assign(
        &mut self,
        target: SymbolId,
        value: &MetaExpr,
        span: &JunoSpan,
    ) -> Result<(), LLVMError> {
        let (ptr, _ty) = {
            let var = self.get_variable(target)?;
            (var.ptr, var.ty)
        };

        let value = self.lower_expr(value, &value.span)?;

        self.builder
            .build_store(ptr, value)
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
        Ok(())
    }
    fn lower_let(
        &mut self,
        name: SymbolId,
        ty: Option<&MetaType>,
        value: Option<&MetaExpr>,
        span: &JunoSpan,
    ) -> Result<(), LLVMError> {
        let ty =
            ty.ok_or_else(|| LLVMError::SpanMessage("let without type".to_string(), span.clone()))?;

        let llvm_ty = self.lower_type(ty, span)?;

        let ptr = self
            .builder
            .build_alloca(llvm_ty, name.as_str())
            .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;

        self.insert_variable(name, ptr, llvm_ty);

        if let Some(expr) = value {
            let value = self.lower_expr(expr, &expr.span)?;

            self.builder
                .build_store(ptr, value)
                .map_err(|e| LLVMError::SpanMessage(e.to_string(), span.clone()))?;
        }

        Ok(())
    }
}
