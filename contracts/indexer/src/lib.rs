#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sas_common::UID;

#[contract]
pub struct Indexer;

#[contractimpl]
impl Indexer {
    pub fn init(_env: Env) {
        // Initialize reverse lookup
    }

    pub fn index_attestation(_env: Env, _uid: UID, _recipient: Address, _schema_uid: UID) {
        // Chunked Recipient -> Vec<UID>
        // Max 100 per chunk to avoid Soroban limits
        let chunk_index = 0u32;
        let mut recipient_uids: soroban_sdk::Vec<UID> = _env.storage().persistent().get(&(_recipient.clone(), chunk_index)).unwrap_or_else(|| soroban_sdk::Vec::new(&_env));
        recipient_uids.push_back(_uid.clone());
        _env.storage().persistent().set(&(_recipient, chunk_index), &recipient_uids);
        
        // Chunked Schema -> Vec<UID>
        let mut schema_uids: soroban_sdk::Vec<UID> = _env.storage().persistent().get(&(_schema_uid.clone(), chunk_index)).unwrap_or_else(|| soroban_sdk::Vec::new(&_env));
        schema_uids.push_back(_uid.clone());
        _env.storage().persistent().set(&(_schema_uid, chunk_index), &schema_uids);
    }

    pub fn get_attestations_by_recipient(env: Env, recipient: Address) -> soroban_sdk::Vec<UID> {
        let chunk_index = 0u32;
        env.storage().persistent().get(&(recipient, chunk_index)).unwrap_or_else(|| soroban_sdk::Vec::new(&env))
    }
}

#[cfg(test)]
mod test;
