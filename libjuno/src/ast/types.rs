//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::JunoSpan;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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
    BitAnd,
    BitOr,
    BitXOR,
    BitSHL,
    BitSHR,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Not,
    Neg,
    BitNot,
    Ref,
    Deref,
}

#[derive(Debug, Clone, PartialEq)]
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
            Type::Named(name, _) => write!(f, "{name}"),
            Type::Pointer(inner, _) => write!(f, "*{inner}"),
            Type::Reference(inner, _) => write!(f, "&{inner}"),
            Type::Array { elem, size, .. } => write!(f, "[{elem}; {size}]"),
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
