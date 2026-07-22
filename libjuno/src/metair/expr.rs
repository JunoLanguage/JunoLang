//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use pest::{Parser, iterators::Pair};

use crate::{
    JunoParser, MetaArg, MetaConst, MetaExpr, MetaExprKind, MetaIRGen, MetaType, Rule,
    ast::{Arg, BinOp, Expr, UnOp},
    builtin_registry,
    parser::JunoASTParser,
};

impl<'a> MetaIRGen<'a> {
    pub(crate) fn lower_expr(&mut self, expr: &Expr) -> MetaExpr {
        match expr.clone() {
            Expr::Integer(value, ty, span) => MetaExpr {
                span,
                kind: MetaExprKind::Const(MetaConst::Int(value, span), span),
                ty: ty
                    .as_ref()
                    .map(|t| self.lower_type(t))
                    .unwrap_or_else(|| MetaType::Named("i32".into(), span)),
            },

            Expr::Fractional(value, ty, span) => MetaExpr {
                span,
                kind: MetaExprKind::Const(MetaConst::Fractional(value, span), span),
                ty: ty
                    .as_ref()
                    .map(|t| self.lower_type(t))
                    .unwrap_or_else(|| MetaType::Named("f32".into(), span)),
            },

            Expr::Boolean(value, span) => MetaExpr {
                span,
                kind: MetaExprKind::Const(MetaConst::Bool(value, span), span),
                ty: MetaType::Named("bool".into(), span),
            },

            Expr::Char(value, span) => MetaExpr {
                span,
                kind: MetaExprKind::Const(MetaConst::Char(value, span), span),
                ty: MetaType::Named("char".into(), span),
            },

            Expr::String(value, span) => {
                let id = self.intern_string(&value);
                MetaExpr {
                    span,
                    kind: MetaExprKind::String(id, span),
                    ty: MetaType::Pointer(Box::new(MetaType::Named("char".into(), span)), span),
                }
            }

            Expr::Var(name, span) => {
                let ty = self.lookup_local_type(name.clone());
                MetaExpr {
                    span,
                    kind: MetaExprKind::Var(name, span),
                    ty,
                }
            }

            Expr::Array(values, span) => {
                let len = values.len();
                let mut it = values.into_iter();

                let first = it.next();
                let mut lowered: Vec<MetaExpr> = Vec::with_capacity(len);
                let mut elem_ty = None;

                if let Some(e) = first {
                    let first_lowered = self.lower_expr(&e);
                    elem_ty = Some(first_lowered.ty.clone());
                    lowered.push(first_lowered);
                }

                for e in it {
                    lowered.push(self.lower_expr(&e));
                }

                let elem_ty = elem_ty.unwrap_or(MetaType::Unit(span));
                let size = lowered.len() as u32;

                MetaExpr {
                    span,
                    kind: MetaExprKind::Array(lowered, span),
                    ty: MetaType::Array {
                        span,
                        elem: Box::new(elem_ty),
                        size,
                    },
                }
            }

            Expr::StructInit(init) => {
                let name = init.name;
                let span = init.span;

                let fields: Vec<_> = init
                    .fields
                    .into_iter()
                    .map(|field| {
                        let index = self.intern_struct_field(name.clone(), &field.name);
                        (index, self.lower_expr(&field.value))
                    })
                    .collect();

                let ty = MetaType::Named(name.clone(), span);

                MetaExpr {
                    span,
                    kind: MetaExprKind::StructInit { name, fields, span },
                    ty,
                }
            }

            Expr::Call(call) => {
                let target = call.target;
                let span = call.span;

                let args: Vec<_> = call
                    .args
                    .into_iter()
                    .map(|arg| match arg {
                        Arg::Positional(expr, s) => MetaArg::Pos(self.lower_expr(&expr), s),
                        Arg::Named(name, expr, s) => {
                            MetaArg::Named(name, self.lower_expr(&expr), s)
                        }
                    })
                    .collect();

                let ty = match self.find_function(&target) {
                    Some(function) => function
                        .return_type
                        .as_ref()
                        .map(|t| self.lower_type(t))
                        .unwrap_or_else(|| MetaType::Named("void".into(), span)),

                    None => {
                        if let Some(decl) = self.declarations.get(&target) {
                            decl.ret
                                .clone()
                                .unwrap_or(MetaType::Named("void".into(), decl.span))
                        } else if let Some(builtin) = builtin_registry::get_builtin(&target) {
                            match &builtin.declare {
                                builtin_registry::BuiltinEnum::Function { return_type, .. } => {
                                    let parsed: Vec<Pair<Rule>> =
                                        JunoParser::parse(Rule::type_, return_type)
                                            .unwrap()
                                            .into_iter()
                                            .collect();

                                    let parser = JunoASTParser::new("_".into());

                                    let ast_ty = parser
                                        .parse_type(parsed.into_iter().next().unwrap())
                                        .unwrap();

                                    self.lower_type(&ast_ty)
                                }
                            }
                        } else {
                            panic!("unknown function {}", target);
                        }
                    }
                };

                MetaExpr {
                    span,
                    kind: MetaExprKind::Call { target, args, span },
                    ty,
                }
            }

            Expr::Binary(binary) => {
                let op = binary.op;
                let span = binary.span;
                let lhs = self.lower_expr(&binary.left);
                let rhs = self.lower_expr(&binary.right);
                let (lhs, rhs) = self.coerce_binary(lhs, rhs).unwrap();

                let ty = match op {
                    BinOp::Eq
                    | BinOp::Neq
                    | BinOp::Lt
                    | BinOp::Lte
                    | BinOp::Gt
                    | BinOp::Gte
                    | BinOp::And
                    | BinOp::Or => MetaType::Named("bool".into(), span),

                    _ => lhs.ty.clone(),
                };

                MetaExpr {
                    span,
                    kind: MetaExprKind::Binary {
                        span,
                        op: self.lower_binop(&op),
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ty,
                }
            }

            Expr::Unary(unary) => {
                let op = unary.op;
                let span = unary.span;
                let expr = self.lower_expr(&unary.expr);

                let ty = match op {
                    UnOp::Ref => MetaType::Pointer(Box::new(expr.ty.clone()), span),
                    UnOp::Deref => match &expr.ty {
                        MetaType::Pointer(inner, _) | MetaType::Reference(inner, _) => {
                            (**inner).clone()
                        }
                        _ => expr.ty.clone(),
                    },
                    _ => expr.ty.clone(),
                };

                MetaExpr {
                    span,
                    kind: MetaExprKind::Unary {
                        span,
                        op: self.lower_unop(&op),
                        expr: Box::new(expr),
                    },
                    ty,
                }
            }
        }
    }
}
