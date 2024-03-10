use caravel::cli;

fn main() {
    let args = cli::run();

    match args {
        cli::CaravelArgs::Client(client_args) => {
            println!("Running client!");
            println!("Manifest: {:?}", client_args.manifest);
            println!("Targets: {:?}", client_args.targets);
            println!("Groups: {:?}", client_args.groups);
            println!("Inventory: {:?}", client_args.inventory);
        }
        cli::CaravelArgs::Agent(agent_args) => {
            println!("Running agent!");
            println!("Config: {:?}", agent_args.config);
        }
    }
}

