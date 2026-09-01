# Soroban SAS Architecture

## Overview
The Soroban Attestation Service (SAS) is composed of three primary components:
1. **Schema Registry**: Stores reusable data layouts (schemas) identified by deterministic UIDs.
2. **SAS Core Contract**: Issues, revokes and verifies attestations based on registered schemas.
3. **Indexer Contract**: Provides off-chain and on-chain reverse lookups for recipients, schemas and attesters.

## Design Goals
- High throughput via parallelized state access.
- Minimal gas overhead.
- Strict payload boundaries to prevent gas exhaustion attacks.

## Trust Boundaries

### Indexer writes

`Indexer::index_attestation` is not a public write path. It records UIDs
into the recipient, schema, and attester lookup tables, and those tables are
only useful if every entry actually corresponds to an attestation the SAS
contract issued. If any caller could invoke it directly, an attacker could
inject arbitrary UIDs and silently poison every reverse lookup the indexer
serves.

`index_attestation` therefore requires `sas.require_auth()`, where `sas` is
the address recorded by `Indexer::init`. Soroban satisfies a contract
address's `require_auth()` without an explicit signature when the call
originates from that contract's own execution — concretely, only
`SAS::attest_internal` invoking `index_attestation` as part of handling
`attest`/`attest_by_delegation`/`multi_attest`/`attest_with_value` can
satisfy it. An external account, or any other contract (including one that
merely forwards the same arguments), cannot produce this authorization and
the call is rejected. A call made before `Indexer::init` has bound a SAS
address is rejected outright, since there is no trusted address to
authorize against yet.

This mirrors how `SAS::init` and `Indexer::init` already gate on a
compatibility probe (`sasreg`/`sasv1`) before trusting a configured
dependency address — the indexer's SAS binding is a similar one-way trust
relationship, just enforced per-call instead of once at initialization.
