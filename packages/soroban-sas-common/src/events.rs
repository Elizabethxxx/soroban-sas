//! Standardized event topics and payloads emitted by the SAS contracts.
//!
//! Off-chain indexing services (e.g. The Graph, Soroban Zephyr) subscribe to
//! these topics to build fast, queryable materialized views of the
//! attestation graph without reading contract storage.

use crate::UID;
use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Symbol};

/// First topic of every `AttestationIssued` event.
pub const ATTESTED: Symbol = symbol_short!("ATTESTED");
/// First topic of every `AttestationRevoked` event.
pub const REVOKED: Symbol = symbol_short!("REVOKED");
/// First topic of every `SchemaRegistered` event.
pub const REGISTERED: Symbol = symbol_short!("REGISTER");
/// First topic of every `AttesterKeyRegistered` event.
pub const ATTESTER_KEY_REGISTERED: Symbol = symbol_short!("ATTKREG");
/// First topic of every `AttesterKeyRotated` event.
pub const ATTESTER_KEY_ROTATED: Symbol = symbol_short!("ATTKROT");
/// First topic of every `AttesterKeyRevoked` event.
pub const ATTESTER_KEY_REVOKED: Symbol = symbol_short!("ATTKREV");

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

/// Payload of the `AttesterKeyRegistered` event.
///
/// Published with topics `(ATTESTER_KEY_REGISTERED, attester)` the first
/// time a delegated-verification key is registered for `attester`, and
/// again if a key is re-registered after a prior one was revoked.
/// `version` starts at `1` and increases by one on every subsequent
/// registration or rotation for the same attester, so off-chain consumers
/// can order key changes without relying on ledger sequence alone.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttesterKeyRegisteredEvent {
    pub attester: Address,
    pub public_key: BytesN<32>,
    pub version: u32,
}

/// Payload of the `AttesterKeyRotated` event.
///
/// Published with topics `(ATTESTER_KEY_ROTATED, attester)` when an
/// already-registered, non-revoked key is replaced with a new one.
/// `old_public_key` and `new_public_key` let an off-chain monitor
/// reconstruct the full key history; `new_version` is the incremented
/// version now in effect.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttesterKeyRotatedEvent {
    pub attester: Address,
    pub old_public_key: BytesN<32>,
    pub new_public_key: BytesN<32>,
    pub new_version: u32,
}

/// Payload of the `AttesterKeyRevoked` event.
///
/// Published with topics `(ATTESTER_KEY_REVOKED, attester)`. Once revoked,
/// `public_key` no longer validates any delegated operation for
/// `attester`, even though the record is retained (rather than deleted)
/// so `version` continues to increase on any future re-registration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttesterKeyRevokedEvent {
    pub attester: Address,
    pub public_key: BytesN<32>,
    pub version: u32,
}
