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
        let parts: Vec<&str> = id.split('.').collect();

        let mut ty = self.lookup_local(parts[0].to_string());

        for field in &parts[1..] {
            let struct_name = match &ty {
                MetaType::Named(name, _) => name,
                _ => panic!("{field} is not a struct field"),
            };

            let strukt = self.structs.get(struct_name).expect("unknown struct");

            let index = self.struct_fields.get(struct_name).unwrap()[*field];

            ty = strukt.fields[index as usize].ty.clone();
        }

        ty
    }
}
