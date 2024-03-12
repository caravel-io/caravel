use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub resources: Vec<Box<dyn Resource>>,
}

#[typetag::serde()]
pub trait Resource {
    fn apply(&self) -> Result<()>;
}