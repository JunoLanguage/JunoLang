#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::utils::{dummy_span, empty_program, make_backend};
    use inkwell::types::BasicTypeEnum;
    use libjuno::{MetaProgram, MetaType};

    #[test]
    fn lower_type_named_primitives() {
        let prog: &'static MetaProgram = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let s = dummy_span();

        assert!(matches!(
            backend.lower_type(&MetaType::Named("bool".into(), s), &s).unwrap(),
            BasicTypeEnum::IntType(t) if t.get_bit_width() == 1
        ));
        assert!(matches!(
            backend.lower_type(&MetaType::Named("i32".into(), s), &s).unwrap(),
            BasicTypeEnum::IntType(t) if t.get_bit_width() == 32
        ));
        assert!(matches!(
            backend.lower_type(&MetaType::Named("f64".into(), s), &s).unwrap(),
            BasicTypeEnum::FloatType(t) if t.get_bit_width() == 64
        ));
    }

    #[test]
    fn lower_type_pointer_and_reference() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let s = dummy_span();
        let ptr_ty = MetaType::Pointer(Box::new(MetaType::Named("i8".into(), s)), s);
        let ref_ty = MetaType::Reference(Box::new(MetaType::Named("i8".into(), s)), s);

        let ptr_res = backend.lower_type(&ptr_ty, &s).unwrap();
        let ref_res = backend.lower_type(&ref_ty, &s).unwrap();
        assert!(matches!(ptr_res, BasicTypeEnum::PointerType(_)));
        assert!(matches!(ref_res, BasicTypeEnum::PointerType(_)));
    }

    #[test]
    fn lower_type_array() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let s = dummy_span();
        let arr = MetaType::Array {
            elem: MetaType::Named("i16".into(), s),
            size: 4,
            span: s,
        };
        let res = backend.lower_type(&arr, &s).unwrap();
        assert!(matches!(res, BasicTypeEnum::ArrayType(t) if t.len() == 4));
    }

    #[test]
    fn lower_type_named_struct_found() {
        let mut prog = empty_program();
        prog.structs.insert(
            "Point".into(),
            MetaStruct {
                name: "Point".into(),
                fields: Vec::new(),
                span: dummy_span(),
            },
        );
        let prog = Box::leak(Box::new(prog));
        let (mut backend, ctx) = make_backend(prog);

        // Manually seed backend structs (as lower_struct does)
        let fields: Vec<_> = Vec::new();
        let struct_ty = ctx.struct_type(&fields, false);
        backend.structs.insert("Point".into(), struct_ty);

        let s = dummy_span();
        let res = backend.lower_type(&MetaType::Named("Point".into(), s), &s);
        assert!(res.is_ok());
    }
}
