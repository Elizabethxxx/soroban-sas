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
