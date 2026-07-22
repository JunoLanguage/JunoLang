//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::wrapper::test_file_utils::test_file;

const TEST0_JUNO: &str = include_str!("test_files/test0.juno");
const TEST_ARITHMETIC_JUNO: &str = include_str!("test_files/test_arithmetic.juno");
const TEST_ARRAYS_JUNO: &str = include_str!("test_files/test_arrays.juno");
const TEST_COMPARISONS_JUNO: &str = include_str!("test_files/test_comparisons.juno");
const TEST_EXPRESSIONS_JUNO: &str = include_str!("test_files/test_expressions.juno");
const TEST_FUNCTIONS_JUNO: &str = include_str!("test_files/test_functions.juno");
const TEST_HELLOWORLD_JUNO: &str = include_str!("test_files/test_helloworld.juno");
const TEST_IF_JUNO: &str = include_str!("test_files/test_if.juno");
const TEST_LOOP_JUNO: &str = include_str!("test_files/test_loop.juno");
const TEST_POINTERS_JUNO: &str = include_str!("test_files/test_pointers.juno");
const TEST_UNARY_JUNO: &str = include_str!("test_files/test_unary.juno");
const TEST_WHILE_JUNO: &str = include_str!("test_files/test_while.juno");

#[test]
fn file_test0_juno() {
    test_file(TEST0_JUNO, "test0.juno");
}

#[test]
fn file_test_arithmetic_juno() {
    test_file(TEST_ARITHMETIC_JUNO, "test_arithmetic.juno");
}

#[test]
fn file_test_arrays_juno() {
    test_file(TEST_ARRAYS_JUNO, "test_arrays.juno");
}

#[test]
fn file_test_comparisons_juno() {
    test_file(TEST_COMPARISONS_JUNO, "test_comparisons.juno");
}

#[test]
fn file_test_expressions_juno() {
    test_file(TEST_EXPRESSIONS_JUNO, "test_expressions.juno");
}

#[test]
fn file_test_functions_juno() {
    test_file(TEST_FUNCTIONS_JUNO, "test_functions.juno");
}

#[test]
fn file_test_helloworld_juno() {
    test_file(TEST_HELLOWORLD_JUNO, "test_helloworld.juno");
}

#[test]
fn file_test_if_juno() {
    test_file(TEST_IF_JUNO, "test_if.juno");
}

#[test]
fn file_test_loop_juno() {
    test_file(TEST_LOOP_JUNO, "test_loop.juno");
}

#[test]
fn file_test_pointers_juno() {
    test_file(TEST_POINTERS_JUNO, "test_pointers.juno");
}

#[test]
fn file_test_unary_juno() {
    test_file(TEST_UNARY_JUNO, "test_unary.juno");
}

#[test]
fn file_test_while_juno() {
    test_file(TEST_WHILE_JUNO, "test_while.juno");
}
