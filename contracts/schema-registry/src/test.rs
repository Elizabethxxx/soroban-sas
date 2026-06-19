#![cfg(test)]

use crate::{SchemaRegistry, SchemaRegistryClient};
use soroban_sdk::{Env, Address, String};
use soroban_sdk::testutils::Address as _;

#[test]
fn test_register_schema() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);
    
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);
    
    let uid = client.register(&schema_str, &resolver, &true);
    let record = client.get_schema(&uid).unwrap();
    
    assert_eq!(record.schema, schema_str);
    assert_eq!(record.revocable, true);
    assert_eq!(record.resolver, resolver);
}

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

use soroban_sdk::BytesN;

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
    
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);
    
    let uid = client.register(&schema_str, &resolver, &true);
    
    // Check it's active
    assert!(client.get_schema(&uid).is_some());
    
    // Deprecate
    client.deprecate(&uid);
    
    // Check it's no longer active
    assert!(client.get_schema(&uid).is_none());
}

#[test]
fn test_validate_schema() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SchemaRegistry);
    let client = SchemaRegistryClient::new(&env, &contract_id);
    
    let schema_str = String::from_str(&env, "bool like_soroban");
    let resolver = Address::generate(&env);
    
    let uid = client.register(&schema_str, &resolver, &true);
    
    assert_eq!(client.validate_schema(&uid), true);
    
    client.deprecate(&uid);
    assert_eq!(client.validate_schema(&uid), false);
}
