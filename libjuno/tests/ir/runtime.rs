#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::utils::{empty_program, make_backend};

    #[test]
    fn declare_runtime_is_no_op() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        backend.declare_runtime(); // should not panic
    }
}
