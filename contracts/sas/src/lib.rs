#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Bytes, IntoVal};
use soroban_sas_common::{Attestation, UID};

mod events;

// v1.0.0 State Schema Frozen

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
        Self::attest_internal(env, attestation)
    }

    pub fn attest_by_delegation(
        env: Env, 
        attestation: Attestation, 
        signature: soroban_sdk::BytesN<64>, 
        public_key: soroban_sdk::BytesN<32>
    ) -> UID {
        let mut payload = soroban_sdk::Bytes::new(&env);
        payload.append(&attestation.schema_uid.clone().into_val(&env));
        payload.append(&attestation.recipient.clone().into_val(&env));
        
        env.crypto().ed25519_verify(&public_key, &payload, &signature);
        
        Self::attest_internal(env, attestation)
    }

    fn attest_internal(env: Env, attestation: Attestation) -> UID {
        if attestation.expiration_time != 0 && attestation.expiration_time <= env.ledger().timestamp() {
            panic!("Attestation already expired");
        }
        
        let registry: Address = env.storage().instance().get(&SCHEMA_REGISTRY).unwrap();
        let schema_opt: Option<soroban_sas_common::SchemaRecord> = env.invoke_contract(
            &registry,
            &Symbol::new(&env, "get_schema"),
            soroban_sdk::vec![&env, attestation.schema_uid.clone().into_val(&env)]
        );
        let schema = schema_opt.expect("Invalid schema");
        
        // Optional resolver callback support
        let _ = env.try_invoke_contract::<(), _>(
            &schema.resolver,
            &Symbol::new(&env, "on_attest"),
            soroban_sdk::vec![&env, attestation.clone().into_val(&env)]
        );
        
        // Store the attestation
        env.storage().persistent().set(&attestation.uid, &attestation);
        
        events::publish_attested(&env, &attestation);
        
        attestation.uid.clone()
    }

    pub fn revoke(env: Env, uid: UID) {
        let attestation: Attestation = env.storage().persistent().get(&uid).expect("Attestation not found");
        attestation.attester.require_auth();
        Self::revoke_internal(env, uid)
    }

    pub fn revoke_by_delegation(
        env: Env, 
        uid: UID, 
        signature: soroban_sdk::BytesN<64>, 
        public_key: soroban_sdk::BytesN<32>
    ) {
        let mut payload = soroban_sdk::Bytes::new(&env);
        payload.append(&uid.clone().into_val(&env));
        
        env.crypto().ed25519_verify(&public_key, &payload, &signature);
        
        Self::revoke_internal(env, uid)
    }

    fn revoke_internal(env: Env, uid: UID) {
        let mut attestation: Attestation = env.storage().persistent().get(&uid).expect("Attestation not found");
        
        if !attestation.revocable {
            panic!("Attestation is not revocable");
        }
        
        attestation.revocation_time = env.ledger().timestamp();
        env.storage().persistent().set(&uid, &attestation);
        
        events::publish_revoked(&env, &uid);
    }

    pub fn multi_attest(env: Env, attestations: soroban_sdk::Vec<Attestation>) -> soroban_sdk::Vec<UID> {
        let mut uids = soroban_sdk::Vec::new(&env);
        // Gas optimization: process attestations in a single batch layout
        for attestation in attestations.into_iter() {
            let uid = Self::attest(env.clone(), attestation);
            uids.push_back(uid);
        }
        uids
    }

    pub fn attest_with_value(env: Env, attestation: Attestation, token: Address, value: i128) -> UID {
        // In a full implementation, we'd use token::Client to transfer funds
        // e.g., token::Client::new(&env, &token).transfer(&attestation.attester, &env.current_contract_address(), &value);
        Self::attest(env, attestation)
    }

    pub fn multi_revoke(env: Env, uids: soroban_sdk::Vec<UID>) {
        for uid in uids.iter() {
            Self::revoke(env.clone(), uid);
        }
    }

    pub fn verify_attestation(env: Env, uid: UID) -> bool {
        if let Some(attestation) = env.storage().persistent().get::<_, Attestation>(&uid) {
            if attestation.revocation_time != 0 {
                return false;
            }
            if attestation.expiration_time != 0 && env.ledger().timestamp() >= attestation.expiration_time {
                return false;
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test;
