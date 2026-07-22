//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{anyhow, bail};
use pest::iterators::Pair;

use super::ParserState;
use crate::{Rule, ast::*};

impl ParserState {
    pub fn parse_type(&self, pair: Pair<Rule>) -> anyhow::Result<Type> {
        let pair_span = pair.as_span();
        let mut prefixes = Vec::new();
        let mut base = None;

        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::type_prefix => prefixes.push(p),
                Rule::base_type => {
                    if base.is_some() {
                        bail!(self.make_span_error(p.as_span(), "duplicate base type"));
                    }
                    base = Some(self.parse_base_type(p)?);
                }
                Rule::generics => {}
                other => bail!(self.make_span_error(
                    pair_span,
                    &format!("unexpected type component: {:?}", other)
                )),
            }
        }

        let mut ty = match base {
            Some(b) => b,
            None => bail!(self.make_span_error(pair_span, "type missing base")),
        };

        for p in prefixes {
            ty = match p.as_str() {
                "&" => Type::Reference(Box::new(ty), self.make_span(p.as_span())),
                "*" => Type::Pointer(Box::new(ty), self.make_span(p.as_span())),
                other => bail!(
                    self.make_span_error(p.as_span(), &format!("invalid type prefix: {}", other))
                ),
            };
        }

        Ok(ty)
    }

    fn parse_base_type(&self, pair: Pair<Rule>) -> anyhow::Result<Type> {
        let mut inner = pair.into_inner();
        let first = inner.next().expect("base type empty");
        let span = self.make_span(first.as_span());

        match first.as_rule() {
            Rule::array_type => {
                let size_token = first.into_inner().next().expect("array size missing");
                let size: u32 = size_token.as_str().parse().map_err(|e| {
                    anyhow!(self.make_span_error(
                        size_token.as_span(),
                        &format!("invalid array size '{}': {}", size_token.as_str(), e)
                    ))
                })?;
                let elem_token = inner.next().expect("array element type missing");
                let elem_ty = Type::Named(
                    self.clean_ident(elem_token.as_str()),
                    self.make_span(elem_token.as_span()),
                );
                Ok(Type::Array {
                    span,
                    elem: Box::new(elem_ty),
                    size,
                })
            }
            Rule::var_ident => Ok(Type::Named(self.clean_ident(first.as_str()), span)),
            other => bail!(self.make_span_error(
                first.as_span(),
                &format!("unexpected base type: {:?}", other)
            )),
        }
    }
}
