use dirs::home_dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use config::{Config, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

type ProfileName = String;
type AuthScript = String;

static CONFIGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = home_dir().unwrap();
    path.push(".awsctx/config.yaml");
    path
});
const CONFIGS_DESCRIPTIONS: &str = r#"# Configurations for awsctx 
# You can manually edit configurations according to the following usage
#
# auth_commands: 
#   # configuration for `foo` profile with aws configure
#   foo: |
#     # you can use pre-defined parameter `{{profile}}` which is replaced by key of this block
#     # In this case, `{{profile}}` is replaced by `foo`
#     aws configure --profile {{profile}}
#   # configuration for `bar` profile with [onelogin-aws-cli](https://github.com/physera/onelogin-aws-cli)
#   bar: |
#     # In this case, name of one-login configuration is same as `profile`
#     onelogin-aws-login -C {{profile}} --profile {{profile}} -u user@example.com
"#;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Configs {
    pub auth_commands: HashMap<ProfileName, AuthScript>,
}

impl Configs {
    pub fn load_configs<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let path = path
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| CONFIGS_PATH.clone());
        let c = Config::builder()
            .add_source(File::with_name(path.to_str().unwrap()))
            .build()
            .context(format!(
                "failed to build configuration from path: {}",
                path.to_str().unwrap()
            ))?;

        c.try_deserialize().context(format!(
            "failed to deserialize configuration from path: {}",
            path.to_str().unwrap()
        ))
    }

    pub fn initialize_default_configs() -> Result<Self> {
        let path: &PathBuf = &CONFIGS_PATH;
        if path.exists() {
            return Self::load_configs(Some(path));
        }
        // if the config directory does not exist, create the directory recursively
        match path.parent() {
            Some(parent) => fs::create_dir_all(parent)
                .context("failed to create a directory of a configuration file")?,
            None => (),
        }
        let c = Configs::default();
        let mut file = fs::File::create(path).context("failed to create a configuration file")?;
        file.write_all(CONFIGS_DESCRIPTIONS.as_bytes())
            .context("failed to write a configuration file")?;
        let mut ser = serde_yaml::Serializer::new(&mut file);
        c.serialize(&mut ser)
            .context("failed to write a configuration file")?;
        file.flush()
            .context("failed to flush a configuration file")?;

        Self::load_configs(Some(path))
    }
}
