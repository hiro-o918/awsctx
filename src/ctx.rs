use std::borrow::Cow;

use skim::SkimItem;
use thiserror::Error;

pub trait CTX {
    fn list_contexts(&self) -> Result<Vec<Context>, CTXError>;
    fn use_context(&self, name: &str) -> Result<Context, CTXError>;
    fn use_context_interactive(&self) -> Result<Context, CTXError>;
}

#[derive(Error, Debug)]
pub enum CTXError {
    #[error("Cannot read configuration")]
    CannotReadConfiguration { source: anyhow::Error },
    #[error("Configuration is broken")]
    ConfigurationIsBroken { source: anyhow::Error },
    #[error("Invalid input")]
    InvalidArgument { source: anyhow::Error },
    #[error("Unknown context name")]
    UnknownContextName { source: anyhow::Error },
    #[error("IOError")]
    IOError { source: anyhow::Error },
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
