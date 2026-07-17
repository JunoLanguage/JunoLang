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
            return_type: "i32",
        },
    },

    "puts" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8"]),
            return_type: "i32",
        },
    },

    "putchar" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
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
            return_type: "i32",
        },
    },
        "strlen" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8"]),
            return_type: "u64",
        },
    },

    "strcmp" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8"]),
            return_type: "i32",
        },
    },

    "strncmp" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8", "u64"]),
            return_type: "i32",
        },
    },

    "strcpy" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8"]),
            return_type: "&u8",
        },
    },

    "strncpy" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8", "u64"]),
            return_type: "&u8",
        },
    },

    "strcat" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8"]),
            return_type: "&u8",
        },
    },

    "strncat" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8", "u64"]),
            return_type: "&u8",
        },
    },

    "strchr" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "i32"]),
            return_type: "&u8",
        },
    },

    "strrchr" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "i32"]),
            return_type: "&u8",
        },
    },

    "strstr" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8"]),
            return_type: "&u8",
        },
    },

    "strtok" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&u8", "&u8"]),
            return_type: "&u8",
        },
    },

    "malloc" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["u64"]),
            return_type: "&u8",
        },
    },

    "calloc" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["u64", "u64"]),
            return_type: "&void",
        },
    },

    "realloc" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void", "u64"]),
            return_type: "&void",
        },
    },

    "free" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void"]),
            return_type: "void",
        },
    },
    "memcpy" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void", "&void", "u64"]),
            return_type: "&void",
        },
    },

    "memmove" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void", "&void", "u64"]),
            return_type: "&void",
        },
    },

    "memset" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void", "i32", "u64"]),
            return_type: "&void",
        },
    },

    "memcmp" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void", "&void", "u64"]),
            return_type: "i32",
        },
    },

    "memchr" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["&void", "i32", "u64"]),
            return_type: "&void",
        },
    },

    "isalnum" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "isalpha" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "isdigit" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "islower" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "isupper" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "isspace" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "isxdigit" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "tolower" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
        },
    },

    "toupper" => Builtin {
        declare: BuiltinEnum::Function {
            param_types: Some(&["i32"]),
            return_type: "i32",
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
