use std::path::PathBuf;

pub struct DevelopArgs {
    pub destination: PathBuf,
}

pub fn run(args: DevelopArgs) {
    println!("Developing!");
}
