#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticRule {
    // Missing
    MissingSemicolon,

    MissingIdentifier,

    MissingType,

    MissingExpression,

    MissingClosingBrace,

    MissingClosingParen,

    MissingClosingBracket,

    // Unexpected
    UnexpectedToken,

    UnexpectedEOF,
    // Invalid
    InvalidLiteral,

    // Misc
    Unknown,
}