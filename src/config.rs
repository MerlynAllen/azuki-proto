
use serde::{Deserialize, Serialize};
use serde_json;
use clap::Subcommand;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Subcommand)]
#[serde(rename_all = "lowercase")]
pub enum AzukiWorkMode {
    Client,
    Server,
    Relay,
}
impl Default for AzukiWorkMode {
    fn default() -> Self {
        AzukiWorkMode::Client
    }
}
impl FromStr for AzukiWorkMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(AzukiWorkMode::Client),
            "server" => Ok(AzukiWorkMode::Server),
            "relay" => Ok(AzukiWorkMode::Relay),
            _ => Err(format!("{s} is not a valid AzukiWorkMode")),
        }
    }
}

use std::{net::IpAddr, str::FromStr};
use log::Level;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub bind_addr: Option<IpAddr>,
    #[serde(default)]
    pub bind_port: Option<u16>,
    #[serde(default)]
    pub peer_addr: Option<IpAddr>,
    #[serde(default)]
    pub peer_port: Option<u16>,
    #[serde(default)]
    pub log_level: Option<String>,
}
use std::fs;
impl Config {
    pub fn from_str(path: &String) -> Self {
        let config_file = fs::read_to_string(path);
        let config_str = match config_file {
            Ok(s) => s,
            Err(e) => {
                panic!("Config not exist!\nError info: {:?}", e);
            }
        };
        serde_json::from_str(&config_str).unwrap_or_else(|e| {
            panic!("Config corrupted! Program exit.\nError info: {:?}", e);
        })
    }
}