use std::path::PathBuf;

pub struct AgentArgs {
    pub config: PathBuf,
}

pub fn run(args: AgentArgs) {
    println!("Starting agent with config: {:?}", args.config);
}
