use std::borrow::Cow;

use skim::SkimItem;
use thiserror::Error;

pub trait CTX {
    fn auth(&self, profile: &str) -> Result<Context, CTXError>;
    fn list_contexts(&self) -> Result<Vec<Context>, CTXError>;
    fn use_context(&self, profile: &str) -> Result<Context, CTXError>;
    fn use_context_interactive(&self) -> Result<Context, CTXError>;
}

#[derive(Error, Debug)]
pub enum CTXError {
    #[error("No auth configuration found for the profile")]
    NoAuthConfiguration { profile: String },
    #[error("Invalid configurations")]
    InvalidConfigurations {
        message: String,
        source: anyhow::Error,
    },
    #[error("Cannot read configuration")]
    CannotReadCredentials { source: anyhow::Error },
    #[error("Configuration is broken")]
    CredentialsIsBroken { source: anyhow::Error },
    #[error("Invalid input")]
    InvalidArgument { source: anyhow::Error },
    #[error("No context is selected")]
    NoContextIsSelected {},
    #[error("Unexpected error")]
    UnexpectedError { source: anyhow::Error },
}

#[derive(Default, Clone, Debug)]
pub struct Context {
    pub name: String,
    pub active: bool,
}

impl SkimItem for Context {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }
}
