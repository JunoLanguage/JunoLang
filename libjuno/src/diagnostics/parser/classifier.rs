use pest::error::ErrorVariant;

use crate::{
    diagnostics::DiagnosticRule,
    Rule,
};

pub fn classify(
    error: &pest::error::Error<Rule>,
) -> DiagnosticRule {

    match &error.variant {

        ErrorVariant::ParsingError { positives, .. } => {

            if positives.contains(&Rule::ident) {
                return DiagnosticRule::MissingIdentifier;
            }

            if positives.contains(&Rule::type_) {
                return DiagnosticRule::MissingType;
            }

            if positives.contains(&Rule::expr) {
                return DiagnosticRule::MissingExpression;
            }

            DiagnosticRule::Unknown
        }

        ErrorVariant::CustomError { .. } => {
            DiagnosticRule::Unknown
        }
    }
}