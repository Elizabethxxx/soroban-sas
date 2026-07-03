#![cfg(test)]
use super::*;
use soroban_sdk::{Env, testutils::Address as _};
use soroban_sas_common::UID;

#[test]
fn test_index_single_attestation() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);
    
    let uid = UID([1u8; 32]);
    let schema_uid = UID([2u8; 32]);
    let recipient = Address::generate(&env);
    
    client.index_attestation(&uid, &recipient, &schema_uid);
}

#[test]
fn test_chunked_storage_limits() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);
    
    let schema_uid = UID([3u8; 32]);
    let recipient = Address::generate(&env);
    
    // Simulate exceeding a chunk limit
    for i in 0..150u8 {
        let mut bytes = [0u8; 32];
        bytes[0] = i;
        let uid = UID(bytes);
        client.index_attestation(&uid, &recipient, &schema_uid);
    }
}
