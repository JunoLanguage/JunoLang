//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::ast::*;
use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;

impl<'a> MetaIRGen<'a> {
    // =======================
    // Blocks
    // =======================

    pub(crate) fn lower_block(
        &mut self,
        block: &Block,
    ) -> (Vec<MetaStmt>, HashMap<String, MetaType>) {
        self.locals.push(HashMap::new());

        let body = block
            .stmts
            .iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect();

        let locals = self.locals.pop().unwrap();

        (body, locals)
    }

    // =======================
    // Statements
    // =======================

    fn lower_stmt(&mut self, stmt: &Stmt) -> MetaStmt {
        match stmt {
            Stmt::Let(stmt) => self.lower_let(stmt),

            Stmt::AssignStmt(assign) => MetaStmt::Assign {
                span: assign.span,
                target: assign.name.clone(),
                value: self.lower_expr(&assign.value),
            },

            Stmt::Expr(expr) => MetaStmt::Expr(self.lower_expr(expr)),

            Stmt::Return(expr, span) => {
                MetaStmt::Return(expr.as_ref().map(|e| self.lower_expr(e)), *span)
            }

            Stmt::Break(span) => MetaStmt::Break(*span),

            Stmt::Continue(span) => MetaStmt::Continue(*span),

            Stmt::If(stmt) => MetaStmt::If {
                span: stmt.span,
                cond: self.lower_expr(&stmt.condition),

                then_body: self.lower_block(&stmt.then_block).0,

                else_ifs: stmt
                    .else_ifs
                    .iter()
                    .map(|(cond, body)| (self.lower_expr(cond), self.lower_block(body).0))
                    .collect(),

                else_body: stmt
                    .else_block
                    .as_ref()
                    .map(|body| self.lower_block(body).0),
            },

            Stmt::While(stmt) => {
                let cond = self.lower_expr(&stmt.condition);

                MetaStmt::Loop {
                    span: stmt.span,
                    body: vec![MetaStmt::If {
                        span: cond.span,

                        cond: MetaExpr {
                            span: cond.span,

                            kind: MetaExprKind::Unary {
                                span: cond.span,
                                op: MetaUnOp::Not,
                                expr: Box::new(cond.clone()),
                            },

                            ty: MetaType::Named("bool".to_string(), cond.span),
                        },

                        then_body: vec![MetaStmt::Break(stmt.span)],

                        else_ifs: Vec::new(),

                        else_body: Some(self.lower_block(&stmt.body).0),
                    }],
                }
            }

            Stmt::Loop(body) => MetaStmt::Loop {
                span: body.span,
                body: self.lower_block(body).0,
            },

            Stmt::For(stmt) => {
                // TODO
                MetaStmt::Break(stmt.span)
            }
        }
    }

    fn lower_let(&mut self, stmt: &LetStmt) -> MetaStmt {
        let declared_ty = self.lower_type(&stmt.ty);

        let value = stmt.value.as_ref().map(|expr| {
            let value = self.lower_expr(expr);
            self.coerce_expr(value, &declared_ty)
        });

        self.insert_local(stmt.name.clone(), declared_ty.clone());

        MetaStmt::Let {
            span: stmt.span,
            name: stmt.name.clone(),
            mutable: stmt.mutable,
            ty: Some(declared_ty),
            value,
        }
    }
}
