use tower_lsp::{jsonrpc::Error, lsp_types::{CompletionParams, CompletionResponse}};

use crate::backend::Backend;
mod get_program;
mod simple_completion;
mod file_global_completion;


pub (self) use get_program::get_program;
pub async fn completion(
    backend: &Backend,
    params: CompletionParams
) -> Result<Option<CompletionResponse>, Error> {
    let mut items = simple_completion::keywords();
    items.append(&mut simple_completion::builtins());
    items.append(&mut simple_completion::snippets());
    items.append(&mut file_global_completion::file_global_completion(backend, params).await.unwrap());
    Ok(Some(CompletionResponse::Array(items)))
}