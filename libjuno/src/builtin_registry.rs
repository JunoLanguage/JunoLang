use phf_macros::phf_map;
use crate::{ ast::Type, * };

pub struct Builtin {
    pub id: u32,
    pub declare: BuiltinEnum,
}

impl Builtin {
    fn new(id: u32, declare: BuiltinEnum) -> Self {
        Self { id, declare }
    }
}
pub enum BuiltinEnum {
    Function {
        param_types: Option<Vec<Type>>,
        return_type: &'static str,
    },
}




const REGISTRY: phf::Map<
    &'static str,
    Builtin
> = phf_map! {
    "printf" => Builtin {id: 0, declare: BuiltinEnum::Function { param_types: None, return_type: "void"  }},
};

pub fn is_builtin(name: &str) -> bool {
    REGISTRY.contains_key(name)
}

pub fn get_builtin_id(name: &str) -> u32 {
    ((1 as u32) << 16) | (REGISTRY.get(name).unwrap().id)
}

pub fn get_builtin(name: &str) -> Option<&Builtin> {
    REGISTRY.get(name)
}