#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};
use soroban_sas_common::{Attestation, UID};

#[contract]
pub struct SAS;

pub const SAS_ADMIN: Symbol = symbol_short!("ADMIN");
pub const SCHEMA_REGISTRY: Symbol = symbol_short!("REGISTRY");

#[contractimpl]
impl SAS {
    pub fn init(env: Env, admin: Address, registry: Address) {
        if env.storage().instance().has(&SAS_ADMIN) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&SAS_ADMIN, &admin);
        env.storage().instance().set(&SCHEMA_REGISTRY, &registry);
    }

    pub fn attest(env: Env, attestation: Attestation) -> UID {
        attestation.attester.require_auth();
        
        let registry: Address = env.storage().instance().get(&SCHEMA_REGISTRY).unwrap();
        let is_valid: bool = env.invoke_contract(
            &registry,
            &Symbol::new(&env, "validate_schema"),
            soroban_sdk::vec![&env, attestation.schema_uid.clone().into_val(&env)]
        );
        if !is_valid {
            panic!("Invalid schema");
        }
        
        // Store the attestation
        env.storage().persistent().set(&attestation.uid, &attestation);
        
        env.events().publish((soroban_sdk::Symbol::new(&env, "ATTESTED"),), attestation.clone());
        
        attestation.uid.clone()
    }

    pub fn revoke(env: Env, uid: UID) {
        let mut attestation: Attestation = env.storage().persistent().get(&uid).expect("Attestation not found");
        attestation.attester.require_auth();
        
        attestation.revocation_time = env.ledger().timestamp();
        env.storage().persistent().set(&uid, &attestation);
        
        env.events().publish((soroban_sdk::Symbol::new(&env, "REVOKED"),), uid.clone());
    }
}

#[cfg(test)]
mod test;
