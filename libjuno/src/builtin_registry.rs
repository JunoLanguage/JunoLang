use crate::SymbolId;
use crate::*;
use phf_macros::phf_map;

pub struct Builtin {
    pub declare: BuiltinEnum,
}

pub enum BuiltinEnum {
    Function {
        param_types: Option<&'static [&'static str]>,
        return_type: &'static str,
    },
}

pub const REGISTRY: phf::Map<&'static str, Builtin> = phf_map! {
    "printf" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: None,
            return_type: "void",
        },
    },

    "puts" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8"]),
            return_type: "void",
        },
    },

    "putchar" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["u8"]),
            return_type: "u8",
        },
    },

    "getchar" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&[]),
            return_type: "i32",
        },
    },
    "scanf" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: None,
            return_type: "u8",
        },
    },


};
pub fn is_builtin(name: &str) -> bool {
    REGISTRY.contains_key(name)
}

pub fn get_builtin_id(name: &str) -> SymbolId {
    name.to_string()
}

pub fn get_builtin(name: &str) -> Option<&Builtin> {
    REGISTRY.get(name)
}
