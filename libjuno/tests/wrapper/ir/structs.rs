//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libjuno::{MetaField, MetaStruct, MetaType, ast::StructField};

use super::*;
use crate::wrapper::ir::utils::{dummy_span, make_backend, test_program};

#[test]
fn lower_struct_adds_to_backend() {
    let mut prog = test_program();
    prog.structs.insert(
        "Point".into(),
        MetaStruct {
            name: "Point".into(),
            fields: vec![MetaField {
                ty: MetaType::Named("i32".into(), dummy_span()),
                name: "dummy".to_string(),
                span: dummy_span(),
                index: 0,
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
    let prog = Box::leak(Box::new(test_program()));
    let (backend, _ctx) = make_backend(prog);
    assert!(backend.get_struct("mod.Point").is_err());
}
