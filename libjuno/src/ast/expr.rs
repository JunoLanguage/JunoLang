//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{JunoSpan, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64, Option<Type>, JunoSpan),
    Fractional(f64, Option<Type>, JunoSpan),
    Boolean(bool, JunoSpan),
    String(String, JunoSpan),
    Char(char, JunoSpan),
    Var(String, JunoSpan),
    Call(Call),
    Array(Vec<Expr>, JunoSpan),
    StructInit(StructInit),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub span: JunoSpan,
    pub target: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Positional(Expr, JunoSpan),
    Named(String, Expr, JunoSpan),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub span: JunoSpan,
    pub left: Box<Expr>,
    pub op: super::types::BinOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub span: JunoSpan,
    pub op: super::types::UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInit {
    pub span: JunoSpan,
    pub name: String,
    pub fields: Vec<StructInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInitField {
    pub span: JunoSpan,
    pub name: String,
    pub value: Expr,
}
