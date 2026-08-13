use clap::{Parser, Subcommand};

mod offchain;

#[derive(Parser)]
#[command(name = "soroban-sas")]
#[command(about = "CLI for Soroban Attestation Service")]
struct Cli {
    #[arg(long, global = true, help = "RPC Network to connect to")]
    network: Option<String>,

    #[arg(long, global = true, help = "Identity to use for signing")]
    identity: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Schema registry commands
    Schema {
        #[command(subcommand)]
        action: SchemaCommands,
    },
    /// Attestation lifecycle commands
    Attest {
        #[command(subcommand)]
        action: AttestCommands,
    },
    /// Indexer query commands
    Query {
        #[command(subcommand)]
        action: QueryCommands,
    },
    /// Generate off-chain delegated signatures
    Delegate {
        #[arg(long, help = "JSON payload to sign")]
        payload: String,
    },
    /// Off-chain attestation signing and verification
    Offchain {
        #[command(subcommand)]
        action: OffchainCommands,
    },
}

#[derive(Subcommand)]
enum OffchainCommands {
    /// Sign an attestation off-chain with an ed25519 key
    Sign {
        #[arg(long, help = "JSON file containing the attestation payload")]
        data_file: String,
        #[arg(
            long,
            help = "Signing key: S... strkey seed or 32-byte hex seed",
            env = "SAS_SECRET_KEY",
            hide_env_values = true
        )]
        secret_key: String,
        #[arg(long, help = "Replay-protection nonce bound into the signature")]
        nonce: u64,
        #[arg(long, help = "Network passphrase the signature is bound to")]
        network_passphrase: String,
        #[arg(long, help = "SAS contract address (C...) the signature is bound to")]
        contract_id: String,
        #[arg(
            long,
            help = "Write the signed attestation to this file instead of stdout"
        )]
        output: Option<String>,
    },
    /// Verify a signed off-chain attestation
    Verify {
        #[arg(long, help = "JSON file containing the signed attestation")]
        file: String,
    },
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Register a new schema. The registration is signed and submitted by
    /// --secret-key's account, which becomes the schema's owner.
    Register {
        #[arg(long, help = "Schema definition string")]
        schema: String,
        #[arg(
            long,
            help = "Resolver contract address (C...) invoked on attest/revoke"
        )]
        resolver: String,
        #[arg(long, help = "Whether attestations against this schema can be revoked")]
        revocable: bool,
        #[arg(
            long,
            help = "Owner's signing key: S... strkey seed or 32-byte hex seed",
            env = "SAS_SECRET_KEY",
            hide_env_values = true
        )]
        secret_key: String,
        #[arg(
            long,
            help = "Network passphrase to sign against",
            env = "SOROBAN_NETWORK_PASSPHRASE"
        )]
        network_passphrase: String,
        #[arg(
            long,
            help = "Schema Registry contract address (C...)",
            env = "SCHEMA_REGISTRY_CONTRACT_ID"
        )]
        registry_contract_id: String,
        #[arg(long, help = "Soroban RPC endpoint URL", env = "SOROBAN_RPC_URL")]
        rpc_url: String,
    },
    /// Get an existing schema by UID
    Get {
        #[arg(long, help = "32-byte schema UID, hex encoded")]
        uid: String,
        #[arg(
            long,
            help = "Schema Registry contract address (C...)",
            env = "SCHEMA_REGISTRY_CONTRACT_ID"
        )]
        registry_contract_id: String,
        #[arg(long, help = "Soroban RPC endpoint URL", env = "SOROBAN_RPC_URL")]
        rpc_url: String,
        #[arg(long, help = "Print raw JSON instead of a human-readable summary")]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AttestCommands {
    /// Create and submit a new on-chain attestation
    Create {
        #[arg(long, help = "JSON file containing attestation data")]
        data_file: String,
        #[arg(
            long,
            help = "Attester signing key: S... strkey seed or 32-byte hex seed",
            env = "SAS_SECRET_KEY",
            hide_env_values = true
        )]
        secret_key: String,
        #[arg(
            long,
            help = "Network passphrase to sign against",
            env = "SOROBAN_NETWORK_PASSPHRASE"
        )]
        network_passphrase: String,
        #[arg(long, help = "SAS contract address (C...)", env = "SAS_CONTRACT_ID")]
        contract_id: String,
        #[arg(long, help = "Soroban RPC endpoint URL", env = "SOROBAN_RPC_URL")]
        rpc_url: String,
    },
    /// Revoke an existing on-chain attestation
    Revoke {
        #[arg(long, help = "32-byte attestation UID, hex encoded")]
        uid: String,
        #[arg(
            long,
            help = "Attester signing key: S... strkey seed or 32-byte hex seed",
            env = "SAS_SECRET_KEY",
            hide_env_values = true
        )]
        secret_key: String,
        #[arg(
            long,
            help = "Network passphrase to sign against",
            env = "SOROBAN_NETWORK_PASSPHRASE"
        )]
        network_passphrase: String,
        #[arg(long, help = "SAS contract address (C...)", env = "SAS_CONTRACT_ID")]
        contract_id: String,
        #[arg(long, help = "Soroban RPC endpoint URL", env = "SOROBAN_RPC_URL")]
        rpc_url: String,
    },
    /// Verify an on-chain attestation's current validity
    Verify {
        #[arg(long, help = "32-byte attestation UID, hex encoded")]
        uid: String,
        #[arg(long, help = "SAS contract address (C...)", env = "SAS_CONTRACT_ID")]
        contract_id: String,
        #[arg(long, help = "Soroban RPC endpoint URL", env = "SOROBAN_RPC_URL")]
        rpc_url: String,
        #[arg(long, help = "Print raw JSON instead of a human-readable result")]
        json: bool,
    },
}

#[derive(Subcommand)]
enum QueryCommands {
    /// Query attestations by recipient address
    ByRecipient {
        #[arg(long)]
        address: String,
    },
    /// Query attestations by schema UID
    BySchema {
        #[arg(long)]
        uid: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Commands::Offchain { action }) => run_offchain(action),
        Some(Commands::Schema { action }) => run_schema(action),
        Some(Commands::Attest { action }) => run_attest(action),
        _ => {
            println!("CLI initialized");
            Ok(())
        }
    };
    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_attest(action: AttestCommands) -> Result<(), String> {
    let env = soroban_sdk::Env::default();
    match action {
        AttestCommands::Create {
            data_file,
            secret_key,
            network_passphrase,
            contract_id,
            rpc_url,
        } => {
            let raw = std::fs::read_to_string(&data_file)
                .map_err(|e| format!("cannot read {data_file}: {e}"))?;
            let input: offchain::AttestationInput =
                serde_json::from_str(&raw).map_err(|e| format!("invalid attestation JSON: {e}"))?;
            let seed = offchain::parse_secret_seed(&secret_key)?;
            let expected_attester = stellar_strkey::ed25519::PublicKey(
                soroban_sas_sdk::signature::derive_public_key(&seed),
            )
            .to_string();
            if input.attester != expected_attester {
                return Err(format!(
                    "attester {} does not match signing key account {expected_attester}",
                    input.attester
                ));
            }
            let attestation = offchain::parse_attestation(&env, &input)?;
            let rpc = soroban_sas_sdk::rpc::RpcClient::new(rpc_url);
            let client = soroban_sas_sdk::client::SASClient::new(contract_id);
            let result = client
                .attest(&env, &rpc, &network_passphrase, &seed, attestation)
                .map_err(|e| format!("{e:?}"))?;
            print_transaction_result(result)
        }
        AttestCommands::Revoke {
            uid,
            secret_key,
            network_passphrase,
            contract_id,
            rpc_url,
        } => {
            let uid = parse_uid(&uid)?;
            let seed = offchain::parse_secret_seed(&secret_key)?;
            let rpc = soroban_sas_sdk::rpc::RpcClient::new(rpc_url);
            let client = soroban_sas_sdk::client::SASClient::new(contract_id);
            let result = client
                .revoke(&env, &rpc, &network_passphrase, &seed, &uid)
                .map_err(|e| format!("{e:?}"))?;
            print_transaction_result(result)
        }
        AttestCommands::Verify {
            uid,
            contract_id,
            rpc_url,
            json,
        } => {
            let uid = parse_uid(&uid)?;
            let rpc = soroban_sas_sdk::rpc::RpcClient::new(rpc_url);
            let client = soroban_sas_sdk::client::SASClient::new(contract_id);
            let valid = client
                .verify_attestation(&env, &rpc, &uid)
                .map_err(|e| format!("{e:?}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "valid": valid }))
                        .map_err(|e| format!("serialization failed: {e}"))?
                );
            } else if valid {
                println!("Attestation is valid");
            } else {
                println!("Attestation is invalid or not found");
            }
            Ok(())
        }
    }
}

fn parse_uid(value: &str) -> Result<[u8; 32], String> {
    hex::decode(value.trim_start_matches("0x"))
        .map_err(|e| format!("invalid hex in uid: {e}"))?
        .try_into()
        .map_err(|_| "uid must be exactly 32 bytes".to_string())
}

fn print_transaction_result(
    result: soroban_sas_sdk::rpc::GetTransactionResult,
) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": result.status,
            "envelopeXdr": result.envelope_xdr,
            "resultXdr": result.result_xdr,
        }))
        .map_err(|e| format!("serialization failed: {e}"))?
    );
    Ok(())
}

fn run_schema(action: SchemaCommands) -> Result<(), String> {
    let env = soroban_sdk::Env::default();
    match action {
        SchemaCommands::Register {
            schema,
            resolver,
            revocable,
            secret_key,
            network_passphrase,
            registry_contract_id,
            rpc_url,
        } => {
            let seed = offchain::parse_secret_seed(&secret_key)?;
            let rpc = soroban_sas_sdk::rpc::RpcClient::new(rpc_url);
            let client = soroban_sas_sdk::client::SASClient::new(registry_contract_id.clone());
            let result = client
                .register_schema(
                    &env,
                    &rpc,
                    &network_passphrase,
                    &seed,
                    &registry_contract_id,
                    &schema,
                    &resolver,
                    revocable,
                )
                .map_err(|e| format!("{e:?}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": result.status,
                    "envelopeXdr": result.envelope_xdr,
                    "resultXdr": result.result_xdr,
                }))
                .map_err(|e| format!("serialization failed: {e}"))?
            );
            Ok(())
        }
        SchemaCommands::Get {
            uid,
            registry_contract_id,
            rpc_url,
            json,
        } => {
            let uid_bytes = parse_uid(&uid)?;
            let rpc = soroban_sas_sdk::rpc::RpcClient::new(rpc_url);
            let client = soroban_sas_sdk::client::SASClient::new(registry_contract_id.clone());
            let schema = client
                .get_schema(&env, &rpc, &registry_contract_id, &uid_bytes)
                .map_err(|e| format!("{e:?}"))?;

            match schema {
                None => {
                    println!("Schema not found");
                }
                Some(record) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "uid": hex::encode(record.uid.0.to_array()),
                                "resolver": soroban_string_to_std(&record.resolver.to_string()),
                                "revocable": record.revocable,
                                "schema": soroban_string_to_std(&record.schema),
                            }))
                            .map_err(|e| format!("serialization failed: {e}"))?
                        );
                    } else {
                        println!("uid:       {}", hex::encode(record.uid.0.to_array()));
                        println!(
                            "resolver:  {}",
                            soroban_string_to_std(&record.resolver.to_string())
                        );
                        println!("revocable: {}", record.revocable);
                        println!("schema:    {}", soroban_string_to_std(&record.schema));
                    }
                }
            }
            Ok(())
        }
    }
}

/// `soroban_sdk::String` (a host value) doesn't implement `Display` off-chain
/// — this copies it into a UTF-8 `std::String` for printing.
fn soroban_string_to_std(s: &soroban_sdk::String) -> String {
    let mut buf = vec![0u8; s.len() as usize];
    s.copy_into_slice(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

fn run_offchain(action: OffchainCommands) -> Result<(), String> {
    match action {
        OffchainCommands::Sign {
            data_file,
            secret_key,
            nonce,
            network_passphrase,
            contract_id,
            output,
        } => {
            let raw = std::fs::read_to_string(&data_file)
                .map_err(|e| format!("cannot read {data_file}: {e}"))?;
            let input: offchain::AttestationInput =
                serde_json::from_str(&raw).map_err(|e| format!("invalid attestation JSON: {e}"))?;
            let seed = offchain::parse_secret_seed(&secret_key)?;
            let signed = offchain::sign_offchain_attestation(
                input,
                nonce,
                &network_passphrase,
                &contract_id,
                &seed,
            )?;
            let json = serde_json::to_string_pretty(&signed)
                .map_err(|e| format!("serialization failed: {e}"))?;
            match output {
                Some(path) => {
                    std::fs::write(&path, &json).map_err(|e| format!("cannot write {path}: {e}"))?
                }
                None => println!("{json}"),
            }
            Ok(())
        }
        OffchainCommands::Verify { file } => {
            let raw =
                std::fs::read_to_string(&file).map_err(|e| format!("cannot read {file}: {e}"))?;
            let signed: offchain::SignedOffchainAttestation = serde_json::from_str(&raw)
                .map_err(|e| format!("invalid signed attestation JSON: {e}"))?;
            offchain::verify_offchain_attestation(&signed)?;
            println!("Signature is valid");
            Ok(())
        }
    }
}

#[cfg(test)]
mod test;
