use std::fmt;

// =======================
// IDs
// =======================

pub type SymbolId = u32;
pub type StringId = u32;
pub type FunctionId = u32;

// =======================
// Program
// =======================
pub type TypeId = u32;

#[derive(Debug, Clone)]
pub struct MetaStruct {
    pub id: TypeId,
    pub name: SymbolId,
    pub fields: Vec<MetaField>,
}

#[derive(Debug, Clone)]
pub struct MetaField {
    pub name: SymbolId,
    pub ty: MetaType,
}

#[derive(Debug, Clone)]
pub struct MetaProgram {
    pub functions: Vec<MetaFunction>,
    pub structs: Vec<MetaStruct>,
    pub string_table: Vec<String>,
    pub symbol_table: Vec<String>,
}

// =======================
// Functions
// =======================

#[derive(Debug, Clone)]
pub struct MetaFunction {
    pub id: FunctionId,
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

    Var (SymbolId),

    String(StringId),

    Call {
        target: Vec<SymbolId>,
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
        fields: Vec<(SymbolId, MetaExpr)>,
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

    Array {
        elem: Box<MetaType>,
        size: u32,
    },

    Unit
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
