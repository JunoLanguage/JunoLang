use crate::diagnostics::{
    DiagnosticCode, Severity, Span, Suggestion,
};

#[derive(Debug, Clone)]
pub struct Diagnostic {

    //pub code: DiagnosticCode,

    pub severity: Severity,

    pub span: Span,

    pub message: String,

    pub notes: Vec<String>,

    pub suggestions: Vec<Suggestion>,
}


