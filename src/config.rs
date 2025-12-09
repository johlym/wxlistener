use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// GW1000/Ecowitt Gateway Weather Station Listener
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IP address of the weather station
    #[arg(short, long)]
    pub ip: Option<String>,

    /// Port number (default: 45000)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output format: text, json
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,

    /// Continuous mode - poll every N seconds
    #[arg(long)]
    pub continuous: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    45000
}

impl Args {
    /// Get IP and port from either command line args or config file
    pub fn get_connection_info(&self) -> Result<(String, u16)> {
        if let Some(config_path) = &self.config {
            let config_str = fs::read_to_string(config_path)
                .context(format!("Failed to read config file: {:?}", config_path))?;
            let config: Config = toml::from_str(&config_str)
                .context("Failed to parse config file")?;
            Ok((config.ip, config.port))
        } else if let Some(ip) = &self.ip {
            let port = self.port.unwrap_or(45000);
            Ok((ip.clone(), port))
        } else {
            anyhow::bail!("Either --ip or --config must be specified");
        }
    }
}
