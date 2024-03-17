// Example usage
// let f = File::new("/tmp/test.txt")
//          .owner("root")
//          .group("root")
//          .mode("0644")
//          .content("Hello, World!");

use std::path::PathBuf;

use crate::manifest::Resource;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum FileState {
    Absent,
    Present,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub path: PathBuf,
    pub state: FileState,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub mode: Option<String>,
    pub source: Option<PathBuf>,
    pub content: Option<String>,
    pub force: Option<bool>,
    pub backup: Option<String>,
}

impl File {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        File {
            state: FileState::Present,
            path: path.into(),
            owner: None,
            group: None,
            mode: None,
            source: None,
            content: None,
            force: None,
            backup: None,
        }
    }
    pub fn state(mut self, state: FileState) -> Self {
        self.state = state;
        self
    }
    pub fn owner(mut self, owner: &str) -> Self {
        self.owner = Some(owner.to_string());
        self
    }
    pub fn group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }
    pub fn mode(mut self, mode: &str) -> Self {
        self.mode = Some(mode.to_string());
        self
    }
    pub fn source(mut self, source: &str) -> Self {
        self.source = Some(source.into());
        self
    }
    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }
    pub fn force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }
    pub fn backup(mut self, backup: &str) -> Self {
        self.backup = Some(backup.to_string());
        self
    }
}

#[typetag::serde]
impl Resource for File {
    /// Apply the resource to the system.
    fn apply(&self) -> Result<()> {
        println!("pretending to create file: {:?}", self.path);
        Ok(())
    }
}
