use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::prelude::*;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum QueryType {
    Health,
    Features,
}

/// Event types that can be sent between the host and the agent
/// These are the primary way the host and agent communicate
/// with each other.
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum EventType {
    /// Generic error event
    Error,
    // Reply,
    // ApplyManifest,
    // ApplySuccess,
    // ApplyFailure,

    /// The initial event sent by the agent to the host to gather 
    /// capabilities in preparation to ship a manifest..
    Query,

    /// The agent responds to the host with a list of capabilities
    /// that it supports, such as version information, installed modules, etc.
    /// Also returns a token that can be used with the remainder of the types
    /// of requests.
    Capabilities {
        token: String
    },

    /// The host compares the capabilities against the depdenencies of the manifest,
    /// gathers missing modules, and sends a gzip-compressed tarball of the modules
    /// to the agent on its /upload endpoint using token authentication.
    Dependencies,

    /// The agent receives the tarball of modules, extracts them, and sends an
    /// Assembed event back to the host when it's ready to apply the manifest.
    Assembled,

    /// The host sends the ApplyManifest event to the agent, which then applies
    /// the manifest to the system.
    ApplyManifest,

    /// Success event sent by the agent to the host when the manifest has been
    /// successfully applied.
    ApplySuccess,
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Invalid event format")]
    InvalidEvent,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum EventPayload {
    /// A manifest that the agent will apply in bytes
    Manifest(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub class: EventType,
    pub id: String,
    pub message: Option<String>,
    pub payload: Option<EventPayload>,
}

impl Event {
    pub fn new(class: EventType) -> Event {
        let id = uuid::Uuid::new_v4().to_string();
        Event {
            class,
            id,
            message: None,
            payload: None,
        }
    }

    pub fn message(mut self, message: String) -> Event {
        self.message = Some(message);
        self
    }

    pub fn payload(mut self, payload: EventPayload) -> Event {
        self.payload = Some(payload);
        self
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
