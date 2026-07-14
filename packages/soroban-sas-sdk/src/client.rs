//! Strongly-typed wrappers for contract clients.

/// The primary client for interacting with the SAS contract.
pub struct SASClient {
    /// The Soroban contract ID.
    pub contract_id: String,
}

impl SASClient {
    /// Instantiates a new SASClient with the given contract ID.
    pub fn new(contract_id: String) -> Self {
        Self { contract_id }
    }
}
