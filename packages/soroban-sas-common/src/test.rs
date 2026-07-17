#![cfg(test)]

use crate::UID;

#[test]
fn test_uid_deterministic() {
    let env = soroban_sdk::Env::default();
    let uid1 = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let uid2 = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    assert_eq!(uid1, uid2);

    let uid3 = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    assert_ne!(uid1, uid3);
}

use crate::validation::validate_ttl;
use soroban_sdk::Env;

#[test]
fn test_validate_ttl() {
    let env = Env::default();

    // Valid cases
    assert!(validate_ttl(&env, 100, 200).is_ok());
    assert!(validate_ttl(&env, 100, 0).is_ok()); // no expiration

    // Invalid cases
    assert!(validate_ttl(&env, 200, 100).is_err());
    assert!(validate_ttl(&env, 100, 100).is_err()); // expired exactly at current time
}

use crate::merkle::MerkleRoot;
use soroban_sdk::BytesN;

#[test]
fn test_merkle_root_generation() {
    let env = Env::default();
    let root_bytes = BytesN::from_array(&env, &[0u8; 32]);
    let merkle_root = MerkleRoot(root_bytes.clone());

    assert_eq!(merkle_root.0, root_bytes);
}
