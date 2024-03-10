use crate::agent::AgentArgs;
use crate::client::ClientArgs;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Caravel is the best thing since sliced bread
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ship a manifest (client mode)
    Ship {
        /// The manifest to ship
        #[arg()]
        manifest: PathBuf,

        /// Target hosts
        #[arg(short, long)]
        targets: Option<Vec<String>>,

        /// Target groups from inventory
        #[arg(short, long)]
        groups: Option<Vec<String>>,

        /// Inventory file
        #[arg(short, long)]
        inventory: Option<PathBuf>,
    },
    /// Run as an agent
    Agent {
        /// Config file path for agent mode
        #[arg()]
        config: PathBuf,
    },
}

pub enum CaravelArgs {
    Client(ClientArgs),
    Agent(AgentArgs),
}

pub fn run() -> CaravelArgs {
    let args = Cli::parse();

    match &args.command {
        Commands::Ship {
            manifest,
            targets,
            groups,
            inventory,
        } => CaravelArgs::Client(ClientArgs {
            manifest: manifest.clone(),
            targets: targets.clone(),
            groups: groups.clone(),
            inventory: inventory.clone(),
        }),
        Commands::Agent { config } => CaravelArgs::Agent(AgentArgs {
            config: config.clone(),
        }),
    }
}
