use crate::diagnostics::{ context::DiagnosticContext, diagnostic::Diagnostic };

pub struct DiagnosticEngine<'a> {
    context: DiagnosticContext<'a>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> DiagnosticEngine<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            context: DiagnosticContext::new(source),
            diagnostics: Vec::new(),
        }
    }

    pub fn context(&self) -> &DiagnosticContext<'a> {
        &self.context
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn finish(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
