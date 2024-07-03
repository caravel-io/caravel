use caravel::agent::Agent;
use caravel::client::Client;
use caravel::module::{CreateModule, ValidateModule};
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
        ///
        /// Caravel expects the following working directory structure.
        /// .
        /// ├── caravel_modules
        /// │   ├── Module1.so
        /// │   └── Module2.so
        /// ├── lua_libs
        /// │   ├── lualib1.lua
        /// │   └── lualib2.lua
        /// └── manifest_entrypoint.lua
        #[clap(verbatim_doc_comment)]
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
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Module actions
    Module {
        /// Module action
        #[command(subcommand, name = "action")]
        action: ModuleAction,
    },
}

#[derive(Clone, Debug, Subcommand)]
enum ModuleAction {
    /// Bootstrap a new module directory for development
    New {
        /// Destination directory
        #[arg(name = "path")]
        destination: PathBuf,
    },
    /// Validate a module directory
    Validate {
        /// Destination directory
        #[arg(short, long)]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Ship {
            manifest,
            targets,
            groups,
            inventory,
        } => Client {
            manifest: manifest.clone(),
            targets: targets.clone(),
            groups: groups.clone(),
            inventory: inventory.clone(),
        }
        .run()
        .await
        .unwrap(),

        Commands::Agent { config } => Agent {
            config_path: config.clone(),
        }
        .run()
        .await
        .expect("Failed to run agent"),

        Commands::Module { action } => match action {
            ModuleAction::New { destination } => CreateModule {
                destination: destination.clone(),
            }
            .run()
            .await
            .unwrap(),
            ModuleAction::Validate { path } => {
                ValidateModule { path: path.clone() }.run().await.unwrap()
            }
        },
    }
}
