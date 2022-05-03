use crate::ctx;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::io::{BufWriter, Read};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use config;
use ini::Ini;

const DEFAULT_PROFILE_NAME_KEY: &str = "default";

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub default: bool,
    #[allow(dead_code)]
    items: HashMap<String, String>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Credentials {
    data: HashMap<String, HashMap<String, String>>,
    default_profile_name: Option<String>,
}

impl fmt::Display for Credentials {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut conf = Ini::new();
        let mut profile_names = Vec::from_iter(self.data.keys());

        // sort profile names by reverse order to write ascending order
        profile_names.sort();
        for profile_name in profile_names {
            let mut sec = conf.with_section(Some(profile_name));
            // NOTE: to use method chain of `&mut SectionSetter`, declare `s` before
            let mut s = sec.borrow_mut();
            let data = self.data.get(profile_name).unwrap();
            let mut data_keys = Vec::from_iter(data.keys());
            data_keys.sort();
            for data_key in data_keys {
                s = s.set(data_key, data.get(data_key).unwrap());
            }
        }

        // write default profile to section first to write last
        if let Some(default_profile_name) = &self.default_profile_name {
            let mut sec = conf.with_section(Some(DEFAULT_PROFILE_NAME_KEY));
            // NOTE: to use method chain of `&mut SectionSetter`, declare `s` before
            let mut s = sec.borrow_mut();
            let data = self.data.get(default_profile_name).unwrap();
            let mut data_keys = Vec::from_iter(data.keys());
            data_keys.sort();
            for data_key in data_keys {
                s = s.set(data_key, data.get(data_key).unwrap());
            }
        }

        let mut buf = vec![];

        {
            let mut f = BufWriter::new(&mut buf);
            conf.write_to(&mut f).unwrap();
        }
        write!(fmt, "{}", String::from_utf8(buf).unwrap())
    }
}

impl Credentials {
    pub fn load_credentials<P: AsRef<Path>>(credentials_path: P) -> Result<Self, ctx::CTXError> {
        let file =
            fs::File::open(credentials_path).map_err(|e| ctx::CTXError::CannotReadCredentials {
                source: Some(e.into()),
            })?;

        let mut data = parse_aws_credentials(&file)?;
        let ck = find_default_from_parsed_aws_credentials(&data);
        // remove DEFAULT_KEY after retrain current key
        data.remove(DEFAULT_PROFILE_NAME_KEY);

        Ok(Credentials {
            data,
            default_profile_name: ck,
        })
    }

    pub fn get_profile(&self, name: &str) -> Result<Profile, ctx::CTXError> {
        let items = self.data.get(name).ok_or(ctx::CTXError::NoSuchProfile {
            profile: name.to_string(),
            source: Some(anyhow!(format!("unknown context name: {}", name))),
        })?;
        Ok(Profile {
            name: name.into(),
            items: items.clone(),
            default: Some(name.to_string()) == self.default_profile_name,
        })
    }

    pub fn get_default_profile(&self) -> Result<Profile, ctx::CTXError> {
        let name = self
            .default_profile_name
            .as_ref()
            .ok_or(ctx::CTXError::NoActiveContext { source: None })?;
        self.get_profile(name)
    }

    pub fn set_default_profile(&mut self, name: &str) -> Result<Profile, ctx::CTXError> {
        let items = self.data.get(name).ok_or(ctx::CTXError::NoSuchProfile {
            profile: name.to_string(),
            source: Some(anyhow!(format!("unknown context name: {}", name))),
        })?;
        self.default_profile_name = Some(name.to_string());
        Ok(Profile {
            name: name.into(),
            items: items.clone(),
            default: true,
        })
    }

    pub fn dump_credentials<P: AsRef<Path>>(
        &self,
        credentials_path: P,
    ) -> Result<(), ctx::CTXError> {
        let mut file = fs::File::create(credentials_path).map_err(|e| {
            ctx::CTXError::CannotWriteCredentials {
                source: Some(e.into()),
            }
        })?;
        file.write_all(self.to_string().as_bytes()).map_err(|e| {
            ctx::CTXError::CannotWriteCredentials {
                source: Some(e.into()),
            }
        })?;
        file.flush()
            .map_err(|e| ctx::CTXError::CannotWriteCredentials {
                source: Some(e.into()),
            })?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<Profile> {
        let mut profiles = self
            .data
            .iter()
            .map(|(key, items)| Profile {
                name: key.to_string(),
                items: items.clone(),
                default: Some(key) == self.default_profile_name.as_ref(),
            })
            .collect::<Vec<Profile>>();
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        profiles
    }
}

fn parse_aws_credentials(
    file: &File,
) -> Result<HashMap<String, HashMap<String, String>>, ctx::CTXError> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader
        .read_to_string(&mut contents)
        .map_err(|e| ctx::CTXError::CannotReadCredentials {
            source: Some(e.into()),
        })?;
    let c = config::Config::builder()
        .add_source(config::File::from_str(
            contents.as_str(),
            config::FileFormat::Ini,
        ))
        .build()
        .context("failed to load aws credentials".to_string())
        .map_err(|e| ctx::CTXError::CredentialsIsBroken { source: Some(e) })?;
    c.try_deserialize()
        .context("failed to deserialize credentials".to_string())
        .map_err(|e| ctx::CTXError::CredentialsIsBroken { source: Some(e) })
}

fn find_default_from_parsed_aws_credentials(
    data: &HashMap<String, HashMap<String, String>>,
) -> Option<String> {
    let default_items = data.get(DEFAULT_PROFILE_NAME_KEY)?;
    for (key, item) in data {
        if key == DEFAULT_PROFILE_NAME_KEY {
            continue;
        }
        if item.clone() == default_items.clone() {
            return Some(key.into());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom};

    use rstest::*;
    use tempfile::NamedTempFile;

    use super::*;

    #[fixture]
    pub fn aws_credentials_text() -> String {
        r#"[bar]
aws_access_key_id=YYYYYYYYYYY
aws_secret_access_key=YYYYYYYYYYY
aws_session_token=YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY

[foo]
aws_access_key_id=XXXXXXXXXXX
aws_secret_access_key=XXXXXXXXXXX
aws_session_token=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

[default]
aws_access_key_id=XXXXXXXXXXX
aws_secret_access_key=XXXXXXXXXXX
aws_session_token=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
"#
        .to_string()
    }

    #[fixture]
    pub fn aws_credentials_text_without_default() -> String {
        r#"[bar]
aws_access_key_id=YYYYYYYYYYY
aws_secret_access_key=YYYYYYYYYYY
aws_session_token=YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY

[foo]
aws_access_key_id=XXXXXXXXXXX
aws_secret_access_key=XXXXXXXXXXX
aws_session_token=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
"#
        .to_string()
    }

    #[fixture(text = aws_credentials_text())]
    pub fn aws_credentials(text: String) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", text).unwrap();
        f.flush().unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        f
    }

    #[fixture(aws_credentials = aws_credentials(aws_credentials_text()))]
    pub fn parsed_aws_credentials(
        aws_credentials: NamedTempFile,
    ) -> HashMap<String, HashMap<String, String>> {
        parse_aws_credentials(aws_credentials.as_file()).unwrap()
    }

    #[fixture]
    pub fn credentials() -> Credentials {
        Credentials {
            data: vec![
                (
                    "foo".to_string(),
                    vec![
                        ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                        (
                            "aws_secret_access_key".to_string(),
                            "XXXXXXXXXXX".to_string(),
                        ),
                        (
                            "aws_session_token".to_string(),
                            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect::<HashMap<String, String>>(),
                ),
                (
                    "bar".to_string(),
                    vec![
                        ("aws_access_key_id".to_string(), "YYYYYYYYYYY".to_string()),
                        (
                            "aws_secret_access_key".to_string(),
                            "YYYYYYYYYYY".to_string(),
                        ),
                        (
                            "aws_session_token".to_string(),
                            "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect::<HashMap<String, String>>(),
                ),
            ]
            .into_iter()
            .collect::<HashMap<String, HashMap<String, String>>>(),
            default_profile_name: Some("foo".to_string()),
        }
    }

    #[fixture]
    pub fn credentials_without_default() -> Credentials {
        Credentials {
            data: vec![
                (
                    "foo".to_string(),
                    vec![
                        ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                        (
                            "aws_secret_access_key".to_string(),
                            "XXXXXXXXXXX".to_string(),
                        ),
                        (
                            "aws_session_token".to_string(),
                            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect::<HashMap<String, String>>(),
                ),
                (
                    "bar".to_string(),
                    vec![
                        ("aws_access_key_id".to_string(), "YYYYYYYYYYY".to_string()),
                        (
                            "aws_secret_access_key".to_string(),
                            "YYYYYYYYYYY".to_string(),
                        ),
                        (
                            "aws_session_token".to_string(),
                            "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect::<HashMap<String, String>>(),
                ),
            ]
            .into_iter()
            .collect::<HashMap<String, HashMap<String, String>>>(),
            default_profile_name: None,
        }
    }

    #[rstest]
    fn test_parse_aws_credentials(aws_credentials: NamedTempFile) {
        let expect = vec![
            (
                "foo".to_string(),
                vec![
                    ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                    (
                        "aws_secret_access_key".to_string(),
                        "XXXXXXXXXXX".to_string(),
                    ),
                    (
                        "aws_session_token".to_string(),
                        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(),
                    ),
                ]
                .into_iter()
                .collect::<HashMap<String, String>>(),
            ),
            (
                "bar".to_string(),
                vec![
                    ("aws_access_key_id".to_string(), "YYYYYYYYYYY".to_string()),
                    (
                        "aws_secret_access_key".to_string(),
                        "YYYYYYYYYYY".to_string(),
                    ),
                    (
                        "aws_session_token".to_string(),
                        "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string(),
                    ),
                ]
                .into_iter()
                .collect::<HashMap<String, String>>(),
            ),
            (
                "default".to_string(),
                vec![
                    ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                    (
                        "aws_secret_access_key".to_string(),
                        "XXXXXXXXXXX".to_string(),
                    ),
                    (
                        "aws_session_token".to_string(),
                        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(),
                    ),
                ]
                .into_iter()
                .collect::<HashMap<String, String>>(),
            ),
        ]
        .into_iter()
        .collect::<HashMap<String, HashMap<String, String>>>();

        let actual = parse_aws_credentials(aws_credentials.as_file()).unwrap();
        assert_eq!(expect, actual);
    }

    #[rstest(::trace)]
    #[case(
        parsed_aws_credentials(aws_credentials(aws_credentials_text())),
        Some("foo".to_string())
    )]
    #[case(
        parsed_aws_credentials(aws_credentials(aws_credentials_text_without_default())),
        None
    )]
    fn test_find_default_from_parsed_aws_credentials(
        #[case] parsed_aws_credentials: HashMap<String, HashMap<String, String>>,
        #[case] expect: Option<String>,
    ) {
        let actual = find_default_from_parsed_aws_credentials(&parsed_aws_credentials);
        assert_eq!(expect, actual);
    }

    #[rstest(::trace)]
    #[case(aws_credentials(aws_credentials_text()), credentials())]
    #[case(
        aws_credentials(aws_credentials_text_without_default()),
        credentials_without_default()
    )]

    fn test_credentials_load_credentials(
        #[case] aws_credentials: NamedTempFile,
        #[case] expect: Credentials,
    ) {
        let actual = Credentials::load_credentials(aws_credentials.path()).unwrap();
        assert_eq!(expect, actual);
    }

    #[rstest(::trace)]
    #[case(
        "foo", 
        Ok(Profile {
            name: "foo".to_string(), 
            default: true,
            items: vec![
                ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                ("aws_secret_access_key".to_string(), "XXXXXXXXXXX".to_string()),
                ("aws_session_token".to_string(), "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string()),
            ]
            .into_iter()
            .collect::<HashMap<String, String>>()
        })
    )]
    #[case(
        "bar", 
        Ok(Profile {
            name: "bar".to_string(), 
            default: false,
            items: vec![
                ("aws_access_key_id".to_string(), "YYYYYYYYYYY".to_string()),
                ("aws_secret_access_key".to_string(), "YYYYYYYYYYY".to_string()),
                ("aws_session_token".to_string(), "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string()),
            ]
            .into_iter()
            .collect::<HashMap<String, String>>()
        })
    )]
    #[case("unknown", Err(ctx::CTXError::NoSuchProfile {
            profile: name.to_string(),
            source: Some(anyhow!(format!("unknown context name: {}", name))),
        }))]
    fn test_credentials_get_profile(
        credentials: Credentials,
        #[case] name: &str,
        #[case] expect: Result<Profile, ctx::CTXError>,
    ) {
        let actual = credentials.get_profile(name);
        match (expect, actual) {
            (Ok(expect), Ok(actual)) => assert_eq!(expect, actual),
            (Err(expect), Err(actual)) => match (&expect, &actual) {
                (
                    ctx::CTXError::NoSuchProfile {
                        profile: expect_profile,
                        source: _expect_source,
                    },
                    ctx::CTXError::NoSuchProfile {
                        profile: actual_profile,
                        source: _actual_source,
                    },
                ) => {
                    assert_eq!(expect_profile, actual_profile);
                }
                _ => panic!("unexpected error: {}", actual),
            },
            _ => panic!("expect and actual are not match"),
        }
    }

    #[rstest(::trace)]
    #[case(
        credentials(),
        Ok(Profile {
            name: "foo".to_string(), 
            default: true,
            items: vec![
                ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                ("aws_secret_access_key".to_string(), "XXXXXXXXXXX".to_string()),
                ("aws_session_token".to_string(), "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string()),
            ]
            .into_iter()
            .collect::<HashMap<String, String>>()
        })
    )]
    #[case(credentials_without_default(), Err(ctx::CTXError::NoActiveContext { source: None }))]
    fn test_credentials_get_default_profile(
        #[case] credentials: Credentials,
        #[case] expect: Result<Profile, ctx::CTXError>,
    ) {
        let actual = credentials.get_default_profile();
        match (expect, actual) {
            (Ok(expect), Ok(actual)) => assert_eq!(expect, actual),
            (Err(expect), Err(actual)) => match (&expect, &actual) {
                (
                    ctx::CTXError::NoActiveContext {
                        source: _expect_source,
                    },
                    ctx::CTXError::NoActiveContext {
                        source: _actual_source,
                    },
                ) => (),
                _ => panic!("unexpected error: {}", actual),
            },
            _ => panic!("expect and actual are not match"),
        }
    }

    #[rstest(::trace)]
    #[case(
        "foo", 
        Ok(Profile {
            name: "foo".to_string(), 
            default: true,
            items: vec![
                ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                ("aws_secret_access_key".to_string(), "XXXXXXXXXXX".to_string()),
                ("aws_session_token".to_string(), "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string()),
            ]
            .into_iter()
            .collect::<HashMap<String, String>>()
        })
    )]
    #[case(
        "bar", 
        Ok(Profile {
            name: "bar".to_string(), 
            default: true,
            items: vec![
                ("aws_access_key_id".to_string(), "YYYYYYYYYYY".to_string()),
                ("aws_secret_access_key".to_string(), "YYYYYYYYYYY".to_string()),
                ("aws_session_token".to_string(), "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string()),
            ]
            .into_iter()
            .collect::<HashMap<String, String>>()
        })
    )]
    #[case("unknown", Err(ctx::CTXError::NoSuchProfile {
            profile: name.to_string(),
            source: Some(anyhow!(format!("unknown context name: {}", name))),
        }))]
    fn test_credentials_set_default_profile(
        mut credentials: Credentials,
        #[case] name: &str,
        #[case] expect: Result<Profile, ctx::CTXError>,
    ) {
        let actual = credentials.set_default_profile(name);
        match (expect, actual) {
            (Ok(expect), Ok(actual)) => {
                assert_eq!(expect, actual);
                // check default profile is updated
                assert_eq!(Some(name.to_string()), credentials.default_profile_name);
            }
            (Err(expect), Err(actual)) => match (&expect, &actual) {
                (
                    ctx::CTXError::NoSuchProfile {
                        profile: expect_profile,
                        source: _expect_source,
                    },
                    ctx::CTXError::NoSuchProfile {
                        profile: actual_profile,
                        source: _actual_source,
                    },
                ) => {
                    assert_eq!(expect_profile, actual_profile);
                }
                _ => panic!("unexpected error: {}", actual),
            },
            _ => panic!("expect and actual are not match"),
        }
    }

    #[rstest(::trace)]
    #[case(credentials(), aws_credentials_text())]
    #[case(credentials_without_default(), aws_credentials_text_without_default())]
    fn test_credentials_dump_credentials(
        #[case] credentials: Credentials,
        #[case] aws_credentials_text: String,
    ) {
        let namedfile = NamedTempFile::new().unwrap();
        let expect = aws_credentials_text;

        credentials.dump_credentials(namedfile.path()).unwrap();
        let actual = fs::read_to_string(namedfile.path()).unwrap();
        assert_eq!(expect, actual);
    }

    #[rstest(::trace)]
    fn test_list_profiles(credentials: Credentials) {
        let expect = vec![
            Profile {
                name: "bar".to_string(),
                default: false,
                items: vec![
                    ("aws_access_key_id".to_string(), "YYYYYYYYYYY".to_string()),
                    (
                        "aws_secret_access_key".to_string(),
                        "YYYYYYYYYYY".to_string(),
                    ),
                    (
                        "aws_session_token".to_string(),
                        "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string(),
                    ),
                ]
                .into_iter()
                .collect::<HashMap<String, String>>(),
            },
            Profile {
                name: "foo".to_string(),
                default: true,
                items: vec![
                    ("aws_access_key_id".to_string(), "XXXXXXXXXXX".to_string()),
                    (
                        "aws_secret_access_key".to_string(),
                        "XXXXXXXXXXX".to_string(),
                    ),
                    (
                        "aws_session_token".to_string(),
                        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(),
                    ),
                ]
                .into_iter()
                .collect::<HashMap<String, String>>(),
            },
        ];

        let actual = credentials.list_profiles();
        assert_eq!(expect, actual);
    }
}
