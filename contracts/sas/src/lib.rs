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
        
        // Basic logic for issuance
        attestation.uid.clone()
    }
}
