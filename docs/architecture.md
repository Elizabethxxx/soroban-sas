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

## Storage Retention Policy

Soroban has two independent expiry mechanisms, and each contract's core
configuration is deliberately held to a stricter policy than the
attestation/schema data it governs:

- **Instance storage** holds a contract's core configuration: SAS's
  `SAS_ADMIN`, `SCHEMA_REGISTRY`, and `INDEXER` bindings; the schema
  registry's `REGISTRY_ADMIN`, `SCHEMA_FEE`, and `TREASURY`; and the
  indexer's `INDEXER_ADMIN` and `SAS_CONTRACT` binding. If instance storage
  expires and is archived, the contract's own configuration becomes
  unreadable and every entry point that depends on it stops working —
  there is no way to "read the admin address to renew the admin address."
  For this reason every contract renews its instance TTL
  (`soroban_sas_common::extend_instance_ttl`, using the shared
  `INSTANCE_TTL_THRESHOLD_LEDGERS`/`INSTANCE_EXTEND_TO_LEDGERS` constants)
  from `init` and from both admin-gated and commonly used public entry
  points, so ordinary traffic keeps configuration alive without any single
  call being solely responsible for it.
- **Persistent storage** holds the data instance configuration governs —
  attestations, schema records, delegation nonces, indexer lookup chunks —
  and is extended independently, per entry, using `LEDGERS_IN_ONE_YEAR`
  wherever it is written or read. An individual attestation or schema
  expiring does not take down the rest of the contract the way a lost
  admin binding would, so persistent entries are extended on their own
  schedule rather than the stricter instance policy.
