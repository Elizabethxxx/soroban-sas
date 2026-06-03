#![no_std]
use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UID(pub [u8; 32]);
