use crate::ctxm::{CTXMError, CTXM};

use anyhow::{anyhow, Result};
use dirs::home_dir;
use regex::Regex;
use skim::prelude::{SkimItemReader, SkimOptionsBuilder};
use skim::Skim;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::Cursor;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct AWS {
    credentials_path: PathBuf,
}

pub struct Credentials {
    data: HashMap<String, Vec<String>>,
    current_key: Option<String>,
}

impl fmt::Display for Credentials {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (profile, items) in self.data.iter() {
            writeln!(fmt, "{}", profile)?;
            for item in items.iter() {
                writeln!(fmt, "{}", item)?;
            }
        }
        match &self.current_key {
            Some(k) => match self.data.get(k) {
                Some(items) => {
                    writeln!(fmt, "{}", DEFAULT_KEY)?;
                    for item in items.iter() {
                        writeln!(fmt, "{}", item)?;
                    }
                }
                None => (),
            },
            None => (),
        }
        Ok(())
    }
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
        // no default key found but it does not match any other keys
        // TODO: handle this as some error or add validation for credentials
        None
    } else {
        None
    }
}

fn load_aws_credentials(credentials_path: &PathBuf) -> Result<Credentials, CTXMError> {
    let file = fs::File::open(credentials_path)
        .map_err(|e| CTXMError::CannotReadConfiguration { source: e.into() })?;

    let reader = BufReader::new(file);
    let mut data = parse_aws_credentials(reader)?;
    let ck = find_current_key(&data);
    // remove DEFAULT_KEY after retrain current key
    data.remove(DEFAULT_KEY);

    Ok(Credentials {
        data: data,
        current_key: ck,
    })
}

fn validate_key(creds: &Credentials, name: &str) -> Result<(), CTXMError> {
    if !creds.data.contains_key(name) {
        return Err(CTXMError::InvalidArgument {
            source: anyhow!(format!("invalid profile: {}", name)),
        });
    }
    Ok(())
}

fn dump_credential(creds: &Credentials, credentials_path: &PathBuf) -> Result<(), CTXMError> {
    let mut file =
        fs::File::create(credentials_path).map_err(|e| CTXMError::IOError { source: e.into() })?;
    file.write_all(creds.to_string().as_bytes())
        .map_err(|e| CTXMError::IOError { source: e.into() })?;
    file.flush()
        .map_err(|e| CTXMError::IOError { source: e.into() })?;
    Ok(())
}

impl CTXM for AWS {
    fn list_contexts(&self) -> Result<Vec<String>, CTXMError> {
        let credentials_path = &self.credentials_path;
        let creds = load_aws_credentials(&credentials_path)?;
        let data = creds.data;
        let current_key = creds.current_key;
        let mut profiles: Vec<String> = data
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
        profiles.sort_by_key(|s| s.as_str().replace("*", " "));
        Ok(profiles)
    }
    fn use_context(&self, name: &str) -> Result<String, CTXMError> {
        let credentials_path = &self.credentials_path;
        let creds = load_aws_credentials(&credentials_path)?;
        let profile = format!("[{}]", name);
        validate_key(&creds, &profile)?;

        let creds = Credentials {
            data: creds.data,
            current_key: Some(profile.to_string()),
        };
        dump_credential(&creds, credentials_path)?;
        Ok(format!("{} is activated", name))
    }

    fn use_context_interactive(&self) -> () {
        let mut contexts = self.list_contexts().unwrap();
        contexts.reverse();
        let options = SkimOptionsBuilder::default()
            .height(Some("30%"))
            .multi(false)
            .build()
            .unwrap();
        let contexts = contexts
            .iter()
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .join("\n");

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(contexts));
        let selected = Skim::run_with(&options, Some(items))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| Vec::new());

        for item in selected.iter() {
            let context = &(item.output()).replace("* ", "");
            println!("{}", self.use_context(context).unwrap());
        }
    }
}
