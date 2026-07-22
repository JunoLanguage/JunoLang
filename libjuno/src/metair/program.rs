//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::ast::*;
use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;

impl<'a> MetaIRGen<'a> {
    // =======================
    // Entry Point
    // =======================

    pub fn lower_program(&mut self, program: &'a Program) -> MetaProgram {
        self.program = program;

        let span = program.span;

        let mut functions = HashMap::new();
        let mut declarations = HashMap::new();

        for item in &program.items {
            match item {
                Item::Function(function, _) => {
                    functions.insert(function.name.clone(), self.lower_function(function));
                }

                Item::Struct(struct_def, _) => {
                    let meta = self.lower_struct(struct_def);
                    self.structs.insert(meta.name.clone(), meta);
                }

                Item::Declaration(decl, _) => {
                    declarations.insert(decl.name.clone(), self.lower_declaration(decl));
                }

                Item::Import(_, _) => {}
            }
        }

        let struct_fields = self
            .struct_fields
            .iter()
            .map(|(struct_id, fields)| {
                let mut ordered = vec![String::new(); fields.len()];

                for (name, index) in fields {
                    ordered[*index as usize] = name.clone();
                }

                (struct_id.clone(), ordered)
            })
            .collect();

        MetaProgram {
            span,
            functions,
            structs: self.structs.clone(),
            declarations,
            string_table: self.string_list.clone(),
            symbol_table: self.symbol_list.clone(),
            struct_fields,
        }
    }

    // =======================
    // Functions
    // =======================

    fn lower_function(&mut self, function: &Function) -> MetaFunction {
        self.locals.push(HashMap::new());

        let name = self.intern_symbol(&function.name);

        let mut params = Vec::new();

        for param in &function.params {
            let ty = self.lower_type(&param.ty);

            self.locals
                .last_mut()
                .unwrap()
                .insert(param.name.clone(), ty.clone());

            params.push(MetaParam {
                span: param.span,
                name: param.name.clone(),
                ty,
            });
        }

        let ret = function.return_type.as_ref().map(|ty| self.lower_type(ty));

        self.declarations.insert(
            name.clone(),
            MetaDeclaration {
                span: function.span,
                name: name.clone(),
                params: params.clone(),
                ret: ret.clone(),
            },
        );

        let (body, locals) = self.lower_block(&function.body);

        self.locals.pop();

        MetaFunction {
            span: function.span,
            name,
            locals,
            params,
            ret,
            body,
        }
    }

    fn lower_declaration(&mut self, declaration: &Declaration) -> MetaDeclaration {
        self.locals.push(HashMap::new());

        let name = self.intern_symbol(&declaration.name);

        let mut params = Vec::new();

        for param in &declaration.params {
            let ty = self.lower_type(&param.ty);

            self.locals
                .last_mut()
                .unwrap()
                .insert(param.name.clone(), ty.clone());

            params.push(MetaParam {
                span: param.span,
                name: param.name.clone(),
                ty,
            });
        }

        self.locals.pop();

        let ret = declaration
            .return_type
            .as_ref()
            .map(|ty| self.lower_type(ty));

        let meta = MetaDeclaration {
            span: declaration.span,
            name: name.clone(),
            params,
            ret,
        };

        self.declarations.insert(name, meta.clone());

        meta
    }

    // =======================
    // Structs
    // =======================

    fn lower_struct(&mut self, strukt: &StructDef) -> MetaStruct {
        let meta = MetaStruct {
            span: strukt.span,
            name: self.intern_symbol(&strukt.name),
            fields: strukt
                .fields
                .iter()
                .map(|field| MetaField {
                    span: field.span,
                    name: field.name.clone(),
                    index: self.intern_struct_field(strukt.name.clone(), &field.name),
                    ty: self.lower_type(&field.ty),
                })
                .collect(),
        };

        // Field indices are local to each struct.
        self.next_struct_field = 0;

        meta
    }
}
