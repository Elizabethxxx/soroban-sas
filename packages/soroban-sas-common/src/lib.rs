#![no_std]
use soroban_sdk::{contracttype, Address, String};

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
