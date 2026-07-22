//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;

impl<'a> MetaIRGen<'a> {
    // =======================
    // Interning
    // =======================

    pub(in crate::metair) fn intern_symbol(&mut self, s: &str) -> SymbolId {
        if self.symbol_set.insert(s.to_string()) {
            self.symbol_list.push(s.to_string());
        }

        s.to_string()
    }

    pub(crate) fn intern_struct_field(&mut self, struct_id: SymbolId, field_name: &str) -> u32 {
        if let Some(fields) = self.struct_fields.get(&struct_id) {
            if let Some(id) = fields.get(field_name) {
                return *id;
            }
        } else {
            self.struct_fields.insert(struct_id.clone(), HashMap::new());
        }

        let id = self.next_struct_field;
        self.next_struct_field += 1;

        self.struct_fields
            .get_mut(&struct_id)
            .unwrap()
            .insert(field_name.to_string(), id);

        id
    }

    pub(crate) fn intern_string(&mut self, s: &str) -> StringId {
        if let Some(id) = self.strings.get(s) {
            return *id;
        }

        let id = self.next_string;
        self.next_string += 1;

        self.strings.insert(s.to_string(), id);
        self.string_list.push(s.to_string());

        id
    }
}
