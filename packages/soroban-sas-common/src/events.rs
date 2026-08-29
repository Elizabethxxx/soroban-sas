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
/// First topic of every `IndexerUpdated` event.
pub const INDEXER_UPDATED: Symbol = symbol_short!("IDXUPD");
/// First topic of every `SchemaFeeUpdated` event.
pub const SCHEMA_FEE_UPDATED: Symbol = symbol_short!("FEEUPD");
/// First topic of every `TreasuryUpdated` event.
pub const TREASURY_UPDATED: Symbol = symbol_short!("TRSUPD");
/// First topic of every `ContractUpgraded` event.
pub const CONTRACT_UPGRADED: Symbol = symbol_short!("UPGRADED");

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

/// Payload of the `IndexerUpdated` event.
///
/// Published with topics `(INDEXER_UPDATED, authorizer)` on a successful
/// `SAS::set_indexer`. `old_indexer` is `None` the first time an indexer is
/// bound. `authorizer` is the address that authorized the change (SAS's
/// admin), included directly in the payload — not just implied by
/// `require_auth` — so an off-chain monitor can attribute the change
/// without cross-referencing a separate admin-lookup call.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexerUpdatedEvent {
    pub old_indexer: Option<Address>,
    pub new_indexer: Address,
    pub authorizer: Address,
}

/// Payload of the `SchemaFeeUpdated` event.
///
/// Published with topics `(SCHEMA_FEE_UPDATED, authorizer)` on a
/// successful `SchemaRegistry::set_fee`. `old_fee` is `None` the first
/// time a fee is set.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaFeeUpdatedEvent {
    pub old_fee: Option<i128>,
    pub new_fee: i128,
    pub authorizer: Address,
}

/// Payload of the `TreasuryUpdated` event.
///
/// Published with topics `(TREASURY_UPDATED, authorizer)` on a successful
/// `SchemaRegistry::set_treasury`. `old_treasury` is `None` the first time
/// a treasury address is set.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryUpdatedEvent {
    pub old_treasury: Option<Address>,
    pub new_treasury: Address,
    pub authorizer: Address,
}

/// Payload of the `ContractUpgraded` event.
///
/// Published with topics `(CONTRACT_UPGRADED, authorizer)` on a successful
/// `SchemaRegistry::upgrade`, immediately before the WASM swap takes
/// effect. `old_wasm_hash` is the hash of the code being replaced, read
/// directly from the ledger's current contract executable so it cannot be
/// spoofed by the caller.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractUpgradedEvent {
    pub old_wasm_hash: BytesN<32>,
    pub new_wasm_hash: BytesN<32>,
    pub authorizer: Address,
}
