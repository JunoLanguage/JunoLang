//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libjuno::{MetaDeclaration, MetaType};

use crate::wrapper::ir::utils::{dummy_span, make_backend, test_program};

#[test]
fn lower_declare_adds_external_function() {
    let mut prog = test_program();
    prog.declarations.insert(
        "add".into(),
        MetaDeclaration {
            name: "add".into(),
            params: vec![
                libjuno::metair::MetaParam {
                    name: "a".into(),
                    ty: MetaType::Named("i32".into(), dummy_span()),
                    span: dummy_span(),
                },
                libjuno::metair::MetaParam {
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
