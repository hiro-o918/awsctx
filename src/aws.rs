use crate::configs::Configs;
use crate::creds::Credentials;
use crate::ctx;

use dirs::home_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde_json::json;
use skim::prelude::{unbounded, Key};
use skim::{Skim, SkimItemReceiver, SkimItemSender, SkimOptions};

pub static CREDENTIALS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = home_dir().unwrap();
    path.push(".aws/credentials");
    path
});

#[derive(Debug)]
pub struct AWS<'a, P: AsRef<Path>> {
    configs: Rc<Configs>,
    credentials_path: P,
    reg: Handlebars<'a>,
}

impl<P: AsRef<Path>> AWS<'_, P> {
    pub fn new(configs: Rc<Configs>, credentials_path: P) -> Result<Self> {
        Ok(Self {
            configs,
            credentials_path,
            reg: Handlebars::new(),
        })
    }
}

impl<P: AsRef<Path>> ctx::CTX for AWS<'_, P> {
    fn auth(&self, profile: &str) -> Result<ctx::Context, ctx::CTXError> {
        let script_template = self.configs.auth_commands.get(profile).ok_or_else(|| {
            ctx::CTXError::NoAuthConfiguration {
                profile: profile.to_string(),
                source: None,
            }
        })?;

        let script = self
            .reg
            .render_template(script_template, &json!({ "profile": profile }))
            .map_err(|e| ctx::CTXError::InvalidConfigurations {
                message: format!("failed to render script of profile {}", profile),
                source: Some(anyhow!("failed to render script {}", e)),
            })?;

        let status = Command::new("sh")
            .arg("-c")
            .arg(script)
            .status()
            .map_err(|e| ctx::CTXError::InvalidConfigurations {
                message: format!(
                    "failed to execute an auth script of profile ({}), check configurations",
                    profile
                ),
                source: Some(anyhow!("failed to execute an auth script: {}", e)),
            })?;
        if !status.success() {
            return Err(ctx::CTXError::InvalidConfigurations {
                message: format!(
                    "failed to execute an auth script of profile ({}), check configurations",
                    profile
                ),
                source: Some(anyhow!("failed to run auth script, check output logs")),
            });
        }
        self.use_context(profile)
    }

    fn list_contexts(&self) -> Result<Vec<ctx::Context>, ctx::CTXError> {
        let creds = Credentials::load_credentials(&self.credentials_path)?;
        Ok(creds
            .list_profiles()
            .into_iter()
            .map(|p| ctx::Context {
                name: p.name.to_string(),
                active: p.default,
            })
            .collect())
    }

    fn get_active_context(&self) -> Result<ctx::Context, ctx::CTXError> {
        let creds = Credentials::load_credentials(&self.credentials_path)?;
        creds.get_default_profile().map(|p| ctx::Context {
            name: p.name.to_string(),
            active: p.default,
        })
    }

    fn use_context(&self, name: &str) -> Result<ctx::Context, ctx::CTXError> {
        let mut creds = Credentials::load_credentials(&self.credentials_path)?;
        let profile = creds.set_default_profile(name)?;
        creds.dump_credentials(&self.credentials_path)?;
        Ok(ctx::Context {
            name: profile.name.to_string(),
            active: profile.default,
        })
    }

    fn use_context_interactive(
        &self,
        skim_options: SkimOptions,
    ) -> Result<ctx::Context, ctx::CTXError> {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
        // skim shows reverse order
        for context in self.list_contexts()?.into_iter().rev() {
            tx_item
                .send(Arc::new(context))
                .context("failed to send an item to skim")
                .map_err(|e| ctx::CTXError::UnexpectedError { source: Some(e) })?;
        }
        drop(tx_item);

        let selected_items = Skim::run_with(&skim_options, Some(rx_item))
            .map(|out| match out.final_key {
                Key::Enter => Ok(out.selected_items),
                _ => Err(ctx::CTXError::NoContextIsSelected { source: None }),
            })
            .unwrap_or(Ok(Vec::new()))?;
        let item = selected_items
            .get(0)
            .ok_or(ctx::CTXError::NoContextIsSelected { source: None })?;
        let context = (*item).as_any().downcast_ref::<ctx::Context>().ok_or(
            ctx::CTXError::UnexpectedError {
                source: Some(anyhow!("unexpected error")),
            },
        )?;
        self.use_context(&context.name)
    }
}
