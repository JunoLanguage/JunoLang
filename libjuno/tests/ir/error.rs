#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_message_variant() {
        let err = LLVMError::Message("hello".into());
        assert_eq!(format!("{}", err), "hello");
    }

    #[test]
    fn display_unknown_variable() {
        let err = LLVMError::UnknownVariable("x".into());
        let msg = format!("{}", err);
        assert!(msg.contains("x"));
    }
}
