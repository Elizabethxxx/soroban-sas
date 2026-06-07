use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureData {
    pub v: u32,
    pub r: BytesN<32>,
    pub s: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegatedAttestation {
    pub attester: Address,
    pub signature: SignatureData,
    pub payload_hash: BytesN<32>,
}
