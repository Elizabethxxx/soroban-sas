//! Strongly-typed wrappers for contract clients.

use crate::errors::SdkError;
use crate::rpc::RpcClient;
use crate::simulate;
use soroban_sas_common::{SchemaRecord, UID};
use soroban_sdk::{BytesN, Env};

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
}

/// Simulates a read-only call to `function_name` on `contract_id` and
/// decodes its return value as `T`.
fn invoke_read_only<T>(
    env: &Env,
    rpc: &RpcClient,
    contract_id: &str,
    function_name: &str,
    args: Vec<soroban_sdk::xdr::ScVal>,
) -> Result<T, SdkError>
where
    T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>,
{
    let tx_xdr = simulate::build_invoke_transaction_xdr(contract_id, function_name, args)?;
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
