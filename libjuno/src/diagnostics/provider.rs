use crate::diagnostics::engine::DiagnosticEngine;

pub trait DiagnosticProvider {
    fn collect(
        &self,
        engine: &mut DiagnosticEngine,
    );
}