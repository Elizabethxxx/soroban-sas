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

### Known Issues
- `SchemaRegistry::deprecate` currently lacks an authorization check.
- Delegated attest/revoke signatures do not bind the full attestation payload or a nonce, permitting potential replay.
