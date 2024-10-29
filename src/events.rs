use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum QueryType {
    Health,
    Features,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum EventType {
    Query,
    Error,
    Reply,
    ApplyManifest,
    ApplySuccess,
    ApplyFailure,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub class: EventType,
    pub id: String,
    pub message: Option<String>,
}

impl Event {
    pub fn new(class: EventType, message: Option<String>) -> Event {
        let id = uuid::Uuid::new_v4().to_string();
        Event { class, id, message }
    }

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
