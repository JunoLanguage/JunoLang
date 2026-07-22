#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;
    use libjuno::ir::scope::Scope;

    #[test]
    fn scope_insert_and_get() {
        let context = Context::create();
        let mut scope = Scope::new();
        let int_ty = context.i32_type();
        let ptr = context.create_builder().build_alloca(int_ty, "x").unwrap();

        scope.insert("x", ptr, int_ty.as_basic_type_enum());
        let v = scope.get("x").expect("should find x");
        // Pointer identity check (value-level equality is enough for smoke testing)
        assert!(v.ptr.as_value_ref() == ptr.as_value_ref());
    }

    #[test]
    fn scope_missing_key_returns_none() {
        let scope = Scope::<Context>::new(); // adjust lifetime as needed
        assert!(scope.get("missing").is_none());
    }
}
