//! Soroban SAS SDK
//! 
//! This library provides strong types, builder patterns, and RPC integration
//! for interacting with the Soroban Attestation Service (SAS).

pub mod client;
pub mod schema_builder;

pub fn init() {}
pub mod attestation_builder;
pub mod signature;
pub mod rpc;
pub mod transaction;
pub mod batch;

#[cfg(test)]
mod test;
pub mod events;
