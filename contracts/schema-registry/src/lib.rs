#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol, String, Bytes};
use soroban_sas_common::{SchemaRecord, UID};

#[contract]
pub struct SchemaRegistry;

pub const REGISTRY_ADMIN: Symbol = symbol_short!("ADMIN");
pub const SCHEMA_COUNT: Symbol = symbol_short!("COUNT");

#[contractimpl]
impl SchemaRegistry {
    pub fn init(env: Env, admin: soroban_sdk::Address) {
        if env.storage().instance().has(&REGISTRY_ADMIN) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&REGISTRY_ADMIN, &admin);
    }

    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
        let admin: soroban_sdk::Address = env.storage().instance().get(&REGISTRY_ADMIN).unwrap();
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn register(env: Env, schema: String, resolver: Address, revocable: bool) -> UID {
        let mut payload = Bytes::new(&env);
        payload.append(&schema.clone().into());
        
        let hash = env.crypto().sha256(&payload);
        let uid = UID(hash.into());
        
        if env.storage().persistent().has(&uid) {
            soroban_sdk::panic_with_error!(&env, soroban_sas_common::SASError::SchemaAlreadyExists);
        }
        
        let record = SchemaRecord {
            uid: uid.clone(),
            resolver,
            revocable,
            schema,
        };
        env.storage().persistent().set(&uid, &record);
        
        env.events().publish((soroban_sas_common::events::REGISTERED,), uid.clone());
        
        uid
    }

    pub fn get_schema(env: Env, uid: UID) -> Option<SchemaRecord> {
        env.storage().persistent().get(&uid)
    }
}

#[cfg(test)]
mod test;
