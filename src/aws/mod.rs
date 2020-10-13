use crate::ctxm::{CTXMError, CTXM};

use anyhow::{anyhow, Result};
use dirs::home_dir;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug)]
pub struct AWS {
    credentials_path: PathBuf,
}

pub struct Credentials {
    data: HashMap<String, Vec<String>>,
    current_key: Option<String>,
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
const DEFAULT_KEY: &str = "[default]";

fn parse_aws_credentials(
    reader: BufReader<fs::File>,
) -> Result<HashMap<String, Vec<String>>, CTXMError> {
    let re_profile = Regex::new(r"^\[.+\]$").unwrap();

    let mut profile_idxs: Vec<usize> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| CTXMError::CannotReadConfiguration { source: e.into() })?;
        lines.push(line.clone());
        if re_profile.is_match(&line) {
            profile_idxs.push(idx)
        }
    }
    if profile_idxs.len() == 0 {
        return Err(CTXMError::ConfigurationIsBroken {
            source: anyhow!("empty credential"),
        });
    }
    let lines = lines;
    let profile_idxs = profile_idxs;

    let first_idx: usize;
    let latter_idxs: Vec<usize>;
    if let Some(first_and_latters) = profile_idxs.split_first() {
        first_idx = first_and_latters.0.clone();
        latter_idxs = first_and_latters.1.iter().cloned().collect();
    } else {
        return Err(CTXMError::ConfigurationIsBroken {
            source: anyhow!("unexpected error"),
        });
    }

    let end_idx: usize;
    let former_idxs: Vec<usize>;
    if let Some(end_and_formers) = profile_idxs.split_last() {
        end_idx = end_and_formers.0.clone();
        former_idxs = end_and_formers.1.iter().cloned().collect();
    } else {
        return Err(CTXMError::ConfigurationIsBroken {
            source: anyhow!("unexpected error"),
        });
    }

    if first_idx == end_idx {
        let mut data: HashMap<String, Vec<String>> = HashMap::new();
        let profile = lines[first_idx].clone();
        let items = lines[first_idx..].iter().cloned().collect();

        data.insert(profile, items);
        return Ok(data);
    }

    let mut data: HashMap<String, Vec<String>> = HashMap::new();
    for (former_idx, latter_idx) in former_idxs.iter().zip(latter_idxs) {
        let profile = lines[*former_idx].clone(); // trim redundant `[` and `]`
        let items = lines[*former_idx + 1..latter_idx].iter().cloned().collect();
        data.insert(profile, items);
    }
    {
        let profile = lines[end_idx].clone();
        let items = lines[end_idx + 1..].iter().cloned().collect();
        data.insert(profile, items);
    }
    Ok(data)
}

fn find_current_key(data: &HashMap<String, Vec<String>>) -> Option<String> {
    if let Some(default_values) = data.get(DEFAULT_KEY) {
        let sep = "-";
        let joined_dv = default_values.join(sep);
        for (k, v) in data {
            if k == DEFAULT_KEY {
                continue;
            }
            if v.join(sep) == joined_dv {
                return Some(k.to_string());
            }
        }
        None
    } else {
        None
    }
}

fn load_aws_credentials(credentials_path: &PathBuf) -> Result<Credentials, CTXMError> {
    let file = fs::File::open(credentials_path)
        .map_err(|e| CTXMError::CannotReadConfiguration { source: e.into() })?;

    let reader = BufReader::new(file);
    let data = parse_aws_credentials(reader)?;
    let ck = find_current_key(&data);

    Ok(Credentials {
        data: data,
        current_key: ck,
    })
}

impl CTXM for AWS {
    fn list_contexts(&self) -> Result<Vec<String>, CTXMError> {
        let credentials_path = &self.credentials_path;
        let creds = load_aws_credentials(&credentials_path)?;
        let data = creds.data;
        let current_key = creds.current_key;
        let profiles: Vec<String> = data
            .keys()
            .filter_map(|k| {
                if k == DEFAULT_KEY {
                    return None;
                }
                let p = k[1..k.len() - 1].to_string();
                if let Some(ck) = &current_key {
                    if ck == k {
                        return Some(format!("* {}", p));
                    }
                }
                Some(format!("  {}", p))
            })
            .collect();
        Ok(profiles)
    }
}
