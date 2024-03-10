use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum QueryType {
    Health,
    Features,
}

#[derive(Serialize, Deserialize)]
pub enum EventType {
    Query(QueryType),
    Apply(Manifest),
    Error(String),
    Reply(String),
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub class: EventType,
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub resources: Vec<Box<dyn Resource>>,
}

#[typetag::serde]
pub trait Resource {
    fn apply(&self) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[typetag::serde]
impl Resource for File {
    fn apply(&self) -> Result<()> {
        // let mut file = std::fs::File::create(&self.name)?;
        // file.write_all(self.content.as_bytes())?;
        // Ok(())
        println!("pretending to create file: {}", self.name);
        Ok(())
    }
}

impl Event {
    pub fn write_to_stdout(&self) -> Result<()> {
        let event_s = serde_json::to_string(&self)?;
        let _ = io::stdout().write(event_s.as_bytes());
        Ok(())
    }

    pub fn write_to_stderr(&self) -> Result<()> {
        let event_s = serde_json::to_string(&self)?;
        let _ = io::stderr().write(event_s.as_bytes());
        Ok(())
    }
}
