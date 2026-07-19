# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Initial workspace structure and foundational crates.
- Schema Registry contract implementation.
- SAS core contract implementation for issuing and revoking attestations.
- Indexer contract for reverse lookups.
- CLI tool (`soroban-sas-cli`) for interacting with contracts.
- SDK wrapper (`soroban-sas-sdk`) for dApp integration.
- EIP-712 style off-chain attestations: deterministic typed-data hashing and
  ed25519 verification utilities in `soroban-sas-common`, a
  `verify_offchain_attestation` entrypoint in the SAS contract, and
  `offchain sign` / `offchain verify` CLI commands.
