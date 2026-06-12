use soroban_sdk::{contracttype, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleRoot(pub BytesN<32>);

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchAttestation {
    pub root: MerkleRoot,
    pub count: u32,
}
