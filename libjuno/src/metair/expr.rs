use pest::Parser;
use pest::iterators::Pair;

use crate::ast::*;
use crate::builtin_registry;
use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;
use crate::parser::JunoASTParser;
use crate::{JunoParser, Rule};

impl<'a> MetaIRGen<'a> {
    // =======================
    // Expressions
    // =======================

    pub(crate) fn lower_expr(&mut self, expr: &Expr) -> MetaExpr {
        match expr {
            Expr::Integer(value, ty, span) => MetaExpr {
                span:*span,
                kind: MetaExprKind::Const(MetaConst::Int(*value,*span),*span),
                ty: ty
                    .as_ref()
                    .map(|t| self.lower_type(t))
                    .unwrap_or_else(|| MetaType::Named("i32".to_string(),*span)),
            },

            Expr::Fractional(value, ty, span) => MetaExpr {
                span:*span,
                kind: MetaExprKind::Const(
                    MetaConst::Fractional(*value,*span),
                   *span,
                ),
                ty: ty
                    .as_ref()
                    .map(|t| self.lower_type(t))
                    .unwrap_or_else(|| MetaType::Named("f32".to_string(),*span)),
            },

            Expr::Boolean(value, span) => MetaExpr {
                span:*span,
                kind: MetaExprKind::Const(MetaConst::Bool(*value,*span),*span),
                ty: MetaType::Named("bool".into(),*span),
            },

            Expr::Char(value, span) => MetaExpr {
                span:*span,
                kind: MetaExprKind::Const(MetaConst::Char(*value,*span),*span),
                ty: MetaType::Named("char".into(),*span),
            },

            Expr::String(value, span) => {
                let id = self.intern_string(value);

                MetaExpr {
                    span:*span,
                    kind: MetaExprKind::String(id,*span),
                    ty: MetaType::Pointer(
                        Box::new(MetaType::Named("char".into(),*span)),
                       *span,
                    ),
                }
            }

            Expr::Var(name, span) => MetaExpr {
                span:*span,
                kind: MetaExprKind::Var(name.clone(),*span),
                ty: self.lookup_local_type(name.clone()),
            },

            Expr::Array(values, span) => {
                let values: Vec<_> = values.iter().map(|e| self.lower_expr(e)).collect();

                let elem_ty = values
                    .first()
                    .map(|e| e.ty.clone())
                    .unwrap_or(MetaType::Unit(*span));

                let size = values.len() as u32;

                MetaExpr {
                    span:*span,
                    kind: MetaExprKind::Array(values,*span),
                    ty: MetaType::Array {
                        span:*span,
                        elem: Box::new(elem_ty),
                        size,
                    },
                }
            }

            Expr::StructInit(init) => {
                let ty = MetaType::Named(init.name.clone(), init.span);

                let fields = init
                    .fields
                    .iter()
                    .map(|field| {
                        let index = self.intern_struct_field(init.name.clone(), &field.name);

                        (index, self.lower_expr(&field.value))
                    })
                    .collect();

                MetaExpr {
                    span: init.span,
                    kind: MetaExprKind::StructInit {
                        name: init.name.clone(),
                        fields,
                        span: init.span,
                    },
                    ty,
                }
            }

            Expr::Call(call) => {
                let target = call.target.clone();

                let args = call
                    .args
                    .iter()
                    .map(|arg| match arg {
                        Arg::Positional(expr, span) => {
                            MetaArg::Pos(self.lower_expr(expr),*span)
                        }

                        Arg::Named(name, expr, span) => {
                            MetaArg::Named(name.clone(), self.lower_expr(expr),*span)
                        }
                    })
                    .collect();

                let ty = match self.find_function(&target) {
                    Some(function) => function
                        .return_type
                        .as_ref()
                        .map(|t| self.lower_type(t))
                        .unwrap_or(MetaType::Named("void".into(), call.span)),

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
                                            .collect();

                                    let mut parser = JunoASTParser::new("_".into());

                                    let ast_ty =
                                        parser.parse_type(parsed.first().unwrap().clone()).unwrap();

                                    self.lower_type(&ast_ty)
                                }
                            }
                        } else {
                            panic!("unknown function {}", target);
                        }
                    }
                };

                MetaExpr {
                    span: call.span,
                    kind: MetaExprKind::Call {
                        target,
                        args,
                        span: call.span,
                    },
                    ty,
                }
            }

            Expr::Binary(binary) => {
                let lhs = self.lower_expr(&binary.left);
                let rhs = self.lower_expr(&binary.right);

                let (lhs, rhs) = self.coerce_binary(lhs, rhs).unwrap();

                let ty = match binary.op {
                    BinOp::Eq
                    | BinOp::Neq
                    | BinOp::Lt
                    | BinOp::Lte
                    | BinOp::Gt
                    | BinOp::Gte
                    | BinOp::And
                    | BinOp::Or => MetaType::Named("bool".into(), binary.span),

                    _ => lhs.ty.clone(),
                };

                MetaExpr {
                    span: binary.span,
                    kind: MetaExprKind::Binary {
                        span: binary.span,
                        op: self.lower_binop(&binary.op),
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ty,
                }
            }

            Expr::Unary(unary) => {
                let expr = self.lower_expr(&unary.expr);

                let ty = match unary.op {
                    UnOp::Ref => MetaType::Pointer(Box::new(expr.ty.clone()), unary.span),

                    UnOp::Deref => match &expr.ty {
                        MetaType::Pointer(inner, _) | MetaType::Reference(inner, _) => {
                            (**inner).clone()
                        }

                        _ => expr.ty.clone(),
                    },

                    _ => expr.ty.clone(),
                };

                MetaExpr {
                    span: unary.span,
                    kind: MetaExprKind::Unary {
                        span: unary.span,
                        op: self.lower_unop(&unary.op),
                        expr: Box::new(expr),
                    },
                    ty,
                }
            }
        }
    }
}
