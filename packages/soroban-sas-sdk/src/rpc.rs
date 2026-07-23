//! Soroban RPC client scaffolding.
//!
//! This does not perform network I/O yet — it builds well-typed JSON-RPC
//! request bodies for the Soroban RPC methods the SDK will need to submit
//! and track transactions. Wiring an actual HTTP transport (and the
//! matching response types) is a follow-up.

use serde::Serialize;

/// A Soroban RPC endpoint the SDK will submit requests to,
/// e.g. `https://soroban-testnet.stellar.org`.
pub struct RpcClient {
    pub network_url: String,
}

impl RpcClient {
    pub fn new(network_url: impl Into<String>) -> Self {
        Self {
            network_url: network_url.into(),
        }
    }

    /// Builds the JSON-RPC request body for Soroban's `sendTransaction`.
    ///
    /// `tx_envelope_xdr` is the base64-encoded `TransactionEnvelope` to submit.
    pub fn build_send_transaction_request(
        &self,
        tx_envelope_xdr: &str,
    ) -> JsonRpcRequest<SendTransactionParams> {
        JsonRpcRequest::new(
            "sendTransaction",
            SendTransactionParams {
                transaction: tx_envelope_xdr.to_string(),
            },
        )
    }

    /// Builds the JSON-RPC request body for Soroban's `getTransaction`,
    /// used to poll for the result of a previously submitted transaction.
    pub fn build_get_transaction_request(
        &self,
        tx_hash: &str,
    ) -> JsonRpcRequest<GetTransactionParams> {
        JsonRpcRequest::new(
            "getTransaction",
            GetTransactionParams {
                hash: tx_hash.to_string(),
            },
        )
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct JsonRpcRequest<P: Serialize> {
    pub jsonrpc: &'static str,
    pub id: u32,
    pub method: &'static str,
    pub params: P,
}

impl<P: Serialize> JsonRpcRequest<P> {
    fn new(method: &'static str, params: P) -> Self {
        Self {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SendTransactionParams {
    pub transaction: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct GetTransactionParams {
    pub hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_send_transaction_request() {
        let client = RpcClient::new("https://soroban-testnet.stellar.org");
        let request = client.build_send_transaction_request("AAAAAgAAAAA=");

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["method"], "sendTransaction");
        assert_eq!(value["params"]["transaction"], "AAAAAgAAAAA=");
    }

    #[test]
    fn builds_get_transaction_request() {
        let client = RpcClient::new("https://soroban-testnet.stellar.org");
        let request = client.build_get_transaction_request("deadbeef");

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["method"], "getTransaction");
        assert_eq!(value["params"]["hash"], "deadbeef");
    }
}
