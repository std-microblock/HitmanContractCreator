#![recursion_limit = "256"]

use std::env;

use log::{debug, info, trace, warn};
mod contract;

use clap::{Parser, Subcommand};
use serde_json::Value;
use tokio::{runtime::Handle, task::block_in_place};

use crate::contract::Contract;

// Hitman Contract Submitter
#[derive(Parser, Debug)]
#[clap(author="MicroBlock", version="1.0.0", about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Debug, Clone,Copy)]
pub enum PublishTypes {
    HITMAN2,
    HITMAN3,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Publish contracts
    #[clap(arg_required_else_help = true)]
    Publish {
        /// The file to submit
        #[clap(value_parser)]
        file: String,
        // The User's ID
        #[clap(value_parser)]
        userid: String,
        // Bearer for auth reasons.
        #[clap(long, value_parser)]
        bearer: String,

        // Publish to hitman2
        #[clap(long)]
        hitman2: bool,

        // Publish to hitman3
        #[clap(long)]
        hitman3: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_BACKTRACE", "1");
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("reqwest", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;

    let args = Args::parse();

    use std::fs;
    match args.command {
        Commands::Publish {
            file,
            bearer,
            userid,
            hitman2,
            hitman3,
        } => {
            let contract = fs::read_to_string(file)?;
            let contract: Value = serde_json::from_str(contract.as_str())?;
            let mut contract = Contract::from_contract_json(
                contract,
                if hitman2 {
                    PublishTypes::HITMAN2
                } else {
                    PublishTypes::HITMAN3
                },
            )?;
            contract.publish_contract(&userid, &bearer).await?;
        }
    }

    Ok(())
}
