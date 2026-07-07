use phf_macros::phf_map;
use crate::{ ast::Type, * };

pub struct Builtin {
    pub id: u32,
    pub declare: BuiltinEnum,
}

pub enum BuiltinEnum {
    Function {
        param_types: Option<&'static [&'static str]>,
        return_type: &'static str,
    },
}




const REGISTRY: phf::Map<&'static str, Builtin> = phf_map! {
    "printf" => Builtin {
        id: 0,
        declare: BuiltinEnum::Function {
            param_types: None,
            return_type: "void",
        },
    },

    "puts" => Builtin {
        id: 1,
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8"]),
            return_type: "void",
        },
    },

    "putchar" => Builtin {
        id: 2,
        declare: BuiltinEnum::Function {
            param_types: Some(&["u8"]),
            return_type: "u8",
        },
    },

    "getchar" => Builtin {
        id: 3,
        declare: BuiltinEnum::Function {
            param_types: Some(&[]),
            return_type: "i32",
        },
    },
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