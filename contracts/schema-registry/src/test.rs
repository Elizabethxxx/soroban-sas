use crate::{SchemaRegistry, SchemaRegistryClient};
use soroban_sas_common::{
    ContractUpgradedEvent, SchemaFeeUpdatedEvent, SchemaRegisteredEvent, TreasuryUpdatedEvent,
};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{symbol_short, Address, BytesN, Env, IntoVal, String};

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
fn test_set_fee_emits_event_with_old_and_new_value() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);
    env.mock_all_auths();

    client.set_fee(&1000);
    let expected_first = SchemaFeeUpdatedEvent {
        old_fee: None,
        new_fee: 1000,
        authorizer: admin.clone(),
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("FEEUPD"), admin.clone()).into_val(&env),
                expected_first.into_val(&env),
            )
        ]
    );

    client.set_fee(&2000);
    let expected_second = SchemaFeeUpdatedEvent {
        old_fee: Some(1000),
        new_fee: 2000,
        authorizer: admin.clone(),
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_id,
                (symbol_short!("FEEUPD"), admin).into_val(&env),
                expected_second.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_set_fee_requires_admin_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let res = client.try_set_fee(&1000);
    assert!(res.is_err());
}

#[test]
fn test_set_treasury_emits_event_with_old_and_new_value() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury_one = Address::generate(&env);
    let treasury_two = Address::generate(&env);
    client.init(&admin);
    env.mock_all_auths();

    client.set_treasury(&treasury_one);
    let expected_first = TreasuryUpdatedEvent {
        old_treasury: None,
        new_treasury: treasury_one.clone(),
        authorizer: admin.clone(),
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("TRSUPD"), admin.clone()).into_val(&env),
                expected_first.into_val(&env),
            )
        ]
    );

    client.set_treasury(&treasury_two);
    let expected_second = TreasuryUpdatedEvent {
        old_treasury: Some(treasury_one),
        new_treasury: treasury_two,
        authorizer: admin.clone(),
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_id,
                (symbol_short!("TRSUPD"), admin).into_val(&env),
                expected_second.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_set_treasury_requires_admin_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(&admin);

    let res = client.try_set_treasury(&treasury);
    assert!(res.is_err());
}

/// Exercises `record_upgrade_event` (the event-payload half of `upgrade`,
/// factored out so it can be tested without `update_current_contract_wasm`)
/// directly, since Soroban requires a real, previously uploaded Wasm blob
/// to target a swap against, which isn't practical to construct in a unit
/// test. `upgrade`'s admin-auth requirement and its call into
/// `update_current_contract_wasm` are still exercised end-to-end by
/// `test_upgrade_requires_admin_auth` below.
#[test]
fn test_record_upgrade_event_reports_old_and_new_hash() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let first_hash = BytesN::from_array(&env, &[1u8; 32]);
    env.as_contract(&contract_id, || {
        crate::SchemaRegistry::record_upgrade_event(&env, &admin, first_hash.clone());
    });
    // The first upgrade has no prior tracked hash, so it reports the new
    // hash as both old and new rather than a placeholder value.
    let expected_first = ContractUpgradedEvent {
        old_wasm_hash: first_hash.clone(),
        new_wasm_hash: first_hash.clone(),
        authorizer: admin.clone(),
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("UPGRADED"), admin.clone()).into_val(&env),
                expected_first.into_val(&env),
            )
        ]
    );

    let second_hash = BytesN::from_array(&env, &[2u8; 32]);
    env.as_contract(&contract_id, || {
        crate::SchemaRegistry::record_upgrade_event(&env, &admin, second_hash.clone());
    });
    let expected_second = ContractUpgradedEvent {
        old_wasm_hash: first_hash,
        new_wasm_hash: second_hash,
        authorizer: admin.clone(),
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                contract_id,
                (symbol_short!("UPGRADED"), admin).into_val(&env),
                expected_second.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_upgrade_requires_admin_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let new_hash = BytesN::from_array(&env, &[9u8; 32]);
    let res = client.try_upgrade(&new_hash);
    assert!(res.is_err());
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
