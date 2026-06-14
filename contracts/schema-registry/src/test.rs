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
