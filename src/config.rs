use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// GW1000/Ecowitt Gateway Weather Station Listener
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Weather station IP address (e.g., 192.168.1.100)
    #[arg(short, long)]
    pub ip: Option<String>,

    /// Weather station port number (default: 45000)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output format: text, json
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,

    /// Continuous mode - poll every N seconds (default: 5)
    #[arg(long, default_value = "5")]
    pub continuous: u64,

    /// Run web server mode
    #[arg(long)]
    pub web: bool,

    /// Web server bind address (default: 0.0.0.0)
    #[arg(long, default_value = "0.0.0.0")]
    pub web_host: String,

    /// Web server port (default: 18888)
    #[arg(long, default_value = "18888")]
    pub web_port: u16,
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
    /// Get IP and port from either command line args, config file, or environment variables
    pub fn get_connection_info(&self) -> Result<(String, u16)> {
        // Priority: CLI args > config file > environment variables
        if let Some(config_path) = &self.config {
            let config_str = fs::read_to_string(config_path)
                .context(format!("Failed to read config file: {:?}", config_path))?;
            let config: Config =
                toml::from_str(&config_str).context("Failed to parse config file")?;
            Ok((config.ip, config.port))
        } else if let Some(ip) = &self.ip {
            let port = self.port.unwrap_or(45000);
            Ok((ip.clone(), port))
        } else if let Ok(ip) = std::env::var("WXLISTENER_IP") {
            // Try environment variables
            let port = std::env::var("WXLISTENER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(45000);
            Ok((ip, port))
        } else {
            anyhow::bail!(
                "Weather station IP must be specified via:\n\
                 - Command line: --ip <WEATHER_STATION_IP>\n\
                 - Config file: --config <FILE>\n\
                 - Environment: WXLISTENER_IP=<WEATHER_STATION_IP>\n\
                 \n\
                 Note: This is the IP of your GW1000/Ecowitt device, not the web server.\n\
                 Web server settings use --web-host and --web-port."
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_connection_info_from_ip() {
        let args = Args {
            ip: Some("192.168.1.100".to_string()),
            port: Some(45000),
            config: None,
            format: "text".to_string(),
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
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
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
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
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
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
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.0.1");
        assert_eq!(port, 45000); // Default port
    }

    #[test]
    fn test_get_connection_info_no_args() {
        // Clean up any environment variables first
        std::env::remove_var("WXLISTENER_IP");
        std::env::remove_var("WXLISTENER_PORT");

        let args = Args {
            ip: None,
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
        };

        let result = args.get_connection_info();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Weather station IP must be specified"));
    }

    #[test]
    fn test_get_connection_info_missing_config_file() {
        let args = Args {
            ip: None,
            port: None,
            config: Some(PathBuf::from("/nonexistent/config.toml")),
            format: "text".to_string(),
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
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
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
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

    #[test]
    #[serial]
    fn test_get_connection_info_from_env() {
        // Clean up first to ensure clean state
        std::env::remove_var("WXLISTENER_IP");
        std::env::remove_var("WXLISTENER_PORT");

        // Set environment variables
        std::env::set_var("WXLISTENER_IP", "192.168.1.50");
        std::env::set_var("WXLISTENER_PORT", "12345");

        let args = Args {
            ip: None,
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.1.50");
        assert_eq!(port, 12345);

        // Clean up
        std::env::remove_var("WXLISTENER_IP");
        std::env::remove_var("WXLISTENER_PORT");
    }

    #[test]
    #[serial]
    fn test_get_connection_info_from_env_default_port() {
        // Clean up first to ensure clean state
        std::env::remove_var("WXLISTENER_IP");
        std::env::remove_var("WXLISTENER_PORT");

        // Set only IP
        std::env::set_var("WXLISTENER_IP", "10.0.0.5");

        let args = Args {
            ip: None,
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "10.0.0.5");
        assert_eq!(port, 45000); // Default

        // Clean up
        std::env::remove_var("WXLISTENER_IP");
        std::env::remove_var("WXLISTENER_PORT");
    }

    #[test]
    #[serial]
    fn test_priority_cli_over_env() {
        // Set environment variable
        std::env::set_var("WXLISTENER_IP", "192.168.1.1");

        // CLI args should take priority
        let args = Args {
            ip: Some("10.10.10.10".to_string()),
            port: Some(9999),
            config: None,
            format: "text".to_string(),
            continuous: 5,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "10.10.10.10"); // CLI value, not env
        assert_eq!(port, 9999);

        // Clean up
        std::env::remove_var("WXLISTENER_IP");
    }
}
