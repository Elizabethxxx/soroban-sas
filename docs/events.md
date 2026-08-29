# Contract Events

All state-changing operations emit standardized Soroban events via
`env.events().publish(...)` so off-chain indexers (The Graph, Soroban
Zephyr, custom RPC consumers) can build queryable materialized views of the
attestation graph without reading contract storage.

Topic constants and payload types live in `soroban-sas-common`
(`soroban_sas_common::events`), so contracts and tooling share a single
definition.

## SchemaRegistered

Emitted by the schema registry on a successful `register`.

- Topics: `("REGISTER", schema_uid: UID)`
- Data: `SchemaRegisteredEvent { schema_uid: UID, owner: Address }`

`register` requires authorization from `owner`, so the address in the event
is always an authenticated caller — indexers can trust it as the schema's
registrant.

## AttestationIssued

Emitted by the SAS contract on every successful attestation (`attest`,
`attest_by_delegation`, `multi_attest`, `attest_with_value`).

- Topics: `("ATTESTED", schema_uid: UID, attester: Address)`
- Data: `AttestationIssuedEvent { uid: UID, schema_uid: UID, attester: Address, recipient: Address }`

`schema_uid` and `attester` are topics so indexers can subscribe to a single
schema or attester without decoding payloads.

## AttestationRevoked

Emitted by the SAS contract on every successful revocation (`revoke`,
`revoke_by_delegation`, `multi_revoke`).

- Topics: `("REVOKED", uid: UID)`
- Data: `AttestationRevokedEvent { uid: UID, timestamp: u64 }`

`timestamp` is the exact ledger timestamp written to the attestation's
`revocation_time`, so event consumers and contract state can never diverge.

## AttesterKeyRegistered

Emitted by the SAS contract on a successful `register_attester_key` — the
first registration for an attester, or a re-registration after a prior key
was revoked.

- Topics: `("ATTKREG", attester: Address)`
- Data: `AttesterKeyRegisteredEvent { attester: Address, public_key: BytesN<32>, version: u32 }`

`version` starts at `1` and increases by one on every subsequent
registration or rotation for the same attester, so consumers can order key
changes without relying on ledger sequence alone.

## AttesterKeyRotated

Emitted by the SAS contract on a successful `rotate_attester_key`.

- Topics: `("ATTKROT", attester: Address)`
- Data: `AttesterKeyRotatedEvent { attester: Address, old_public_key: BytesN<32>, new_public_key: BytesN<32>, new_version: u32 }`

Carrying both the old and new key lets an off-chain monitor reconstruct an
attester's full key history from the event log alone. A signature made
under `old_public_key` stops validating any delegated operation as soon as
this event is emitted.

## AttesterKeyRevoked

Emitted by the SAS contract on a successful `revoke_attester_key`.

- Topics: `("ATTKREV", attester: Address)`
- Data: `AttesterKeyRevokedEvent { attester: Address, public_key: BytesN<32>, version: u32 }`

Once revoked, `public_key` no longer validates any delegated operation for
`attester`. The underlying record is retained (not deleted) so a future
`register_attester_key` call continues the `version` sequence instead of
restarting at `1`.

## Parsing events off-chain

`soroban-sas-sdk` ships decoding utilities in `soroban_sas_sdk::events`:

```rust
use soroban_sas_sdk::events::{parse_contract_event, SasEvent};

// `event` is an xdr::ContractEvent from a transaction meta or getEvents.
match parse_contract_event(&event) {
    Ok(SasEvent::AttestationIssued(issued)) => {
        // issued.uid, issued.schema_uid, issued.attester, issued.recipient
    }
    Ok(SasEvent::AttestationRevoked(revoked)) => {
        // revoked.uid, revoked.timestamp
    }
    Ok(SasEvent::SchemaRegistered(registered)) => {
        // registered.schema_uid, registered.owner
    }
    Err(_) => { /* not a SAS event */ }
}
```

`parse_events` filters a whole batch, and `parse_event` accepts raw
`ScVal` topics and data for consumers that decode XDR themselves.
