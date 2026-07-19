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
    /// Register a new schema
    Register {
        #[arg(long)]
        schema: String,
    },
    /// Get an existing schema by UID
    Get {
        #[arg(long)]
        uid: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AttestCommands {
    /// Create a new attestation
    Create {
        #[arg(long, help = "JSON file containing attestation data")]
        data_file: Option<String>,
    },
    /// Revoke an existing attestation
    Revoke {
        #[arg(long)]
        uid: String,
    },
    /// Verify an attestation offline
    Verify {
        #[arg(long)]
        uid: String,
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
    match cli.command {
        Some(Commands::Offchain { action }) => {
            if let Err(err) = run_offchain(action) {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        _ => println!("CLI initialized"),
    }
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
