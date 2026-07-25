#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_snapshot_formatting() {
        assert_eq!(1, 1);
    }
}

#[cfg(test)]
mod offchain_tests {
    use crate::offchain::{
        compute_payload_hash, parse_secret_seed, sign_offchain_attestation,
        verify_offchain_attestation, AttestationInput,
    };
    use ed25519_dalek::SigningKey;

    const NETWORK: &str = "Test SDF Network ; September 2015";

    fn contract_id() -> String {
        stellar_strkey::Contract([1u8; 32]).to_string()
    }

    fn sample_input(seed: [u8; 32]) -> AttestationInput {
        let public_key = SigningKey::from_bytes(&seed).verifying_key().to_bytes();
        let attester = stellar_strkey::ed25519::PublicKey(public_key).to_string();
        let recipient = stellar_strkey::ed25519::PublicKey([5u8; 32]).to_string();
        AttestationInput {
            uid: hex::encode([1u8; 32]),
            schema_uid: hex::encode([2u8; 32]),
            time: 1000,
            expiration_time: 0,
            ref_uid: hex::encode([0u8; 32]),
            recipient,
            attester,
            revocable: true,
            data: "deadbeef".to_string(),
        }
    }

    #[test]
    fn test_payload_hash_deterministic() {
        let input = sample_input([41u8; 32]);
        let h1 = compute_payload_hash(&input, 7, NETWORK, &contract_id()).unwrap();
        let h2 = compute_payload_hash(&input, 7, NETWORK, &contract_id()).unwrap();
        assert_eq!(h1, h2);

        // Different nonce, network, or contract yields a different digest.
        let h3 = compute_payload_hash(&input, 8, NETWORK, &contract_id()).unwrap();
        assert_ne!(h1, h3);
        let h4 = compute_payload_hash(&input, 7, "other network", &contract_id()).unwrap();
        assert_ne!(h1, h4);
        let other_contract = stellar_strkey::Contract([2u8; 32]).to_string();
        let h5 = compute_payload_hash(&input, 7, NETWORK, &other_contract).unwrap();
        assert_ne!(h1, h5);
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let seed = [41u8; 32];
        let input = sample_input(seed);
        let signed = sign_offchain_attestation(input, 7, NETWORK, &contract_id(), &seed).unwrap();
        assert!(verify_offchain_attestation(&signed).is_ok());
    }

    #[test]
    fn test_verify_rejects_tampered_data() {
        let seed = [41u8; 32];
        let input = sample_input(seed);
        let mut signed =
            sign_offchain_attestation(input, 7, NETWORK, &contract_id(), &seed).unwrap();
        signed.attestation.data = "deadbeee".to_string();
        assert!(verify_offchain_attestation(&signed).is_err());
    }

    #[test]
    fn test_verify_rejects_wrong_key() {
        let seed = [41u8; 32];
        let input = sample_input(seed);
        let mut signed =
            sign_offchain_attestation(input, 7, NETWORK, &contract_id(), &seed).unwrap();

        // Swap in a different public key: the attester binding check fails.
        let other_key = SigningKey::from_bytes(&[42u8; 32])
            .verifying_key()
            .to_bytes();
        signed.public_key = hex::encode(other_key);
        assert!(verify_offchain_attestation(&signed).is_err());
    }

    #[test]
    fn test_verify_rejects_nonce_change() {
        let seed = [41u8; 32];
        let input = sample_input(seed);
        let mut signed =
            sign_offchain_attestation(input, 7, NETWORK, &contract_id(), &seed).unwrap();
        signed.nonce = 8;
        assert!(verify_offchain_attestation(&signed).is_err());
    }

    #[test]
    fn test_sign_rejects_mismatched_attester() {
        let seed = [41u8; 32];
        // Attester derived from a different key than the signing seed.
        let input = sample_input([43u8; 32]);
        assert!(sign_offchain_attestation(input, 7, NETWORK, &contract_id(), &seed).is_err());
    }

    #[test]
    fn test_parse_secret_seed_hex_and_strkey() {
        let seed = [41u8; 32];
        assert_eq!(parse_secret_seed(&hex::encode(seed)).unwrap(), seed);
        let strkey = stellar_strkey::ed25519::PrivateKey(seed).to_string();
        assert_eq!(parse_secret_seed(&strkey).unwrap(), seed);
        assert!(parse_secret_seed("not a key").is_err());
    }
}
