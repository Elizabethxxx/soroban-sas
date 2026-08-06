//! Submits a transaction and polls until it settles.

use crate::errors::SdkError;
use crate::rpc::{GetTransactionResult, RpcClient};
use std::time::Duration;

const SETTLING_STATUSES: [&str; 2] = ["NOT_FOUND", "PENDING"];

pub struct TransactionSubmitter;

impl TransactionSubmitter {
    /// Sends `tx_envelope_xdr` via `client`, then polls `getTransaction`
    /// (waiting `poll_interval` between attempts) until it leaves
    /// `NOT_FOUND`/`PENDING`, or returns an error once `max_attempts` is
    /// exhausted.
    pub fn submit_with_retries(
        client: &RpcClient,
        tx_envelope_xdr: &str,
        max_attempts: u32,
        poll_interval: Duration,
    ) -> Result<GetTransactionResult, SdkError> {
        let sent = client.send_transaction(tx_envelope_xdr)?;
        Self::poll_until_settled(max_attempts, poll_interval, || {
            client.get_transaction(&sent.hash)
        })
    }

    fn poll_until_settled<F>(
        max_attempts: u32,
        poll_interval: Duration,
        mut fetch: F,
    ) -> Result<GetTransactionResult, SdkError>
    where
        F: FnMut() -> Result<GetTransactionResult, SdkError>,
    {
        for attempt in 0..max_attempts {
            let result = fetch()?;
            if !SETTLING_STATUSES.contains(&result.status.as_str()) {
                return Ok(result);
            }
            if attempt + 1 < max_attempts {
                std::thread::sleep(poll_interval);
            }
        }
        Err(SdkError::RpcError(format!(
            "transaction did not settle after {max_attempts} attempts"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(status: &str) -> GetTransactionResult {
        GetTransactionResult {
            status: status.to_string(),
            latest_ledger: 1,
            envelope_xdr: None,
            result_xdr: None,
        }
    }

    #[test]
    fn returns_immediately_once_settled() {
        let mut calls = 0;
        let outcome = TransactionSubmitter::poll_until_settled(5, Duration::ZERO, || {
            calls += 1;
            Ok(result("SUCCESS"))
        });

        assert_eq!(calls, 1);
        assert_eq!(outcome.unwrap().status, "SUCCESS");
    }

    #[test]
    fn keeps_polling_while_pending() {
        let mut calls = 0;
        let outcome = TransactionSubmitter::poll_until_settled(5, Duration::ZERO, || {
            calls += 1;
            Ok(if calls < 3 {
                result("PENDING")
            } else {
                result("SUCCESS")
            })
        });

        assert_eq!(calls, 3);
        assert_eq!(outcome.unwrap().status, "SUCCESS");
    }

    #[test]
    fn errors_once_attempts_are_exhausted() {
        let mut calls = 0;
        let outcome = TransactionSubmitter::poll_until_settled(3, Duration::ZERO, || {
            calls += 1;
            Ok(result("NOT_FOUND"))
        });

        assert_eq!(calls, 3);
        assert!(matches!(outcome, Err(SdkError::RpcError(_))));
    }

    #[test]
    fn propagates_rpc_errors_immediately() {
        let mut calls = 0;
        let outcome = TransactionSubmitter::poll_until_settled(5, Duration::ZERO, || {
            calls += 1;
            Err(SdkError::RpcError("boom".to_string()))
        });

        assert_eq!(calls, 1);
        assert!(matches!(outcome, Err(SdkError::RpcError(_))));
    }
}
