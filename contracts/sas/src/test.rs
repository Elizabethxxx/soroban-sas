#![cfg(test)]

use crate::{SAS, SASClient};
use soroban_sdk::{contract, contractimpl, Env, Address, Bytes};
use soroban_sdk::testutils::Address as _;
use soroban_sas_common::{Attestation, UID};

#[contract]
pub struct MockRegistry;

#[contractimpl]
impl MockRegistry {
    pub fn validate_schema(_env: Env, _uid: UID) -> bool {
        true
    }
    pub fn get_schema(env: Env, uid: UID) -> Option<soroban_sas_common::SchemaRecord> {
        Some(soroban_sas_common::SchemaRecord {
            uid: uid.clone(),
            resolver: Address::generate(&env),
            revocable: true,
            schema: soroban_sdk::String::from_str(&env, "bool like"),
        })
    }
}

#[contract]
pub struct MockRejectRegistry;

#[contractimpl]
impl MockRejectRegistry {
    pub fn validate_schema(_env: Env, _uid: UID) -> bool {
        false
    }
    pub fn get_schema(_env: Env, _uid: UID) -> Option<soroban_sas_common::SchemaRecord> {
        None
    }
}

#[contract]
pub struct MockResolver;

#[contractimpl]
impl MockResolver {
    pub fn on_attest(_env: Env, _attestation: Attestation) {
        // Mock execution
    }
}

#[test]
fn test_happy_path_attestation() {
    let env = Env::default();
    
    // Deploy Mock Registry
    let registry_id = env.register_contract(None, MockRegistry);
    
    // Deploy SAS
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid = UID([1u8; 32]);
    let schema_uid = UID([2u8; 32]);
    let ref_uid = UID([0u8; 32]);
    
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid,
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid,
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    
    let result_uid = sas_client.attest(&attestation);
    assert_eq!(result_uid, uid);
}

#[test]
#[should_panic]
fn test_auth_failure_missing_signature() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid = UID([1u8; 32]);
    let schema_uid = UID([2u8; 32]);
    let ref_uid = UID([0u8; 32]);
    
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid,
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid,
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    // Attempting to attest without mock_all_auths or explicitly providing signatures should panic
    sas_client.attest(&attestation);
}

#[test]
#[should_panic(expected = "Invalid schema")]
fn test_schema_validation_rejection() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRejectRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid = UID([1u8; 32]);
    let schema_uid = UID([2u8; 32]);
    let ref_uid = UID([0u8; 32]);
    
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid,
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid,
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    sas_client.attest(&attestation);
}

#[test]
fn test_revocation_success() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid = UID([1u8; 32]);
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID([2u8; 32]),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID([0u8; 32]),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    sas_client.attest(&attestation);
    
    // Revoke
    sas_client.revoke(&uid);
}

#[test]
#[should_panic(expected = "Attestation is not revocable")]
fn test_revocation_failure() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid = UID([1u8; 32]);
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID([2u8; 32]),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID([0u8; 32]),
        recipient,
        attester: attester.clone(),
        revocable: false, // NOT REVOCABLE
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    sas_client.attest(&attestation);
    
    // Should panic
    sas_client.revoke(&uid);
}

#[test]
fn test_batch_operations() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid1 = UID([3u8; 32]);
    let uid2 = UID([4u8; 32]);
    
    let att1 = Attestation {
        uid: uid1.clone(),
        schema_uid: UID([2u8; 32]),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID([0u8; 32]),
        recipient: recipient.clone(),
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    let att2 = Attestation {
        uid: uid2.clone(),
        schema_uid: UID([2u8; 32]),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID([0u8; 32]),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    let batch = soroban_sdk::vec![&env, att1, att2];
    
    let result = sas_client.multi_attest(&batch);
    assert_eq!(result.len(), 2);
    
    let revoke_batch = soroban_sdk::vec![&env, uid1.clone(), uid2.clone()];
    sas_client.multi_revoke(&revoke_batch);
}

#[test]
fn test_resolver_callback() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRegistry);
    let resolver_id = env.register_contract(None, MockResolver);
    
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    let uid = UID([5u8; 32]);
    
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID([2u8; 32]),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID([0u8; 32]),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    sas_client.attest(&attestation);
    // Verifies it doesn't panic on try_invoke_contract
}

#[test]
fn test_attest_with_value() {
    let env = Env::default();
    
    let registry_id = env.register_contract(None, MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);
    
    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);
    
    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);
    
    let uid = UID([7u8; 32]);
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID([2u8; 32]),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID([0u8; 32]),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };
    
    env.mock_all_auths();
    sas_client.attest_with_value(&attestation, &token, &500);
}
