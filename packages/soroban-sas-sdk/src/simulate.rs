//! Builds unsigned transactions for Soroban RPC's `simulateTransaction`, and
//! converts between Rust values and `ScVal` contract-call arguments/results.
//!
//! Simulation never touches ledger state or requires a valid signature, so
//! read-only contract calls (`get_schema`, `verify_attestation`, ...) can go
//! through it without any signing key. A fixed placeholder source account is
//! used for every simulated call, since RPC only needs *a* syntactically
//! valid account to build the envelope, not one that actually exists — this
//! was verified against live `soroban-testnet.stellar.org`: a simulated call
//! built this way is accepted and actually executed by the host (it fails
//! only on `Storage, MissingValue` for the placeholder contract address,
//! confirming the envelope, and the invoked function/args, are well-formed).
//!
//! Argument/result conversion goes through the host `Val` type rather than
//! the direct `TryFrom<T> for ScVal` impls `soroban-sdk` generates for
//! `#[contracttype]` types, because those direct impls are gated behind the
//! `test`/`testutils` cfg (they exist for test-assertion convenience, not
//! for production use) — the `Val`-mediated path is not gated and is the
//! one contract tooling is meant to use off-chain.

use crate::errors::SdkError;
use soroban_sdk::xdr::{
    Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Limits, Memo, MuxedAccount,
    Operation, OperationBody, Preconditions, ReadXdr, ScAddress, ScSymbol, ScVal, SequenceNumber,
    StringM, Transaction, TransactionEnvelope, TransactionExt, TransactionV1Envelope, Uint256,
    VecM, WriteXdr,
};
use soroban_sdk::{Env, TryFromVal, TryIntoVal, Val};

/// Placeholder source account used for every simulated (never submitted,
/// never signed) transaction.
const PLACEHOLDER_SOURCE_ACCOUNT: [u8; 32] = [0; 32];

/// Builds a base64-encoded unsigned `TransactionEnvelope` invoking
/// `function_name` on `contract_id` with `args`, suitable for
/// `simulateTransaction`.
pub fn build_invoke_transaction_xdr(
    contract_id: &str,
    function_name: &str,
    args: Vec<ScVal>,
) -> Result<String, SdkError> {
    let contract = stellar_strkey::Contract::from_string(contract_id)
        .map_err(|e| SdkError::RpcError(format!("invalid contract id {contract_id}: {e:?}")))?;

    let function_name = ScSymbol(
        StringM::try_from(function_name.as_bytes().to_vec())
            .map_err(|e| SdkError::RpcError(format!("invalid function name: {e:?}")))?,
    );
    let args: VecM<ScVal> = args
        .try_into()
        .map_err(|e| SdkError::RpcError(format!("too many arguments: {e:?}")))?;

    let host_function = HostFunction::InvokeContract(InvokeContractArgs {
        contract_address: ScAddress::Contract(Hash(contract.0)),
        function_name,
        args,
    });

    let operation = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function,
            auth: VecM::default(),
        }),
    };

    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(PLACEHOLDER_SOURCE_ACCOUNT)),
        fee: 100,
        seq_num: SequenceNumber(0),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![operation]
            .try_into()
            .expect("a single operation is always within the 100-operation limit"),
        ext: TransactionExt::V0,
    };

    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx,
        signatures: VecM::default(),
    });

    envelope
        .to_xdr_base64(Limits::none())
        .map_err(|e| SdkError::RpcError(format!("failed to encode transaction xdr: {e:?}")))
}

/// Converts a Rust value into an `ScVal` contract-call argument, via the
/// host `Val` bridge (see module docs for why not the direct `ScVal` impl).
pub fn encode_arg<T>(env: &Env, value: &T) -> Result<ScVal, SdkError>
where
    T: TryIntoVal<Env, Val>,
{
    let val: Val = value
        .try_into_val(env)
        .map_err(|_| SdkError::RpcError("failed to convert value to host Val".to_string()))?;
    ScVal::try_from_val(env, &val)
        .map_err(|_| SdkError::RpcError("failed to convert Val to ScVal".to_string()))
}

/// Decodes the base64 `ScVal` XDR returned in a successful simulation's
/// `results[0].xdr` field into a typed value `T`.
///
/// Goes through the host `Val` type (`ScVal` -> `Val` -> `T`) rather than a
/// direct `T: TryFromVal<Env, ScVal>` bound, since `soroban-sdk` only
/// generates that direct impl for `#[contracttype]` types behind the
/// `test`/`testutils` cfg (see module docs).
pub fn decode_result<T>(env: &Env, result_xdr_base64: &str) -> Result<T, SdkError>
where
    T: TryFromVal<Env, Val>,
{
    let sc_val = ScVal::from_xdr_base64(result_xdr_base64, Limits::none())
        .map_err(|e| SdkError::RpcError(format!("failed to decode result xdr: {e:?}")))?;
    let val: Val = Val::try_from_val(env, &sc_val)
        .map_err(|_| SdkError::RpcError("failed to convert ScVal to host Val".to_string()))?;
    T::try_from_val(env, &val)
        .map_err(|_| SdkError::RpcError("failed to convert Val to target type".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sas_common::UID;
    use soroban_sdk::xdr::Limits as XdrLimits;
    use soroban_sdk::BytesN;

    #[test]
    fn builds_a_well_formed_invoke_transaction() {
        let contract = stellar_strkey::Contract([0u8; 32]).to_string();
        let xdr = build_invoke_transaction_xdr(&contract, "get_schema", vec![ScVal::Void]).unwrap();

        // Round-trips through the same XDR parser Soroban RPC uses, and has
        // exactly the InvokeHostFunction operation we asked for.
        let envelope = TransactionEnvelope::from_xdr_base64(xdr, XdrLimits::none()).unwrap();
        let TransactionEnvelope::Tx(v1) = envelope else {
            panic!("expected a V1 transaction envelope");
        };
        assert_eq!(v1.tx.operations.len(), 1);
        let OperationBody::InvokeHostFunction(op) = &v1.tx.operations[0].body else {
            panic!("expected an InvokeHostFunction operation");
        };
        let HostFunction::InvokeContract(args) = &op.host_function else {
            panic!("expected an InvokeContract host function");
        };
        assert_eq!(args.function_name.0.to_string(), "get_schema");
        assert_eq!(args.args.len(), 1);
    }

    #[test]
    fn rejects_an_invalid_contract_id() {
        let err = build_invoke_transaction_xdr("not-a-contract-id", "get_schema", vec![]);
        assert!(matches!(err, Err(SdkError::RpcError(_))));
    }

    #[test]
    fn encode_then_decode_round_trips_a_uid() {
        let env = Env::default();
        let uid = UID(BytesN::from_array(&env, &[9u8; 32]));

        let encoded = encode_arg(&env, &uid).unwrap();
        let decoded: UID = decode_result_from_scval(&env, &encoded);

        assert_eq!(decoded, uid);
    }

    #[test]
    fn encode_then_decode_round_trips_a_bool() {
        let env = Env::default();
        let encoded = encode_arg(&env, &true).unwrap();
        let decoded: bool = decode_result_from_scval(&env, &encoded);
        assert!(decoded);
    }

    /// Test-only helper: same conversion `decode_result` does, but starting
    /// from an in-memory `ScVal` instead of a base64 string, so encode/decode
    /// round-trip tests don't need to go through XDR text encoding.
    fn decode_result_from_scval<T: TryFromVal<Env, Val>>(env: &Env, sc_val: &ScVal) -> T {
        let val: Val = Val::try_from_val(env, sc_val).unwrap();
        T::try_from_val(env, &val).unwrap()
    }
}
