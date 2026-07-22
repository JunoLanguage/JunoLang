#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::utils::{dummy_meta_function, dummy_span, empty_program, make_backend};

    #[test]
    fn lower_const_int_and_bool() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let dummy_fn = Box::leak(Box::new(dummy_meta_function()));
        backend.current_meta_function = Some(dummy_fn);

        let span = dummy_span();

        let int_expr = MetaExpr {
            kind: MetaExprKind::Const(MetaConst::Int(42, span), span),
            ty: MetaType::Named("i32".into(), span),
            span,
        };
        assert!(backend.lower_expr(&int_expr, &span).is_ok());

        let bool_expr = MetaExpr {
            kind: MetaExprKind::Const(MetaConst::Bool(true, span), span),
            ty: MetaType::Named("bool".into(), span),
            span,
        };
        assert!(backend.lower_expr(&bool_expr, &span).is_ok());
    }

    #[test]
    fn lower_unary_neg() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let dummy_fn = Box::leak(Box::new(dummy_meta_function()));
        backend.current_meta_function = Some(dummy_fn);
        let span = dummy_span();

        let expr = MetaExpr {
            kind: MetaExprKind::Unary {
                op: MetaUnOp::Neg,
                expr: Box::new(MetaExpr {
                    kind: MetaExprKind::Const(MetaConst::Int(5, span), span),
                    ty: MetaType::Named("i32".into(), span),
                    span,
                }),
                span,
            },
            ty: MetaType::Named("i32".into(), span),
            span,
        };
        assert!(backend.lower_expr(&expr, &span).is_ok());
    }

    #[test]
    fn lower_expr_variable_lookup() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, ctx) = make_backend(prog);
        backend.push_scope();

        let int_ty = ctx.i32_type();
        let ptr = ctx.create_builder().build_alloca(int_ty, "x").unwrap();
        backend.insert_variable("x", ptr, int_ty.as_basic_type_enum());

        let dummy_fn = Box::leak(Box::new(dummy_meta_function()));
        backend.current_meta_function = Some(dummy_fn);

        let var_expr = MetaExpr {
            kind: MetaExprKind::Var("x".into(), dummy_span()),
            ty: MetaType::Named("i32".into(), dummy_span()),
            span: dummy_span(),
        };
        assert!(backend.lower_expr(&var_expr, &dummy_span()).is_ok());
    }

    #[test]
    fn lower_expr_struct_init() {
        let mut prog = empty_program();
        prog.structs.insert(
            "Point".into(),
            MetaStruct {
                name: "Point".into(),
                fields: vec![
                    crate::metair::StructField {
                        ty: MetaType::Named("i32".into(), dummy_span()),
                        span: dummy_span(),
                    },
                    crate::metair::StructField {
                        ty: MetaType::Named("i32".into(), dummy_span()),
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            },
        );
        prog.struct_fields
            .insert("Point".into(), vec!["x".into(), "y".into()]);
        let prog = Box::leak(Box::new(prog));
        let (mut backend, ctx) = make_backend(prog);

        let meta_struct = prog.structs.get("Point").unwrap();
        backend.lower_struct(meta_struct, &dummy_span()).unwrap();

        let dummy_fn = Box::leak(Box::new(dummy_meta_function()));
        backend.current_meta_function = Some(dummy_fn);
        backend.push_scope();

        let init_expr = MetaExpr {
            kind: MetaExprKind::StructInit {
                span: dummy_span(),
                name: "Point".into(),
                fields: vec![
                    (
                        0,
                        MetaExpr {
                            kind: MetaExprKind::Const(
                                MetaConst::Int(1, dummy_span()),
                                dummy_span(),
                            ),
                            ty: MetaType::Named("i32".into(), dummy_span()),
                            span: dummy_span(),
                        },
                    ),
                    (
                        1,
                        MetaExpr {
                            kind: MetaExprKind::Const(
                                MetaConst::Int(2, dummy_span()),
                                dummy_span(),
                            ),
                            ty: MetaType::Named("i32".into(), dummy_span()),
                            span: dummy_span(),
                        },
                    ),
                ],
            },
            ty: MetaType::Named("Point".into(), dummy_span()),
            span: dummy_span(),
        };
        assert!(backend.lower_expr(&init_expr, &dummy_span()).is_ok());
    }
}
