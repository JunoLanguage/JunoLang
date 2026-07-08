use super::{ classify, message, from_pest };

use crate::diagnostics::{ Diagnostic, DiagnosticBuilder, DiagnosticContext, Severity };

use crate::Rule;

pub fn parse(_ctx: &DiagnosticContext, error: pest::error::Error<Rule>) -> Diagnostic {
    let span = from_pest(&error);

    let rule = classify(&error);

    let message = message(rule);

    DiagnosticBuilder::new(Severity::Error, span, message).build()
}
