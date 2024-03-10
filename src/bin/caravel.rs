use caravel::cli;

fn main() {
    let args = cli::run();

    match args {
        cli::CaravelArgs::Client(client_args) => {
            caravel::client::run(client_args);
        }
        cli::CaravelArgs::Agent(agent_args) => {
            caravel::agent::run(agent_args);
        }
    }
}

