use crate::manifest::Manifest;
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
    pub id: String,
    pub payload: Option<String>,
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
