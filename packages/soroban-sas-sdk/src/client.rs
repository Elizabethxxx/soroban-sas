//! Strongly-typed wrappers for contract clients.

use crate::account;
use crate::errors::SdkError;
use crate::rpc::{GetTransactionResult, RpcClient};
use crate::signature;
use crate::simulate;
use crate::transaction::TransactionSubmitter;
use soroban_sas_common::{Attestation, SchemaRecord, UID};
use soroban_sdk::xdr::{Limits, ReadXdr, ScVal, SorobanTransactionData, TransactionExt};
use soroban_sdk::{Bytes, BytesN, Env};
use std::time::Duration;

/// Classic per-operation fee, in stroops, before the Soroban resource fee
/// simulation reports is added on top.
const BASE_FEE: u32 = 100;
const DEFAULT_MAX_POLL_ATTEMPTS: u32 = 10;
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(2);

/// The primary client for interacting with the SAS contract.
pub struct SASClient {
    /// The Soroban contract ID.
    pub contract_id: String,
}

impl SASClient {
    /// Instantiates a new SASClient with the given contract ID.
    pub fn new(contract_id: String) -> Self {
        Self { contract_id }
    }

    /// Calls `SAS::verify_attestation(uid)` via `simulateTransaction` — a
    /// pure read: no signing key or transaction submission required.
    pub fn verify_attestation(
        &self,
        env: &Env,
        rpc: &RpcClient,
        uid: &[u8; 32],
    ) -> Result<bool, SdkError> {
        let uid = UID(BytesN::from_array(env, uid));
        let arg = simulate::encode_arg(env, &uid)?;
        invoke_read_only(env, rpc, &self.contract_id, "verify_attestation", vec![arg])
    }

    /// Calls `SchemaRegistry::get_schema(uid)` on `registry_contract_id` via
    /// `simulateTransaction` — a pure read, same as `verify_attestation`.
    ///
    /// Takes the registry's contract ID explicitly: `get_schema` lives on
    /// the Schema Registry contract, a separate deployment from the SAS
    /// contract this client otherwise talks to.
    pub fn get_schema(
        &self,
        env: &Env,
        rpc: &RpcClient,
        registry_contract_id: &str,
        uid: &[u8; 32],
    ) -> Result<Option<SchemaRecord>, SdkError> {
        let uid = UID(BytesN::from_array(env, uid));
        let arg = simulate::encode_arg(env, &uid)?;
        invoke_read_only(env, rpc, registry_contract_id, "get_schema", vec![arg])
    }

    /// Calls `SAS::attest(attestation)`: builds the invoke transaction,
    /// signs it with the ed25519 key derived from `secret_seed`, and
    /// submits it — then polls until it settles.
    ///
    /// Requires `secret_seed`'s account to be both the transaction's source
    /// account and `attestation.attester` (see `simulate::sign_transaction`
    /// for why): the common case of an attester submitting and authorizing
    /// its own attestation. A relayer submitting on someone else's behalf
    /// needs a separately signed `SorobanAuthorizationEntry`, which this
    /// does not build.
    pub fn attest(
        &self,
        env: &Env,
        rpc: &RpcClient,
        network_passphrase: &str,
        secret_seed: &[u8; 32],
        attestation: Attestation,
    ) -> Result<GetTransactionResult, SdkError> {
        let arg = simulate::encode_arg(env, &attestation)?;
        invoke_write(
            env,
            rpc,
            network_passphrase,
            secret_seed,
            &self.contract_id,
            "attest",
            vec![arg],
        )
    }

    /// Calls `SAS::revoke(uid)`, same signing/submission flow as `attest`.
    /// Requires `secret_seed`'s account to be the attestation's attester.
    pub fn revoke(
        &self,
        env: &Env,
        rpc: &RpcClient,
        network_passphrase: &str,
        secret_seed: &[u8; 32],
        uid: &[u8; 32],
    ) -> Result<GetTransactionResult, SdkError> {
        let uid = UID(BytesN::from_array(env, uid));
        let arg = simulate::encode_arg(env, &uid)?;
        invoke_write(
            env,
            rpc,
            network_passphrase,
            secret_seed,
            &self.contract_id,
            "revoke",
            vec![arg],
        )
    }
}

/// Simulates a read-only call to `function_name` on `contract_id` and
/// decodes its return value as `T`.
fn invoke_read_only<T>(
    env: &Env,
    rpc: &RpcClient,
    contract_id: &str,
    function_name: &str,
    args: Vec<ScVal>,
) -> Result<T, SdkError>
where
    T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>,
{
    let tx_xdr = simulate::build_simulate_transaction_xdr(contract_id, function_name, args)?;
    let result = rpc.simulate_transaction(&tx_xdr)?;
    if let Some(error) = result.error {
        return Err(SdkError::RpcError(error));
    }
    let xdr = result
        .results
        .first()
        .ok_or_else(|| SdkError::RpcError("simulateTransaction returned no results".to_string()))?
        .xdr
        .clone();
    simulate::decode_result(env, &xdr)
}

/// Builds, simulates (to get the real resource footprint/fee), signs, and
/// submits a write call to `function_name` on `contract_id`, then polls
/// until the transaction settles.
fn invoke_write(
    env: &Env,
    rpc: &RpcClient,
    network_passphrase: &str,
    secret_seed: &[u8; 32],
    contract_id: &str,
    function_name: &str,
    args: Vec<ScVal>,
) -> Result<GetTransactionResult, SdkError> {
    let public_key = signature::derive_public_key(secret_seed);
    let next_seq = account::fetch_sequence_number(rpc, &public_key)? + 1;

    // 1. Simulate a draft (V0, base-fee) transaction to get the real
    //    resource footprint and fee a submittable one needs to carry.
    let draft_tx = simulate::build_invoke_transaction(
        &public_key,
        next_seq,
        BASE_FEE,
        TransactionExt::V0,
        contract_id,
        function_name,
        args.clone(),
    )?;
    let draft_xdr = simulate::unsigned_envelope_xdr(draft_tx)?;
    let sim = rpc.simulate_transaction(&draft_xdr)?;
    if let Some(error) = sim.error {
        return Err(SdkError::RpcError(error));
    }
    let transaction_data_b64 = sim.transaction_data.ok_or_else(|| {
        SdkError::RpcError("simulation succeeded but returned no transactionData".to_string())
    })?;
    let soroban_data =
        SorobanTransactionData::from_xdr_base64(transaction_data_b64, Limits::none())
            .map_err(|e| SdkError::RpcError(format!("failed to decode transactionData: {e:?}")))?;
    let resource_fee: i64 = sim
        .min_resource_fee
        .as_deref()
        .unwrap_or("0")
        .parse()
        .map_err(|e| SdkError::RpcError(format!("invalid minResourceFee: {e:?}")))?;
    let fee = u32::try_from(i64::from(BASE_FEE) + resource_fee)
        .map_err(|_| SdkError::RpcError("computed fee overflowed u32".to_string()))?;

    // 2. Build the real transaction with that resource data and fee, and
    //    sign it.
    let final_tx = simulate::build_invoke_transaction(
        &public_key,
        next_seq,
        fee,
        TransactionExt::V1(soroban_data),
        contract_id,
        function_name,
        args,
    )?;
    let network_id: [u8; 32] = env
        .crypto()
        .sha256(&Bytes::from_slice(env, network_passphrase.as_bytes()))
        .to_array();
    let signed_xdr = simulate::sign_transaction(env, &network_id, final_tx, secret_seed)?;

    // 3. Submit and poll until it settles.
    TransactionSubmitter::submit_with_retries(
        rpc,
        &signed_xdr,
        DEFAULT_MAX_POLL_ATTEMPTS,
        DEFAULT_POLL_INTERVAL,
    )
}
