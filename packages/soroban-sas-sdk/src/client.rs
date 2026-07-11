//! Strongly-typed wrappers for contract clients.

pub struct SASClient {
    pub contract_id: String,
}

impl SASClient {
    pub fn new(contract_id: String) -> Self {
        Self { contract_id }
    }
}
