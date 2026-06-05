#![no_std]
use soroban_sdk::{contracttype, Address, String, Bytes};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UID(pub [u8; 32]);

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaRecord {
    pub uid: UID,
    pub resolver: Address,
    pub revocable: bool,
    pub schema: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub uid: UID,
    pub schema_uid: UID,
    pub time: u64,
    pub expiration_time: u64,
    pub revocation_time: u64,
    pub ref_uid: UID,
    pub recipient: Address,
    pub attester: Address,
    pub revocable: bool,
    pub data: Bytes,
}
