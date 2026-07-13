//! Specialized error handling for SDK

#[derive(Debug)]
pub enum SdkError {
    ContractError(u32),
    RpcError(String),
}
