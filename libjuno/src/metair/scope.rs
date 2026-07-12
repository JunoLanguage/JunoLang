use std::collections::HashMap;

use super::*;
impl<'a> MetaIRGen<'a> {
    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn insert_local(&mut self, id: SymbolId, ty: MetaType) {
        self.locals.last_mut().unwrap().insert(id, ty);
    }

    pub fn lookup_local(&self, id: SymbolId) -> MetaType {
        for scope in self.locals.iter().rev() {
            if let Some(ty) = scope.get(&id) {
                return ty.clone();
            }
        }

        panic!("unknown variable {}", id);
    }
    pub fn lookup_local_type(&self, id: SymbolId) -> MetaType {
        for scope in self.locals.iter().rev() {
            if let Some(ty) = scope.get(&id) {
                return ty.clone();
            }
        }
        panic!("unknown local variable '{}'", id);
    }
}
