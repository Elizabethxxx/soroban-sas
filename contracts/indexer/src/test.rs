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
