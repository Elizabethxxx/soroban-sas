use crate::{SchemaRegistry, SchemaRegistryClient};
use soroban_sas_common::{SchemaRegisteredEvent, INSTANCE_EXTEND_TO_LEDGERS};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger};
use soroban_sdk::{symbol_short, Address, Env, IntoVal, String};

#[test]
fn test_register_schema() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    env.mock_all_auths();
    let uid = client.register(&owner, &schema_str, &resolver, &true);
    let record = client.get_schema(&uid).unwrap();

    assert_eq!(record.schema, schema_str);
    assert!(record.revocable);
    assert_eq!(record.resolver, resolver);
}

#[test]
fn test_register_rejects_malformed_schema_strings() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let resolver = Address::generate(&env);

    env.mock_all_auths();

    for schema in ["!!!", " ", "12345"] {
        let schema = String::from_str(&env, schema);
        let res = client.try_register(&owner, &schema, &resolver, &true);
        assert_eq!(res, Err(Ok(soroban_sas_common::SASError::InvalidSchema.into())));
    }
}

#[test]
fn test_register_emits_schema_registered_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    env.mock_all_auths();
    let uid = client.register(&owner, &schema_str, &resolver, &true);

    let expected = SchemaRegisteredEvent {
        schema_uid: uid.clone(),
        owner: owner.clone(),
    };
    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("REGISTER"), uid.clone()).into_val(&env),
                expected.into_val(&env),
            )
        ]
    );
}

/*
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_duplicate_schema() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    // First registration succeeds
    client.register(&schema_str, &resolver, &true);

    // Second registration with exactly the same parameters should panic
    // (SASError::SchemaAlreadyExists is #2 assuming it's the second variant)
    client.register(&schema_str, &resolver, &true);
}
*/

/*
#[test]
fn test_upgrade() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    // Simulate upgrade call (we mock the wasm hash)
    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);

    // In tests, environment requires mock auth setup for `admin.require_auth()`
    env.mock_all_auths();

    client.upgrade(&new_wasm_hash);
}
*/

#[test]
fn test_fee_and_treasury() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(&admin);

    env.mock_all_auths();
    client.set_fee(&1000);
    client.set_treasury(&treasury);
    client.withdraw_fees(&500);
}

#[test]
fn test_deprecate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin);
    let uid = client.register(&owner, &schema_str, &resolver, &true);

    // Check it's active
    assert!(client.get_schema(&uid).is_some());

    // Deprecate
    client.deprecate(&uid, &owner);

    // Check it's no longer active
    assert!(client.get_schema(&uid).is_none());
}

#[test]
fn test_deprecate_by_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin);
    let uid = client.register(&owner, &schema_str, &resolver, &true);
    client.deprecate(&uid, &admin);

    assert!(client.get_schema(&uid).is_none());
}

#[test]
fn test_deprecate_rejects_unrelated_authorizer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let unrelated = Address::generate(&env);
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin);
    let uid = client.register(&owner, &schema_str, &resolver, &true);

    let res = client.try_deprecate(&uid, &unrelated);
    assert_eq!(
        res,
        Err(Ok(soroban_sas_common::SASError::Unauthorized.into()))
    );
}

#[test]
fn test_validate_schema() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin);
    let uid = client.register(&owner, &schema_str, &resolver, &true);

    assert!(client.validate_schema(&uid));

    client.deprecate(&uid, &owner);
    assert!(!client.validate_schema(&uid));
}

#[test]
fn test_init_twice_is_rejected() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let res = client.try_init(&admin);
    assert_eq!(
        res,
        Err(Ok(soroban_sas_common::SASError::AlreadyInitialized.into()))
    );
}

/// After the ledger has advanced far past deployment, `init`'s instance-TTL
/// extension must still be in effect: reading configuration through any
/// admin-gated entry point (here, a second `init`, which reads
/// REGISTRY_ADMIN before rejecting the call) must not panic on an expired
/// instance. Before this extension existed, an instance created this long
/// ago and never renewed would already be archived and unreadable.
#[test]
fn test_instance_configuration_survives_long_after_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    env.ledger().with_mut(|li| {
        li.sequence_number += INSTANCE_EXTEND_TO_LEDGERS - 1000;
    });

    let res = client.try_init(&admin);
    assert_eq!(
        res,
        Err(Ok(soroban_sas_common::SASError::AlreadyInitialized.into()))
    );
}

/// Ordinary public traffic (here, `register`, which requires no special
/// admin access) must also renew the instance TTL, not just admin-only
/// entry points — so a schema registry that only ever receives
/// registrations, and no admin calls, still keeps its own configuration
/// alive. Exercised by advancing the ledger twice in a row by nearly the
/// full renewal window and registering in between each jump; if `register`
/// did not renew the TTL, the second `register` call would panic on an
/// archived instance.
#[test]
fn test_ordinary_traffic_renews_decayed_instance_ttl() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let owner = Address::generate(&env);
    let resolver = Address::generate(&env);
    env.mock_all_auths();

    env.ledger().with_mut(|li| {
        li.sequence_number += INSTANCE_EXTEND_TO_LEDGERS - 1000;
    });
    client.register(
        &owner,
        &String::from_str(&env, "schema one"),
        &resolver,
        &true,
    );

    env.ledger().with_mut(|li| {
        li.sequence_number += INSTANCE_EXTEND_TO_LEDGERS - 1000;
    });
    client.register(
        &owner,
        &String::from_str(&env, "schema two"),
        &resolver,
        &true,
    );
}
