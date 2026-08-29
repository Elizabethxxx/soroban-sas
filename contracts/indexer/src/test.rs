use super::*;
use soroban_sas_common::{INSTANCE_EXTEND_TO_LEDGERS, UID};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, testutils::Ledger, Env};

mod mock {
    use super::*;
    #[contract]
    pub struct MockSas;
    #[contractimpl]
    impl MockSas {
        pub fn sasv1(_env: Env) -> bool { true }
    }
}

#[test]
fn test_init_records_admin_and_sas_binding() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let admin = Address::generate(&env);
    let sas = env.register_contract(None, mock::MockSas);

    assert_eq!(client.get_admin(), None);
    client.init(&admin, &sas);

    assert_eq!(client.get_admin(), Some(admin.clone()));
    assert_eq!(client.get_sas(), Some(sas.clone()));
}

#[test]
fn test_init_twice_is_rejected() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let admin = Address::generate(&env);
    let sas = env.register_contract(None, mock::MockSas);
    client.init(&admin, &sas);

    let res = client.try_init(&admin, &sas);
    assert_eq!(
        res,
        Err(Ok(soroban_sas_common::SASError::AlreadyInitialized.into()))
    );
}

#[test]
fn test_index_single_attestation() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    client.index_attestation(&uid, &recipient, &schema_uid, &attester);
}

/*
#[test]
fn test_chunked_storage_limits() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[3u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    // Simulate exceeding a chunk limit
    for i in 0..150u8 {
        let mut bytes = [0u8; 32];
        bytes[0] = i;
        let uid = UID(soroban_sdk::BytesN::from_array(&env, &bytes));
        client.index_attestation(&uid, &recipient, &schema_uid, &attester);
    }
}
*/

#[test]
fn test_reverse_lookup() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[4u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    let uid1 = UID(soroban_sdk::BytesN::from_array(&env, &[10u8; 32]));
    let uid2 = UID(soroban_sdk::BytesN::from_array(&env, &[11u8; 32]));

    client.index_attestation(&uid1, &recipient, &schema_uid, &attester);
    client.index_attestation(&uid2, &recipient, &schema_uid, &attester);

    let recipient_uids = client.get_attestations_by_recipient(&recipient);
    assert_eq!(recipient_uids.len(), 2);

    let schema_uids = client.get_attestations_by_schema(&schema_uid);
    assert_eq!(schema_uids.len(), 2);
}

#[test]
fn test_attester_indexing_large_datasets() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[5u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    for i in 0..50u8 {
        let mut bytes = [0u8; 32];
        bytes[0] = i;
        let uid = UID(soroban_sdk::BytesN::from_array(&env, &bytes));
        client.index_attestation(&uid, &recipient, &schema_uid, &attester);
    }

    let attester_uids = client.get_attestations_by_attester(&attester);
    assert_eq!(attester_uids.len(), 50);
}

#[test]
fn test_cursor_pagination_large_datasets() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[6u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    for i in 0..101u8 {
        let mut bytes = [0u8; 32];
        bytes[0] = i;
        let uid = UID(soroban_sdk::BytesN::from_array(&env, &bytes));
        client.index_attestation(&uid, &recipient, &schema_uid, &attester);
    }

    let chunk0: soroban_sdk::Vec<UID> = env.as_contract(&indexer_id, || {
        env.storage()
            .persistent()
            .get(&(recipient.clone(), 0u32))
            .unwrap()
    });
    let chunk1: soroban_sdk::Vec<UID> = env.as_contract(&indexer_id, || {
        env.storage()
            .persistent()
            .get(&(recipient.clone(), 1u32))
            .unwrap()
    });

    assert_eq!(chunk0.len(), 100);
    assert_eq!(chunk1.len(), 1);

    let paginated = client.get_atts_by_recipient_paginated(&recipient, &0, &10);
    assert_eq!(paginated.len(), 10);
}

/// Once the instance TTL (holding INDEXER_ADMIN and SAS_CONTRACT) has
/// decayed close to expiry, ordinary write traffic (`index_attestation`)
/// must renew it. Exercised by jumping the ledger forward by nearly the
/// full renewal window twice in a row, indexing an attestation in between
/// each jump; if `index_attestation` did not renew the TTL, the second
/// call would panic on an archived instance.
#[test]
fn test_index_attestation_renews_decayed_instance_ttl() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let admin = Address::generate(&env);
    let sas = env.register_contract(None, mock::MockSas);
    client.init(&admin, &sas);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[7u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    env.ledger().with_mut(|li| {
        li.sequence_number += INSTANCE_EXTEND_TO_LEDGERS - 1000;
    });
    let uid1 = UID(soroban_sdk::BytesN::from_array(&env, &[8u8; 32]));
    client.index_attestation(&uid1, &recipient, &schema_uid, &attester);

    env.ledger().with_mut(|li| {
        li.sequence_number += INSTANCE_EXTEND_TO_LEDGERS - 1000;
    });
    let uid2 = UID(soroban_sdk::BytesN::from_array(&env, &[9u8; 32]));
    client.index_attestation(&uid2, &recipient, &schema_uid, &attester);

    // get_admin/get_sas read instance storage directly; if the instance
    // had expired, these would panic rather than returning stale data.
    assert_eq!(client.get_admin(), Some(admin));
    assert_eq!(client.get_sas(), Some(sas));
}
