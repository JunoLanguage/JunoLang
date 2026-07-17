use crate::ast::JunoSpan;
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
    pub span: JunoSpan,
    pub name: SymbolId,
    pub fields: Vec<MetaField>,
}

#[derive(Debug, Clone)]
pub struct MetaField {
    pub span: JunoSpan,
    pub index: u32,
    pub ty: MetaType,
}

#[derive(Debug, Clone)]
pub struct MetaProgram {
    pub span: JunoSpan,
    pub functions: HashMap<SymbolId, MetaFunction>,
    pub declarations: HashMap<SymbolId, MetaDeclaration>,
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
    pub span: JunoSpan,
    pub name: SymbolId,
    pub params: Vec<MetaParam>,
    pub ret: Option<MetaType>,
    pub body: Vec<MetaStmt>,
}

#[derive(Debug, Clone)]
pub struct MetaDeclaration {
    pub span: JunoSpan,
    pub name: SymbolId,
    pub params: Vec<MetaParam>,
    pub ret: Option<MetaType>,
}

#[derive(Debug, Clone)]
pub struct MetaParam {
    pub span: JunoSpan,
    pub name: SymbolId,
    pub ty: MetaType,
}

// =======================
// Statements
// =======================

#[derive(Debug, Clone)]
pub enum MetaStmt {
    Let {
        span: JunoSpan,
        name: SymbolId,
        mutable: bool,
        ty: Option<MetaType>,
        value: Option<MetaExpr>,
    },

    Assign {
        span: JunoSpan,
        target: SymbolId,
        value: MetaExpr,
    },

    Expr(MetaExpr),

    Return(Option<MetaExpr>, JunoSpan),

    Break(JunoSpan),
    Continue(JunoSpan),

    If {
        span: JunoSpan,
        cond: MetaExpr,
        then_body: Vec<MetaStmt>,
        else_ifs: Vec<(MetaExpr, Vec<MetaStmt>)>,
        else_body: Option<Vec<MetaStmt>>,
    },

    Loop {
        span: JunoSpan,
        body: Vec<MetaStmt>,
    },
}
impl MetaStmt {
    pub fn span(&self) -> &JunoSpan {
        match self {
            MetaStmt::Assign { span, .. } => span,
            MetaStmt::Let { span, .. } => span,
            MetaStmt::Assign { span, .. } => span,
            MetaStmt::Expr(meta_expr) => &meta_expr.span,
            MetaStmt::Return(meta_expr, juno_span) => juno_span,
            MetaStmt::Break(juno_span) => juno_span,
            MetaStmt::Continue(juno_span) => juno_span,
            MetaStmt::If { span, .. } => span,
            MetaStmt::Loop { span, .. } => span,
        }
    }
}
// =======================
// Expressions
// =======================
#[derive(Debug, Clone)]
pub struct MetaExpr {
    pub span: JunoSpan,
    pub kind: MetaExprKind,
    pub ty: MetaType,
}
#[derive(Debug, Clone)]
pub enum MetaExprKind {
    Const(MetaConst, JunoSpan),

    Var(SymbolId, JunoSpan),

    String(StringId, JunoSpan),

    Call {
        span: JunoSpan,
        target: SymbolId,
        args: Vec<MetaArg>,
    },

    Binary {
        span: JunoSpan,
        op: MetaBinOp,
        lhs: Box<MetaExpr>,
        rhs: Box<MetaExpr>,
    },

    Unary {
        span: JunoSpan,
        op: MetaUnOp,
        expr: Box<MetaExpr>,
    },

    Array(Vec<MetaExpr>, JunoSpan),

    Void(JunoSpan),

    StructInit {
        span: JunoSpan,
        name: SymbolId,
        fields: Vec<(u32, MetaExpr)>,
    },
}
#[derive(Debug, Clone)]
pub enum MetaArg {
    Pos(MetaExpr, JunoSpan),
    Named(SymbolId, MetaExpr, JunoSpan),
}

// =======================
// Constants
// =======================

#[derive(Debug, Clone)]
pub enum MetaConst {
    Int(i64, JunoSpan),
    Bool(bool, JunoSpan),
    Char(char, JunoSpan),
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

#[derive(Debug, Clone)]
pub enum MetaType {
    Named(SymbolId, JunoSpan),

    Pointer(Box<MetaType>, JunoSpan),
    Reference(Box<MetaType>, JunoSpan),

    Array {
        span: JunoSpan,
        elem: Box<MetaType>,
        size: u32,
    },

    Unit(JunoSpan),
}

impl PartialEq for MetaType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MetaType::Named(a, _), MetaType::Named(b, _)) => a == b,

            (MetaType::Pointer(a, _), MetaType::Pointer(b, _)) => a == b,

            (MetaType::Reference(a, _), MetaType::Reference(b, _)) => a == b,

            (
                MetaType::Array {
                    elem: ae,
                    size: asize,
                    ..
                },
                MetaType::Array {
                    elem: be,
                    size: bsize,
                    ..
                },
            ) => ae == be && asize == bsize,

            _ => false,
        }
    }
}

impl Eq for MetaType {}

impl fmt::Display for MetaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaType::Named(name, _) => {
                write!(f, "{name}")
            }

            MetaType::Pointer(inner, _) => {
                write!(f, "*{inner} (ptr)")
            }

            MetaType::Reference(inner, _) => {
                write!(f, "&{inner} (ref)")
            }

            MetaType::Array { elem, size, .. } => {
                write!(f, "[{elem}; {size}] (arr)")
            }
            MetaType::Unit(juno_span) => {
                write!(f, "Unit")
            }
        }
    }
}
// =======================
// Debug helpers
// =======================

impl fmt::Display for MetaConst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaConst::Int(i, span) => write!(f, "{i} at {:#?}", span),
            MetaConst::Bool(b, span) => write!(f, "{b} at {:#?}", span),
            MetaConst::Char(c, span) => write!(f, "{c} at {:#?}", span),
        }
    }
}
