//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::{Error, ErrorKind};

use tower_lsp::lsp_types::{CompletionItem, CompletionParams, MessageType};
use tracing::Instrument;
use tracing::instrument::WithSubscriber;

use crate::backend::Backend;
use libjuno::ast::*;
use libjuno::{pest::Parser, *};

pub(super) fn get_program(backend: &Backend, params: CompletionParams) -> Result<Program, Error> {
    let document = backend
        .workspace
        .source(&params.text_document_position.text_document.uri)
        .unwrap();
    let pairs = match JunoParser::parse(Rule::program, &document) {
        Ok(pairs) => pairs,
        Err(e) => {
            backend
                .client
                .log_message(MessageType::ERROR, format!("{}", e));
            return Err(Error::new(ErrorKind::Other, "oh no!"));
        }
    };
    let expr_owned =
        match parse_program(pairs.into_iter().next().unwrap(), "debug::lsp".to_string()) {
            Ok(e) => e,
            Err(e) => {
                backend
                    .client
                    .log_message(MessageType::ERROR, format!("{}", e));
                return Err(Error::new(ErrorKind::Other, "oh no!"));
            }
        };
    Ok(expr_owned)
}
