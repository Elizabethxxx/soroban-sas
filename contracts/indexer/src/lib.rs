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
        // Recipient -> Vec<UID>
        let mut recipient_uids: soroban_sdk::Vec<UID> = _env.storage().persistent().get(&_recipient).unwrap_or_else(|| soroban_sdk::Vec::new(&_env));
        recipient_uids.push_back(_uid.clone());
        _env.storage().persistent().set(&_recipient, &recipient_uids);
    }
}
