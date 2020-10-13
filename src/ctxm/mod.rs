use thiserror::Error;
pub mod view;

pub trait CTXM {
    fn list_contexts(&self) -> Result<Vec<String>, CTXMError>;
}

#[derive(Error, Debug)]
pub enum CTXMError {
    #[error("Cannot read configuration")]
    CannotReadConfiguration { source: anyhow::Error },
    #[error("configuration is broken")]
    ConfigurationIsBroken { source: anyhow::Error },
}
