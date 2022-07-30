use dirs::home_dir;
use maplit::hashmap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Context, Result};
use config::{Config, File, FileFormat};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::ctx;

type ProfileName = String;
type AuthScript = String;

pub static CONFIGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = home_dir().unwrap();
    path.push(".awsctx/configs.yaml");
    path
});

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Configs {
    pub auth_commands: HashMap<ProfileName, AuthScript>,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            auth_commands: hashmap! {
            Self::DEFAULT_AUTH_COMMAND_KEY.to_string()  => r#"echo "This is default configuration for auth commands."
echo "You can edit this configuration on ~/.awsctx/configs.yaml according to your needs."
aws configure --profile {{profile}}
"#.to_string(),
                },
        }
    }
}

impl Configs {
    const CONFIGS_DESCRIPTIONS: &'static str = r#"# # Configurations for awsctx
# # You can manually edit configurations according to the following usage

# # To use subcommand `auth` or `refresh`, fill the below configs for each profile.
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
#   # default configuration for profiles without auth configuration
#   __default: |
#     aws configure --profile {{profile}}
"#;

    pub const DEFAULT_AUTH_COMMAND_KEY: &'static str = "__default";

    pub fn load_configs<P: AsRef<Path>>(path: Option<P>) -> Result<Self, ctx::CTXError> {
        let path = path
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| CONFIGS_PATH.clone());
        let c = Config::builder()
            .add_source(File::new(path.to_str().unwrap(), FileFormat::Yaml))
            .build()
            .context(format!(
                "failed to build configuration from path: {}",
                path.to_str().unwrap()
            ))
            .map_err(|e| ctx::CTXError::InvalidConfigurations {
                message:
                    "failed to load configurations, check your configurations (~/.aws/configs.yaml)"
                        .to_string(),
                source: Some(e),
            })?;

        c.try_deserialize()
            .context(format!(
                "failed to deserialize configurations from path: {}",
                path.to_str().unwrap()
            ))
            .map_err(|e| ctx::CTXError::InvalidConfigurations {
                message: "failed to deserialize configurations, check your configurations (~/.aws/configs.yaml)".to_string(),
                source: Some(e),
            })
    }

    pub fn initialize_default_configs<P: AsRef<Path>>(
        path: Option<P>,
    ) -> Result<Self, ctx::CTXError> {
        let path = path
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| CONFIGS_PATH.clone());
        if path.exists() {
            return Self::load_configs(Some(path));
        }
        // if the config directory does not exist, create the directory recursively
        path.parent()
            .map_or_else(
                || {
                    Err(anyhow!(
                        "no parent directory found for config path: {}",
                        path.to_str().unwrap()
                    ))
                },
                |parent| fs::create_dir_all(parent).context("failed to create config directory"),
            )
            .map_err(|e| ctx::CTXError::UnexpectedError { source: Some(e) })?;

        let c = Configs::default();
        let mut file = fs::File::create(&path)
            .context("failed to create a configuration file")
            .map_err(|e| ctx::CTXError::UnexpectedError { source: Some(e) })?;
        file.write_all(Self::CONFIGS_DESCRIPTIONS.as_bytes())
            .context("failed to write a configuration file")
            .map_err(|e| ctx::CTXError::UnexpectedError { source: Some(e) })?;

        let mut ser = serde_yaml::Serializer::new(&mut file);
        c.serialize(&mut ser)
            .context("failed to serialize configuration")
            .map_err(|e| ctx::CTXError::UnexpectedError { source: Some(e) })?;
        file.flush()
            .context("failed to flush a configuration file")
            .map_err(|e| ctx::CTXError::UnexpectedError { source: Some(e) })?;

        Self::load_configs(Some(path))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom};

    use rstest::*;
    use tempfile::{NamedTempFile, TempDir};

    use super::*;

    #[fixture]
    pub fn configs_text() -> String {
        r#"auth_commands:
  foo: |
    echo 1"#
            .to_string()
    }

    #[fixture]
    pub fn configs_file(configs_text: String) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", configs_text).unwrap();
        f.flush().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        f
    }

    #[fixture]
    pub fn configs() -> Configs {
        Configs {
            auth_commands: vec![("foo".to_string(), "echo 1".to_string())]
                .into_iter()
                .collect::<HashMap<String, String>>(),
        }
    }

    #[rstest(input, expect)]
    #[case(configs_file(configs_text()), Ok(configs()))]
    #[case(
        configs_file("invalid_yaml_format: a:a:".to_string()),
        Err(
            ctx::CTXError::InvalidConfigurations {
                message: "failed to load configurations, check your configurations (~/.aws/configs.yaml)".to_string(),
                source: None
            }
        )
    )]
    #[case(
        configs_file("unknown_key: foo".to_string()),
        Err(
            ctx::CTXError::InvalidConfigurations {
                message: "failed to deserialize configurations, check your configurations (~/.aws/configs.yaml)".to_string(),
                source: None
            }
        )
    )]
    fn test_configs_load_configs(input: NamedTempFile, expect: Result<Configs, ctx::CTXError>) {
        let actual = Configs::load_configs(Some(input.path()));
        match (expect, actual) {
            (Ok(expect), Ok(actual)) => {
                assert_eq!(expect, actual);
            }
            (Err(expect), Err(actual)) => match (&expect, &actual) {
                (
                    ctx::CTXError::InvalidConfigurations {
                        message: expect_message,
                        source: _,
                    },
                    ctx::CTXError::InvalidConfigurations {
                        message: actual_message,
                        source: _,
                    },
                ) => assert_eq!(expect_message, actual_message),
                _ => panic!("unexpected error"),
            },
            _ => panic!("expect and actual are not match"),
        }
    }

    #[rstest]
    fn test_initialize_default_configs() {
        let tmpdir = TempDir::new().unwrap();
        let tmpfile = tmpdir.path().join("configs.yaml");
        Configs::initialize_default_configs(Some(&tmpfile)).unwrap();
        let expect = r#"# # Configurations for awsctx
# # You can manually edit configurations according to the following usage

# # To use subcommand `auth` or `refresh`, fill the below configs for each profile.
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
#   # default configuration for profiles without auth configuration
#   __default: |
#     aws configure --profile {{profile}}
auth_commands:
  __default: |
    echo "This is default configuration for auth commands."
    echo "You can edit this configuration on ~/.awsctx/configs.yaml according to your needs."
    aws configure --profile {{profile}}
"#;
        let actual = fs::read_to_string(tmpfile).unwrap();
        assert_eq!(expect, actual);
    }
}
