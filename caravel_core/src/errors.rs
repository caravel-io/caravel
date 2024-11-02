use thiserror::Error;

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum RunnableError {
    #[error("Invalid manifest")]
    InvalidManifest,
    #[error("Invalid module")]
    InvalidModule,
    #[error("Invalid module path")]
    InvalidConfig,
    #[error("Invalid inventory")]
    InvalidInventory,
    #[error("Invalid target")]
    InvalidTarget,
    #[error("Invalid group")]
    InvalidGroup,
    #[error("Invalid agent config")]
    InvalidPath,
    #[error("Invalid destination")]
    InvalidDestination,
}

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum ClientError {
    #[error("Manifest not found at path {0}")]
    ManifestNotFound(String),
    #[error("You must provide either targets or groups!")]
    TargetsOrGroupsRequired,
}

#[derive(Error, Debug, Clone, Eq, PartialEq)]
#[error("Invalid agent config path {0}")]
pub struct InvalidAgentConfigPath(String);
