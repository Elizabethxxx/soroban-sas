#[cfg(test)]
mod tests {
    #[test]
    fn test_signature_generation() {
        let seed = [1u8; 32];
        let signature = crate::signature::generate_delegated_signature(&seed, b"message");
        assert_eq!(signature.len(), 64);
        assert_ne!(signature, [0u8; 64]);
    }
}

#[test]
fn test_rpc_mock_parsing() {}
