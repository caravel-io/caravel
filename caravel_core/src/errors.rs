use thiserror::Error;

pub enum RunnableError {
    InvalidManifest,
    InvalidModule,
    InvalidConfig,
    InvalidInventory,
    InvalidTarget,
    InvalidGroup,
    InvalidPath,
    InvalidDestination,
}

#[derive(Error, Debug, Clone, Eq, PartialEq)]
#[error("Invalid agent config path {0}")]
pub struct InvalidAgentConfigPath(String);
