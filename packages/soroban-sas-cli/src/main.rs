use clap::{Parser, Subcommand};

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
    let _cli = Cli::parse();
    println!("CLI initialized");
}

#[cfg(test)]
mod test;
