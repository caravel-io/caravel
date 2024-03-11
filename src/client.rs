use crate::cli::Runnable;
use std::path::PathBuf;

pub struct Client {
    pub manifest: PathBuf,
    pub targets: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub inventory: Option<PathBuf>,
}

impl Runnable for Client {
    fn run(&self) {
        println!("Running client!");
        println!("Manifest: {:?}", self.manifest);
        println!("Targets: {:?}", self.targets);
        println!("Groups: {:?}", self.groups);
        println!("Inventory: {:?}", self.inventory);
    }
}
