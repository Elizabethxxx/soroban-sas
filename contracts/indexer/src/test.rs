use super::*;
use soroban_sas_common::UID;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Env, IntoVal};

mod mock {
    use super::*;
    #[contract]
    pub struct MockSas;
    #[contractimpl]
    impl MockSas {
        pub fn sasv1(_env: Env) -> bool { true }

        /// Mirrors the real SAS contract's cross-contract call into the
        /// indexer, so tests can prove the indexer accepts writes that
        /// originate from the bound SAS contract's own execution context.
        pub fn relay_index(
            env: Env,
            indexer: Address,
            uid: UID,
            recipient: Address,
            schema_uid: UID,
            attester: Address,
        ) {
            env.invoke_contract::<()>(
                &indexer,
                &Symbol::new(&env, "index_attestation"),
                soroban_sdk::vec![
                    &env,
                    uid.into_val(&env),
                    recipient.into_val(&env),
                    schema_uid.into_val(&env),
                    attester.into_val(&env),
                ],
            );
        }
    }

}

mod mock_attacker {
    use super::*;
    #[contract]
    pub struct MockAttacker;
    #[contractimpl]
    impl MockAttacker {
        /// A contract with no special relationship to the indexer, used to
        /// prove an arbitrary caller cannot poison the index.
        pub fn relay_index(
            env: Env,
            indexer: Address,
            uid: UID,
            recipient: Address,
            schema_uid: UID,
            attester: Address,
        ) {
            env.invoke_contract::<()>(
                &indexer,
                &Symbol::new(&env, "index_attestation"),
                soroban_sdk::vec![
                    &env,
                    uid.into_val(&env),
                    recipient.into_val(&env),
                    schema_uid.into_val(&env),
                    attester.into_val(&env),
                ],
            );
        }
    }
}

/// Registers an indexer bound to a freshly registered `mock::MockSas`, and
/// returns `(indexer_id, indexer_client, sas_id)`.
fn setup_indexed(env: &Env) -> (Address, IndexerClient<'_>, Address) {
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(env, &indexer_id);
    let admin = Address::generate(env);
    let sas = env.register_contract(None, mock::MockSas);
    client.init(&admin, &sas);
    (indexer_id, client, sas)
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
    let (indexer_id, _client, sas) = setup_indexed(&env);
    let sas_client = mock::MockSasClient::new(&env, &sas);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    sas_client.relay_index(&indexer_id, &uid, &recipient, &schema_uid, &attester);
}

/// A direct call from a bare test invocation (no authorizing contract in
/// the call stack) must be rejected: the indexer's trust boundary is the
/// bound SAS contract, not "whoever calls the entry point."
#[test]
fn test_index_attestation_rejects_direct_caller() {
    let env = Env::default();
    let (_indexer_id, client, _sas) = setup_indexed(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    let res = client.try_index_attestation(&uid, &recipient, &schema_uid, &attester);
    assert!(res.is_err());
}

/// An arbitrary contract that is not the bound SAS contract cannot inject
/// UIDs into any of the three lookup tables.
#[test]
fn test_index_attestation_rejects_unrelated_contract() {
    let env = Env::default();
    let (indexer_id, _client, _sas) = setup_indexed(&env);
    let attacker = env.register_contract(None, mock_attacker::MockAttacker);
    let attacker_client = mock_attacker::MockAttackerClient::new(&env, &attacker);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[9u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[9u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    let res = attacker_client.try_relay_index(
        &indexer_id,
        &uid,
        &recipient,
        &schema_uid,
        &attester,
    );
    assert!(res.is_err());

    let indexer_client = IndexerClient::new(&env, &indexer_id);
    assert_eq!(
        indexer_client.get_attestations_by_recipient(&recipient).len(),
        0
    );
    assert_eq!(
        indexer_client.get_attestations_by_schema(&schema_uid).len(),
        0
    );
    assert_eq!(
        indexer_client.get_attestations_by_attester(&attester).len(),
        0
    );
}

/// Before `init`, there is no trusted SAS address to authorize against, so
/// writes must be rejected outright rather than silently accepted.
#[test]
fn test_index_attestation_rejects_before_init() {
    let env = Env::default();
    let indexer_id = env.register_contract(None, Indexer);
    let client = IndexerClient::new(&env, &indexer_id);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    let res = client.try_index_attestation(&uid, &recipient, &schema_uid, &attester);
    assert_eq!(
        res,
        Err(Ok(soroban_sas_common::SASError::Unauthorized.into()))
    );
}

#[test]
fn test_reverse_lookup() {
    let env = Env::default();
    let (indexer_id, client, sas) = setup_indexed(&env);
    let sas_client = mock::MockSasClient::new(&env, &sas);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[4u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    let uid1 = UID(soroban_sdk::BytesN::from_array(&env, &[10u8; 32]));
    let uid2 = UID(soroban_sdk::BytesN::from_array(&env, &[11u8; 32]));

    sas_client.relay_index(&indexer_id, &uid1, &recipient, &schema_uid, &attester);
    sas_client.relay_index(&indexer_id, &uid2, &recipient, &schema_uid, &attester);

    let recipient_uids = client.get_attestations_by_recipient(&recipient);
    assert_eq!(recipient_uids.len(), 2);

    let schema_uids = client.get_attestations_by_schema(&schema_uid);
    assert_eq!(schema_uids.len(), 2);
}

#[test]
fn test_attester_indexing_large_datasets() {
    let env = Env::default();
    let (indexer_id, client, sas) = setup_indexed(&env);
    let sas_client = mock::MockSasClient::new(&env, &sas);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[5u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    for i in 0..50u8 {
        let mut bytes = [0u8; 32];
        bytes[0] = i;
        let uid = UID(soroban_sdk::BytesN::from_array(&env, &bytes));
        sas_client.relay_index(&indexer_id, &uid, &recipient, &schema_uid, &attester);
    }

    let attester_uids = client.get_attestations_by_attester(&attester);
    assert_eq!(attester_uids.len(), 50);
}

#[test]
fn test_cursor_pagination_large_datasets() {
    let env = Env::default();
    let (indexer_id, client, sas) = setup_indexed(&env);
    let sas_client = mock::MockSasClient::new(&env, &sas);

    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[6u8; 32]));
    let recipient = Address::generate(&env);
    let attester = Address::generate(&env);

    for i in 0..101u8 {
        let mut bytes = [0u8; 32];
        bytes[0] = i;
        let uid = UID(soroban_sdk::BytesN::from_array(&env, &bytes));
        sas_client.relay_index(&indexer_id, &uid, &recipient, &schema_uid, &attester);
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
