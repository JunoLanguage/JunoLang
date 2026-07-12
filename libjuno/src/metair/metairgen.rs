use std::collections::HashMap;
use std::hash::Hash;

use crate::metair::metair::*;
use crate::{ast::*, builtin_registry, get_builtin_id, is_builtin};

// =======================
// Generator State
// =======================

pub struct MetaIRGen<'a> {
    pub program: &'a Program,
    pub symbols: HashMap<String, SymbolId>,
    pub struct_fields: HashMap<SymbolId, HashMap<String, u32>>, // struct_id => {field_name1 => field_id1, ...}
    pub strings: HashMap<String, StringId>,

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
        let mut functions = HashMap::new();
        let mut structs = HashMap::new();

        for item in &p.items {
            match item {
                Item::Function(f) => {
                    functions.insert(f.name.clone(), self.lower_function(f));
                }

                Item::Struct(s) => {
                    structs.insert(s.name.clone(), self.lower_struct(s));
                }

                Item::Import(_) => {}
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
            functions,
            structs,
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
            let sym = self.intern_symbol(&p.name);
            let ty = self.lower_type(&p.ty);

            self.locals
                .last_mut()
                .unwrap()
                .insert(sym.clone(), ty.clone());

            params.push(MetaParam { name: sym, ty });
        }

        let ret = f.return_type.as_ref().map(|t| self.lower_type(t));

        let body = self.lower_block(&f.body);

        self.locals.pop();

        let f = MetaFunction {
            name,
            params,
            ret,
            body,
        };
        dbg!(&f);
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
                target: self.intern_symbol(&e.name),
                value: self.lower_expr(&e.value),
            },

            Stmt::Expr(e) => MetaStmt::Expr(self.lower_expr(e)),

            Stmt::Return(e) => MetaStmt::Return(e.as_ref().map(|x| self.lower_expr(x))),

            Stmt::Break => MetaStmt::Break,

            Stmt::Continue => MetaStmt::Continue,

            Stmt::If(i) => MetaStmt::If {
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
                    body: vec![MetaStmt::If {
                        cond: MetaExpr {
                            kind: MetaExprKind::Unary {
                                op: MetaUnOp::Not,
                                expr: Box::new(cond),
                            },
                            ty: MetaType::Named(self.intern_symbol("bool")),
                        },

                        then_body: vec![MetaStmt::Break],

                        else_ifs: vec![],
                        else_body: Some(self.lower_block(&w.body)),
                    }],
                }
            }

            Stmt::Loop(b) => MetaStmt::Loop {
                body: self.lower_block(b),
            },

            Stmt::For(_) => MetaStmt::Break,
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
            name: self.intern_symbol(&stmt.name),
            mutable: stmt.mutable,
            ty: Some(declared_ty),
            value,
        }
    }
    fn coerce_expr(&mut self, mut expr: MetaExpr, expected: &MetaType) -> MetaExpr {
        match (&mut expr.kind, &expr.ty, expected) {
            (_, actual, expected) if actual == expected => expr,

            (MetaExprKind::Const(MetaConst::Int(_)), MetaType::Named(_), MetaType::Named(_)) => {
                expr.ty = expected.clone();
                expr
            }

            (
                MetaExprKind::Array(values),
                MetaType::Array { .. },
                MetaType::Array {
                    elem: expected_elem,
                    size,
                },
            ) => {
                if values.len() > (*size as usize) {
                    panic!("array too large");
                }

                for value in values.iter_mut() {
                    *value = self.coerce_expr(value.clone(), expected_elem);
                }

                expr.ty = expected.clone();
                expr
            }

            _ => panic!("type mismatch: expected {:?}, got {:?}", expected, expr.ty),
        }
    }
    fn lower_struct(&mut self, s: &StructDef) -> MetaStruct {
        let s = MetaStruct {
            name: self.intern_symbol(&s.name),
            fields: s
                .fields
                .iter()
                .map(|f| MetaField {
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
            Expr::Number(n) => MetaExpr {
                kind: MetaExprKind::Const(MetaConst::Int(*n)),
                ty: MetaType::Named(self.intern_symbol("i32")),
            },

            Expr::Boolean(b) => MetaExpr {
                kind: MetaExprKind::Const(MetaConst::Bool(*b)),
                ty: MetaType::Named(self.intern_symbol("bool")),
            },

            Expr::Char(c) => MetaExpr {
                kind: MetaExprKind::Const(MetaConst::Char(*c)),
                ty: MetaType::Named(self.intern_symbol("char")),
            },

            Expr::String(s) => {
                let id = self.intern_string(s);

                MetaExpr {
                    kind: MetaExprKind::String(id),
                    ty: MetaType::Pointer(Box::new(MetaType::Named(self.intern_symbol("char")))),
                }
            }

            Expr::Var(name) => {
                let id = self.intern_symbol(name);
                let ty = self.lookup_local_type(id.clone());

                MetaExpr {
                    kind: MetaExprKind::Var(id.clone()),
                    ty,
                }
            }

            Expr::Array(values) => {
                let values: Vec<_> = values.iter().map(|e| self.lower_expr(e)).collect();

                let elem_ty = values
                    .first()
                    .map(|e| e.ty.clone())
                    .unwrap_or(MetaType::Unit);
                let size = values.len() as u32;
                MetaExpr {
                    kind: MetaExprKind::Array(values),
                    ty: MetaType::Array {
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
                let ty = MetaType::Named(self.intern_symbol(&s.name));

                MetaExpr {
                    kind: MetaExprKind::StructInit {
                        name: self.intern_symbol(&s.name),
                        fields,
                    },
                    ty,
                }
            }

            Expr::Call(c) => {
                dbg!(&c.target);
                let target: SymbolId = c.target.clone();
                dbg!(&target);
                let args: Vec<_> = c
                    .args
                    .iter()
                    .map(|a| match a {
                        Arg::Positional(e) => MetaArg::Pos(self.lower_expr(e)),

                        Arg::Named(name, e) => {
                            MetaArg::Named(self.intern_symbol(name), self.lower_expr(e))
                        }
                    })
                    .collect();

                let ty: MetaType = match self.find_function(target.as_str()) {
                    None => {
                        let builtin = builtin_registry::get_builtin(target.as_str());
                        match builtin {
                            None => {
                                todo!()
                            }
                            Some(b) => {
                                match &b.declare {
                                    builtin_registry::BuiltinEnum::Function {
                                        param_types: _,
                                        return_type,
                                    } => self.lower_type(&Type::Named(return_type.to_string())), //_ => panic!("Function not declared: {}", target_name) // unreachable
                                }
                            }
                        }
                    }
                    Some(f) => self.lower_type(&f.return_type.as_ref().unwrap()),
                };

                MetaExpr {
                    kind: MetaExprKind::Call { target, args },
                    ty,
                }
            }

            Expr::Binary(b) => {
                let lhs = self.lower_expr(&b.left);
                let rhs = self.lower_expr(&b.right);

                let ty = match b.op {
                    BinOp::Eq
                    | BinOp::Neq
                    | BinOp::Lt
                    | BinOp::Lte
                    | BinOp::Gt
                    | BinOp::Gte
                    | BinOp::And
                    | BinOp::Or => MetaType::Named(self.intern_symbol("bool")),

                    _ => lhs.ty.clone(),
                };

                MetaExpr {
                    kind: MetaExprKind::Binary {
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
                    UnOp::Ref => MetaType::Pointer(Box::new(expr.ty.clone())),

                    UnOp::Deref => match &expr.ty {
                        MetaType::Pointer(inner) => (**inner).clone(),
                        MetaType::Reference(inner) => (**inner).clone(),
                        _ => expr.ty.clone(),
                    },

                    _ => expr.ty.clone(),
                };

                MetaExpr {
                    kind: MetaExprKind::Unary {
                        op: self.lower_unop(&u.op),
                        expr: Box::new(expr),
                    },
                    ty,
                }
            }
        }
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
            Type::Named(n) => MetaType::Named(self.intern_symbol(n)),

            Type::Pointer(inner) => MetaType::Pointer(Box::new(self.lower_type(inner))),

            Type::Reference(inner) => MetaType::Reference(Box::new(self.lower_type(inner))),

            Type::Array { elem, size } => MetaType::Array {
                elem: Box::new(self.lower_type(elem)),
                size: *size,
            },

            Type::Generic { base, args: _ } => MetaType::Named(self.intern_symbol(base)),
        }
    }

    pub(crate) fn find_function(&self, name: &str) -> Option<&'a Function> {
        self.program.items.iter().find_map(move |item| match item {
            Item::Function(f) if f.name == name => Some(f),
            _ => None,
        })
    }
}
