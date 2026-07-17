use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::ops::Deref;

use pest::Parser;
use pest::iterators::Pair;

use crate::metair::metair::*;
use crate::parser::JunoASTParser;
use crate::{JunoParser, Rule, ast::*, builtin_registry, get_builtin_id, is_builtin};

// =======================
// Generator State
// =======================

pub struct MetaIRGen<'a> {
    pub program: &'a Program,
    pub symbols: HashMap<String, SymbolId>,
    pub struct_fields: HashMap<SymbolId, HashMap<String, u32>>, // struct_id => {field_name1 => field_id1, ...}
    pub strings: HashMap<String, StringId>,
    pub declarations: HashMap<String, MetaDeclaration>,
    pub symbol_list: Vec<String>,
    pub string_list: Vec<String>,
    pub locals: Vec<HashMap<SymbolId, MetaType>>,
    next_symbol: u32,
    next_string: u32,
    next_func: u32,
    next_type: u32,
    next_struct_field: u32,
    counter: u32,
}

impl MetaProgram {
    pub fn get_struct(&self, name: SymbolId) -> Option<&MetaStruct> {
        self.structs.get(&name)
    }
}

impl<'a> MetaIRGen<'a> {
    pub fn new(p: &'a Program) -> Self {
        Self {
            program: p,
            declarations: HashMap::new(),
            locals: Vec::new(),
            symbols: HashMap::new(),
            struct_fields: HashMap::new(),
            strings: HashMap::new(),
            symbol_list: vec![],
            string_list: vec![],
            next_symbol: 0,
            next_string: 0,
            next_func: 0,
            next_type: 0,
            next_struct_field: 0,
            counter: 0,
        }
    }

    // =======================
    // Interning
    // =======================

    pub(in crate::metair) fn intern_symbol(&mut self, s: &str) -> SymbolId {
        return s.to_string();
    }
    fn intern_struct_field(&mut self, struct_id: SymbolId, field_name: &str) -> u32 {
        if let Some(fields) = self.struct_fields.get(&struct_id) {
            if let Some(id) = fields.get(field_name) {
                return id.clone();
            }
        } else {
            self.struct_fields.insert(struct_id.clone(), HashMap::new());
        }
        let id = self.next_struct_field;
        self.next_struct_field += 1;

        let fields = &mut self.struct_fields.get_mut(&struct_id.clone()).unwrap();
        fields.insert(field_name.to_string(), id);

        id
    }
    fn intern_string(&mut self, s: &str) -> StringId {
        if let Some(id) = self.strings.get(s) {
            return id.clone();
        }

        let id = self.next_string;
        self.next_string += 1;

        self.strings.insert(s.to_string(), id);
        self.string_list.push(s.to_string());

        id
    }
    fn counter(&mut self) -> u32 {
        let c = self.counter;
        self.counter += 1;
        c
    }
    fn reset_counter(&mut self) {
        self.counter = 0;
    }
    // =======================
    // Entry Point
    // =======================

    pub fn lower_program(&mut self, p: &'a Program) -> MetaProgram {
        self.program = p;
        let span = p.span.clone().to_owned();
        let mut functions = HashMap::new();
        let mut structs = HashMap::new();
        let mut declarations = HashMap::new();
        for item in &p.items {
            match item {
                Item::Function(f, span) => {
                    functions.insert(f.name.clone(), self.lower_function(f));
                }

                Item::Struct(s, span) => {
                    structs.insert(s.name.clone(), self.lower_struct(s));
                }

                Item::Import(_, _) => {}
                Item::Declaration(d, span) => {
                    declarations.insert(d.name.clone(), self.lower_declaration(d));
                }
            }
        }
        let struct_fields: HashMap<SymbolId, Vec<String>> =
            HashMap::from_iter(self.struct_fields.clone().into_iter().map(
                |(struct_id, fields)| {
                    (
                        struct_id,
                        fields.clone().into_iter().map(|(key, _)| key).collect(),
                    )
                },
            ));

        MetaProgram {
            span,
            functions,
            structs,
            declarations,
            string_table: self.string_list.clone(),
            symbol_table: self.symbol_list.clone(),
            struct_fields: struct_fields,
        }
    }

    // =======================
    // Functions
    // =======================
    fn lower_function(&mut self, f: &Function) -> MetaFunction {
        let id = self.next_func;
        self.next_func += 1;

        self.locals.push(HashMap::new());

        let name = self.intern_symbol(&f.name);

        let mut params = Vec::new();

        for p in &f.params {
            let span = p.span.clone().to_owned();
            let sym = self.intern_symbol(&p.name);
            let ty = self.lower_type(&p.ty);

            self.locals
                .last_mut()
                .unwrap()
                .insert(sym.clone(), ty.clone());

            params.push(MetaParam {
                span,
                name: sym,
                ty,
            });
        }

        let ret = f.return_type.as_ref().map(|t| self.lower_type(t));
        self.declarations.insert(
            name.clone(),
            MetaDeclaration {
                span: f.span.clone().to_owned(),
                name: name.clone(),
                params: params.clone(),
                ret: ret.clone(),
            },
        );
        let body = self.lower_block(&f.body);

        self.locals.pop();

        let f = MetaFunction {
            span: f.span.clone().to_owned(),
            name,
            params,
            ret,
            body,
        };
        f
    }

    fn lower_declaration(&mut self, f: &Declaration) -> MetaDeclaration {
        let id = self.next_func;
        self.next_func += 1;

        self.locals.push(HashMap::new());

        let name = self.intern_symbol(&f.name);

        let mut params = Vec::new();

        for p in &f.params {
            let sym = self.intern_symbol(&p.name);
            let ty = self.lower_type(&p.ty);

            self.locals
                .last_mut()
                .unwrap()
                .insert(sym.clone(), ty.clone());

            params.push(MetaParam {
                span: p.span.clone(),
                name: sym,
                ty,
            });
        }

        let ret = f.return_type.as_ref().map(|t| self.lower_type(t));

        let f = MetaDeclaration {
            span: f.span.clone(),
            name: name.clone(),
            params,
            ret,
        };
        self.declarations.insert(name, f.clone());
        f
    }

    // =======================
    // Blocks / Stmts
    // =======================

    fn lower_block(&mut self, b: &Block) -> Vec<MetaStmt> {
        self.locals.push(HashMap::new());

        let body = b.stmts.iter().map(|s| self.lower_stmt(s)).collect();

        self.locals.pop();

        body
    }

    fn lower_stmt(&mut self, s: &Stmt) -> MetaStmt {
        match s {
            Stmt::Let(l) => self.lower_let(l),

            Stmt::AssignStmt(e) => MetaStmt::Assign {
                span: e.span.clone(),
                target: self.intern_symbol(&e.name),
                value: self.lower_expr(&e.value),
            },

            Stmt::Expr(e) => MetaStmt::Expr(self.lower_expr(e)),

            Stmt::Return(e, span) => {
                MetaStmt::Return(e.as_ref().map(|x| self.lower_expr(x)), span.clone())
            }

            Stmt::Break(span) => MetaStmt::Break(span.clone()),

            Stmt::Continue(span) => MetaStmt::Continue(span.clone()),

            Stmt::If(i) => MetaStmt::If {
                span: i.span.clone(),
                cond: self.lower_expr(&i.condition),
                then_body: self.lower_block(&i.then_block),
                else_ifs: i
                    .else_ifs
                    .iter()
                    .map(|(c, b)| (self.lower_expr(c), self.lower_block(b)))
                    .collect(),
                else_body: i.else_block.as_ref().map(|b| self.lower_block(b)),
            },

            Stmt::While(w) => {
                let cond = self.lower_expr(&w.condition);

                MetaStmt::Loop {
                    span: w.span.clone(),
                    body: vec![MetaStmt::If {
                        span: cond.span.clone(),
                        cond: MetaExpr {
                            span: cond.span.clone(),
                            kind: MetaExprKind::Unary {
                                span: cond.span.clone(),
                                op: MetaUnOp::Not,
                                expr: Box::new(cond.clone()),
                            },
                            ty: MetaType::Named(self.intern_symbol("bool"), cond.span.clone()),
                        },

                        then_body: vec![MetaStmt::Break(w.span.clone())],

                        else_ifs: vec![],
                        else_body: Some(self.lower_block(&w.body)),
                    }],
                }
            }

            Stmt::Loop(b) => MetaStmt::Loop {
                span: b.span.clone(),
                body: self.lower_block(b),
            },

            Stmt::For(f) => MetaStmt::Break(f.span.clone()), // TODO
        }
    }
    fn lower_let(&mut self, stmt: &LetStmt) -> MetaStmt {
        let declared_ty = self.lower_type(&stmt.ty);

        let value = stmt.value.as_ref().map(|expr| {
            let value = self.lower_expr(expr);

            let value = self.coerce_expr(value, &declared_ty);

            value
        });

        let id = self.intern_symbol(&stmt.name);

        self.insert_local(id, declared_ty.clone());
        MetaStmt::Let {
            span: stmt.span.clone(),
            name: self.intern_symbol(&stmt.name),
            mutable: stmt.mutable,
            ty: Some(declared_ty),
            value,
        }
    }
    fn coerce_expr(&mut self, mut expr: MetaExpr, expected: &MetaType) -> MetaExpr {
        match (&mut expr.kind, &expr.ty, expected) {
            (_, actual, expected) if actual == expected => expr,

            (
                MetaExprKind::Const(MetaConst::Int(_, _), _),
                MetaType::Named(_, _),
                MetaType::Named(_, _),
            ) => {
                expr.ty = expected.clone();
                expr
            }

            (
                MetaExprKind::Array(values, _),
                MetaType::Array { .. },
                MetaType::Array {
                    span,
                    elem: expected_elem,
                    size,
                },
            ) => {
                if values.len() > (*size as usize) {
                    panic!("{:?}", span.err_to_report("array too large"));
                }

                for value in values.iter_mut() {
                    *value = self.coerce_expr(value.clone(), expected_elem);
                }

                expr.ty = expected.clone();
                expr
            }

            _ => panic!(
                "{:?}",
                expr.span.err_to_report(&format!(
                    "type mismatch: expected {}, got {}",
                    expected, expr.ty
                ))
            ),
        }
    }
    fn lower_struct(&mut self, s: &StructDef) -> MetaStruct {
        let s = MetaStruct {
            span: s.span.clone(),
            name: self.intern_symbol(&s.name),
            fields: s
                .fields
                .iter()
                .map(|f| MetaField {
                    span: f.span.clone(),
                    index: self.intern_struct_field(s.name.clone(), &f.name),
                    ty: self.lower_type(&f.ty),
                })
                .collect(),
        };
        self.next_struct_field = 0;
        s
    }
    // =======================
    // Expressions
    // =======================

    fn lower_expr(&mut self, e: &Expr) -> MetaExpr {
        match e {
            Expr::Number(n, span) => MetaExpr {
                span: span.clone(),
                kind: MetaExprKind::Const(MetaConst::Int(*n, span.clone()), span.clone()),
                ty: MetaType::Named(self.intern_symbol("i32"), span.clone()),
            },

            Expr::Boolean(b, span) => MetaExpr {
                span: span.clone(),
                kind: MetaExprKind::Const(MetaConst::Bool(*b, span.clone()), span.clone()),
                ty: MetaType::Named(self.intern_symbol("bool"), span.clone()),
            },

            Expr::Char(c, span) => MetaExpr {
                span: span.clone(),
                kind: MetaExprKind::Const(MetaConst::Char(*c, span.clone()), span.clone()),
                ty: MetaType::Named(self.intern_symbol("char"), span.clone()),
            },

            Expr::String(s, span) => {
                let id = self.intern_string(s);

                MetaExpr {
                    span: span.clone(),
                    kind: MetaExprKind::String(id, span.clone()),
                    ty: MetaType::Pointer(
                        Box::new(MetaType::Named(self.intern_symbol("char"), span.clone())),
                        span.clone(),
                    ),
                }
            }

            Expr::Var(name, span) => {
                let id = self.intern_symbol(name);
                let ty = self.lookup_local_type(id.clone());

                MetaExpr {
                    span: span.clone(),
                    kind: MetaExprKind::Var(id.clone(), span.clone()),
                    ty,
                }
            }

            Expr::Array(values, span) => {
                let values: Vec<_> = values.iter().map(|e| self.lower_expr(e)).collect();

                let elem_ty = values
                    .first()
                    .map(|e| e.ty.clone())
                    .unwrap_or(MetaType::Unit(span.clone()));
                let size = values.len() as u32;
                MetaExpr {
                    span: span.clone(),
                    kind: MetaExprKind::Array(values, span.clone()),
                    ty: MetaType::Array {
                        span: span.clone(),
                        elem: Box::new(elem_ty),
                        size: size,
                    },
                }
            }

            Expr::StructInit(s) => {
                let fields = s
                    .fields
                    .iter()
                    .map(|f| (self.counter(), self.lower_expr(&f.value)))
                    .collect();
                self.reset_counter();
                let ty = MetaType::Named(self.intern_symbol(&s.name), s.span.clone());

                MetaExpr {
                    span: s.span.clone(),
                    kind: MetaExprKind::StructInit {
                        name: self.intern_symbol(&s.name),
                        fields,
                        span: s.span.clone(),
                    },
                    ty,
                }
            }

            Expr::Call(c) => {
                let target: SymbolId = c.target.clone();
                let args: Vec<_> = c
                    .args
                    .iter()
                    .map(|a| match a {
                        Arg::Positional(e, span) => MetaArg::Pos(self.lower_expr(e), span.clone()),

                        Arg::Named(name, e, span) => MetaArg::Named(
                            self.intern_symbol(name),
                            self.lower_expr(e),
                            span.clone(),
                        ),
                    })
                    .collect();

                let ty: MetaType = match self.find_function(target.as_str()) {
                    None => {
                        let builtin = builtin_registry::get_builtin(target.as_str());
                        match builtin {
                            None => match self.declarations.entry(target.clone()) {
                                Entry::Occupied(occupied) => {
                                    let value: &mut MetaDeclaration = occupied.into_mut();
                                    value.ret.clone().unwrap_or(MetaType::Named(
                                        "void".to_string(),
                                        value.span.clone(),
                                    ))
                                }
                                Entry::Vacant(vacant) => todo!(),
                            },
                            Some(b) => {
                                match &b.declare {
                                    builtin_registry::BuiltinEnum::Function {
                                        param_types: _,
                                        return_type,
                                    } => {
                                        let ty_pair: Vec<Pair<Rule>> =
                                            JunoParser::parse(Rule::type_, return_type)
                                                .unwrap()
                                                .collect();
                                        let mut ast_parser = JunoASTParser::new("_".to_string());
                                        let ast_ty = ast_parser
                                            .parse_type(ty_pair.first().unwrap().clone())
                                            .unwrap();
                                        self.lower_type(&ast_ty)
                                    } //_ => panic!("Function not declared: {}", target_name) // unreachable
                                }
                            }
                        }
                    }
                    Some(f) => self.lower_type(&f.return_type.as_ref().unwrap()),
                };

                MetaExpr {
                    kind: MetaExprKind::Call {
                        target,
                        args,
                        span: c.span.clone(),
                    },
                    ty,
                    span: c.span.clone(),
                }
            }

            Expr::Binary(b) => {
                let lhs = self.lower_expr(&b.left);
                let rhs = self.lower_expr(&b.right);
                let (lhs, rhs) = self.coerce_binary(lhs, rhs).unwrap();
                let ty = match b.op {
                    BinOp::Eq
                    | BinOp::Neq
                    | BinOp::Lt
                    | BinOp::Lte
                    | BinOp::Gt
                    | BinOp::Gte
                    | BinOp::And
                    | BinOp::Or => MetaType::Named(self.intern_symbol("bool"), b.span.clone()),

                    _ => lhs.ty.clone(),
                };

                MetaExpr {
                    span: b.span.clone(),
                    kind: MetaExprKind::Binary {
                        span: b.span.clone(),
                        op: self.lower_binop(&b.op),
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ty,
                }
            }

            Expr::Unary(u) => {
                let expr = self.lower_expr(&u.expr);

                let ty = match u.op {
                    UnOp::Ref => MetaType::Pointer(Box::new(expr.ty.clone()), u.span.clone()),

                    UnOp::Deref => match &expr.ty {
                        MetaType::Pointer(inner, span) => (**inner).clone(),
                        MetaType::Reference(inner, span) => (**inner).clone(),
                        _ => expr.ty.clone(),
                    },

                    _ => expr.ty.clone(),
                };

                MetaExpr {
                    span: u.span.clone(),
                    kind: MetaExprKind::Unary {
                        span: u.span.clone(),
                        op: self.lower_unop(&u.op),
                        expr: Box::new(expr),
                    },
                    ty,
                }
            }
        }
    }
    fn coerce_binary(
        &mut self,
        mut lhs: MetaExpr,
        mut rhs: MetaExpr,
    ) -> Result<(MetaExpr, MetaExpr), miette::Error> {
        // Bereits gleicher Typ
        if lhs.ty == rhs.ty {
            return Ok((lhs, rhs));
        }

        match (&lhs.kind, &rhs.kind) {
            (.., MetaExprKind::Const(MetaConst::Int(_, _), _)) => {
                rhs = self.coerce_expr(rhs, &lhs.ty);
                return Ok((lhs, rhs));
            }

            // linke Seite ist Integer-Literal
            (MetaExprKind::Const(MetaConst::Int(_, _), _), ..) => {
                lhs = self.coerce_expr(lhs, &rhs.ty);
                return Ok((lhs, rhs));
            }

            _ => {}
        }

        Err(lhs
            .span
            .err_to_report(&format!("type mismatch: {} vs {}", lhs.ty, rhs.ty)))
    }
    // =======================
    // Ops
    // =======================

    fn lower_binop(&self, op: &BinOp) -> MetaBinOp {
        match op {
            BinOp::Add => MetaBinOp::Add,
            BinOp::Sub => MetaBinOp::Sub,
            BinOp::Mul => MetaBinOp::Mul,
            BinOp::Div => MetaBinOp::Div,
            BinOp::Mod => MetaBinOp::Mod,

            BinOp::Eq => MetaBinOp::Eq,
            BinOp::Neq => MetaBinOp::Neq,
            BinOp::Lt => MetaBinOp::Lt,
            BinOp::Gt => MetaBinOp::Gt,
            BinOp::Lte => MetaBinOp::Lte,
            BinOp::Gte => MetaBinOp::Gte,

            BinOp::And => MetaBinOp::And,
            BinOp::Or => MetaBinOp::Or,

            _ => MetaBinOp::Add,
        }
    }

    fn lower_unop(&self, op: &UnOp) -> MetaUnOp {
        match op {
            UnOp::Neg => MetaUnOp::Neg,
            UnOp::Not => MetaUnOp::Not,
            UnOp::Ref => MetaUnOp::Ref,
            UnOp::Deref => MetaUnOp::Deref,
        }
    }

    // =======================
    // Types
    // =======================

    fn lower_type(&mut self, t: &Type) -> MetaType {
        match t {
            Type::Named(n, span) => MetaType::Named(self.intern_symbol(n), span.clone()),

            Type::Pointer(inner, span) => {
                MetaType::Pointer(Box::new(self.lower_type(inner)), span.clone())
            }

            Type::Reference(inner, span) => {
                MetaType::Reference(Box::new(self.lower_type(inner)), span.clone())
            }

            Type::Array { elem, size, span } => MetaType::Array {
                span: span.clone(),
                elem: Box::new(self.lower_type(elem)),
                size: *size,
            },

            Type::Generic {
                base,
                args: _,
                span,
            } => MetaType::Named(self.intern_symbol(base), span.clone()),
        }
    }

    pub(crate) fn find_function(&self, name: &str) -> Option<&'a Function> {
        self.program.items.iter().find_map(move |item| match item {
            Item::Function(f, span) if f.name == name => Some(f),
            _ => None,
        })
    }
}
