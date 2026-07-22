#[cfg(test)]
mod tests {
    use libjuno::{MetaStruct, MetaType, ast::StructField};

    use super::*;
    use crate::ir::utils::{dummy_span, empty_program, make_backend};

    #[test]
    fn lower_struct_adds_to_backend() {
        let mut prog = empty_program();
        prog.structs.insert(
            "Point".into(),
            MetaStruct {
                name: "Point".into(),
                fields: vec![StructField {
                    ty: MetaType::Named("i32".into(), dummy_span()),
                    name: "dummy".to_string(),
                    span: dummy_span(),
                }],
                span: dummy_span(),
            },
        );
        prog.struct_fields.insert("Point".into(), vec!["x".into()]);
        let prog = Box::leak(Box::new(prog));
        let (mut backend, _ctx) = make_backend(prog);

        let meta_struct = prog.structs.get("Point").unwrap();
        assert!(backend.lower_struct(meta_struct, &dummy_span()).is_ok());
        assert!(backend.get_struct("Point").is_ok());
    }

    #[test]
    fn get_struct_qualified_lookup_fails() {
        let prog = Box::leak(Box::new(empty_program()));
        let (backend, _ctx) = make_backend(prog);
        assert!(backend.get_struct("mod.Point").is_err());
    }
}
