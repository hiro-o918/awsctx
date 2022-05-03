use std::borrow::Cow;

use skim::{SkimItem, SkimOptions};
use thiserror::Error;

pub trait CTX {
    fn auth(&self, profile: &str) -> Result<Context, CTXError>;
    fn list_contexts(&self) -> Result<Vec<Context>, CTXError>;
    fn get_active_context(&self) -> Result<Context, CTXError>;
    fn use_context(&self, profile: &str) -> Result<Context, CTXError>;
    fn use_context_interactive(&self, skim_options: SkimOptions) -> Result<Context, CTXError>;
}

#[derive(Error, Debug)]
pub enum CTXError {
    #[error("Cannot read configuration")]
    CannotReadCredentials { source: Option<anyhow::Error> },
    #[error("Cannot write configuration")]
    CannotWriteCredentials { source: Option<anyhow::Error> },
    #[error("Configuration is broken")]
    CredentialsIsBroken { source: Option<anyhow::Error> },
    #[error("Invalid configurations")]
    InvalidConfigurations {
        message: String,
        source: Option<anyhow::Error>,
    },
    #[error("No active context found")]
    NoActiveContext { source: Option<anyhow::Error> },
    #[error("No auth configuration found for the profile")]
    NoAuthConfiguration {
        profile: String,
        source: Option<anyhow::Error>,
    },
    #[error("No context is selected")]
    NoContextIsSelected { source: Option<anyhow::Error> },
    #[error("No such profile")]
    NoSuchProfile {
        profile: String,
        source: Option<anyhow::Error>,
    },
    #[error("Unexpected error")]
    UnexpectedError { source: Option<anyhow::Error> },
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub name: String,
    pub active: bool,
}

impl SkimItem for Context {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }
}
