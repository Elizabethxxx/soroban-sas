//! Fetches an account's current sequence number, needed to build a
//! submittable (not just simulated) transaction.

use crate::errors::SdkError;
use crate::rpc::RpcClient;
use soroban_sdk::xdr::{
    AccountId, LedgerEntryData, LedgerKey, LedgerKeyAccount, Limits, PublicKey, ReadXdr, Uint256,
    WriteXdr,
};

/// Fetches the current sequence number of the account belonging to
/// `public_key`. The next valid transaction from this account uses
/// `sequence_number + 1`.
pub fn fetch_sequence_number(rpc: &RpcClient, public_key: &[u8; 32]) -> Result<i64, SdkError> {
    let key = account_ledger_key_base64(public_key)?;
    let result = rpc.get_ledger_entries(vec![key])?;

    let entry = result.entries.first().ok_or_else(|| {
        SdkError::RpcError(
            "account does not exist on this network (fund it first, e.g. via friendbot on testnet)"
                .to_string(),
        )
    })?;

    let data = LedgerEntryData::from_xdr_base64(&entry.xdr, Limits::none())
        .map_err(|e| SdkError::RpcError(format!("failed to decode ledger entry xdr: {e:?}")))?;

    match data {
        LedgerEntryData::Account(account) => Ok(account.seq_num.0),
        other => Err(SdkError::RpcError(format!(
            "expected an Account ledger entry, got {other:?}"
        ))),
    }
}

fn account_ledger_key_base64(public_key: &[u8; 32]) -> Result<String, SdkError> {
    let key = LedgerKey::Account(LedgerKeyAccount {
        account_id: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(*public_key))),
    });
    key.to_xdr_base64(Limits::none())
        .map_err(|e| SdkError::RpcError(format!("failed to encode ledger key: {e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::xdr::{AccountEntry, AccountEntryExt, SequenceNumber, String32, Thresholds};

    #[test]
    fn account_ledger_key_round_trips_through_xdr() {
        let public_key = [5u8; 32];
        let key_b64 = account_ledger_key_base64(&public_key).unwrap();

        let key = LedgerKey::from_xdr_base64(key_b64, Limits::none()).unwrap();
        let LedgerKey::Account(account_key) = key else {
            panic!("expected a LedgerKey::Account");
        };
        let AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(bytes))) = account_key.account_id;
        assert_eq!(bytes, public_key);
    }

    #[test]
    fn extracts_seq_num_from_an_account_entry() {
        let public_key = [6u8; 32];
        let account_entry = AccountEntry {
            account_id: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(public_key))),
            balance: 100_000_000,
            seq_num: SequenceNumber(42),
            num_sub_entries: 0,
            inflation_dest: None,
            flags: 0,
            home_domain: String32::default(),
            thresholds: Thresholds([1, 0, 0, 0]),
            signers: Default::default(),
            ext: AccountEntryExt::V0,
        };
        let entry_xdr = LedgerEntryData::Account(account_entry)
            .to_xdr_base64(Limits::none())
            .unwrap();

        let data = LedgerEntryData::from_xdr_base64(entry_xdr, Limits::none()).unwrap();
        let LedgerEntryData::Account(decoded) = data else {
            panic!("expected an Account ledger entry");
        };
        assert_eq!(decoded.seq_num.0, 42);
    }
}
