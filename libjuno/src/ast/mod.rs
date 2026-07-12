//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Import(Import),
    Struct(StructDef),
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub raw_name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    AssignStmt(AssignStmt),
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Loop(Block),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_ifs: Vec<(Expr, Block)>,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub init: Expr,
    pub iter: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Boolean(bool),
    String(String),
    Char(char),

    Var(String),

    Call(Call),

    Array(Vec<Expr>),

    StructInit(StructInit),

    Binary(BinaryExpr),
    Unary(UnaryExpr),
}
#[derive(Debug, Clone)]
pub struct Call {
    pub target: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Expr),
    Named(String, Expr),
}
#[derive(Debug, Clone)]
pub struct BinaryExpr {
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
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct StructInit {
    pub name: String,
    pub fields: Vec<StructInitField>,
}

#[derive(Debug, Clone)]
pub struct StructInitField {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),

    Pointer(Box<Type>),
    Reference(Box<Type>),

    Array { elem: Box<Type>, size: u32 },

    Generic { base: String, args: Vec<Type> },
}

pub type FilePath = String;
