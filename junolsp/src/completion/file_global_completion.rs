//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libjuno::ast::{Function, Item};
use tower_lsp::{
    jsonrpc::Error,
    lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, InsertTextFormat},
};

use crate::backend::Backend;

pub(super) async fn file_global_completion(
    backend: &Backend,
    params: CompletionParams,
) -> Result<Vec<CompletionItem>, Error> {
    let mut items = vec![];
    let program = match super::get_program(backend, params).await {
        Ok(p) => p,
        Err(_e) => return Ok(vec![]),
    }; // TODO

    for i in program.items {
        match i {
            Item::Function(f, span) => {
                create_completion_for_function(f, &mut items);
            }
            _ => {
                continue;
            }
        }
    }

    Ok(items)
}

fn create_completion_for_function<'a>(function: Function, items: &mut Vec<CompletionItem>) {
    let param_len = function.params.len();
    let mut param_text_v = vec![];
    for p in 0..param_len {
        param_text_v.push(format!("${{{}}}", p + 1));
    }
    let param_text = param_text_v.join(", ");
    items.push(create_completion_for_ident(
        function.name.clone(),
        format!("{}({});", function.name, param_text),
        CompletionItemKind::FUNCTION,
    ));
}

fn create_completion_for_ident(
    ident: String,
    insert_text: String,
    kind: CompletionItemKind,
) -> CompletionItem {
    CompletionItem {
        label: ident.clone(),
        kind: Some(kind),
        sort_text: Some(ident.clone()),
        filter_text: Some(ident),
        insert_text: Some(insert_text),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}
