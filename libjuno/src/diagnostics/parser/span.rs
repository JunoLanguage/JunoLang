#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

use pest::error::LineColLocation;

use crate::Rule;

pub fn from_pest(
    error: &pest::error::Error<Rule>,
) -> Span {

    match error.line_col {

        LineColLocation::Pos((line, column)) => {

            Span {
                start: Position {
                    line: line - 1,
                    column: column - 1,
                },

                end: Position {
                    line: line - 1,
                    column,
                },
            }

        }

        LineColLocation::Span(
            (sl, sc),
            (el, ec),
        ) => {

            Span {
                start: Position {
                    line: sl - 1,
                    column: sc - 1,
                },

                end: Position {
                    line: el - 1,
                    column: ec,
                },
            }

        }

    }

}