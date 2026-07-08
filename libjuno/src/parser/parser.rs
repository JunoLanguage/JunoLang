use pest::error::{ Error, ErrorVariant, InputLocation };
use pest::iterators::Pair;

use crate::*;
use crate::ast::*;
type JunoPair<'a> = Pair<'a, Rule>;

pub fn parse_program(_pair: Pair<Rule>) -> Result<Program, Error<Rule>> {
    let mut items = vec![];
    let pairs = _pair.into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::item => {
                items.push(match parse_item(pair) {
                    Err(e) => {
                        return Err(e);
                    }
                    Ok(i) => i,
                });
            }
            Rule::EOI => {}
            other => {
                return Err(
                    Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("unexpected rule in program: {:?}", other),
                        },
                        pair.as_span()
                    )
                );
            }
        }
    }

    Ok(Program { items })
}

fn parse_item(pair: JunoPair) -> Result<Item, Error<Rule>> {
    let p = pair.clone().into_inner().last().expect("Error");
    match p.as_rule() {
        Rule::function => {
            match parse_function(p) {
                Ok(f) => Ok(Item::Function(f)),
                Err(e) => Err(e),
            }
        }
        Rule::import_stmt => {
            match parse_import(p) {
                Ok(i) => Ok(Item::Import(i)),
                Err(e) => Err(e),
            }
        }
        Rule::struct_def => {
            match parse_struct(p) {
                Ok(i) => Ok(Item::Struct(i)),
                Err(e) => Err(e),
            }
        }
        other => panic!("unhandled rule in pair: {:#?}, parse_item: {:?}", pair, other),
    }
}

fn parse_import(pair: JunoPair) -> Result<Import, Error<Rule>> {
    let mut inner = pair.into_inner();

    let path = clean_ident(inner.next().unwrap().as_str());

    Ok(Import { path })
}

fn parse_function(pair: JunoPair) -> Result<Function, Error<Rule>> {
    let mut inner = pair.into_inner();

    let name = clean_ident(inner.next().unwrap().as_str());

    let mut params = vec![];
    let mut return_type = None;

    for p in inner.by_ref() {
        match p.as_rule() {
            Rule::params => {
                params = parse_params(p)?;
            }
            Rule::type_ => {
                return_type = Some(parse_type(p)?);
            }
            Rule::block => {
                let body = parse_block(p)?;
                return Ok(Function {
                    name,
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

fn parse_params(pair: JunoPair) -> Result<Vec<Param>, Error<Rule>> {
    let mut params: Vec<Param> = vec![];
    for p in pair.into_inner().into_iter() {
        let mut inner = p.into_inner();
        let name = clean_ident(inner.next().unwrap().as_str());
        let ty = match parse_type(inner.next().unwrap()) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };

        params.push(Param { name, ty });
    }
    Ok(params)
}

fn parse_block(pair: JunoPair) -> Result<Block, Error<Rule>> {
    let stmt_pairs = pair.into_inner();
    let mut stmts = vec![];
    for s in stmt_pairs {
        stmts.push(match parse_stmt(s) {
            Ok(s) => s,
            Err(e) => {
                return Err(e);
            }
        });
    }

    Ok(Block { stmts })
}

fn parse_stmt(pair: JunoPair) -> Result<Stmt, Error<Rule>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::let_stmt => parse_let(inner),
        Rule::assign_stmt => parse_assign_stmt(inner),
        Rule::expr_stmt => {
            match parse_expr(inner.into_inner().next().unwrap()) {
                Ok(e) => Ok(Stmt::Expr(e)),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Rule::return_stmt => {
            let i = match inner.into_inner().next().map(parse_expr) {
                None => None,
                Some(a) => Some(a?),
            };

            Ok(Stmt::Return(i))
        }
        Rule::break_stmt => Ok(Stmt::Break),
        Rule::continue_stmt => Ok(Stmt::Continue),
        Rule::if_stmt => parse_if(inner),
        Rule::while_stmt => parse_while(inner),
        Rule::for_stmt => parse_for(inner),
        Rule::loop_stmt => parse_loop(inner),
        _ => unreachable!("bad stmt"),
    }
}

fn parse_if(pair: JunoPair) -> Result<Stmt, Error<Rule>> {
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap())?;

    let then_block = parse_block(inner.next().unwrap())?;

    let mut else_ifs = vec![];
    let mut else_block = None;

    for p in inner {
        match p.as_rule() {
            Rule::else_if => {
                let mut i = p.into_inner();
                let cond = parse_expr(i.next().unwrap())?;
                let block = parse_block(i.next().unwrap())?;
                else_ifs.push((cond, block));
            }

            Rule::else_block => {
                let block = parse_block(p.into_inner().next().unwrap())?;
                else_block = Some(block);
            }

            _ => {}
        }
    }

    Ok(
        Stmt::If(IfStmt {
            condition,
            then_block,
            else_ifs,
            else_block,
        })
    )
}
fn parse_while(pair: JunoPair) -> Result<Stmt, Error<Rule>> {
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(
        Stmt::While(WhileStmt {
            condition,
            body,
        })
    )
}
fn parse_loop(pair: JunoPair) -> Result<Stmt, Error<Rule>> {
    let mut inner = pair.into_inner();

    let body = parse_block(inner.next().unwrap())?;

    Ok(Stmt::Loop(body))
}
fn parse_for(pair: JunoPair) -> Result<Stmt, Error<Rule>> {
    let mut inner = pair.into_inner();

    let init = parse_expr(inner.next().unwrap())?;
    let iter = parse_expr(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(
        Stmt::For(ForStmt {
            init,
            iter,
            body,
        })
    )
}

fn parse_array(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut items = vec![];

    for e in pair.into_inner() {
        items.push(parse_expr(e)?);
    }

    Ok(Expr::Array(items))
}

fn parse_primary(pair: JunoPair) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::expr => parse_expr(first),

        Rule::number => Ok(Expr::Number(match first.as_str().parse() {
            Err(e) => return Err(Error::new_from_span(ErrorVariant::CustomError { message: format!("{}", e) }, first.as_span())),
            Ok(n) => n
        })),

        Rule::boolean => Ok(Expr::Boolean(first.as_str() == "true")),

        Rule::string => parse_string(first),

        Rule::char => Ok(Expr::Char(parse_char_literal(first.as_str()))),

        Rule::var_ident => Ok(Expr::Var(clean_ident(first.as_str()))),

        Rule::call => parse_call(first),

        Rule::array => parse_array(first),

        Rule::struct_init => parse_struct_init(first),

        other =>
            Err(
                Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("unexpected primary: {:?}", other),
                    },
                    first.as_span()
                )
            ),
    }
}
fn parse_string(pair: JunoPair) -> Result<Expr, Error<Rule>> {
    let raw = pair.as_str();
    let inner = &raw[1..raw.len() - 1];
    let mut s = inner.to_string();
    s = s.replace("\\n", "\n");
    s = s.replace("\\t", "\n");
    s = s.replace("\\r", "\n");
    s = s.replace("\\\"", "\"");
    s = s.replace("\\\\", "\\");

    Ok(Expr::String(s))
}
fn parse_char_literal(s: &str) -> char {
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
fn parse_let(pair: JunoPair) -> Result<Stmt, Error<Rule>> {
    let mut inner = pair.into_inner();
    let possible_mutable_pair = inner.next().unwrap();
    let mutable = possible_mutable_pair.as_str() == "mut";
    let name: String;
    if !mutable {
        name = clean_ident(possible_mutable_pair.as_str());
    } else {
        name = clean_ident(inner.next().unwrap().as_str());
    }
    let ty = parse_type(inner.next().unwrap())?;
    let value = match inner.next().map(parse_expr) {
        None => None,
        Some(a) =>
            match a {
                Err(e) => {
                    return Err(e);
                }
                Ok(v) => Some(v),
            }
    };

    Ok(
        Stmt::Let(LetStmt {
            mutable,
            name,
            ty,
            value,
        })
    )
}

fn parse_expr(pair: JunoPair) -> Result<Expr, Error<Rule>> {
    let inner = pair.into_inner().next().unwrap().into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::logical => parse_logical(inner),
        Rule::comparison => parse_comparison(inner),
        Rule::arithmetic => parse_arithmetic(inner),
        Rule::term => parse_term(inner),
        Rule::unary => parse_unary(inner),
        Rule::primary => parse_primary(inner.into_inner().next().unwrap()),
        e => panic!("bad expr {:#?}", e),
    }
}

fn parse_call(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();

    let target = clean_ident(inner.next().unwrap().as_str())
        .split('.')
        .map(str::to_owned)
        .collect();

    let mut args = Vec::new();

    if let Some(arg_list) = inner.next() {
        for arg in arg_list.into_inner() {
            let inner = arg.into_inner().next().unwrap();

            match inner.as_rule() {
                Rule::positional_arg => {
                    args.push(Arg::Positional(parse_expr(inner.into_inner().next().unwrap())?));
                }

                Rule::named_arg => {
                    let mut i = inner.into_inner();

                    args.push(
                        Arg::Named(
                            clean_ident(i.next().unwrap().as_str()),
                            parse_expr(i.next().unwrap())?
                        )
                    );
                }

                _ => unreachable!(),
            }
        }
    }

    Ok(Expr::Call(Call { target, args }))
}

fn parse_arithmetic(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();

    let mut left = parse_term(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let right = parse_term(inner.next().unwrap())?;

        let op = match op.as_str() {
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            _ => unreachable!("invalid arithmetic operator"),
        };

        left = Expr::Binary(BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        });
    }

    Ok(left)
}

fn parse_term(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();

    let mut left = parse_unary(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let right = parse_unary(inner.next().unwrap())?;

        let op = match op.as_str() {
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            "%" => BinOp::Mod,
            "//" => BinOp::DivFloor,
            _ => unreachable!("invalid term operator"),
        };

        left = Expr::Binary(BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        });
    }

    Ok(left)
}
fn parse_unary(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner().peekable();

    let mut ops = Vec::new();

    while let Some(p) = inner.peek() {
        if p.as_rule() == Rule::unary_ops {
            ops.push(inner.next().unwrap());
        } else {
            break;
        }
    }

    let mut expr = parse_primary(inner.next().unwrap())?;

    for op in ops.into_iter().rev() {
        expr = Expr::Unary(UnaryExpr {
            op: parse_unary_op(op)?,
            expr: Box::new(expr),
        });
    }

    Ok(expr)
}

fn parse_assign_stmt(pair: Pair<Rule>) -> Result<Stmt, Error<Rule>> {
    let mut inner = pair.into_inner();
    let name = clean_ident(inner.next().unwrap().as_str());
    let value = parse_expr(inner.next().expect("ERROR"))?;

    Ok(
        Stmt::AssignStmt(AssignStmt {
            name,
            value,
        })
    )
}

fn parse_unary_op(pair: Pair<Rule>) -> Result<UnOp, Error<Rule>> {
    match pair.as_str() {
        "&" => Ok(UnOp::Ref),
        "*" => Ok(UnOp::Deref),
        "!" => Ok(UnOp::Not),
        "-" => Ok(UnOp::Neg),
        other =>
            Err(
                Error::new_from_span(
                    ErrorVariant::CustomError { message: format!("unknown unary op: {}", other) },
                    pair.as_span()
                )
            ),
    }
}
fn parse_comparison(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();
    let mut left = parse_arithmetic(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let right = parse_arithmetic(inner.next().unwrap())?;

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
            left: Box::new(left),
            op,
            right: Box::new(right),
        });
    }

    Ok(left)
}

fn parse_struct_init(pair: JunoPair) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();

    let name = clean_ident(inner.next().unwrap().as_str());

    let mut fields = vec![];
    let field_pairs = inner.next().unwrap();
    for f in field_pairs.into_inner() {
        let mut i = f.into_inner();
        let name = clean_ident(i.next().unwrap().as_str());
        let value = parse_expr(i.next().unwrap())?;

        fields.push(StructInitField { name, value });
    }

    Ok(Expr::StructInit(StructInit { name, fields }))
}

fn parse_base_type(pair: Pair<Rule>) -> Result<Type, Error<Rule>> {
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::array_type => {
            let size = first.into_inner().next().unwrap().as_str().parse().unwrap();

            let elem = Type::Named(clean_ident(inner.next().unwrap().as_str()));

            Ok(Type::Array {
                elem: Box::new(elem),
                size,
            })
        }

        Rule::var_ident => { Ok(Type::Named(clean_ident(first.as_str()))) }

        _ => unreachable!(),
    }
}
fn parse_type(pair: Pair<Rule>) -> Result<Type, Error<Rule>> {
    let mut prefixes = Vec::new();
    let mut base = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::type_prefix => prefixes.push(p.as_str()),
            Rule::base_type => {
                base = Some(parse_base_type(p));
            }
            Rule::generics => {}
            _ => unreachable!(),
        }
    }

    let mut ty = base.unwrap()?;

    for p in prefixes.into_iter().rev() {
        ty = match p {
            "&" => Type::Reference(Box::new(ty)),
            "*" => Type::Pointer(Box::new(ty)),
            _ => unreachable!(),
        };
    }

    Ok(ty)
}
fn parse_struct(pair: Pair<Rule>) -> Result<StructDef, Error<Rule>> {
    let mut inner = pair.into_inner();

    let name = clean_ident(inner.next().unwrap().as_str());

    let mut fields = vec![];
    let fields_pairs = inner.next().unwrap();
    for f in fields_pairs.into_inner() {
        let mut i = f.into_inner();
        let name = clean_ident(i.next().unwrap().as_str());
        let ty = parse_type(i.next().unwrap())?;

        fields.push(StructField { name, ty });
    }

    Ok(StructDef { name, fields })
}

fn parse_logical(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let mut inner = pair.into_inner();

    let mut left = parse_comparison(inner.next().unwrap())?;

    while let Some(op) = inner.next() {
        let right = parse_comparison(inner.next().unwrap())?;

        let op = match op.as_str() {
            "&&" => BinOp::And,
            "||" => BinOp::Or,
            _ => unreachable!("invalid logical operator"),
        };

        left = Expr::Binary(BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        });
    }

    Ok(left)
}

fn clean_ident(s: &str) -> String {
    let s = s.trim();

    if s.contains(' ') {
        panic!("invalid identifier: contains whitespace: '{}'", s);
    }

    s.to_string()
}
