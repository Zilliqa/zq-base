#![allow(unused_imports)]
use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use ethers::abi::AbiEncode;
use ethers::core::rand::thread_rng;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::H160;
use serde_json;
use std::env;
use zutils::{bq, queries};

#[derive(Parser, Debug)]
#[clap(about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct PrivKey {
    privkey: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // Get the addresses for a privkey
    Addresses(PrivKey),
    // Generate some accounts
    Accounts(AccountParams),
    // Ask BigQuery something
    #[clap(name = "bigquery")]
    BigQuery(BQParams),
}

/// These are the only two networks for which we have datasets.
#[derive(ValueEnum, Subcommand, Debug, Clone, PartialEq, Eq)]
enum BQNetwork {
    Mainnet,
    Testnet,
}

#[derive(Debug, Subcommand, Clone)]
enum BQQuery {
    Involved(Address),
}

#[derive(Debug, Args, Clone)]
struct AccountParams {
    #[clap(subcommand)]
    subcommand: AccountSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
enum AccountSubcommand {
    Generate(AccountNum),
}

#[derive(Args, Debug, Clone)]
struct AccountNum {
    accounts: u32,
}

#[derive(Args, Debug, Clone)]
struct Address {
    address: H160,
}

#[derive(Args, Debug)]
struct BQParams {
    network: BQNetwork,
    #[clap(subcommand)]
    query: BQQuery,
}

/// Returns the project_id and DSN
fn bq_params_from_network(net: &BQNetwork) -> Result<bq::Location> {
    match net {
        BQNetwork::Testnet => Ok(bq::Location::new(
            "prj-c-data-analytics-3xs14wez",
            "ds_zq1_testnet",
        )),
        BQNetwork::Mainnet => Ok(bq::Location::new(
            "prj-c-data-analytics-3xs14wez",
            "ds_zq1_mainnet",
        )),
    }
}

impl Cli {
    async fn addresses_for_privkey(&self, pk: &str) -> Result<()> {
        let wallet = pk.parse::<LocalWallet>()?;
        println!("eth: {0:#020x}", wallet.address());
        Ok(())
    }

    async fn query_involved(&self, bq: &bq::BQ, address: &H160) -> Result<()> {
        let result = queries::query_involved(bq, address).await?;
        let output = serde_json::to_string(&result)?;
        println!("{0}", output);
        Ok(())
    }

    async fn generate_accounts(&self, _nr: u32) -> Result<()> {
        let wallet = LocalWallet::new(&mut thread_rng());
        let priv_key = wallet.signer().to_bytes();
        println!("Private key: {0:#020x}", &priv_key);
        println!("Address    : {0:#020x}", wallet.address());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_from(env::args());

    match &cli.command {
        Commands::Addresses(s) => cli.addresses_for_privkey(&s.privkey).await?,
        Commands::BigQuery(q) => {
            let bq_loc = bq_params_from_network(&q.network)?;
            let bq = bq::BQ::new(&bq_loc).await?;
            match &q.query {
                BQQuery::Involved(addr) => cli.query_involved(&bq, &addr.address).await?,
            }
        }
        Commands::Accounts(q) => match &q.subcommand {
            AccountSubcommand::Generate(an) => cli.generate_accounts(an.accounts).await?,
        },
    }
    Ok(())
}
