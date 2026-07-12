use inkwell::types::BasicTypeEnum;
use inkwell::types::FunctionType;
use inkwell::types::StructType;
use inkwell::values::StructValue;

use std::collections::HashMap;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
};

use crate::metair::MetaProgram;

use super::LLVMError;
use crate::ir::scope::Scope;
use crate::metair::*;

pub struct LoopFrame<'ctx> {
    pub break_block: BasicBlock<'ctx>,
    pub continue_block: BasicBlock<'ctx>,
}

pub struct LLVMBackend<'ctx> {
    pub context: &'ctx Context,

    pub module: Module<'ctx>,

    pub builder: Builder<'ctx>,

    pub program: &'ctx MetaProgram,

    pub functions: HashMap<FunctionId, FunctionValue<'ctx>>,

    pub structs: HashMap<TypeId, StructType<'ctx>>,
    pub scopes: Vec<Scope<'ctx>>,

    pub current_function: Option<FunctionValue<'ctx>>,

    pub current_struct: Option<StructType<'ctx>>,

    pub loop_stack: Vec<LoopFrame<'ctx>>,

    pub builtins: HashMap<&'ctx str, FunctionValue<'ctx>>,
}

impl<'ctx> LLVMBackend<'ctx> {
    pub fn new(context: &'ctx Context, program: &'ctx MetaProgram, module_name: &str) -> Self {
        let mut s = Self {
            context,

            module: context.create_module(module_name),

            builder: context.create_builder(),

            program,

            functions: HashMap::new(),
            structs: HashMap::new(),
            builtins: HashMap::new(),
            scopes: Vec::new(),

            loop_stack: Vec::new(),

            current_function: None,
            current_struct: None,
        };
        s.declare_builtins();
        s
    }
    pub fn dump_ir(&self) {
        self.module.print_to_stderr();
    }

    pub fn compile(&mut self) -> Result<Module<'ctx>, LLVMError> {
        self.lower_program()?;
        self.module
            .verify()
            .map_err(|e| LLVMError::Message(e.to_string()))?;

        Ok(self.module.clone())
    }
}

impl<'ctx> LLVMBackend<'ctx> {
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn current_function(&self) -> FunctionValue<'ctx> {
        self.current_function.unwrap()
    }
    pub fn insert_variable(
        &mut self,
        id: SymbolId,
        ptr: PointerValue<'ctx>,
        ty: BasicTypeEnum<'ctx>,
    ) {
        self.scopes.last_mut().unwrap().insert(id, ptr, ty);
    }

    pub fn get_variable(
        &self,
        id: SymbolId,
    ) -> Result<&crate::ir::scope::Variable<'ctx>, LLVMError> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(id.clone()) {
                return Ok(var);
            }
        }

        Err(LLVMError::UnknownVariable(id))
    }
}
