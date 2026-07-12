use crate::metair::*;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;

use super::*;

use inkwell::types::BasicMetadataTypeEnum;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn lower_program(&mut self) -> Result<(), LLVMError> {
        for (function_name, function) in &self.program.functions {
            self.declare_function(function)?;
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
            Some(ret) => self.lower_type(ret)?.fn_type(&params, false),

            None => self.context.void_type().fn_type(&params, false),
        };

        let llvm_function = self
            .module
            .add_function(function.name.as_str(), fn_type, None);

        self.functions.insert(function.name.clone(), llvm_function);

        Ok(())
    }

    fn lower_function(&mut self, function: &MetaFunction) -> Result<(), LLVMError> {
        self.declare_runtime();

        let llvm_function = *self.functions.get(&function.name).unwrap();
        self.current_function = Some(llvm_function);

        let entry = self.context.append_basic_block(llvm_function, "entry");
        self.builder.position_at_end(entry);

        self.push_scope();

        for (index, param) in function.params.iter().enumerate() {
            let llvm_param = llvm_function
                .get_nth_param(index as u32)
                .ok_or_else(|| LLVMError::Message(format!("missing llvm parameter {}", index)))?;

            llvm_param.set_name(&param.name.as_str());

            let llvm_type = self.lower_type(&param.ty)?;
            let ptr = self
                .builder
                .build_alloca(llvm_type, &param.name.as_str())
                .map_err(|e| LLVMError::Message(e.to_string()))?;

            self.builder
                .build_store(ptr, llvm_param)
                .map_err(|e| LLVMError::Message(e.to_string()))?;

            self.insert_variable(param.name.clone(), ptr, llvm_param.get_type());
        }

        for stmt in &function.body {
            self.lower_stmt(stmt)?;
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
                    return Err(LLVMError::Message(format!(
                        "function '{}' is missing a return statement",
                        function.name.as_str()
                    )));
                }

                None => {
                    self.builder
                        .build_return(None)
                        .map_err(|e| LLVMError::Message(e.to_string()))?;
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

    pub fn get_function(&self, target: String) -> Result<FunctionValue<'ctx>, LLVMError> {
        let id = target;

        if let Some(f) = self.functions.get(&id) {
            return Ok(*f);
        }
        Err(LLVMError::Message(format!("unknown function '{}'", id)))
    }
}
