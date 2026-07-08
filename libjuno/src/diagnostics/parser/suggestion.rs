use crate::diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,

    pub replacement: String,

    pub span: Span,

    pub applicability: Applicability,
}
#[derive(Debug, Clone)]
pub enum Applicability {
    MachineApplicable,

    MaybeIncorrect,

    Manual,
}
