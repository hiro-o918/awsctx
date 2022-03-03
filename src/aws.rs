use crate::creds::Credentials;
use crate::ctx;

use anyhow::{anyhow, Result};
use dirs::home_dir;
use skim::prelude::{unbounded, SkimOptionsBuilder};
use skim::{Skim, SkimItemReceiver, SkimItemSender};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub struct AWS {
    credentials_path: PathBuf,
}

impl Default for AWS {
    fn default() -> Self {
        let mut path = home_dir().unwrap();
        path.push(".aws/credentials");
        Self {
            credentials_path: path,
        }
    }
}

impl ctx::CTX for AWS {
    fn list_contexts(&self) -> Result<Vec<ctx::Context>, ctx::CTXError> {
        let creds = Credentials::load_credentials(&self.credentials_path)?;
        let profiles = creds.list_profiles();
        Ok(profiles
            .iter()
            .map(|p| ctx::Context {
                name: p.name.clone(),
                active: p.default,
            })
            .collect())
    }

    fn use_context(&self, name: &str) -> Result<ctx::Context, ctx::CTXError> {
        let mut creds = Credentials::load_credentials(&self.credentials_path)?;
        let profile = creds.set_default_profile(name)?;
        creds.dump_credential(&self.credentials_path)?;
        Ok(ctx::Context {
            name: profile.name,
            active: profile.default,
        })
    }

    fn use_context_interactive(&self) -> Result<ctx::Context, ctx::CTXError> {
        let mut contexts = self.list_contexts().unwrap();
        // skim shows reverse order
        contexts.reverse();
        let options = SkimOptionsBuilder::default()
            .height(Some("30%"))
            .multi(false)
            .build()
            .unwrap();

        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
        for context in contexts {
            let _ = tx_item.send(Arc::new(context));
        }
        drop(tx_item);

        let selected_items = Skim::run_with(&options, Some(rx_item))
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);
        let item = selected_items
            .get(0)
            .ok_or(ctx::CTXError::InvalidArgument {
                source: anyhow!("no context is selected"),
            })?;
        (*item)
            .as_any()
            .downcast_ref::<ctx::Context>()
            .cloned()
            .ok_or(ctx::CTXError::UnexpectedError {
                source: anyhow!("unexpected error"),
            })
    }
}
