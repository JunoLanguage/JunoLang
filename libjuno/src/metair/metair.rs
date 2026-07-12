use std::{collections::HashMap, fmt};

// =======================
// IDs
// =======================

pub type SymbolId = String;
pub type StringId = u32;
pub type FunctionId = String;
pub type TypeId = String;
// =======================
// Program
// =======================

#[derive(Debug, Clone)]
pub struct MetaStruct {
    pub name: SymbolId,
    pub fields: Vec<MetaField>,
}

#[derive(Debug, Clone)]
pub struct MetaField {
    pub index: u32,
    pub ty: MetaType,
}

#[derive(Debug, Clone)]
pub struct MetaProgram {
    pub functions: HashMap<SymbolId, MetaFunction>,
    pub structs: HashMap<SymbolId, MetaStruct>,
    pub struct_fields: HashMap<SymbolId, Vec<String>>,
    pub string_table: Vec<String>,
    pub symbol_table: Vec<String>,
}

// =======================
// Functions
// =======================

#[derive(Debug, Clone)]
pub struct MetaFunction {
    pub name: SymbolId,
    pub params: Vec<MetaParam>,
    pub ret: Option<MetaType>,
    pub body: Vec<MetaStmt>,
}

#[derive(Debug, Clone)]
pub struct MetaParam {
    pub name: SymbolId,
    pub ty: MetaType,
}

// =======================
// Statements
// =======================

#[derive(Debug, Clone)]
pub enum MetaStmt {
    Let {
        name: SymbolId,
        mutable: bool,
        ty: Option<MetaType>,
        value: Option<MetaExpr>,
    },

    Assign {
        target: SymbolId,
        value: MetaExpr,
    },

    Expr(MetaExpr),

    Return(Option<MetaExpr>),

    Break,
    Continue,

    If {
        cond: MetaExpr,
        then_body: Vec<MetaStmt>,
        else_ifs: Vec<(MetaExpr, Vec<MetaStmt>)>,
        else_body: Option<Vec<MetaStmt>>,
    },

    Loop {
        body: Vec<MetaStmt>,
    },
}

// =======================
// Expressions
// =======================
#[derive(Debug, Clone)]
pub struct MetaExpr {
    pub kind: MetaExprKind,
    pub ty: MetaType,
}
#[derive(Debug, Clone)]
pub enum MetaExprKind {
    Const(MetaConst),

    Var(SymbolId),

    String(StringId),

    Call {
        target: SymbolId,
        args: Vec<MetaArg>,
    },

    Binary {
        op: MetaBinOp,
        lhs: Box<MetaExpr>,
        rhs: Box<MetaExpr>,
    },

    Unary {
        op: MetaUnOp,
        expr: Box<MetaExpr>,
    },

    Array(Vec<MetaExpr>),

    Void,

    StructInit {
        name: SymbolId,
        fields: Vec<(u32, MetaExpr)>,
    },
}
#[derive(Debug, Clone)]
pub enum MetaArg {
    Pos(MetaExpr),
    Named(SymbolId, MetaExpr),
}

// =======================
// Constants
// =======================

#[derive(Debug, Clone)]
pub enum MetaConst {
    Int(i64),
    Bool(bool),
    Char(char),
}

// =======================
// Operations
// =======================

#[derive(Debug, Clone)]
pub enum MetaBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

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
pub enum MetaUnOp {
    Neg,
    Not,
    Ref,
    Deref,
}

// =======================
// Types
// =======================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaType {
    Named(SymbolId),

    Pointer(Box<MetaType>),
    Reference(Box<MetaType>),

    Array { elem: Box<MetaType>, size: u32 },

    Unit,
}

// =======================
// Debug helpers
// =======================

impl fmt::Display for MetaConst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaConst::Int(i) => write!(f, "{i}"),
            MetaConst::Bool(b) => write!(f, "{b}"),
            MetaConst::Char(c) => write!(f, "{c}"),
        }
    }
}
