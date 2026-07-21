use std::collections::HashMap;

use inkwell::{types::BasicTypeEnum, values::PointerValue};

use crate::SymbolId;

pub struct Variable<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub ty: BasicTypeEnum<'ctx>,
}
#[derive(Default)]
pub struct Scope<'ctx> {
    variables: HashMap<SymbolId, Variable<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn insert(&mut self, id: &str, ptr: PointerValue<'ctx>, ty: BasicTypeEnum<'ctx>) {
        self.variables.insert(id.to_string(), Variable { ptr, ty });
    }

    pub fn get(&self, id: &str) -> Option<&Variable<'ctx>> {
        self.variables.get(id)
    }
}