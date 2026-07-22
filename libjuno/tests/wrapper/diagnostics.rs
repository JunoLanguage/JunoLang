//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fs::File, io::Read, path::Path};

#[test]
fn test_diagnostics_ok_file() {
    let mut source = "".to_string();
    File::open(Path::new("../test/test.juno"))
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();
    let diagnostics = libjuno::diagnostics::analyze(source.as_str());
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics, got:\n{:#?}",
        diagnostics
    );
}

#[test]
fn test_diagnostics_wrong_file() {
    let mut source = "".to_string();
    File::open(Path::new("../test/wrong_syntax/semicolon.juno"))
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();
    let diagnostics = libjuno::diagnostics::analyze(source.as_str());
    assert!(
        !diagnostics.is_empty(),
        "Expected diagnostics, got:\n{:#?}",
        diagnostics
    );
}
