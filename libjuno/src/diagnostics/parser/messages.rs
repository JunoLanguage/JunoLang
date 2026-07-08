use crate::diagnostics::DiagnosticRule;

pub fn message(
    rule: DiagnosticRule,
) -> &'static str {

    match rule {

        DiagnosticRule::MissingSemicolon =>
            "Expected ';' after statement.",

        DiagnosticRule::MissingIdentifier =>
            "Expected an identifier.",

        DiagnosticRule::MissingType =>
            "Expected a type.",

        DiagnosticRule::MissingExpression =>
            "Expected an expression.",

        DiagnosticRule::MissingClosingBrace =>
            "Missing closing '}'.",

        DiagnosticRule::MissingClosingParen =>
            "Missing closing ')'.",

        DiagnosticRule::MissingClosingBracket =>
            "Missing closing ']'.",

        DiagnosticRule::UnexpectedToken =>
            "Unexpected token.",

        DiagnosticRule::UnexpectedEOF =>
            "Unexpected end of file.",

        DiagnosticRule::InvalidLiteral =>
            "Invalid literal.",

        DiagnosticRule::Unknown =>
            "Syntax error.",
    }
}