use std::{fs::OpenOptions, io::Write};

use clap::{FromArgMatches, Parser};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Parser, PartialEq, Deserialize, Serialize)]
pub struct File {
    pub name: String,
    pub path: String,
    pub etag: String,
    pub url: String,
}

impl std::str::FromStr for File {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 4 {
            return Err(format!("invalid file argument: {}", s));
        }

        let name = parts[0].to_string();
        let path = parts[1].to_string();
        let etag = parts[2].to_string();
        let url = parts[3].to_string();

        Ok(File {
            name,
            path,
            etag,
            url,
        })
    }
}

#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    #[clap(long, env, required = true, num_args = 1.., value_delimiter = ' ', use_value_delimiter=true)]
    pub files: Vec<File>,
    #[clap(long, env, required = true)]
    pub path: String,
    #[clap(long, env, required = true)]
    pub wow: String,
    #[clap(long, env, required = true)]
    pub realmlist: String,
}

impl Configuration {
    pub fn write(&self) {
        let mut file = OpenOptions::new()
            .append(false)
            .write(true)
            .truncate(true)
            .open("config.json")
            .unwrap();
        println!("Writing: {:#?}", self);
        file.write_all(serde_json::to_string_pretty(self).unwrap().as_bytes())
            .unwrap();
    }
}

pub fn parse() -> Configuration {
    Configuration::parse()
}

pub fn parse_config() -> Configuration {
    match std::fs::File::open("config.json") {
        Ok(f) => serde_json::from_reader(f).unwrap(),
        Err(_) => Configuration::parse(),
    }
}
