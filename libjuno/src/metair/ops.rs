//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ast::{BinOp, UnOp};
use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;

impl<'a> MetaIRGen<'a> {
    // =======================
    // Binary Operators
    // =======================

    pub(crate) fn lower_binop(&self, op: &BinOp) -> MetaBinOp {
        match op {
            BinOp::Add => MetaBinOp::Add,
            BinOp::Sub => MetaBinOp::Sub,
            BinOp::Mul => MetaBinOp::Mul,
            BinOp::Div => MetaBinOp::Div,
            BinOp::Mod => MetaBinOp::Mod,

            BinOp::Eq => MetaBinOp::Eq,
            BinOp::Neq => MetaBinOp::Neq,
            BinOp::Lt => MetaBinOp::Lt,
            BinOp::Lte => MetaBinOp::Lte,
            BinOp::Gt => MetaBinOp::Gt,
            BinOp::Gte => MetaBinOp::Gte,

            BinOp::And => MetaBinOp::And,
            BinOp::Or => MetaBinOp::Or,

            BinOp::BitAnd => MetaBinOp::BitAnd,
            BinOp::BitOr => MetaBinOp::BitOr,
            BinOp::BitXOR => MetaBinOp::BitXOR,
            BinOp::BitSHL => MetaBinOp::BitSHL,
            BinOp::BitSHR => MetaBinOp::BitSHR,

            _ => unreachable!("unsupported binary operator: {:?}", op),
        }
    }

    // =======================
    // Unary Operators
    // =======================

    pub(crate) fn lower_unop(&self, op: &UnOp) -> MetaUnOp {
        match op {
            UnOp::Neg => MetaUnOp::Neg,
            UnOp::Not => MetaUnOp::Not,

            UnOp::Ref => MetaUnOp::Ref,
            UnOp::Deref => MetaUnOp::Deref,

            UnOp::BitNot => MetaUnOp::BitNot,
        }
    }
}
