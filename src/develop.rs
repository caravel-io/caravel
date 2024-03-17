use std::path::PathBuf;

pub struct DevelopArgs {
    pub destination: PathBuf,
}

pub async fn run(args: DevelopArgs) {
    println!("Developing!");
}
