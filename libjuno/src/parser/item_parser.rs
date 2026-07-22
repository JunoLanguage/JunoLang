//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::bail;
use pest::iterators::Pair;

use super::ParserState;
use crate::{Rule, ast::*};

impl ParserState {
    pub fn parse_program(&mut self, pair: Pair<Rule>) -> anyhow::Result<Program> {
        let span = pair.as_span();
        let mut items = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::item => {
                    let item = self.parse_item(inner)?;
                    if let Item::Function(f, _) = &item {
                        self.functions.insert(f.raw_name.clone());
                    }
                    items.push(item);
                }
                Rule::EOI => {}
                other => bail!(
                    self.make_span_error(span, &format!("unexpected rule in program: {:?}", other))
                ),
            }
        }

        Ok(Program {
            span: self.make_span(span),
            items,
        })
    }

    fn parse_item(&self, pair: Pair<Rule>) -> anyhow::Result<Item> {
        let inner = pair.into_inner().last().expect("empty item");
        let span = self.make_span(inner.as_span());

        match inner.as_rule() {
            Rule::function => {
                let f = self.parse_function(inner)?;
                Ok(Item::Function(f, span))
            }
            Rule::import_stmt => {
                let i = self.parse_import(inner)?;
                Ok(Item::Import(i, span))
            }
            Rule::struct_def => {
                let s = self.parse_struct_def(inner)?;
                Ok(Item::Struct(s, span))
            }
            Rule::declaration => {
                let d = self.parse_declaration(inner)?;
                Ok(Item::Declaration(d, span))
            }
            other => bail!(self.make_span_error(
                inner.as_span(),
                &format!("unexpected item rule: {:?}", other)
            )),
        }
    }

    fn parse_import(&self, pair: Pair<Rule>) -> anyhow::Result<Import> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let path_token = inner.next().expect("import path missing");
        let path = self.clean_ident(path_token.as_str());

        Ok(Import {
            span: self.make_span(span),
            path,
        })
    }

    fn parse_function(&self, pair: Pair<Rule>) -> anyhow::Result<Function> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let raw_name_token = inner.next().expect("function name missing");
        let raw_name = self.clean_ident(raw_name_token.as_str());

        let mut name = self.with_namespace(&raw_name);
        if raw_name == "main" {
            name = "main".to_string();
        }

        let mut params = Vec::new();
        let mut return_type = None;

        for p in inner {
            match p.as_rule() {
                Rule::params => params = self.parse_params(p)?,
                Rule::type_ => return_type = Some(self.parse_type(p)?),
                Rule::block => {
                    let body = self.parse_block(p)?;
                    return Ok(Function {
                        span: self.make_span(span),
                        name,
                        raw_name,
                        params,
                        return_type,
                        body,
                    });
                }
                _ => {}
            }
        }

        bail!(self.make_span_error(span, "function missing body"))
    }

    fn parse_declaration(&self, pair: Pair<Rule>) -> anyhow::Result<Declaration> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let name_token = inner.next().expect("declaration name missing");
        let name = self.clean_ident(name_token.as_str());

        let mut params = Vec::new();

        for p in inner {
            match p.as_rule() {
                Rule::params => params = self.parse_params(p)?,
                Rule::type_ => {
                    return Ok(Declaration {
                        span: self.make_span(span),
                        name,
                        params,
                        return_type: Some(self.parse_type(p)?),
                    });
                }
                _ => {}
            }
        }

        bail!(self.make_span_error(span, "declaration missing return type"))
    }

    fn parse_params(&self, pair: Pair<Rule>) -> anyhow::Result<Vec<Param>> {
        let mut params = Vec::new();

        for p in pair.into_inner() {
            let p_span = p.as_span();
            let mut tokens = p.into_inner();
            let name_token = tokens.next().expect("param name missing");
            let name = self.clean_ident(name_token.as_str());

            let type_token = tokens.next().expect("param type missing");
            let ty = self.parse_type(type_token)?;

            params.push(Param {
                span: self.make_span(p_span),
                name,
                ty,
            });
        }

        Ok(params)
    }

    fn parse_struct_def(&self, pair: Pair<Rule>) -> anyhow::Result<StructDef> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let name_token = inner.next().expect("struct name missing");
        let name = self.clean_ident(name_token.as_str());

        let fields_token = inner.next().expect("struct fields missing");
        let mut fields = Vec::new();

        for f in fields_token.into_inner() {
            let f_span = f.as_span();
            let mut i = f.into_inner();
            let field_name_token = i.next().expect("field name missing");
            let field_name = self.clean_ident(field_name_token.as_str());

            let field_ty_token = i.next().expect("field type missing");
            let field_ty = self.parse_type(field_ty_token)?;

            fields.push(StructField {
                span: self.make_span(f_span),
                name: field_name,
                ty: field_ty,
            });
        }

        Ok(StructDef {
            span: self.make_span(span),
            name,
            fields,
        })
    }
}
