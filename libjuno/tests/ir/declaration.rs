#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::utils::{dummy_span, empty_program, make_backend};

    #[test]
    fn lower_declare_adds_external_function() {
        let mut prog = empty_program();
        prog.declarations.insert(
            "add".into(),
            MetaDeclaration {
                name: "add".into(),
                params: vec![
                    crate::metair::MetaParam {
                        name: "a".into(),
                        ty: MetaType::Named("i32".into(), dummy_span()),
                        span: dummy_span(),
                    },
                    crate::metair::MetaParam {
                        name: "b".into(),
                        ty: MetaType::Named("i32".into(), dummy_span()),
                        span: dummy_span(),
                    },
                ],
                ret: Some(MetaType::Named("i32".into(), dummy_span())),
                span: dummy_span(),
            },
        );
        let prog = Box::leak(Box::new(prog));
        let (mut backend, _ctx) = make_backend(prog);

        let decl = prog.declarations.get("add").unwrap();
        assert!(backend.lower_declaration(decl, &dummy_span()).is_ok());
        assert!(backend.get_function("add").is_ok());
    }
}
