use thiserror::Error;
pub mod view;

pub trait CTXM {
    fn list_contexts(&self) -> Result<Vec<String>, CTXMError>;
    fn use_context(&self, name: &str) -> Result<String, CTXMError>;
    fn use_context_interactive(&self) -> ();
}

#[derive(Error, Debug)]
pub enum CTXMError {
    #[error("Cannot read configuration")]
    CannotReadConfiguration { source: anyhow::Error },
    #[error("Configuration is broken")]
    ConfigurationIsBroken { source: anyhow::Error },
    #[error("Invalid input")]
    InvalidArgument { source: anyhow::Error },
    #[error("IOError")]
    IOError { source: anyhow::Error },
}
