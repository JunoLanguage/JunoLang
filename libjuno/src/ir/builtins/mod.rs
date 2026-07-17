use inkwell::module::Linkage;
use inkwell::{types::FunctionType, values::FunctionValue};

use crate::{LLVMBackend, get_builtin_id};

pub mod character;
pub mod memory;
pub mod memory_ops;
pub mod reading;
pub mod strings;
pub mod writing;

impl<'ctx> LLVMBackend<'ctx> {
    pub fn declare_builtins(&mut self) {
        // writing
        self.add_printf();
        self.add_puts();
        self.add_putchar();

        // reading
        self.add_getchar();
        self.add_scanf();

        // strings
        self.add_strlen();
        self.add_strcmp();
        self.add_strncmp();
        self.add_strcpy();
        self.add_strncpy();
        self.add_strcat();
        self.add_strncat();
        self.add_strchr();
        self.add_strrchr();
        self.add_strstr();
        self.add_strtok();

        // memory
        self.add_malloc();
        self.add_calloc();
        self.add_realloc();
        self.add_free();

        // memory operations
        self.add_memcpy();
        self.add_memmove();
        self.add_memset();
        self.add_memcmp();
        self.add_memchr();

        // character
        self.add_isalnum();
        self.add_isalpha();
        self.add_isdigit();
        self.add_islower();
        self.add_isupper();
        self.add_isspace();
        self.add_isxdigit();
        self.add_tolower();
        self.add_toupper();
    }
}

impl<'ctx> LLVMBackend<'ctx> {
    pub(in crate::ir::builtins) fn declare_builtin(
        &mut self,
        name: &'static str,
        ty: FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        let function = self.module.add_function(name, ty, Some(Linkage::External));

        self.add_function(get_builtin_id(name), &function)
            .expect("builtin already registered");

        self.builtins.insert(name, function);

        function
    }
}
