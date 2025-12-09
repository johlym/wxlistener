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
            let config: Config =
                toml::from_str(&config_str).context("Failed to parse config file")?;
            Ok((config.ip, config.port))
        } else if let Some(ip) = &self.ip {
            let port = self.port.unwrap_or(45000);
            Ok((ip.clone(), port))
        } else {
            anyhow::bail!("Either --ip or --config must be specified");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_connection_info_from_ip() {
        let args = Args {
            ip: Some("192.168.1.100".to_string()),
            port: Some(45000),
            config: None,
            format: "text".to_string(),
            continuous: None,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.1.100");
        assert_eq!(port, 45000);
    }

    #[test]
    fn test_get_connection_info_from_ip_default_port() {
        let args = Args {
            ip: Some("10.0.0.1".to_string()),
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: None,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "10.0.0.1");
        assert_eq!(port, 45000); // Default port
    }

    #[test]
    fn test_get_connection_info_from_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ip = \"172.16.0.1\"").unwrap();
        writeln!(temp_file, "port = 12345").unwrap();

        let args = Args {
            ip: None,
            port: None,
            config: Some(temp_file.path().to_path_buf()),
            format: "text".to_string(),
            continuous: None,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "172.16.0.1");
        assert_eq!(port, 12345);
    }

    #[test]
    fn test_get_connection_info_from_config_default_port() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ip = \"192.168.0.1\"").unwrap();
        // No port specified, should use default

        let args = Args {
            ip: None,
            port: None,
            config: Some(temp_file.path().to_path_buf()),
            format: "text".to_string(),
            continuous: None,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.0.1");
        assert_eq!(port, 45000); // Default port
    }

    #[test]
    fn test_get_connection_info_no_args() {
        let args = Args {
            ip: None,
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: None,
        };

        let result = args.get_connection_info();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Either --ip or --config must be specified"));
    }

    #[test]
    fn test_get_connection_info_missing_config_file() {
        let args = Args {
            ip: None,
            port: None,
            config: Some(PathBuf::from("/nonexistent/config.toml")),
            format: "text".to_string(),
            continuous: None,
        };

        let result = args.get_connection_info();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read config file"));
    }

    #[test]
    fn test_get_connection_info_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "this is not valid toml {{{{").unwrap();

        let args = Args {
            ip: None,
            port: None,
            config: Some(temp_file.path().to_path_buf()),
            format: "text".to_string(),
            continuous: None,
        };

        let result = args.get_connection_info();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config file"));
    }

    #[test]
    fn test_default_port() {
        assert_eq!(default_port(), 45000);
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            ip = "10.31.100.42"
            port = 45000
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ip, "10.31.100.42");
        assert_eq!(config.port, 45000);
    }

    #[test]
    fn test_config_deserialization_default_port() {
        let toml_str = r#"
            ip = "10.31.100.42"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ip, "10.31.100.42");
        assert_eq!(config.port, 45000); // Default
    }
}
