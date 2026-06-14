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
