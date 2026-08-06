//! Ed25519 signing for delegated attestation/revocation calls.
//!
//! Message construction (what bytes actually get signed for a given
//! contract call) is the caller's responsibility — this module only wraps
//! `ed25519-dalek` for the two primitives every delegated write call needs:
//! deriving the public key that goes alongside a signature on-chain, and
//! producing the signature itself.

use ed25519_dalek::{Signer, SigningKey};

/// Derives the 32-byte ed25519 public key for `secret_seed`.
pub fn derive_public_key(secret_seed: &[u8; 32]) -> [u8; 32] {
    SigningKey::from_bytes(secret_seed)
        .verifying_key()
        .to_bytes()
}

/// Signs `message` with the ed25519 key derived from `secret_seed`.
pub fn generate_delegated_signature(secret_seed: &[u8; 32], message: &[u8]) -> [u8; 64] {
    SigningKey::from_bytes(secret_seed).sign(message).to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    const SEED: [u8; 32] = [7; 32];

    #[test]
    fn derives_the_matching_public_key() {
        let public_key = derive_public_key(&SEED);
        let expected = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
        assert_eq!(public_key, expected);
    }

    #[test]
    fn produces_a_signature_that_verifies() {
        let message = b"schema_uid || recipient";
        let signature_bytes = generate_delegated_signature(&SEED, message);

        let verifying_key = VerifyingKey::from_bytes(&derive_public_key(&SEED)).unwrap();
        let signature = Signature::from_bytes(&signature_bytes);
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn signature_does_not_verify_against_a_different_message() {
        let signature_bytes = generate_delegated_signature(&SEED, b"original payload");

        let verifying_key = VerifyingKey::from_bytes(&derive_public_key(&SEED)).unwrap();
        let signature = Signature::from_bytes(&signature_bytes);
        assert!(verifying_key
            .verify(b"tampered payload", &signature)
            .is_err());
    }

    #[test]
    fn different_seeds_produce_different_signatures() {
        let message = b"same message";
        let sig_a = generate_delegated_signature(&SEED, message);
        let sig_b = generate_delegated_signature(&[9; 32], message);
        assert_ne!(sig_a, sig_b);
    }
}
