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
