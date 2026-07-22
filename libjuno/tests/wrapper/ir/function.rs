//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use libjuno::{MetaConst, MetaExpr, MetaExprKind, MetaFunction, MetaStmt, MetaType};

use crate::wrapper::ir::utils::{dummy_span, make_backend, test_program};

#[test]
fn lower_simple_return_function() {
    let mut prog = test_program();
    prog.functions.insert(
        "main".into(),
        MetaFunction {
            name: "main".into(),
            params: Vec::new(),
            ret: Some(MetaType::Named("i32".into(), dummy_span())),
            body: vec![MetaStmt::Return(
                Some(MetaExpr {
                    kind: MetaExprKind::Const(MetaConst::Int(0, dummy_span()), dummy_span()),
                    ty: MetaType::Named("i32".into(), dummy_span()),
                    span: dummy_span(),
                }),
                dummy_span(),
            )],
            span: dummy_span(),
            locals: HashMap::new(),
        },
    );
    let prog = Box::leak(Box::new(prog));
    let mut backend = make_backend(prog).0;
    // lower_program needs the meta function reference set
    // It does this internally, but we need to call lower_program manually.
    assert!(backend.lower_program().is_ok());
    assert!(backend.get_function("main").is_ok());
}
