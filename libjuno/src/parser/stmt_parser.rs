//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::bail;
use pest::iterators::Pair;

use super::ParserState;
use crate::{Rule, ast::*};

impl ParserState {
    pub fn parse_block(&self, pair: Pair<Rule>) -> anyhow::Result<Block> {
        let span = pair.as_span();
        let mut stmts = Vec::new();

        for s in pair.into_inner() {
            stmts.push(self.parse_stmt(s)?);
        }

        Ok(Block {
            span: self.make_span(span),
            stmts,
        })
    }

    pub fn parse_stmt(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let stmt_span = pair.as_span();
        let span = self.make_span(stmt_span);
        let inner = pair.into_inner().next().expect("empty statement");

        match inner.as_rule() {
            Rule::let_stmt => self.parse_let(inner),
            Rule::assign_stmt => self.parse_assign_stmt(inner),
            Rule::expr_stmt => {
                let expr_token = inner.into_inner().next().expect("empty expr stmt");
                self.parse_expr(expr_token).map(Stmt::Expr)
            }
            Rule::return_stmt => {
                let maybe_expr = inner.into_inner().next();
                let expr = match maybe_expr {
                    Some(tok) => Some(self.parse_expr(tok)?),
                    None => None,
                };
                Ok(Stmt::Return(expr, span))
            }
            Rule::break_stmt => Ok(Stmt::Break(span)),
            Rule::continue_stmt => Ok(Stmt::Continue(span)),
            Rule::if_stmt => self.parse_if(inner),
            Rule::while_stmt => self.parse_while(inner),
            Rule::for_stmt => self.parse_for(inner),
            Rule::loop_stmt => self.parse_loop(inner),
            other => bail!(self.make_span_error(
                stmt_span,
                &format!("unexpected statement rule: {:?}", other)
            )),
        }
    }

    fn parse_let(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let mut_token = inner.next().expect("missing mutable token");
        let mutable = mut_token.as_str() == "mut";

        let name_token = if mutable {
            inner.next().expect("missing variable name")
        } else {
            mut_token
        };
        let name = self.clean_ident(name_token.as_str());

        let type_token = inner.next().expect("missing type annotation");
        let ty = self.parse_type(type_token)?;

        let value = match inner.next() {
            Some(tok) => Some(self.parse_expr(tok)?),
            None => None,
        };

        Ok(Stmt::Let(LetStmt {
            span,
            mutable,
            name,
            ty,
            value,
        }))
    }

    fn parse_assign_stmt(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let name_token = inner.next().expect("assignment target missing");
        let name = self.clean_ident(name_token.as_str());

        let value_token = inner.next().expect("assignment value missing");
        let value = self.parse_expr(value_token)?;

        Ok(Stmt::AssignStmt(AssignStmt { span, name, value }))
    }

    fn parse_if(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let condition = self.parse_expr(inner.next().expect("missing if condition"))?;
        let then_block = self.parse_block(inner.next().expect("missing then block"))?;

        let mut else_ifs = Vec::new();
        let mut else_block = None;

        for p in inner {
            match p.as_rule() {
                Rule::else_if => {
                    let mut i = p.into_inner();
                    let cond = self.parse_expr(i.next().expect("missing else-if condition"))?;
                    let block = self.parse_block(i.next().expect("missing else-if block"))?;
                    else_ifs.push((cond, block));
                }
                Rule::else_block => {
                    let token = p.into_inner().next().expect("empty else block");
                    else_block = Some(self.parse_block(token)?);
                }
                _ => {}
            }
        }

        Ok(Stmt::If(IfStmt {
            span,
            condition,
            then_block,
            else_ifs,
            else_block,
        }))
    }

    fn parse_while(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let condition = self.parse_expr(inner.next().expect("missing while condition"))?;
        let body = self.parse_block(inner.next().expect("missing while body"))?;

        Ok(Stmt::While(WhileStmt {
            span,
            condition,
            body,
        }))
    }

    fn parse_for(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let init = self.parse_expr(inner.next().expect("missing for init"))?;
        let iter = self.parse_expr(inner.next().expect("missing for iter"))?;
        let body = self.parse_block(inner.next().expect("missing for body"))?;

        Ok(Stmt::For(ForStmt {
            span,
            init,
            iter,
            body,
        }))
    }

    fn parse_loop(&self, pair: Pair<Rule>) -> anyhow::Result<Stmt> {
        let body_token = pair.into_inner().next().expect("missing loop body");
        let body = self.parse_block(body_token)?;
        Ok(Stmt::Loop(body))
    }
}
