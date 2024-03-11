use std::path::PathBuf;
use crate::cli::Runnable;

pub struct Agent {
    pub config: PathBuf,
}

impl Runnable for Agent {
    fn run(&self) {
        println!("Starting agent with config: {:?}", self.config);
    }
}

