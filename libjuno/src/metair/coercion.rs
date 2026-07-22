//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;

impl<'a> MetaIRGen<'a> {
    // =======================
    // Type Coercion
    // =======================

    pub(crate) fn coerce_expr(&self, mut expr: MetaExpr, expected: &MetaType) -> MetaExpr {
        match (&mut expr.kind, &expr.ty, expected) {
            (_, actual, expected) if actual == expected => expr,

            (
                MetaExprKind::Const(MetaConst::Int(_, _), _),
                MetaType::Named(_, _),
                MetaType::Named(_, _),
            ) => {
                expr.ty = expected.clone();
                expr
            }

            (
                MetaExprKind::Array(values, _),
                MetaType::Array { .. },
                MetaType::Array {
                    span,
                    elem: expected_elem,
                    size,
                },
            ) => {
                assert!(
                    values.len() <= *size as usize,
                    "{:?}",
                    self.make_span_error("array too large", *span)
                );

                for value in values.iter_mut() {
                    *value = self.coerce_expr(value.clone(), expected_elem);
                }

                expr.ty = expected.clone();
                expr
            }

            _ => {
                panic!(
                    "{:?}",
                    self.make_span_error(
                        &format!("type mismatch: expected {}, got {}", expected, expr.ty),
                        expr.span
                    )
                )
            }
        }
    }

    pub(crate) fn coerce_binary(
        &self,
        mut lhs: MetaExpr,
        mut rhs: MetaExpr,
    ) -> Result<(MetaExpr, MetaExpr), miette::Error> {
        if lhs.ty == rhs.ty {
            return Ok((lhs, rhs));
        }

        match (&lhs.kind, &rhs.kind) {
            (_, MetaExprKind::Const(MetaConst::Int(_, _), _)) => {
                rhs = self.coerce_expr(rhs, &lhs.ty);
                Ok((lhs, rhs))
            }

            (MetaExprKind::Const(MetaConst::Int(_, _), _), _) => {
                lhs = self.coerce_expr(lhs, &rhs.ty);
                Ok((lhs, rhs))
            }

            _ => Err(self.make_span_error(
                &format!("type mismatch: {} vs {}", lhs.ty, rhs.ty,),
                lhs.span,
            )),
        }
    }
}
