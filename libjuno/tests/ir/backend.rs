#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{empty_program, make_backend};

    #[test]
    fn scope_push_pop_roundtrip() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        backend.push_scope();
        backend.pop_scope();
    }

    #[test]
    fn variable_lifecycle() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, ctx) = make_backend(prog);
        backend.push_scope();
        let int_ty = ctx.i32_type();
        let ptr = ctx.create_builder().build_alloca(int_ty, "v").unwrap();
        backend.insert_variable("v", ptr, int_ty.as_basic_type_enum());
        assert!(backend.get_variable("v").is_ok());
        backend.pop_scope();
    }
}
