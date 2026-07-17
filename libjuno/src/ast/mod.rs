//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::anyhow;
use miette::{LabeledSpan, NamedSource, Report, SourceOffset, SourceSpan};
use pest::Span;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JunoSpan {
    pub source_code: Arc<str>,
    pub source_file_name: Arc<str>,
    pub start: usize,
    pub end: usize,
}

impl JunoSpan {
    pub fn text(&self) -> &str {
        &self.source_code[self.start..self.end]
    }
    pub fn new(
        start: usize,
        end: usize,
        source_code: Arc<str>,
        source_file_name: Arc<str>,
    ) -> Self {
        Self {
            start,
            end,
            source_code,
            source_file_name,
        }
    }
    pub fn with_source(&self, source_code: &str, source_file_name: &str) -> Self {
        return Self {
            start: self.start,
            end: self.end,
            source_code: source_code.into(),
            source_file_name: source_file_name.into(),
        };
    }
}
use std::convert::TryFrom;

impl<'a> TryFrom<&'a JunoSpan> for Span<'a> {
    type Error = anyhow::Error;
    fn try_from(span: &'a JunoSpan) -> anyhow::Result<Self> {
        Span::new(&span.text(), span.start, span.end).ok_or(anyhow!("invalid span"))
    }
}
impl<'a> From<pest::Span<'a>> for JunoSpan {
    fn from(span: pest::Span<'a>) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
            ..Default::default()
        }
    }
}

impl JunoSpan {
    pub fn err_to_report(&self, label: &str) -> Report {
        let source = NamedSource::new(&self.source_file_name, self.source_code.to_string());

        let report = miette::miette!(
            labels = vec![LabeledSpan::at(self.start..self.end, label)],
            "Error"
        )
        .with_source_code(source);

        report
    }
}
#[derive(Debug, Clone)]
pub struct Program {
    pub span: JunoSpan,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function, JunoSpan),
    Import(Import, JunoSpan),
    Struct(StructDef, JunoSpan),
    Declaration(Declaration, JunoSpan),
}

#[derive(Debug, Clone)]
pub struct Import {
    pub span: JunoSpan,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub span: JunoSpan,
    pub name: String,
    pub raw_name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}
#[derive(Debug, Clone)]
pub struct Declaration {
    pub span: JunoSpan,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub span: JunoSpan,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub span: JunoSpan,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub span: JunoSpan,
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub span: JunoSpan,
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub span: JunoSpan,
    pub condition: Expr,
    pub then_block: Block,
    pub else_ifs: Vec<(Expr, Block)>,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub span: JunoSpan,
    pub condition: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub span: JunoSpan,
    pub init: Expr,
    pub iter: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64, JunoSpan),
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
#[derive(Debug, Clone)]
pub struct Call {
    pub span: JunoSpan,
    pub target: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Expr, JunoSpan),
    Named(String, Expr, JunoSpan),
}
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub span: JunoSpan,
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    DivFloor,

    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,

    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub span: JunoSpan,
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Not,
    Neg,
    Ref,
    Deref,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub span: JunoSpan,
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub span: JunoSpan,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct StructInit {
    pub span: JunoSpan,
    pub name: String,
    pub fields: Vec<StructInitField>,
}

#[derive(Debug, Clone)]
pub struct StructInitField {
    pub span: JunoSpan,
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Type {
    Named(String, JunoSpan),

    Pointer(Box<Type>, JunoSpan),
    Reference(Box<Type>, JunoSpan),

    Array {
        span: JunoSpan,
        elem: Box<Type>,
        size: u32,
    },

    Generic {
        span: JunoSpan,
        base: String,
        args: Vec<Type>,
    },
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Named(name, _) => {
                write!(f, "{name}")
            }

            Type::Pointer(inner, _) => {
                write!(f, "*{inner}")
            }

            Type::Reference(inner, _) => {
                write!(f, "&{inner}")
            }

            Type::Array { elem, size, .. } => {
                write!(f, "[{elem}; {size}]")
            }

            Type::Generic { base, args, .. } => {
                write!(f, "{base}<")?;

                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }

                write!(f, ">")
            }
        }
    }
}

pub type FilePath = String;
