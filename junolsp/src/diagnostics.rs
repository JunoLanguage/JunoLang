use tower_lsp::lsp_types::{
    Diagnostic,
    DiagnosticSeverity,
    Position,
    Range,
    Url,
};

use libjuno::diagnostics;

use crate::backend::Backend;

pub async fn publish(
    backend: &Backend,
    uri: Url,
) {

    let Some(source) = backend.workspace.source(&uri) else {
        return;
    };

    let diagnostics = diagnostics::analyze(&source);

    let diagnostics = diagnostics
        .into_iter()
        .map(to_lsp)
        .collect();

    backend
        .client
        .publish_diagnostics(uri, diagnostics, None)
        .await;
}


fn to_lsp(
    diagnostic: libjuno::diagnostics::Diagnostic,
) -> tower_lsp::lsp_types::Diagnostic {

    use tower_lsp::lsp_types::*;

    Diagnostic {
        range: Range {
            start: Position {
                line: diagnostic.span.start.line as u32,
                character: diagnostic.span.start.column as u32,
            },
            end: Position {
                line: diagnostic.span.end.line as u32,
                character: diagnostic.span.end.column as u32,
            },
        },

        severity: Some(match diagnostic.severity {
            libjuno::diagnostics::Severity::Error => DiagnosticSeverity::ERROR,
            libjuno::diagnostics::Severity::Warning => DiagnosticSeverity::WARNING,
            libjuno::diagnostics::Severity::Info => DiagnosticSeverity::INFORMATION,
            libjuno::diagnostics::Severity::Hint => DiagnosticSeverity::HINT,
        }),

        source: Some("junolsp".into()),

        message: diagnostic.message,

        ..Default::default()
    }
}