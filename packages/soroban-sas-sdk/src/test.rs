#[cfg(test)]
mod tests {
    #[test]
    fn test_signature_generation() {
        assert_eq!(crate::signature::generate_delegated_signature().len(), 64);
    }
}
