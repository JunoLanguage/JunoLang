//This Source Code Form is subject to the terms of the Mozilla Public
//License, v. 2.0. If a copy of the MPL was not distributed with this
//file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{Block, JunoSpan, Type};

pub type FilePath = String;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Program {
    pub span: JunoSpan,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function, JunoSpan),
    Import(Import, JunoSpan),
    Struct(StructDef, JunoSpan),
    Declaration(Declaration, JunoSpan),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Import {
    pub span: JunoSpan,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub span: JunoSpan,
    pub name: String,
    pub raw_name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub span: JunoSpan,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub span: JunoSpan,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub span: JunoSpan,
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub span: JunoSpan,
    pub name: String,
    pub ty: Type,
}
