//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ast::JunoSpan;
use crate::metair::*;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;

use super::*;

use inkwell::types::BasicMetadataTypeEnum;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_program(&mut self) -> Result<(), LLVMError> {
        for s in self.program.structs.values() {
            self.lower_struct(s, &s.span)?;
        }
        for declaration in self.program.declarations.values() {
            self.lower_declaration(declaration, &declaration.span)?;
        }

        for function in self.program.functions.values() {
            self.declare_function(function)?;
        }

        for function in self.program.functions.values() {
            self.lower_function(function, &function.span)?;
        }

        Ok(())
    }

    fn declare_function(&mut self, function: &MetaFunction) -> Result<(), LLVMError> {
        let mut params = Vec::<BasicMetadataTypeEnum>::with_capacity(function.params.len());

        for param in &function.params {
            params.push(self.lower_type(&param.ty, &param.span)?.into());
        }

        let fn_type = match &function.ret {
            Some(ret) => self
                .lower_type(ret, &function.span)?
                .fn_type(&params, false),

            None => self.context.void_type().fn_type(&params, false),
        };

        let llvm_function = self
            .module
            .add_function(function.name.as_str(), fn_type, None);

        self.functions.insert(function.name.clone(), llvm_function);

        Ok(())
    }

    fn lower_function(
        &mut self,
        function: &'ctx MetaFunction,
        span: &JunoSpan,
    ) -> Result<(), LLVMError> {
        self.declare_runtime();

        let llvm_function = *self.functions.get(&function.name).unwrap();

        self.current_function = Some(llvm_function);
        self.current_meta_function = Some(function);

        let entry = self.context.append_basic_block(llvm_function, "entry");
        self.builder.position_at_end(entry);

        self.push_scope();

        for (index, param) in function.params.iter().enumerate() {
            let llvm_param = llvm_function.get_nth_param(index as u32).ok_or_else(|| {
                self.make_span_error(format!("missing llvm parameter {}", index), *span)
            })?;

            llvm_param.set_name(param.name.as_str());

            let llvm_type = self.lower_type(&param.ty, &param.span)?;
            let ptr = self
                .builder
                .build_alloca(llvm_type, param.name.as_str())
                .map_err(|e| self.make_span_error(e.to_string(), *span))?;

            self.builder
                .build_store(ptr, llvm_param)
                .map_err(|e| self.make_span_error(e.to_string(), *span))?;

            self.insert_variable(&param.name, ptr, llvm_param.get_type());
        }

        for stmt in &function.body {
            self.lower_stmt(stmt, stmt.span())?;
        }

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            match function.ret {
                Some(_) => {
                    return Err(self.make_span_error(
                        format!(
                            "function '{}' is missing a return statement",
                            function.name.as_str()
                        ),
                        *span,
                    ));
                }

                None => {
                    self.builder
                        .build_return(None)
                        .map_err(|e| self.make_span_error(e.to_string(), *span))?;
                }
            }
        }

        self.pop_scope();

        Ok(())
    }

    pub fn add_function(
        &mut self,
        id: String,
        function: &FunctionValue<'ctx>,
    ) -> Result<(), LLVMError> {
        self.functions.insert(id, *function);
        Ok(())
    }

    pub fn get_function(&self, target: &str) -> Result<FunctionValue<'ctx>, LLVMError> {
        if let Some(f) = self.functions.get(target) {
            return Ok(*f);
        }
        Err(LLVMError::Message(format!("unknown function '{}'", target)))
    }
}
