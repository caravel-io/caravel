use std::path::PathBuf;

pub struct ClientArgs {
    pub manifest: PathBuf,
    pub targets: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub inventory: Option<PathBuf>,
}

pub fn run(args: ClientArgs) {
    println!("Running client!");
    println!("Manifest: {:?}", args.manifest);
    println!("Targets: {:?}", args.targets);
    println!("Groups: {:?}", args.groups);
    println!("Inventory: {:?}", args.inventory);
}
