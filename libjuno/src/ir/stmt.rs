use crate::ir::backend::LoopFrame;
use crate::metair::*;

use super::*;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_stmt(&mut self, stmt: &MetaStmt) -> Result<(), LLVMError> {
        match stmt {
            MetaStmt::Let {
                name,
                mutable: _,
                ty,
                value,
            } => self.lower_let(name.clone(), ty.as_ref(), value.as_ref()),
            MetaStmt::Break => self.lower_break(),
            MetaStmt::If {
                cond,
                then_body,
                else_ifs,
                else_body,
            } => self.lower_if(cond, then_body, else_ifs, else_body.as_ref()),
            MetaStmt::Continue => self.lower_continue(),
            MetaStmt::Loop { body } => self.lower_loop(body),
            MetaStmt::Return(expr) => self.lower_return(expr.as_ref()),
            MetaStmt::Assign { target, value } => self.lower_assign(target.clone(), value),
            MetaStmt::Expr(expr) => self.lower_expr_stmt(expr),
        }
    }
    fn lower_expr_stmt(&mut self, expr: &MetaExpr) -> Result<(), LLVMError> {
        match &expr.kind {
            MetaExprKind::Call { target, args } => {
                self.lower_call(target.clone(), &args)?;
            }

            _ => {
                self.lower_expr(expr)?;
            }
        }

        Ok(())
    }
    fn lower_continue(&mut self) -> Result<(), LLVMError> {
        let frame = self
            .loop_stack
            .last()
            .ok_or_else(|| LLVMError::Message("continue used outside of a loop".into()))?;

        self.builder
            .build_unconditional_branch(frame.continue_block)
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        Ok(())
    }
    fn lower_break(&mut self) -> Result<(), LLVMError> {
        let frame = self
            .loop_stack
            .last()
            .ok_or_else(|| LLVMError::Message("break used outside of a loop".into()))?;

        self.builder
            .build_unconditional_branch(frame.break_block)
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        Ok(())
    }
    fn lower_loop(&mut self, body: &[MetaStmt]) -> Result<(), LLVMError> {
        let function = self.current_function();

        let header_bb = self.context.append_basic_block(function, "loop.header");

        let body_bb = self.context.append_basic_block(function, "loop.body");

        let exit_bb = self.context.append_basic_block(function, "loop.exit");

        self.builder
            .build_unconditional_branch(header_bb)
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        self.builder.position_at_end(header_bb);

        self.builder
            .build_unconditional_branch(body_bb)
            .map_err(|e| LLVMError::Message(e.to_string()))?;

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

            self.lower_stmt(stmt)?;
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
                .map_err(|e| LLVMError::Message(e.to_string()))?;
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
    ) -> Result<(), LLVMError> {
        let function = self.current_function();

        let merge_bb = self.context.append_basic_block(function, "if.merge");

        let then_bb = self.context.append_basic_block(function, "if.then");

        let first_else_bb = self.context.append_basic_block(function, "if.else");

        let cond = self.lower_expr(cond)?.into_int_value();

        self.builder
            .build_conditional_branch(cond, then_bb, first_else_bb)
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        self.builder.position_at_end(then_bb);

        self.push_scope();

        for stmt in then_body {
            self.lower_stmt(stmt)?;

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
                .map_err(|e| LLVMError::Message(e.to_string()))?;
        }

        self.builder.position_at_end(first_else_bb);

        if else_ifs.is_empty() {
            if let Some(body) = else_body {
                self.push_scope();

                for stmt in body {
                    self.lower_stmt(stmt)?;

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

            self.lower_if(cond, body, &else_ifs[1..], else_body)?;
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
                .map_err(|e| LLVMError::Message(e.to_string()))?;
        }

        self.builder.position_at_end(merge_bb);

        Ok(())
    }
    fn lower_return(&mut self, value: Option<&MetaExpr>) -> Result<(), LLVMError> {
        match value {
            Some(expr) => {
                let value = self.lower_expr(expr)?;

                self.builder
                    .build_return(Some(&value))
                    .map_err(|e| LLVMError::Message(e.to_string()))?;
            }

            None => {
                self.builder
                    .build_return(None)
                    .map_err(|e| LLVMError::Message(e.to_string()))?;
            }
        }

        Ok(())
    }
    fn lower_assign(&mut self, target: SymbolId, value: &MetaExpr) -> Result<(), LLVMError> {
        let (ptr, _ty) = {
            let var = self.get_variable(target)?;
            (var.ptr, var.ty)
        };

        let value = self.lower_expr(value)?;

        self.builder
            .build_store(ptr, value)
            .map_err(|e| LLVMError::Message(e.to_string()))?;
        Ok(())
    }
    fn lower_let(
        &mut self,
        name: SymbolId,
        ty: Option<&MetaType>,
        value: Option<&MetaExpr>,
    ) -> Result<(), LLVMError> {
        let ty = ty.ok_or_else(|| LLVMError::Message("let without type".into()))?;

        let llvm_ty = self.lower_type(ty)?;

        let ptr = self
            .builder
            .build_alloca(llvm_ty, name.as_str())
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        self.insert_variable(name, ptr, llvm_ty);

        if let Some(expr) = value {
            let value = self.lower_expr(expr)?;

            self.builder
                .build_store(ptr, value)
                .map_err(|e| LLVMError::Message(e.to_string()))?;
        }

        Ok(())
    }
}
