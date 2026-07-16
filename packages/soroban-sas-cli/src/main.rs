use clap::{Parser};

#[derive(Parser)]
#[command(name = "soroban-sas")]
#[command(about = "CLI for Soroban Attestation Service")]
struct Cli {
    #[arg(long, global = true, help = "RPC Network to connect to")]
    network: Option<String>,

    #[arg(long, global = true, help = "Identity to use for signing")]
    identity: Option<String>,
}

fn main() {
    let _cli = Cli::parse();
    println!("CLI initialized");
}
