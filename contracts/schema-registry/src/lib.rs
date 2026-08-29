#![allow(unexpected_cfgs)]
#![no_std]
#![allow(unused_variables)]

use soroban_sas_common::{
    events::{CONTRACT_UPGRADED, SCHEMA_FEE_UPDATED, TREASURY_UPDATED},
    validate_schema_syntax, ContractUpgradedEvent, SASError, SchemaFeeUpdatedEvent, SchemaRecord,
    TreasuryUpdatedEvent, LEDGERS_IN_ONE_YEAR, UID,
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, xdr::ToXdr, Address, Bytes, BytesN, Env, String,
};

#[contract]
pub struct SchemaRegistry;

mod storage;
use storage::*;

#[contractimpl]
impl SchemaRegistry {
    /// Compatibility probe used by SAS::init before storing this registry.
    pub fn sasreg(_env: Env) -> bool {
        true
    }

    pub fn init(env: Env, admin: soroban_sdk::Address) {
        if env.storage().instance().has(&REGISTRY_ADMIN) {
            panic_with_error!(&env, SASError::AlreadyInitialized);
        }
        env.storage().instance().set(&REGISTRY_ADMIN, &admin);
    }

    /// Replaces this contract's installed WASM. Requires the registry
    /// admin's authorization. Emits `ContractUpgraded` with the hash being
    /// replaced and the new hash immediately before the swap takes effect,
    /// so a failed or unauthorized call never emits the event: if the swap
    /// itself then fails (e.g. `new_wasm_hash` has no uploaded WASM),
    /// Soroban rolls back the whole invocation, discarding the event and
    /// the storage write below along with it.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&REGISTRY_ADMIN).unwrap();
        admin.require_auth();

        Self::record_upgrade_event(&env, &admin, new_wasm_hash.clone());

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Records the WASM-hash rotation and emits `ContractUpgraded`.
    /// Factored out of `upgrade` so its event-payload logic (reading the
    /// previously tracked hash, building the event) can be exercised in
    /// tests without going through `update_current_contract_wasm`, which
    /// requires a real, previously uploaded WASM blob to target.
    fn record_upgrade_event(env: &Env, admin: &Address, new_wasm_hash: BytesN<32>) {
        let old_wasm_hash: Option<BytesN<32>> = env.storage().instance().get(&CURRENT_WASM_HASH);
        // Soroban does not expose a way to read the currently installed
        // WASM hash from within the contract itself, so the first upgrade
        // on a given deployment has no prior tracked hash to report; every
        // upgrade after that carries the hash it is replacing.
        let old_wasm_hash = old_wasm_hash.unwrap_or_else(|| new_wasm_hash.clone());

        env.storage()
            .instance()
            .set(&CURRENT_WASM_HASH, &new_wasm_hash);

        env.events().publish(
            (CONTRACT_UPGRADED, admin.clone()),
            ContractUpgradedEvent {
                old_wasm_hash,
                new_wasm_hash,
                authorizer: admin.clone(),
            },
        );
    }

    /// Sets the fee charged for schema registration. Requires the registry
    /// admin's authorization. Emits `SchemaFeeUpdated` with the previous
    /// fee (`None` the first time a fee is set) after the new fee has
    /// already been written to storage.
    pub fn set_fee(env: Env, fee: i128) {
        let admin: Address = env.storage().instance().get(&REGISTRY_ADMIN).unwrap();
        admin.require_auth();

        let old_fee: Option<i128> = env.storage().instance().get(&SCHEMA_FEE);
        env.storage().instance().set(&SCHEMA_FEE, &fee);

        env.events().publish(
            (SCHEMA_FEE_UPDATED, admin.clone()),
            SchemaFeeUpdatedEvent {
                old_fee,
                new_fee: fee,
                authorizer: admin,
            },
        );
    }

    /// Sets the treasury address that receives registration fees. Requires
    /// the registry admin's authorization. Emits `TreasuryUpdated` with the
    /// previous treasury (`None` the first time a treasury is set) after
    /// the new address has already been written to storage.
    pub fn set_treasury(env: Env, treasury: Address) {
        let admin: Address = env.storage().instance().get(&REGISTRY_ADMIN).unwrap();
        admin.require_auth();

        let old_treasury: Option<Address> = env.storage().instance().get(&TREASURY);
        env.storage().instance().set(&TREASURY, &treasury);

        env.events().publish(
            (TREASURY_UPDATED, admin.clone()),
            TreasuryUpdatedEvent {
                old_treasury,
                new_treasury: treasury,
                authorizer: admin,
            },
        );
    }

    pub fn withdraw_fees(env: Env, amount: i128) {
        let admin: soroban_sdk::Address = env.storage().instance().get(&REGISTRY_ADMIN).unwrap();
        admin.require_auth();
        // Native token transfer logic goes here
    }

    /// Deprecates a schema. Only its original registrant or the registry
    /// administrator may authorize this operation.
    pub fn deprecate(env: Env, uid: UID, authorizer: Address) {
        authorizer.require_auth();

        let admin: Address = env.storage().instance().get(&REGISTRY_ADMIN).unwrap();
        let creator: Option<Address> = env
            .storage()
            .persistent()
            .get(&(SCHEMA_CREATOR, uid.clone()));
        // Schemas registered before creator tracking was introduced have no
        // mapping. The registry admin remains able to deprecate those legacy
        // records; new records also permit their creator.
        if authorizer != admin && creator.as_ref() != Some(&authorizer) {
            panic_with_error!(&env, SASError::Unauthorized);
        }

        let deprecated_key = (DEPRECATED, uid);
        env.storage().persistent().set(&deprecated_key, &true);
        env.storage().persistent().extend_ttl(
            &deprecated_key,
            LEDGERS_IN_ONE_YEAR,
            LEDGERS_IN_ONE_YEAR,
        );
    }

    pub fn register(
        env: Env,
        owner: Address,
        schema: String,
        resolver: Address,
        revocable: bool,
    ) -> UID {
        if let Err(err) = validate_schema_syntax(&env, &schema) {
            panic_with_error!(&env, err);
        }

        // The owner must authorize the registration so the emitted event
        // carries a caller identity that off-chain indexers can trust.
        owner.require_auth();

        let mut payload = Bytes::new(&env);
        payload.append(&schema.clone().to_xdr(&env));

        let hash = env.crypto().sha256(&payload);
        let uid = UID(hash);

        if env.storage().persistent().has(&uid) {
            panic_with_error!(&env, SASError::SchemaAlreadyExists);
        }

        let record = SchemaRecord {
            uid: uid.clone(),
            resolver,
            revocable,
            schema,
        };
        env.storage().persistent().set(&uid, &record);
        env.storage()
            .persistent()
            .extend_ttl(&uid, LEDGERS_IN_ONE_YEAR, LEDGERS_IN_ONE_YEAR);
        let creator_key = (SCHEMA_CREATOR, uid.clone());
        env.storage().persistent().set(&creator_key, &owner);
        env.storage().persistent().extend_ttl(
            &creator_key,
            LEDGERS_IN_ONE_YEAR,
            LEDGERS_IN_ONE_YEAR,
        );

        let mut count: u32 = env.storage().persistent().get(&SCHEMA_COUNT).unwrap_or(0);
        env.storage().persistent().set(&count, &uid);
        env.storage()
            .persistent()
            .extend_ttl(&count, LEDGERS_IN_ONE_YEAR, LEDGERS_IN_ONE_YEAR);
        count += 1;
        env.storage().persistent().set(&SCHEMA_COUNT, &count);
        env.storage().persistent().extend_ttl(
            &SCHEMA_COUNT,
            LEDGERS_IN_ONE_YEAR,
            LEDGERS_IN_ONE_YEAR,
        );

        env.events().publish(
            (soroban_sas_common::events::REGISTERED, uid.clone()),
            soroban_sas_common::SchemaRegisteredEvent {
                schema_uid: uid.clone(),
                owner,
            },
        );

        uid
    }

    pub fn get_schema(env: Env, uid: UID) -> Option<SchemaRecord> {
        if env
            .storage()
            .persistent()
            .get(&(DEPRECATED, uid.clone()))
            .unwrap_or(false)
        {
            return None;
        }
        env.storage().persistent().get(&uid)
    }

    pub fn validate_schema(env: Env, uid: UID) -> bool {
        if env
            .storage()
            .persistent()
            .get(&(DEPRECATED, uid.clone()))
            .unwrap_or(false)
        {
            return false;
        }
        env.storage().persistent().has(&uid)
    }

    pub fn get_schemas(env: Env, start: u32, limit: u32) -> soroban_sdk::Vec<SchemaRecord> {
        let mut schemas = soroban_sdk::Vec::new(&env);
        let count: u32 = env.storage().persistent().get(&SCHEMA_COUNT).unwrap_or(0);

        let end = if start + limit > count {
            count
        } else {
            start + limit
        };
        for i in start..end {
            if let Some(uid) = env.storage().persistent().get::<u32, UID>(&i) {
                if let Some(record) = env.storage().persistent().get(&uid) {
                    schemas.push_back(record);
                }
            }
        }
        schemas
    }
}

#[cfg(test)]
mod test;
