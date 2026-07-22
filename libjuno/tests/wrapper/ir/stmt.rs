//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libjuno::MetaStmt;

use super::*;
use crate::wrapper::ir::utils::{dummy_span, make_backend, test_program};

#[test]
fn break_outside_loop_errors() {
    let prog = Box::leak(Box::new(test_program()));
    let (mut backend, _ctx) = make_backend(prog);
    let stmt = MetaStmt::Break(dummy_span());
    assert!(backend.lower_stmt(&stmt, &dummy_span()).is_err());
}

#[test]
fn continue_outside_loop_errors() {
    let prog = Box::leak(Box::new(test_program()));
    let (mut backend, _ctx) = make_backend(prog);
    let stmt = MetaStmt::Continue(dummy_span());
    assert!(backend.lower_stmt(&stmt, &dummy_span()).is_err());
}
