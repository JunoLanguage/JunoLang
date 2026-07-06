use inkwell::types::BasicTypeEnum::PointerType;
use inkwell::values::FunctionValue;
use inkwell::types::BasicType;
use crate::metair::*;

use super::*;

use inkwell::types::BasicMetadataTypeEnum;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_program(&mut self) -> Result<(), LLVMError> {
        for function in &self.program.functions {
            self.declare_function(function)?;
        }

        for function in &self.program.functions {
            self.lower_function(function)?;
        }

        Ok(())
    }

    fn declare_function(&mut self, function: &MetaFunction) -> Result<(), LLVMError> {
        let mut params = Vec::<BasicMetadataTypeEnum>::new();

        for param in &function.params {
            params.push(self.lower_type(&param.ty)?.into());
        }

        let fn_type = match &function.ret {
            Some(ret) => { self.lower_type(ret)?.fn_type(&params, false) }

            None => { self.context.void_type().fn_type(&params, false) }
        };

        let name = &self.program.symbol_table[function.name as usize];

        let llvm_function = self.module.add_function(name, fn_type, None);

        self.functions.insert(function.id, llvm_function);

        Ok(())
    }

    fn lower_function(&mut self, function: &MetaFunction) -> Result<(), LLVMError> {
        self.declare_runtime();

        let llvm_function = *self.functions.get(&function.id).unwrap();
        self.current_function = Some(llvm_function);

        let entry = self.context.append_basic_block(llvm_function, "entry");
        self.builder.position_at_end(entry);

        self.push_scope();

        for (index, param) in function.params.iter().enumerate() {
            let llvm_param = llvm_function
                .get_nth_param(index as u32)
                .ok_or_else(|| {
                    LLVMError::Message(format!("missing llvm parameter {}", index))
                })?;

            llvm_param.set_name(&self.program.symbol_table[param.name as usize]);

            let ptr = self.builder
                .build_alloca(
                    self.lower_type(&param.ty)?,
                    &self.program.symbol_table[param.name as usize]
                )
                .map_err(|e| LLVMError::Message(e.to_string()))?;

            self.builder
                .build_store(ptr, llvm_param)
                .map_err(|e| LLVMError::Message(e.to_string()))?;

            self.insert_variable(param.name, ptr, llvm_param.get_type());
        }

        for stmt in &function.body {
            self.lower_stmt(stmt)?;
        }

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            match function.ret {
                Some(_) => {
                    return Err(
                        LLVMError::Message(
                            format!(
                                "function '{}' is missing a return statement",
                                self.program.symbol_table[function.name as usize]
                            )
                        )
                    );
                }

                None => {
                    self.builder.build_return(None).map_err(|e| LLVMError::Message(e.to_string()))?;
                }
            }
        }

        self.pop_scope();

        Ok(())
    }
    pub fn add_function_in_namespace(
        &mut self,
        namespace: u16,
        id: u16,
        function: &FunctionValue<'ctx>
    ) -> Result<(), LLVMError> {
        let id = ((namespace as u32) << 16) | (id as u32);
        self.functions.insert(id, *function);

        Ok(())
    }
    pub fn add_function(
        &mut self,
        id: u32,
        function: &FunctionValue<'ctx>
    ) -> Result<(), LLVMError> {
        self.functions.insert(id, *function);
        Ok(())
    }

    pub fn get_function_in_namespace(
        &mut self,
        namespace: u16,
        id: u16
    ) -> Result<FunctionValue, LLVMError> {
        let id = ((namespace as u32) << 16) | (id as u32);
        if let Some(f) = self.functions.get(&id) {
            return Ok(*f);
        }

        Err(
            LLVMError::Message(
                format!("unknown function '{}'", self.program.symbol_table[id as usize])
            )
        )
    }
    pub fn get_function(&self, target: &[SymbolId]) -> Result<FunctionValue<'ctx>, LLVMError> {
        if target.len() != 1 {
            return Err(LLVMError::Message("qualified calls are not implemented".into()));
        }

        let id = target[0];

        if let Some(f) = self.functions.get(&id) {
            return Ok(*f);
        }
        let namespace = (id >> 16) as u16;
        let local_id = (id & 0xffff) as u32;
        match namespace {
            0 => {
                Err(
                    LLVMError::Message(
                        format!("unknown function '{}'", self.program.symbol_table[id as usize])
                    )
                )
            }
            _ =>
                Err(
                    LLVMError::Message(
                        format!("unknown function with id {} in namespace {}", local_id, namespace)
                    )
                ),
        }
    }
}
