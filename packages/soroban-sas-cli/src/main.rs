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

fn main() {
    let _cli = Cli::parse();
    println!("CLI initialized");
}
