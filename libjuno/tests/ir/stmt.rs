#[cfg(test)]
mod tests {
    use libjuno::MetaStmt;

    use super::*;
    use crate::ir::utils::{dummy_span, empty_program, make_backend};

    #[test]
    fn break_outside_loop_errors() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let stmt = MetaStmt::Break(dummy_span());
        assert!(backend.lower_stmt(&stmt, &dummy_span()).is_err());
    }

    #[test]
    fn continue_outside_loop_errors() {
        let prog = Box::leak(Box::new(empty_program()));
        let (mut backend, _ctx) = make_backend(prog);
        let stmt = MetaStmt::Continue(dummy_span());
        assert!(backend.lower_stmt(&stmt, &dummy_span()).is_err());
    }

    #[test]
    fn loop_lower_creates_blocks() {
        let prog = Box::leak(Box::new(empty_program()));
        let mut backend = make_backend(prog).0;
        // Set up a dummy meta function so loop lowering can use current_function()
        let dummy = Box::leak(Box::new(crate::ir::utils::dummy_meta_function()));
        backend.current_meta_function = Some(dummy);

        // We just need a loop body that terminates quickly
        let stmt = MetaStmt::Loop {
            span: dummy_span(),
            body: vec![MetaStmt::Break(dummy_span())],
        };

        // Break inside loop is fine; we only check it doesn't panic.
        assert!(backend.lower_stmt(&stmt, &dummy_span()).is_ok());
    }
}
