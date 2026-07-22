//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{anyhow, bail};
use pest::iterators::Pair;

use super::ParserState;
use crate::{Rule, ast::*};

impl ParserState {
    pub fn parse_expr(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let pair_span = pair.as_span();
        let wrapper = pair.into_inner().next().expect("expr missing wrapper");
        let inner = wrapper.into_inner().next().expect("expr wrapper empty");

        match inner.as_rule() {
            Rule::logical => self.parse_logical(inner),
            Rule::comparison => self.parse_comparison(inner),
            Rule::arithmetic => self.parse_arithmetic(inner),
            Rule::term => self.parse_term(inner),
            Rule::unary => self.parse_unary(inner),
            Rule::primary => {
                let token = inner.into_inner().next().expect("primary empty");
                self.parse_primary(token)
            }
            Rule::bitwise => self.parse_bitwise(inner),
            other => bail!(self.make_span_error(
                pair_span,
                &format!("unexpected expression core: {:?}", other)
            )),
        }
    }

    fn parse_primary(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let pair_span = pair.as_span();
        let span = self.make_span(pair_span);
        let mut inner = pair.into_inner();
        let first = inner.next().expect("primary missing inner token");

        match first.as_rule() {
            Rule::expr => self.parse_expr(first),
            Rule::integer => self.parse_integer(first),
            Rule::fractional => self.parse_fractional(first),
            Rule::boolean => Ok(Expr::Boolean(first.as_str() == "true", span)),
            Rule::string => self.parse_string(first),
            Rule::char => Ok(Expr::Char(self.parse_char_literal(first.as_str()), span)),
            Rule::var_ident => Ok(Expr::Var(self.clean_ident(first.as_str()), span)),
            Rule::call => self.parse_call(first),
            Rule::array => self.parse_array(first),
            Rule::struct_init => self.parse_struct_init(first),
            other => bail!(
                self.make_span_error(pair_span, &format!("unexpected primary token: {:?}", other))
            ),
        }
    }

    fn parse_integer(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let mut inner = pair.clone().into_inner();
        let value_token = inner.next().expect("integer value missing");
        let value_str = value_token.as_str();

        let ty_token = inner.next();
        let ty =
            ty_token.map(|t| Type::Named(t.as_str().replace('_', ""), self.make_span(t.as_span())));

        let value: i64 = value_str.parse().map_err(|e| {
            anyhow!(self.make_span_error(
                pair.as_span(),
                &format!("invalid integer '{}': {}", value_str, e)
            ))
        })?;

        Ok(Expr::Integer(value, ty, self.make_span(pair.as_span())))
    }

    fn parse_fractional(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let mut inner = pair.clone().into_inner();
        let value_token = inner.next().expect("fractional value missing");
        let value_str = value_token.as_str();

        let ty_token = inner.next();
        let ty =
            ty_token.map(|t| Type::Named(t.as_str().replace('_', ""), self.make_span(t.as_span())));

        let value: f64 = value_str.parse().map_err(|e| {
            anyhow!(self.make_span_error(
                pair.as_span(),
                &format!("invalid float '{}': {}", value_str, e)
            ))
        })?;

        Ok(Expr::Fractional(value, ty, self.make_span(pair.as_span())))
    }

    fn parse_string(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = self.make_span(pair.as_span());
        let raw = pair.as_str();
        if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
            bail!(self.make_span_error(
                pair.as_span(),
                "invalid string literal: expected quoted string"
            ));
        }
        let inner = &raw[1..raw.len() - 1];
        let mut s = inner.to_string();

        s = s.replace("\\n", "\n");
        s = s.replace("\\t", "\t");
        s = s.replace("\\r", "\r");
        s = s.replace("\\\"", "\"");
        s = s.replace("\\\\", "\\");

        Ok(Expr::String(s, span))
    }

    fn parse_char_literal(&self, s: &str) -> char {
        let inner = if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        match inner {
            "\\n" => '\n',
            "\\t" => '\t',
            "\\r" => '\r',
            "\\'" => '\'',
            "\\\\" => '\\',
            _ => inner.chars().next().unwrap_or('\0'),
        }
    }

    fn parse_call(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let target_token = inner.next().expect("call target missing");
        let raw_target = self.clean_ident(target_token.as_str());

        let mut target = if self.functions.contains(&raw_target) {
            self.with_namespace(&raw_target)
        } else {
            raw_target
        };
        if target == "main" {
            target = "main".to_string();
        }

        let mut args = Vec::new();

        if let Some(arg_list) = inner.next() {
            for arg_pair in arg_list.into_inner() {
                let token = arg_pair.into_inner().next().expect("empty argument");
                let arg_span = self.make_span(token.as_span());

                match token.as_rule() {
                    Rule::positional_arg => {
                        let expr_token = token.into_inner().next().expect("empty positional arg");
                        args.push(Arg::Positional(self.parse_expr(expr_token)?, arg_span));
                    }
                    Rule::named_arg => {
                        let mut i = token.into_inner();
                        let name_token = i.next().expect("named arg name missing");
                        let name = self.clean_ident(name_token.as_str());
                        let value_token = i.next().expect("named arg value missing");
                        args.push(Arg::Named(name, self.parse_expr(value_token)?, arg_span));
                    }
                    other => bail!(self.make_span_error(
                        token.as_span(),
                        &format!("unexpected argument type: {:?}", other)
                    )),
                }
            }
        }

        Ok(Expr::Call(Call { span, target, args }))
    }

    fn parse_array(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = self.make_span(pair.as_span());
        let mut items = Vec::new();

        for token in pair.into_inner() {
            items.push(self.parse_expr(token)?);
        }

        Ok(Expr::Array(items, span))
    }

    fn parse_struct_init(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let name_token = inner.next().expect("struct init name missing");
        let name = self.clean_ident(name_token.as_str());

        let fields_token = inner.next().expect("struct init fields missing");
        let mut fields = Vec::new();

        for f in fields_token.into_inner() {
            let f_span = f.as_span();
            let mut i = f.into_inner();
            let field_name_token = i.next().expect("init field name missing");
            let field_name = self.clean_ident(field_name_token.as_str());

            let field_value_token = i.next().expect("init field value missing");
            let value = self.parse_expr(field_value_token)?;

            fields.push(StructInitField {
                span: self.make_span(f_span),
                name: field_name,
                value,
            });
        }

        Ok(Expr::StructInit(StructInit { span, name, fields }))
    }

    fn parse_logical(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_bitwise(inner.next().expect("logical left missing"))?;

        while let Some(op_token) = inner.next() {
            let right_token = inner.next().expect("logical right missing");
            let right = self.parse_bitwise(right_token)?;

            let op = match op_token.as_str() {
                "&&" => BinOp::And,
                "||" => BinOp::Or,
                other => bail!(self.make_span_error(
                    op_token.as_span(),
                    &format!("invalid logical operator: {}", other)
                )),
            };

            left = Expr::Binary(BinaryExpr {
                span: self.make_span(span),
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_bitwise(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_comparison(inner.next().expect("bitwise left missing"))?;

        while let Some(op_token) = inner.next() {
            let right_token = inner.next().expect("bitwise right missing");
            let right = self.parse_comparison(right_token)?;

            let op = match op_token.as_str() {
                "&" => BinOp::BitAnd,
                "|" => BinOp::BitOr,
                "^" => BinOp::BitXOR,
                "<<" => BinOp::BitSHL,
                ">>" => BinOp::BitSHR,
                other => bail!(self.make_span_error(
                    op_token.as_span(),
                    &format!("invalid bitwise operator: {}", other)
                )),
            };

            left = Expr::Binary(BinaryExpr {
                span: self.make_span(span),
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_comparison(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_arithmetic(inner.next().expect("comparison left missing"))?;

        while let Some(op_token) = inner.next() {
            let right_token = inner.next().expect("comparison right missing");
            let right = self.parse_arithmetic(right_token)?;

            let op = match op_token.as_str() {
                "==" => BinOp::Eq,
                "!=" => BinOp::Neq,
                ">" => BinOp::Gt,
                "<" => BinOp::Lt,
                ">=" => BinOp::Gte,
                "<=" => BinOp::Lte,
                other => bail!(self.make_span_error(
                    op_token.as_span(),
                    &format!("invalid comparison operator: {}", other)
                )),
            };

            left = Expr::Binary(BinaryExpr {
                span: self.make_span(span),
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_arithmetic(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_term(inner.next().expect("arithmetic left missing"))?;

        while let Some(op_token) = inner.next() {
            let right_token = inner.next().expect("arithmetic right missing");
            let right = self.parse_term(right_token)?;

            let op = match op_token.as_str() {
                "+" => BinOp::Add,
                "-" => BinOp::Sub,
                other => bail!(self.make_span_error(
                    op_token.as_span(),
                    &format!("invalid arithmetic operator: {}", other)
                )),
            };

            left = Expr::Binary(BinaryExpr {
                span: self.make_span(span),
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_term(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_unary(inner.next().expect("term left missing"))?;

        while let Some(op_token) = inner.next() {
            let right_token = inner.next().expect("term right missing");
            let right = self.parse_unary(right_token)?;

            let op = match op_token.as_str() {
                "*" => BinOp::Mul,
                "/" => BinOp::Div,
                "%" => BinOp::Mod,
                "//" => BinOp::DivFloor,
                other => bail!(self.make_span_error(
                    op_token.as_span(),
                    &format!("invalid term operator: {}", other)
                )),
            };

            left = Expr::Binary(BinaryExpr {
                span: self.make_span(span),
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_unary(&self, pair: Pair<Rule>) -> anyhow::Result<Expr> {
        let span = pair.as_span();
        let mut inner = pair.into_inner().peekable();

        let mut ops = Vec::new();
        while let Some(p) = inner.peek() {
            if p.as_rule() == Rule::unary_ops {
                ops.push(inner.next().unwrap());
            } else {
                break;
            }
        }

        let expr_token = inner.next().expect("unary missing operand");
        let mut expr = self.parse_primary(expr_token)?;

        for op_token in ops.into_iter().rev() {
            expr = Expr::Unary(UnaryExpr {
                span: self.make_span(span),
                op: self.parse_unary_op(op_token)?,
                expr: Box::new(expr),
            });
        }

        Ok(expr)
    }

    fn parse_unary_op(&self, pair: Pair<Rule>) -> anyhow::Result<UnOp> {
        match pair.as_str() {
            "&" => Ok(UnOp::Ref),
            "*" => Ok(UnOp::Deref),
            "!" => Ok(UnOp::Not),
            "-" => Ok(UnOp::Neg),
            "~" => Ok(UnOp::BitNot),
            other => bail!(self.make_span_error(
                pair.as_span(),
                &format!("unknown unary operator: {}", other)
            )),
        }
    }
}
