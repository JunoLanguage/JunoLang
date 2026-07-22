//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::{LabeledSpan, NamedSource};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct JunoSpan {
    pub start: usize,
    pub end: usize,
}

impl JunoSpan {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl<'a> From<pest::Span<'a>> for JunoSpan {
    fn from(span: pest::Span<'a>) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl JunoSpan {
    pub fn err_to_report(
        &self,
        label: &str,
        source_code: String,
        source_file_name: impl AsRef<str>,
    ) -> miette::Error {
        let named = NamedSource::new(source_file_name.as_ref(), source_code);
        miette::miette!(
            labels = vec![LabeledSpan::at(self.start..self.end, label)],
            "Error"
        )
        .with_source_code(named)
    }
}
