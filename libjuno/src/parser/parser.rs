use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::hash::Hash;
use std::mem::take;
use std::sync::Arc;

use pest::Span;
use pest::error::{Error, ErrorVariant, InputLocation};
use pest::iterators::Pair;

use crate::ast::*;
use crate::*;
type JunoPair<'a> = Pair<'a, Rule>;

#[derive(Clone, Default)]
pub struct JunoASTParser {
    namespace: String,
    functions: HashSet<String>,
    source_code: Arc<str>,
    source_file_name: Arc<str>,
}
impl JunoASTParser {
    pub fn new(namespace: String) -> JunoASTParser {
        return JunoASTParser {
            functions: HashSet::new(),
            namespace,
            ..Default::default()
        };
    }
    pub fn with_source(&mut self, source_code: Arc<str>, source_file_name: Arc<str>) -> &mut Self {
        self.source_code = source_code;
        self.source_file_name = source_file_name;
        self
    }
}

pub fn parse_program(
    pair: Pair<'_, Rule>,
    namespace: String,
    source_code: Arc<str>,
    source_file_name: Arc<str>,
) -> Result<Program, Error<Rule>> {
    JunoASTParser::new(namespace)
        .with_source(source_code, source_file_name)
        .parse_program(pair)
}

impl<'a> JunoASTParser {
    pub fn make_span(&self, span: Span) -> JunoSpan {
        JunoSpan::from(span).with_source(&self.source_code, &self.source_file_name)
    }
    pub fn parse_program(&mut self, pair: Pair<Rule>) -> Result<Program, Error<Rule>> {
        let span = pair.as_span();
        let mut items = vec![];
        let pairs = pair.into_inner();
        for pair in pairs {
            match pair.as_rule() {
                Rule::item => {
                    items.push(
                        match self.parse_item(pair) {
                            Err(e) => {
                                return Err(e.clone());
                            }
                            Ok(i) => {
                                match i.clone() {
                                    Item::Function(f, _) => {
                                        self.functions.insert(f.raw_name.clone());
                                    }
                                    _ => (),
                                };
                                i
                            }
                        }
                        .clone(),
                    );
                }
                Rule::EOI => {}
                other => {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("unexpected rule in program: {:?}", other),
                        },
                        span,
                    ));
                }
            }
        }

        Ok(Program {
            span: self.make_span(span),
            items,
        })
    }

    fn parse_item(&mut self, pair: JunoPair) -> Result<Item, Error<Rule>> {
        let p = pair.clone().into_inner().last().expect("Error");
        let span = p.as_span().into();
        match p.as_rule() {
            Rule::function => match self.parse_function(p) {
                Ok(f) => Ok(Item::Function(f, span)),
                Err(e) => Err(e),
            },
            Rule::import_stmt => match self.parse_import(p) {
                Ok(i) => Ok(Item::Import(i, span)),
                Err(e) => Err(e),
            },
            Rule::struct_def => match self.parse_struct(p) {
                Ok(i) => Ok(Item::Struct(i, span)),
                Err(e) => Err(e),
            },
            Rule::declaration => match self.parse_declaration(p) {
                Ok(d) => Ok(Item::Declaration(d, span)),
                Err(e) => Err(e),
            },
            other => panic!(
                "unhandled rule in pair: {:#?}, parse_item: {:?}",
                pair, other
            ),
        }
    }

    fn parse_import(&self, pair: JunoPair) -> Result<Import, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let path = self.clean_ident(inner.next().unwrap().as_str()).to_string();
        print!("{}", path);
        Ok(Import {
            span: self.make_span(span),
            path,
        })
    }

    fn parse_function(&mut self, pair: JunoPair) -> Result<Function, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let raw_name: String = self.clean_ident(inner.next().unwrap().as_str());
        let mut name = self.with_namespace(&raw_name).to_string();
        if raw_name == "main".to_string() {
            name = "main".to_string();
        }
        let mut params = vec![];
        let mut return_type = None;

        for p in inner.by_ref() {
            match p.as_rule() {
                Rule::params => {
                    params = self.parse_params(p)?;
                }
                Rule::type_ => {
                    return_type = Some(self.parse_type(p)?);
                }
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

        unreachable!()
    }

    fn parse_declaration(&mut self, pair: JunoPair) -> Result<Declaration, Error<Rule>> {
        // decl test(param1: i32, param2: i32) -> i32;
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let name = self.clean_ident(inner.next().unwrap().as_str());

        let mut params = vec![];

        for p in inner.by_ref() {
            match p.as_rule() {
                Rule::params => {
                    params = self.parse_params(p)?;
                }
                Rule::type_ => {
                    let declaration = Declaration {
                        span: self.make_span(span),
                        name,
                        params,
                        return_type: Some(self.parse_type(p)?),
                    };
                    return Ok(declaration);
                }
                _ => {}
            }
        }

        unreachable!()
    }

    fn parse_params(&mut self, pair: JunoPair) -> Result<Vec<Param>, Error<Rule>> {
        let span = pair.as_span();
        let mut params: Vec<Param> = vec![];
        for p in pair.into_inner().into_iter() {
            let mut inner = p.into_inner();
            let name = self.clean_ident(inner.next().unwrap().as_str());
            let ty = match self.parse_type(inner.next().unwrap()) {
                Ok(t) => t,
                Err(e) => {
                    return Err(e);
                }
            };

            params.push(Param {
                span: self.make_span(span),
                name,
                ty,
            });
        }
        Ok(params)
    }

    fn parse_block(&mut self, pair: JunoPair) -> Result<Block, Error<Rule>> {
        let span = pair.as_span();
        let stmt_pairs = pair.into_inner();
        let mut stmts = vec![];
        for s in stmt_pairs {
            stmts.push(match self.parse_stmt(s) {
                Ok(s) => s,
                Err(e) => {
                    return Err(e);
                }
            });
        }

        Ok(Block {
            span: self.make_span(span),
            stmts,
        })
    }

    fn parse_stmt(&mut self, pair: JunoPair) -> Result<Stmt, Error<Rule>> {
        let span = self.make_span(pair.as_span());
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::let_stmt => self.parse_let(inner),
            Rule::assign_stmt => self.parse_assign_stmt(inner),
            Rule::expr_stmt => match self.parse_expr(inner.into_inner().next().unwrap()) {
                Ok(e) => Ok(Stmt::Expr(e)),
                Err(e) => {
                    return Err(e);
                }
            },
            Rule::return_stmt => {
                let i = match inner.into_inner().next().map(|x| self.parse_expr(x)) {
                    None => None,
                    Some(a) => Some(a?),
                };

                Ok(Stmt::Return(i, span))
            }
            Rule::break_stmt => Ok(Stmt::Break(span)),
            Rule::continue_stmt => Ok(Stmt::Continue(span)),
            Rule::if_stmt => self.parse_if(inner),
            Rule::while_stmt => self.parse_while(inner),
            Rule::for_stmt => self.parse_for(inner),
            Rule::loop_stmt => self.parse_loop(inner),
            _ => unreachable!("bad stmt"),
        }
    }

    fn parse_if(&mut self, pair: JunoPair) -> Result<Stmt, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let condition = self.parse_expr(inner.next().unwrap())?;

        let then_block = self.parse_block(inner.next().unwrap())?;

        let mut else_ifs = vec![];
        let mut else_block = None;

        for p in inner {
            match p.as_rule() {
                Rule::else_if => {
                    let mut i = p.into_inner();
                    let cond = self.parse_expr(i.next().unwrap())?;
                    let block = self.parse_block(i.next().unwrap())?;
                    else_ifs.push((cond, block));
                }

                Rule::else_block => {
                    let block = self.parse_block(p.into_inner().next().unwrap())?;
                    else_block = Some(block);
                }

                _ => {}
            }
        }

        Ok(Stmt::If(IfStmt {
            span: self.make_span(span),
            condition,
            then_block,
            else_ifs,
            else_block,
        }))
    }
    fn parse_while(&mut self, pair: JunoPair) -> Result<Stmt, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let condition = self.parse_expr(inner.next().unwrap())?;
        let body = self.parse_block(inner.next().unwrap())?;

        Ok(Stmt::While(WhileStmt {
            span: self.make_span(span),
            condition,
            body,
        }))
    }
    fn parse_loop(&mut self, pair: JunoPair) -> Result<Stmt, Error<Rule>> {
        let mut inner = pair.into_inner();

        let body = self.parse_block(inner.next().unwrap())?;

        Ok(Stmt::Loop(body))
    }
    fn parse_for(&mut self, pair: JunoPair) -> Result<Stmt, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let init = self.parse_expr(inner.next().unwrap())?;
        let iter = self.parse_expr(inner.next().unwrap())?;
        let body = self.parse_block(inner.next().unwrap())?;

        Ok(Stmt::For(ForStmt {
            span: self.make_span(span),
            init,
            iter,
            body,
        }))
    }

    fn parse_array(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
        let span: JunoSpan = self.make_span(pair.as_span());
        let mut items = vec![];

        for e in pair.into_inner() {
            items.push(self.parse_expr(e)?);
        }

        Ok(Expr::Array(items, span))
    }

    fn parse_primary(&mut self, pair: JunoPair) -> Result<Expr, Error<Rule>> {
        let span: JunoSpan = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let first = inner.next().unwrap();

        match first.as_rule() {
            Rule::expr => self.parse_expr(first),

            Rule::number => Ok(Expr::Number(
                match first.as_str().parse() {
                    Err(e) => {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("{}", e),
                            },
                            first.as_span(),
                        ));
                    }
                    Ok(n) => n,
                },
                span,
            )),

            Rule::boolean => Ok(Expr::Boolean(first.as_str() == "true", span)),

            Rule::string => self.parse_string(first),

            Rule::char => Ok(Expr::Char(self.parse_char_literal(first.as_str()), span)),

            Rule::var_ident => Ok(Expr::Var(self.clean_ident(first.as_str()), span)),

            Rule::call => self.parse_call(first),

            Rule::array => self.parse_array(first),

            Rule::struct_init => self.parse_struct_init(first),

            other => Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!("unexpected primary: {:?}", other),
                },
                first.as_span(),
            )),
        }
    }
    fn parse_string(&mut self, pair: JunoPair) -> Result<Expr, Error<Rule>> {
        let span: JunoSpan = self.make_span(pair.as_span());
        let raw = pair.as_str();
        let inner = &raw[1..raw.len() - 1];
        let mut s: String = inner.to_string();
        s = s.replace("\\n", "\n");
        s = s.replace("\\t", "\n");
        s = s.replace("\\r", "\n");
        s = s.replace("\\\"", "\"");
        s = s.replace("\\\\", "\\");

        Ok(Expr::String(s, span))
    }
    fn parse_char_literal(&mut self, s: &str) -> char {
        let inner = &s[1..s.len() - 1];

        match inner {
            "\\n" => '\n',
            "\\t" => '\t',
            "\\r" => '\r',
            "\\'" => '\'',
            "\\\\" => '\\',
            _ => inner.chars().next().unwrap(),
        }
    }
    fn parse_let(&mut self, pair: JunoPair) -> Result<Stmt, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let possible_mutable_pair = inner.next().unwrap();
        let mutable = possible_mutable_pair.as_str() == "mut";
        let name: String;
        if !mutable {
            name = self.clean_ident(possible_mutable_pair.as_str());
        } else {
            name = self.clean_ident(inner.next().unwrap().as_str());
        }
        let ty = self.parse_type(inner.next().unwrap())?;
        let value = match inner.next().map(|x| self.parse_expr(x)) {
            None => None,
            Some(a) => match a {
                Err(e) => {
                    return Err(e);
                }
                Ok(v) => Some(v),
            },
        };

        Ok(Stmt::Let(LetStmt {
            span: self.make_span(span),
            mutable,
            name,
            ty,
            value,
        }))
    }

    fn parse_expr(&mut self, pair: JunoPair) -> Result<Expr, Error<Rule>> {
        let inner = pair
            .into_inner()
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap();

        match inner.as_rule() {
            Rule::logical => self.parse_logical(inner),
            Rule::comparison => self.parse_comparison(inner),
            Rule::arithmetic => self.parse_arithmetic(inner),
            Rule::term => self.parse_term(inner),
            Rule::unary => self.parse_unary(inner),
            Rule::primary => self.parse_primary(inner.into_inner().next().unwrap()),
            e => panic!("bad expr {:#?}", e),
        }
    }

    fn parse_call(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
        let span: JunoSpan = self.make_span(pair.as_span());
        let mut inner = pair.into_inner();

        let mut raw_target: String = self.clean_ident(inner.next().unwrap().as_str());

        let mut target = match self.functions.contains(&raw_target) {
            true => self.with_namespace(&raw_target),
            false => raw_target,
        };
        if *target == "main".to_string() {
            target = "main".to_string();
        }
        let mut args = Vec::new();

        if let Some(arg_list) = inner.next() {
            for arg in arg_list.into_inner() {
                let inner = arg.into_inner().next().unwrap();
                let span = inner.as_span().into();
                match inner.as_rule() {
                    Rule::positional_arg => {
                        args.push(Arg::Positional(
                            self.parse_expr(inner.into_inner().next().unwrap())?,
                            span,
                        ));
                    }

                    Rule::named_arg => {
                        let mut i = inner.into_inner();
                        args.push(Arg::Named(
                            self.clean_ident(i.next().unwrap().as_str()),
                            self.parse_expr(i.next().unwrap())?,
                            span,
                        ));
                    }

                    _ => unreachable!(),
                }
            }
        }

        Ok(Expr::Call(Call { span, target, args }))
    }

    fn parse_arithmetic(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_term(inner.next().unwrap())?;

        while let Some(op) = inner.next() {
            let right = self.parse_term(inner.next().unwrap())?;

            let op = match op.as_str() {
                "+" => BinOp::Add,
                "-" => BinOp::Sub,
                _ => unreachable!("invalid arithmetic operator"),
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

    fn parse_term(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_unary(inner.next().unwrap())?;

        while let Some(op) = inner.next() {
            let right = self.parse_unary(inner.next().unwrap())?;

            let op = match op.as_str() {
                "*" => BinOp::Mul,
                "/" => BinOp::Div,
                "%" => BinOp::Mod,
                "//" => BinOp::DivFloor,
                _ => unreachable!("invalid term operator"),
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
    fn parse_unary(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
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

        let mut expr = self.parse_primary(inner.next().unwrap())?;

        for op in ops.into_iter().rev() {
            expr = Expr::Unary(UnaryExpr {
                span: self.make_span(span),
                op: self.parse_unary_op(op)?,
                expr: Box::new(expr),
            });
        }

        Ok(expr)
    }

    fn parse_assign_stmt(&mut self, pair: Pair<Rule>) -> Result<Stmt, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let name = self.clean_ident(inner.next().unwrap().as_str());
        let value = self.parse_expr(inner.next().expect("ERROR"))?;

        Ok(Stmt::AssignStmt(AssignStmt {
            span: self.make_span(span),
            name,
            value,
        }))
    }

    fn parse_unary_op(&mut self, pair: Pair<Rule>) -> Result<UnOp, Error<Rule>> {
        match pair.as_str() {
            "&" => Ok(UnOp::Ref),
            "*" => Ok(UnOp::Deref),
            "!" => Ok(UnOp::Not),
            "-" => Ok(UnOp::Neg),
            other => Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!("unknown unary op: {}", other),
                },
                pair.as_span(),
            )),
        }
    }
    fn parse_comparison(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let mut left = self.parse_arithmetic(inner.next().unwrap())?;

        while let Some(op) = inner.next() {
            let right = self.parse_arithmetic(inner.next().unwrap())?;

            let op = match op.as_str() {
                "==" => BinOp::Eq,
                "!=" => BinOp::Neq,
                ">" => BinOp::Gt,
                "<" => BinOp::Lt,
                ">=" => BinOp::Gte,
                "<=" => BinOp::Lte,
                _ => unreachable!("invalid comparison operator"),
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

    fn parse_struct_init(&mut self, pair: JunoPair) -> Result<Expr, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let name = self.clean_ident(inner.next().unwrap().as_str());

        let mut fields = vec![];
        let field_pairs = inner.next().unwrap();
        for f in field_pairs.into_inner() {
            let mut i = f.into_inner();
            let name = self.clean_ident(i.next().unwrap().as_str());
            let value = self.parse_expr(i.next().unwrap())?;

            fields.push(StructInitField {
                span: self.make_span(span),
                name,
                value,
            });
        }

        Ok(Expr::StructInit(StructInit {
            span: self.make_span(span),
            name,
            fields,
        }))
    }

    fn parse_base_type(&mut self, pair: Pair<Rule>) -> Result<Type, Error<Rule>> {
        let mut inner = pair.into_inner();

        let first = inner.next().unwrap();
        let span: JunoSpan = first.as_span().into();
        match first.as_rule() {
            Rule::array_type => {
                let size = first.into_inner().next().unwrap().as_str().parse().unwrap();
                let i = inner.next().unwrap();
                let elem = Type::Named(self.clean_ident(i.as_str()), i.as_span().into());

                Ok(Type::Array {
                    elem: Box::new(elem),
                    size,
                    span: span,
                })
            }

            Rule::var_ident => Ok(Type::Named(
                self.clean_ident(first.as_str()),
                first.as_span().into(),
            )),

            _ => unreachable!(),
        }
    }
    pub fn parse_type(&mut self, pair: Pair<Rule>) -> Result<Type, Error<Rule>> {
        let span: JunoSpan = self.make_span(pair.as_span());
        let mut prefixes = Vec::new();
        let mut base = None;

        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::type_prefix => prefixes.push(p),
                Rule::base_type => {
                    base = Some(self.parse_base_type(p));
                }
                Rule::generics => {}
                _ => unreachable!(),
            }
        }

        let mut ty = base.unwrap()?;
        for p in prefixes.into_iter() {
            ty = match p.as_str() {
                "&" => Type::Reference(Box::new(ty), p.as_span().into()),
                "*" => Type::Pointer(Box::new(ty), p.as_span().into()),
                _ => unreachable!(),
            };
        }

        Ok(ty)
    }
    fn parse_struct(&mut self, pair: Pair<Rule>) -> Result<StructDef, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let name = self.clean_ident(inner.next().unwrap().as_str());

        let mut fields = vec![];
        let fields_pairs = inner.next().unwrap();
        for f in fields_pairs.into_inner() {
            let mut i = f.into_inner();
            let name = self.clean_ident(i.next().unwrap().as_str());
            let ty = self.parse_type(i.next().unwrap())?;

            fields.push(StructField {
                span: self.make_span(span),
                name,
                ty,
            });
        }

        Ok(StructDef {
            span: self.make_span(span),
            name,
            fields,
        })
    }

    fn parse_logical(&mut self, pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();

        let mut left = self.parse_comparison(inner.next().unwrap())?;

        while let Some(op) = inner.next() {
            let right = self.parse_comparison(inner.next().unwrap())?;

            let op = match op.as_str() {
                "&&" => BinOp::And,
                "||" => BinOp::Or,
                _ => unreachable!("invalid logical operator"),
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

    fn clean_ident(&self, s: &str) -> String {
        let s: &str = s.trim();

        if s.contains(' ') {
            panic!("invalid identifier: contains whitespace: '{}'", s);
        }

        s.to_string()
    }

    fn with_namespace(&self, s: &str) -> String {
        format!("{}::{}", self.namespace, s)
    }
}
