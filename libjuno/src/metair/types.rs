//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ast::Type;
use crate::metair::generator::MetaIRGen;
use crate::metair::metair::*;

impl<'a> MetaIRGen<'a> {
    // =======================
    // Types
    // =======================

    pub(crate) fn lower_type(&mut self, ty: &Type) -> MetaType {
        match ty {
            Type::Named(name, span) => MetaType::Named(name.clone(), *span),

            Type::Pointer(inner, span) => {
                MetaType::Pointer(Box::new(self.lower_type(inner)), *span)
            }

            Type::Reference(inner, span) => {
                MetaType::Reference(Box::new(self.lower_type(inner)), *span)
            }

            Type::Array { elem, size, span } => MetaType::Array {
                span: *span,
                elem: Box::new(self.lower_type(elem)),
                size: *size,
            },

            Type::Generic {
                base,
                args: _,
                span,
            } => {
                // Generic arguments are currently erased.
                MetaType::Named(base.clone(), *span)
            }
        }
    }
}
