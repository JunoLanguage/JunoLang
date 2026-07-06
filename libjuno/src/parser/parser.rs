use pest::iterators::{ Pair, Pairs };

use crate::*;
use crate::ast::*;
type JunoPair<'a> = Pair<'a, Rule>;

pub fn parse_program(_pair: Pair<Rule>) -> Program {
    let mut items = vec![];
    let pairs = _pair.into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::item => {
                items.push(parse_item(pair));
            }
            Rule::EOI => {}
            other => panic!("unexpected rule in program: {:?}", other),
        }
    }

    Program { items }
}

fn parse_item(pair: JunoPair) -> Item {
    let p = pair.clone().into_inner().last().expect("Error");
    match p.as_rule() {
        Rule::function => Item::Function(parse_function(p)),
        Rule::import_stmt => Item::Import(parse_import(p)),
        Rule::struct_def => Item::Struct(parse_struct(p)),
        other => panic!("unhandled rule in pair: {:#?}, parse_item: {:?}", pair, other),
    }
}

fn parse_import(pair: JunoPair) -> Import {
    let mut inner = pair.into_inner();

    let path = clean_ident(inner.next().unwrap().as_str());

    Import { path }
}

fn parse_function(pair: JunoPair) -> Function {
    let mut inner = pair.into_inner();

    let name = clean_ident(inner.next().unwrap().as_str());

    let mut params = vec![];
    let mut return_type = None;

    for p in inner.by_ref() {
        match p.as_rule() {
            Rule::params => {
                params = parse_params(p);
            }
            Rule::type_ => {
                return_type = Some(parse_type(p));
            }
            Rule::block => {
                let body = parse_block(p);
                return Function {
                    name,
                    params,
                    return_type,
                    body,
                };
            }
            _ => {}
        }
    }

    unreachable!()
}

fn parse_params(pair: JunoPair) -> Vec<Param> {
    pair.into_inner()
        .map(|p| {
            let mut inner = p.into_inner();
            let name = clean_ident(inner.next().unwrap().as_str());
            let ty = parse_type(inner.next().unwrap());

            Param { name, ty }
        })
        .collect()
}

fn parse_block(pair: JunoPair) -> Block {
    let stmts = pair.into_inner().map(parse_stmt).collect();

    Block { stmts }
}

fn parse_stmt(pair: JunoPair) -> Stmt {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::let_stmt => parse_let(inner),
        Rule::assign_stmt => parse_assign_stmt(inner),
        Rule::expr_stmt => { Stmt::Expr(parse_expr(inner.into_inner().next().unwrap())) }
        Rule::return_stmt => {
            let mut i = inner.into_inner();
            Stmt::Return(i.next().map(parse_expr))
        }
        Rule::break_stmt => Stmt::Break,
        Rule::continue_stmt => Stmt::Continue,
        Rule::if_stmt => parse_if(inner),
        Rule::while_stmt => parse_while(inner),
        Rule::for_stmt => parse_for(inner),
        Rule::loop_stmt => parse_loop(inner),
        _ => unreachable!("bad stmt"),
    }
}

fn parse_if(pair: JunoPair) -> Stmt {
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap());

    let then_block = parse_block(inner.next().unwrap());

    let mut else_ifs = vec![];
    let mut else_block = None;

    for p in inner {
        match p.as_rule() {
            Rule::else_if => {
                let mut i = p.into_inner();
                let cond = parse_expr(i.next().unwrap());
                let block = parse_block(i.next().unwrap());
                else_ifs.push((cond, block));
            }

            Rule::else_block => {
                let block = parse_block(p.into_inner().next().unwrap());
                else_block = Some(block);
            }

            _ => {}
        }
    }

    Stmt::If(IfStmt {
        condition,
        then_block,
        else_ifs,
        else_block,
    })
}
fn parse_while(pair: JunoPair) -> Stmt {
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap());
    let body = parse_block(inner.next().unwrap());

    Stmt::While(WhileStmt {
        condition,
        body,
    })
}
fn parse_loop(pair: JunoPair) -> Stmt {
    let mut inner = pair.into_inner();

    let body = parse_block(inner.next().unwrap());

    Stmt::Loop(body)
}
fn parse_for(pair: JunoPair) -> Stmt {
    let mut inner = pair.into_inner();

    let init = parse_expr(inner.next().unwrap());
    let iter = parse_expr(inner.next().unwrap());
    let body = parse_block(inner.next().unwrap());

    Stmt::For(ForStmt {
        init,
        iter,
        body,
    })
}

fn parse_array(pair: Pair<Rule>) -> Expr {
    let mut items = vec![];

    for e in pair.into_inner() {
        items.push(parse_expr(e));
    }

    Expr::Array(items)
}

fn parse_primary(pair: JunoPair) -> Expr {
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::expr => parse_expr(first),

        Rule::number => Expr::Number(first.as_str().parse().unwrap()),

        Rule::boolean => Expr::Boolean(first.as_str() == "true"),

        Rule::string => parse_string(first),

        Rule::char => Expr::Char(parse_char_literal(first.as_str())),

        Rule::var_ident => Expr::Var(clean_ident(first.as_str())),

        Rule::call => parse_call(first),

        Rule::array => parse_array(first),

        Rule::struct_init => parse_struct_init(first),

        other => panic!("unexpected primary: {:?}", other),
    }
}
fn parse_string(pair: JunoPair) -> Expr {
    let raw = pair.as_str();
    let inner = &raw[1..raw.len() - 1];
    let mut s = inner.to_string();
    s = s.replace("\\n", "\n");
    s = s.replace("\\t", "\n");
    s = s.replace("\\r", "\n");
    s = s.replace("\\\"", "\"");
    s = s.replace("\\\\", "\\");

    Expr::String(s)
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
fn parse_let(pair: JunoPair) -> Stmt {
    let mut inner = pair.into_inner();
    let possible_mutable_pair = inner.next().unwrap();
    let mutable = possible_mutable_pair.as_str() == "mut";
    let mut name: String;
    if !mutable {
        name = clean_ident(possible_mutable_pair.as_str());
    } else {
        name = clean_ident(inner.next().unwrap().as_str());
    }
    let ty = parse_type(inner.next().unwrap());
    let value = inner.next().map(parse_expr);

    Stmt::Let(LetStmt {
        mutable,
        name,
        ty,
        value,
    })
}

fn parse_expr(pair: JunoPair) -> Expr {
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

fn parse_call(pair: Pair<Rule>) -> Expr {
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
                    args.push(Arg::Positional(parse_expr(inner.into_inner().next().unwrap())));
                }

                Rule::named_arg => {
                    let mut i = inner.into_inner();

                    args.push(
                        Arg::Named(
                            clean_ident(i.next().unwrap().as_str()),
                            parse_expr(i.next().unwrap())
                        )
                    );
                }

                _ => unreachable!(),
            }
        }
    }

    Expr::Call(Call { target, args })
}

fn parse_arithmetic(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let mut left = parse_term(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let right = parse_term(inner.next().unwrap());

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

    left
}

fn parse_term(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let mut left = parse_unary(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let right = parse_unary(inner.next().unwrap());

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

    left
}
fn parse_unary(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner().peekable();

    let mut ops = Vec::new();

    while let Some(p) = inner.peek() {
        if p.as_rule() == Rule::unary_ops {
            ops.push(inner.next().unwrap());
        } else {
            break;
        }
    }

    let mut expr = parse_primary(inner.next().unwrap());

    for op in ops.into_iter().rev() {
        expr = Expr::Unary(UnaryExpr {
            op: parse_unary_op(op),
            expr: Box::new(expr),
        });
    }

    expr
}

fn parse_assign_stmt(pair: Pair<Rule>) -> Stmt {
    let mut inner = pair.into_inner();
    let name = clean_ident(inner.next().unwrap().as_str());
    let value = parse_expr(inner.next().expect("ERROR"));

    Stmt::AssingStmt(AssingStmt {
        name,
        value,
    })
}

fn parse_unary_op(pair: Pair<Rule>) -> UnOp {
    match pair.as_str() {
        "&" => UnOp::Ref,
        "*" => UnOp::Deref,
        "!" => UnOp::Not,
        "-" => UnOp::Neg,
        other => panic!("unknown unary op: {}", other),
    }
}
fn parse_comparison(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut left = parse_arithmetic(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let right = parse_arithmetic(inner.next().unwrap());

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

    left
}

fn parse_struct_init(pair: JunoPair) -> Expr {
    let mut inner = pair.into_inner();

    let name = clean_ident(inner.next().unwrap().as_str());

    let mut fields = vec![];

    for f in inner {
        let mut i = f.into_inner();
        let name = clean_ident(i.next().unwrap().as_str());
        let value = parse_expr(i.next().unwrap());

        fields.push(StructInitField { name, value });
    }

    Expr::StructInit(StructInit { name, fields })
}

fn parse_base_type(pair: Pair<Rule>) -> Type {
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::array_type => {
            let size = first.into_inner().next().unwrap().as_str().parse().unwrap();

            let elem = Type::Named(clean_ident(inner.next().unwrap().as_str()));

            Type::Array {
                elem: Box::new(elem),
                size,
            }
        }

        Rule::var_ident => { Type::Named(clean_ident(first.as_str())) }

        _ => unreachable!(),
    }
}
fn parse_type(pair: Pair<Rule>) -> Type {
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

    let mut ty = base.unwrap();

    for p in prefixes.into_iter().rev() {
        ty = match p {
            "&" => Type::Reference(Box::new(ty)),
            "*" => Type::Pointer(Box::new(ty)),
            _ => unreachable!(),
        };
    }

    ty
}
fn parse_struct(pair: JunoPair) -> StructDef {
    let mut inner = pair.into_inner();

    let name = clean_ident(inner.next().unwrap().as_str());

    let mut fields = vec![];

    for f in inner {
        let mut i = f.into_inner();
        let name = clean_ident(i.next().unwrap().as_str());
        let ty = parse_type(i.next().unwrap());

        fields.push(StructField { name, ty });
    }

    StructDef { name, fields }
}

fn parse_logical(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let mut left = parse_comparison(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let right = parse_comparison(inner.next().unwrap());

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

    left
}

fn clean_ident(s: &str) -> String {
    let s = s.trim();

    if s.contains(' ') {
        panic!("invalid identifier: contains whitespace: '{}'", s);
    }

    s.to_string()
}
