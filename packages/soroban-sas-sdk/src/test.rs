#[cfg(test)]
mod tests {
    #[test]
    fn test_signature_generation() {
        assert_eq!(crate::signature::generate_delegated_signature().len(), 64);
    }
}

#[test]
fn test_rpc_mock_parsing() {
}
