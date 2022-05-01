use crate::configs::Configs;
use crate::creds::Credentials;
use crate::ctx;

use dirs::home_dir;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde_json::json;
use skim::prelude::{unbounded, Key, SkimOptionsBuilder};
use skim::{Skim, SkimItemReceiver, SkimItemSender};

static CREDENTIALS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = home_dir().unwrap();
    path.push(".aws/credentials");
    path
});

#[derive(Debug)]
pub struct AWS<'a> {
    configs: Rc<Configs>,
    credentials_path: PathBuf,
    reg: Handlebars<'a>,
}

impl AWS<'_> {
    pub fn new(configs: Rc<Configs>) -> Result<Self> {
        Ok(Self {
            configs,
            credentials_path: CREDENTIALS_PATH.clone(),
            reg: Handlebars::new(),
        })
    }
}

impl ctx::CTX for AWS<'_> {
    fn auth(&self, profile: &str) -> Result<ctx::Context, ctx::CTXError> {
        let script_template = self.configs.auth_commands.get(profile).ok_or_else(|| {
            ctx::CTXError::NoAuthConfiguration {
                profile: profile.to_string(),
            }
        })?;

        let script = self
            .reg
            .render_template(script_template, &json!({ "profile": profile }))
            .map_err(|e| ctx::CTXError::InvalidConfigurations {
                message: format!("failed to render script of profile {}", profile),
                source: anyhow!("failed to render script {}", e),
            })?;

        let status = Command::new("sh")
            .arg("-c")
            .arg(script)
            .status()
            .map_err(|e| ctx::CTXError::UnexpectedError {
                source: anyhow!("failed to execute an auth script: {}", e),
            })?;
        if !status.success() {
            return Err(ctx::CTXError::UnexpectedError {
                source: anyhow!("failed to run auth script, check output logs"),
            });
        }
        self.use_context(profile)
    }

    fn list_contexts(&self) -> Result<Vec<ctx::Context>, ctx::CTXError> {
        let creds = Credentials::load_credentials(&self.credentials_path)?;
        let profiles = creds.list_profiles();
        Ok(profiles
            .into_iter()
            .map(|p| ctx::Context {
                name: p.name,
                active: p.default,
            })
            .collect())
    }

    fn use_context(&self, name: &str) -> Result<ctx::Context, ctx::CTXError> {
        let mut creds = Credentials::load_credentials(&self.credentials_path)?;
        let profile = creds.set_default_profile(name)?;
        creds.dump_credentials(&self.credentials_path)?;
        Ok(ctx::Context {
            name: profile.name,
            active: profile.default,
        })
    }

    fn use_context_interactive(&self) -> Result<ctx::Context, ctx::CTXError> {
        let mut contexts = self.list_contexts()?;
        // skim shows reverse order
        contexts.reverse();
        let options = SkimOptionsBuilder::default()
            .height(Some("30%"))
            .multi(false)
            .build()
            .map_err(|err| ctx::CTXError::UnexpectedError {
                source: anyhow!(err),
            })?;

        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
        for context in contexts {
            let _ = tx_item.send(Arc::new(context));
        }
        drop(tx_item);

        let selected_items = Skim::run_with(&options, Some(rx_item))
            .map(|out| match out.final_key {
                Key::Enter => Ok(out.selected_items),
                _ => Err(ctx::CTXError::NoContextIsSelected {}),
            })
            .unwrap_or(Ok(Vec::new()))?;
        let item = selected_items
            .get(0)
            .ok_or(ctx::CTXError::NoContextIsSelected {})?;
        let context = (*item)
            .as_any()
            .downcast_ref::<ctx::Context>()
            .cloned()
            .ok_or(ctx::CTXError::UnexpectedError {
                source: anyhow!("unexpected error"),
            })?;
        self.use_context(&context.name)
    }
}
