//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::*;
use crate::wrapper::ir::utils::{make_backend, test_program};

#[test]
fn declare_runtime_is_no_op() {
    let prog = Box::leak(Box::new(test_program()));
    let (mut backend, _ctx) = make_backend(prog);
    backend.declare_runtime(); // should not panic
}
