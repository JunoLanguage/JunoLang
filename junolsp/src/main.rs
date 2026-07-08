mod backend;
mod completion;
mod config;
mod diagnostics;
mod goto_definition;
mod hover;
mod semantic_tokens;
mod workspace;

use std::io::Error;

use backend::Backend;
use tower_lsp::{ LspService, Server };

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().with_writer(std::io::stderr).init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}