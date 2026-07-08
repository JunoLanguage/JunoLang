use tower_lsp::lsp_types::{
    Diagnostic,
    DiagnosticSeverity,
    Position,
    Range,
    Url,
};

use libjuno::{
    pest::{
        error::LineColLocation,
        Parser,
    },
    JunoParser,
    Rule,
};

use crate::backend::Backend;

pub async fn publish(
    backend: &Backend,
    uri: Url,
) {
    let Some(source) = backend.workspace.source(&uri) else {
        return;
    };

    let diagnostics = match JunoParser::parse(Rule::program, &source) {
        Ok(_) => Vec::new(),
        Err(error) => {
            vec![pest_error_to_diagnostic(error)]
        }
    };

    backend
        .client
        .publish_diagnostics(uri, diagnostics, None)
        .await;
}

fn pest_error_to_diagnostic(
    error: libjuno::pest::error::Error<Rule>,
) -> Diagnostic {

    let range = match error.line_col {

        LineColLocation::Pos((line, column)) => {
            Range {
                start: Position {
                    line: (line - 1) as u32,
                    character: (column - 1) as u32,
                },
                end: Position {
                    line: (line - 1) as u32,
                    character: column as u32,
                },
            }
        }

        LineColLocation::Span((start_line, start_column), (end_line, end_column)) => {
            Range {
                start: Position {
                    line: (start_line - 1) as u32,
                    character: (start_column - 1) as u32,
                },
                end: Position {
                    line: (end_line - 1) as u32,
                    character: (end_column - 1) as u32,
                },
            }
        }
    };

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("junolsp".into()),
        message: error.to_string(),
        ..Default::default()
    }
}