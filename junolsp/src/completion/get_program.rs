//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::{Error, ErrorKind};
use std::sync::Arc;

use tower_lsp::lsp_types::{CompletionItem, CompletionParams, MessageType};
use tracing::Instrument;
use tracing::instrument::WithSubscriber;

use crate::backend::Backend;
use libjuno::ast::*;
use libjuno::{pest::Parser, *};

pub(super) async fn get_program(
    backend: &Backend,
    params: CompletionParams,
) -> Result<Program, Error> {
    let document = backend
        .workspace
        .source(&params.text_document_position.text_document.uri)
        .unwrap();

    let pairs = match JunoParser::parse(Rule::program, &document) {
        Ok(pairs) => pairs,
        Err(e) => {
            let msg = format!("{}", e);
            //backend.client.log_message(MessageType::ERROR, msg);

            return Err(Error::new(ErrorKind::Other, "parse error"));
        }
    };

    let pair = pairs.into_iter().next().unwrap();

    let result = parse_program(
        pair,
        "debug::lsp".to_string(),
        Arc::from(document.as_str()),
        params
            .text_document_position
            .text_document
            .uri
            .to_string()
            .into(),
    );
    match result {
        Ok(program) => Ok(program),

        Err(e) => {
            let msg = format!("{}", e);

            backend.client.log_message(MessageType::ERROR, msg).await;

            Err(Error::new(ErrorKind::Other, "program error"))
        }
    }
}
