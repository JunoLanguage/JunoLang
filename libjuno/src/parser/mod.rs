//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod expr_parser;
pub mod item_parser;
pub mod stmt_parser;
pub mod type_parser;

use pest::Span;
use pest::iterators::Pair;
use std::collections::HashSet;
use std::sync::Arc;

use crate::Rule;
use crate::ast::*;

#[derive(Clone, Default)]
pub struct ParserState {
    pub namespace: String,
    pub functions: HashSet<String>,
    pub source_code: Arc<str>,
    pub source_file_name: Arc<str>,
}

// Backward-compat alias for the rest of the codebase
pub type JunoASTParser = ParserState;

impl ParserState {
    pub fn new(namespace: String) -> Self {
        Self {
            namespace,
            functions: HashSet::new(),
            source_code: Arc::default(),
            source_file_name: Arc::default(),
        }
    }

    pub fn with_source(mut self, source_code: Arc<str>, source_file_name: Arc<str>) -> Self {
        self.source_code = source_code;
        self.source_file_name = source_file_name;
        self
    }

    pub fn make_span(&self, span: Span) -> JunoSpan {
        JunoSpan::from(span)
    }

    pub fn make_span_error(&self, span: Span, label: &str) -> miette::Error {
        self.make_span(span).err_to_report(
            label,
            self.source_code.to_string(),
            &self.source_file_name,
        )
    }

    pub fn clean_ident(&self, s: &str) -> String {
        let s = s.trim();
        assert!(
            !s.contains(' '),
            "invalid identifier: contains whitespace: '{}'",
            s
        );
        s.to_string()
    }

    pub fn with_namespace(&self, s: &str) -> String {
        format!("{}::{}", self.namespace, s)
    }
}

pub fn parse_program(
    pair: Pair<'_, Rule>,
    namespace: String,
    source_code: Arc<str>,
    source_file_name: Arc<str>,
) -> anyhow::Result<Program> {
    ParserState::new(namespace)
        .with_source(source_code, source_file_name)
        .parse_program(pair)
}
