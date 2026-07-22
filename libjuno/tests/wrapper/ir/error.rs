//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libjuno::ir::LLVMError;

use super::*;

#[test]
fn display_message_variant() {
    let err = LLVMError::Message("hello".into());
    assert_eq!(format!("{}", err), "hello");
}

#[test]
fn display_unknown_variable() {
    let err = LLVMError::UnknownVariable("x".into());
    let msg = format!("{}", err);
    assert!(msg.contains("x"));
}
