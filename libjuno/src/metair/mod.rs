//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod coercion;
mod expr;
mod generator;
mod intern;
mod lookup;
mod metair;
mod ops;
mod program;
mod stmt;
mod types;
pub use generator::MetaIRGen;
pub use metair::*;
//pub use metairgen::MetaIRGen;
