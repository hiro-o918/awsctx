use crate::ctx;

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const DEFAULT_PROFILE_KEY: &str = "[default]";

#[derive(Default, Debug)]
pub struct Profile {
    pub name: String,
    pub default: bool,
    items: Vec<String>,
}

#[derive(Default, Debug)]
pub struct Credentials {
    data: HashMap<String, Vec<String>>,
    current_key: Option<String>,
}

impl fmt::Display for Credentials {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // sort profiles by key
        let mut profiles = self.data.iter().collect::<Vec<(&String, &Vec<String>)>>();
        profiles.sort_by(|a, b| a.0.cmp(b.0));
        for (profile, items) in profiles {
            writeln!(fmt, "{}", profile)?;
            for item in items.iter() {
                writeln!(fmt, "{}", item)?;
            }
        }
        if let Some(items) = self.data.get(&self.current_key.clone().unwrap_or_default()) {
            writeln!(fmt, "{}", DEFAULT_PROFILE_KEY)?;
            for item in items.iter() {
                writeln!(fmt, "{}", item)?;
            }
        }
        Ok(())
    }
}

impl Credentials {
    pub fn load_credentials(credentials_path: &Path) -> Result<Self, ctx::CTXError> {
        let file = fs::File::open(credentials_path)
            .map_err(|e| ctx::CTXError::CannotReadConfiguration { source: e.into() })?;

        let reader = BufReader::new(file);
        let mut data = parse_aws_credentials(reader)?;
        let ck = Self::find_default_key_from_profiles(&data);
        // remove DEFAULT_KEY after retrain current key
        data.remove(DEFAULT_PROFILE_KEY);

        Ok(Credentials {
            data,
            current_key: ck,
        })
    }

    pub fn get_profile(&self, name: &str) -> Result<Profile, ctx::CTXError> {
        let items =
            self.data
                .get(&format!("[{}]", name))
                .ok_or(ctx::CTXError::UnknownContextName {
                    source: anyhow!(format!("unknown context name: {}", name)),
                })?;
        Ok(Profile {
            name: name.into(),
            items: items.clone(),
            default: Some(name) == self.current_key.as_deref(),
        })
    }

    pub fn set_default_profile(&mut self, name: &str) -> Result<Profile, ctx::CTXError> {
        let key = name_to_profile_key(name);
        let items = self
            .data
            .get(&key)
            .ok_or(ctx::CTXError::UnknownContextName {
                source: anyhow!(format!("unknown context name: {}", name)),
            })?;
        self.current_key = Some(key);
        Ok(Profile {
            name: name.into(),
            items: items.clone(),
            default: Some(name) == self.current_key.as_deref(),
        })
    }

    pub fn dump_credential(&self, credentials_path: &Path) -> Result<(), ctx::CTXError> {
        let mut file = fs::File::create(credentials_path)
            .map_err(|e| ctx::CTXError::IOError { source: e.into() })?;
        file.write_all(self.to_string().as_bytes())
            .map_err(|e| ctx::CTXError::IOError { source: e.into() })?;
        file.flush()
            .map_err(|e| ctx::CTXError::IOError { source: e.into() })?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<Profile> {
        let mut profiles = self
            .data
            .iter()
            .map(|(key, items)| Profile {
                // remove `[` and `]` from name
                name: profile_key_to_name(key),
                items: items.clone(),
                default: Some(key) == self.current_key.as_ref(),
            })
            .collect::<Vec<Profile>>();
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        profiles
    }

    fn find_default_key_from_profiles(data: &HashMap<String, Vec<String>>) -> Option<String> {
        let default_items = data.get(DEFAULT_PROFILE_KEY)?;
        for (key, item) in data {
            if key == DEFAULT_PROFILE_KEY {
                continue;
            }
            if item.join("") == default_items.join("") {
                return Some(key.into());
            }
        }
        None
    }
}

fn parse_aws_credentials(
    reader: BufReader<fs::File>,
) -> Result<HashMap<String, Vec<String>>, ctx::CTXError> {
    let re_profile = Regex::new(r"^\[.+\]$").unwrap();

    let mut profile_idxs: Vec<usize> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| ctx::CTXError::CannotReadConfiguration { source: e.into() })?;
        lines.push(line.clone());
        if re_profile.is_match(&line) {
            profile_idxs.push(idx)
        }
    }
    if profile_idxs.is_empty() {
        return Err(ctx::CTXError::ConfigurationIsBroken {
            source: anyhow!("empty credential"),
        });
    }
    let lines = lines;
    let profile_idxs = profile_idxs;

    let first_idx: usize;
    let latter_idxs: Vec<usize>;
    if let Some(first_and_latters) = profile_idxs.split_first() {
        first_idx = *first_and_latters.0;
        latter_idxs = first_and_latters.1.to_vec();
    } else {
        return Err(ctx::CTXError::ConfigurationIsBroken {
            source: anyhow!("unexpected error"),
        });
    }

    let end_idx: usize;
    let former_idxs: Vec<usize>;
    if let Some(end_and_formers) = profile_idxs.split_last() {
        end_idx = *end_and_formers.0;
        former_idxs = end_and_formers.1.to_vec();
    } else {
        return Err(ctx::CTXError::ConfigurationIsBroken {
            source: anyhow!("unexpected error"),
        });
    }

    if first_idx == end_idx {
        let mut data: HashMap<String, Vec<String>> = HashMap::new();
        let profile = lines[first_idx].clone();
        let items = lines[first_idx..].to_vec();

        data.insert(profile, items);
        return Ok(data);
    }

    let mut data: HashMap<String, Vec<String>> = HashMap::new();
    for (former_idx, latter_idx) in former_idxs.iter().zip(latter_idxs) {
        let profile = lines[*former_idx].clone(); // trim redundant `[` and `]`
        let items = lines[*former_idx + 1..latter_idx].to_vec();
        data.insert(profile, items);
    }
    {
        let profile = lines[end_idx].clone();
        let items = lines[end_idx + 1..].to_vec();
        data.insert(profile, items);
    }
    Ok(data)
}

fn name_to_profile_key(name: &str) -> String {
    format!("[{}]", name)
}

fn profile_key_to_name(key: &str) -> String {
    key[1..key.len() - 1].to_string()
}
