use crate::ctx;

pub fn fatal_ctxerr<T>(result: Result<T, ctx::CTXError>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => match e {
            ctx::CTXError::CannotReadCredentials { source } => {
                error!("<red>failed to read credentials, check your ~/.aws/credentials file</>");
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::CredentialsIsBroken { source } => {
                error!("<red>broken credentials, check your ~/.aws/credentials file</>");
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::InvalidConfigurations { message, source } => {
                error!("<red>invalid configurations: {}</>", message);
                error!("");
                error!("modify ~/.awsctx/configs.yaml manually and try again");
                error!("<bold>Example Usage</>: <u>https://github.com/hiro-o918/awsctx/tree/v{}#configs.yaml</>", env!("CARGO_PKG_VERSION"));
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::NoActiveContext { source } => {
                info!("<red>no active context</>");
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::NoAuthConfiguration { profile, source } => {
                error!(
                    "<red>no auth configuration found for the profile: {}</>",
                    profile
                );
                error!("");
                error!("modify ~/.awsctx/configs.yaml manually and try again");
                error!("<bold>Example Usage</>: <u>https://github.com/hiro-o918/awsctx/tree/v{}#configs.yaml</>", env!("CARGO_PKG_VERSION"));
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::NoContextIsSelected { source } => {
                error!("<red>no context is selected</>");
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::NoSuchProfile { profile, source } => {
                error!(
                    "<red>no such profile: {}, check your ~/.aws/credentials file</>",
                    profile
                );
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
            ctx::CTXError::UnexpectedError { source } => {
                error!("<red>unexpected error occurred, you can check detailed error by `verbose` option</>");
                if let Some(source) = source {
                    debug!("caused error: {:?}", source);
                }
                std::process::exit(1);
            }
        },
    }
}

pub fn show_contexts(contexts: &[ctx::Context]) {
    for c in contexts.iter() {
        if c.active {
            info!("<green>* {}</>", c.name);
        } else {
            info!("  {}", c.name);
        }
    }
}

pub fn show_context(contexts: &ctx::Context) {
    info!("{}", contexts.name)
}
