## Summary

<!-- What does this PR change and why? Link the issue it addresses, if any (e.g. Closes #05). -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change (contract storage layout, public API, or CLI/SDK interface)
- [ ] Documentation
- [ ] Testing / tooling

## Area(s) touched

- [ ] `contracts/schema-registry`
- [ ] `contracts/sas`
- [ ] `contracts/indexer`
- [ ] `packages/soroban-sas-common`
- [ ] `packages/soroban-sas-sdk`
- [ ] `packages/soroban-sas-cli`
- [ ] Docs / specs
- [ ] CI / tooling / scripts

## How was this tested?

<!-- e.g. `TMPDIR=/tmp cargo test --workspace`, new unit tests added, manual CLI/testnet run. -->

## Checklist

- [ ] `cargo fmt --all` run
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `TMPDIR=/tmp cargo test --workspace` passes
- [ ] Added/updated tests for the change
- [ ] Updated `README.md` if the change affects the public interface (contract methods, CLI commands, SDK types)
- [ ] Updated `CHANGELOG.md` under `[Unreleased]`
- [ ] For contract changes: considered authorization (`require_auth`), storage layout compatibility, and event emission

## Security considerations

<!-- Anything auth-related, delegated-signature-related, or storage-migration-related? Call it out explicitly, even if the answer is "none". -->
