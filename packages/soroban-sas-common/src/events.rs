//! Standardized event topics and payloads emitted by the SAS contracts.
//!
//! Off-chain indexing services (e.g. The Graph, Soroban Zephyr) subscribe to
//! these topics to build fast, queryable materialized views of the
//! attestation graph without reading contract storage.

use crate::UID;
use soroban_sdk::{contracttype, symbol_short, Address, Symbol};

/// First topic of every `AttestationIssued` event.
pub const ATTESTED: Symbol = symbol_short!("ATTESTED");
/// First topic of every `AttestationRevoked` event.
pub const REVOKED: Symbol = symbol_short!("REVOKED");
/// First topic of every `SchemaRegistered` event.
pub const REGISTERED: Symbol = symbol_short!("REGISTER");

/// Payload of the `SchemaRegistered` event.
///
/// Published with topics `(REGISTERED, schema_uid)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaRegisteredEvent {
    pub schema_uid: UID,
    pub owner: Address,
}

/// Payload of the `AttestationIssued` event.
///
/// Published with topics `(ATTESTED, schema_uid, attester)` so indexers can
/// filter by schema or attester without decoding the payload.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestationIssuedEvent {
    pub uid: UID,
    pub schema_uid: UID,
    pub attester: Address,
    pub recipient: Address,
}

/// Payload of the `AttestationRevoked` event.
///
/// Published with topics `(REVOKED, uid)`. `timestamp` is the ledger
/// timestamp recorded as the attestation's revocation time.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestationRevokedEvent {
    pub uid: UID,
    pub timestamp: u64,
}
