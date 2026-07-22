//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{Block, Expr, JunoSpan, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    AssignStmt(AssignStmt),
    Expr(Expr),
    Return(Option<Expr>, JunoSpan),
    Break(JunoSpan),
    Continue(JunoSpan),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Loop(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub span: JunoSpan,
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignStmt {
    pub span: JunoSpan,
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub span: JunoSpan,
    pub condition: Expr,
    pub then_block: Block,
    pub else_ifs: Vec<(Expr, Block)>,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub span: JunoSpan,
    pub condition: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub span: JunoSpan,
    pub init: Expr,
    pub iter: Expr,
    pub body: Block,
}
