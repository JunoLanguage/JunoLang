use std::collections::HashMap;

use inkwell::{
    types::BasicTypeEnum,
    values::PointerValue,
};

pub struct Variable<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub ty: BasicTypeEnum<'ctx>,
}

pub struct Scope<'ctx> {
    variables: HashMap<u32, Variable<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        id: u32,
        ptr: PointerValue<'ctx>,
        ty: BasicTypeEnum<'ctx>,
    ) {
        self.variables.insert(
            id,
            Variable {
                ptr,
                ty,
            },
        );
    }

    pub fn get(
        &self,
        id: u32,
    ) -> Option<&Variable<'ctx>> {
        self.variables.get(&id)
    }
}