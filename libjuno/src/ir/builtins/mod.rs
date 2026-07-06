use crate::LLVMBackend;



pub mod writing;


impl<'ctx> LLVMBackend<'ctx> {
    pub fn declare_builtins(&mut self) {
        self.add_printf();
    }
}