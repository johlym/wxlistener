use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::database::DatabaseConfig;
use crate::http_output::HttpConfig;
use crate::mqtt::MqttConfig;

/// Local HTTP API port on Ecowitt gateways (GW3000 and the same API on GW1100/GW2000).
pub const HTTP_GATEWAY_PORT: u16 = 80;
/// Legacy telenet TCP port.
pub const LEGACY_GATEWAY_PORT: u16 = 45000;

/// Which rain array fills `rain_*` keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainSource {
    /// Traditional `rain` array. Default.
    Rain,
    /// Piezoelectric `piezoRain` array.
    Piezo,
}

impl RainSource {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "rain" => Ok(Self::Rain),
            "piezo" => Ok(Self::Piezo),
            other => anyhow::bail!(
                "rain_source must be \"rain\" or \"piezo\", got \"{}\"",
                other
            ),
        }
    }
}

/// Resolved gateway address and transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewaySettings {
    pub ip: String,
    pub port: u16,
    pub legacy: bool,
    pub rain_source: RainSource,
}

/// GW1000/GW3000/Ecowitt Gateway Weather Station Listener
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "A fast, standalone command-line tool to read live data from GW3000/GW1100/GW2000/Ecowitt Gateway weather stations.\n\n\
                  The local HTTP API is the default. Pass --legacy for the old TCP API on port 45000.\n\
                  Supports continuous monitoring, web interface, JSON/text output, and database logging to PostgreSQL or MySQL."
)]
pub struct Args {
    /// Weather station IP address (e.g., 192.168.1.100)
    #[arg(short, long)]
    pub ip: Option<String>,

    /// Weather station port. HTTP default is 80. Legacy TCP default is 45000.
    /// Port 45000 without --legacy, or port 80 with --legacy, is an error.
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Use the legacy TCP API instead of the local HTTP API.
    #[arg(long)]
    pub legacy: bool,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output format: text, json
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,

    /// Continuous mode - poll every N seconds (default: 16)
    #[arg(long, default_value = "16")]
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

    /// Create database table and exit (requires database config in config file)
    #[arg(long)]
    pub db_create_table: bool,

    /// Probe SENSOR_ID_NEW + livedata for WN34/WN34S ITEM_TF_USR mapping, then exit
    #[arg(long)]
    pub debug_sensors: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ip: String,
    /// Explicit gateway port. Absent means the transport default (80 or 45000).
    #[serde(default)]
    pub port: Option<u16>,
    /// Use the legacy TCP API. Absent leaves the choice to the CLI flag or environment.
    #[serde(default)]
    pub legacy: Option<bool>,
    /// `rain` or `piezo`. Absent means traditional rain.
    #[serde(default)]
    pub rain_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpConfig>,
}

impl Args {
    fn load_config(&self) -> Result<Option<Config>> {
        let Some(config_path) = &self.config else {
            return Ok(None);
        };
        let config_str = fs::read_to_string(config_path)
            .context(format!("Failed to read config file: {:?}", config_path))?;
        let config: Config =
            toml::from_str(&config_str).context("Failed to parse config file")?;
        Ok(Some(config))
    }

    /// IP, resolved port, transport, and rain source.
    pub fn gateway_settings(&self) -> Result<GatewaySettings> {
        let config = self.load_config()?;
        let legacy = self.legacy_mode_from(config.as_ref())?;
        let (ip, explicit_port) = self.endpoint_parts(config.as_ref())?;
        let port = explicit_port.unwrap_or(if legacy {
            LEGACY_GATEWAY_PORT
        } else {
            HTTP_GATEWAY_PORT
        });
        if let Some(explicit) = explicit_port {
            check_port_transport(legacy, explicit)?;
        }
        let rain_source = match config.as_ref().and_then(|c| c.rain_source.as_deref()) {
            Some(value) => RainSource::parse(value)?,
            None => RainSource::Rain,
        };
        Ok(GatewaySettings {
            ip,
            port,
            legacy,
            rain_source,
        })
    }

    fn legacy_mode_from(&self, config: Option<&Config>) -> Result<bool> {
        if self.legacy {
            return Ok(true);
        }
        if let Some(config) = config {
            if let Some(legacy) = config.legacy {
                return Ok(legacy);
            }
        }
        if let Ok(value) = std::env::var("WXLISTENER_LEGACY") {
            return parse_env_bool("WXLISTENER_LEGACY", &value);
        }
        Ok(false)
    }

    fn endpoint_parts(&self, config: Option<&Config>) -> Result<(String, Option<u16>)> {
        if let Some(config) = config {
            return Ok((config.ip.clone(), config.port));
        }
        if let Some(ip) = &self.ip {
            return Ok((ip.clone(), self.port));
        }
        if let Ok(ip) = std::env::var("WXLISTENER_IP") {
            let port = match std::env::var("WXLISTENER_PORT") {
                Ok(value) => Some(value.parse::<u16>().context(
                    "WXLISTENER_PORT must be an integer port number",
                )?),
                Err(_) => None,
            };
            return Ok((ip, port));
        }
        anyhow::bail!(
            "Weather station IP must be specified via:\n\
             - Command line: --ip <WEATHER_STATION_IP>\n\
             - Config file: --config <FILE>\n\
             - Environment: WXLISTENER_IP=<WEATHER_STATION_IP>\n\
             \n\
             Note: This is the IP of your GW1000/GW3000/Ecowitt device, not the web server.\n\
             Web server settings use --web-host and --web-port."
        )
    }

    /// Get IP and port from either command line args, config file, or environment variables.
    pub fn get_connection_info(&self) -> Result<(String, u16)> {
        let settings = self.gateway_settings()?;
        Ok((settings.ip, settings.port))
    }

    /// Get database configuration from config file if present
    pub fn get_database_config(&self) -> Result<Option<DatabaseConfig>> {
        Ok(self.load_config()?.and_then(|config| config.database))
    }

    /// Get MQTT configuration from config file if present
    pub fn get_mqtt_config(&self) -> Result<Option<MqttConfig>> {
        Ok(self.load_config()?.and_then(|config| config.mqtt))
    }

    /// Get HTTP configuration from config file or environment variables
    pub fn get_http_config(&self) -> Result<Option<HttpConfig>> {
        if let Some(config) = self.load_config()? {
            Ok(config.http)
        } else if std::env::var("WXLISTENER_HTTP_URL").is_ok() {
            Ok(Some(HttpConfig::default()))
        } else {
            Ok(None)
        }
    }
}

fn parse_env_bool(name: &str, value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" | "" => Ok(false),
        other => anyhow::bail!(
            "{} must be true/false or 1/0, got \"{}\"",
            name,
            other
        ),
    }
}

fn check_port_transport(legacy: bool, port: u16) -> Result<()> {
    if !legacy && port == LEGACY_GATEWAY_PORT {
        anyhow::bail!(
            "Port 45000 is the legacy TCP port, but the HTTP API is selected.\n\
             Pass --legacy, set legacy = true in the config file, or set WXLISTENER_LEGACY=1.\n\
             Or omit the port (HTTP uses 80) instead of setting 45000."
        );
    }
    if legacy && port == HTTP_GATEWAY_PORT {
        anyhow::bail!(
            "Port 80 is the HTTP API port, but legacy TCP is selected.\n\
             Remove --legacy, legacy = true, or WXLISTENER_LEGACY, or set the port to 45000."
        );
    }
    Ok(())
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
            port: Some(8080),
            config: None,
            format: "text".to_string(),
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.1.100");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_get_connection_info_from_ip_default_port() {
        let args = Args {
            ip: Some("10.0.0.1".to_string()),
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "10.0.0.1");
        assert_eq!(port, HTTP_GATEWAY_PORT);
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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.0.1");
        assert_eq!(port, HTTP_GATEWAY_PORT);
    }

    #[test]
    #[serial]
    fn test_get_connection_info_no_args() {
        // Clean up any environment variables first
        std::env::remove_var("WXLISTENER_IP");
        std::env::remove_var("WXLISTENER_PORT");

        let args = Args {
            ip: None,
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        };

        let result = args.get_connection_info();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config file"));
    }

    #[test]
    fn test_default_ports() {
        assert_eq!(HTTP_GATEWAY_PORT, 80);
        assert_eq!(LEGACY_GATEWAY_PORT, 45000);
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            ip = "10.31.100.42"
            port = 45000
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ip, "10.31.100.42");
        assert_eq!(config.port, Some(45000));
    }

    #[test]
    fn test_config_deserialization_default_port() {
        let toml_str = r#"
            ip = "10.31.100.42"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ip, "10.31.100.42");
        assert_eq!(config.port, None);
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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "10.0.0.5");
        assert_eq!(port, HTTP_GATEWAY_PORT);

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
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "10.10.10.10"); // CLI value, not env
        assert_eq!(port, 9999);

        // Clean up
        std::env::remove_var("WXLISTENER_IP");
    }

    fn bare_args() -> Args {
        Args {
            ip: Some("10.0.0.1".to_string()),
            port: None,
            config: None,
            format: "text".to_string(),
            continuous: 16,
            web: false,
            web_host: "0.0.0.0".to_string(),
            web_port: 18888,
            db_create_table: false,
            debug_sensors: false,
            legacy: false,
        }
    }

    #[test]
    fn test_explicit_legacy_port_without_legacy_is_an_error() {
        let mut args = bare_args();
        args.port = Some(LEGACY_GATEWAY_PORT);
        let err = args.gateway_settings().unwrap_err().to_string();
        assert!(err.contains("45000"));
        assert!(err.contains("--legacy"));
    }

    #[test]
    fn test_http_port_with_legacy_is_an_error() {
        let mut args = bare_args();
        args.legacy = true;
        args.port = Some(HTTP_GATEWAY_PORT);
        let err = args.gateway_settings().unwrap_err().to_string();
        assert!(err.contains("80"));
    }

    #[test]
    fn test_legacy_flag_uses_tcp_port() {
        let mut args = bare_args();
        args.legacy = true;
        let settings = args.gateway_settings().unwrap();
        assert!(settings.legacy);
        assert_eq!(settings.port, LEGACY_GATEWAY_PORT);
        assert_eq!(settings.rain_source, RainSource::Rain);
    }

    #[test]
    fn test_custom_port_is_allowed_for_http() {
        let mut args = bare_args();
        args.port = Some(8088);
        let settings = args.gateway_settings().unwrap();
        assert!(!settings.legacy);
        assert_eq!(settings.port, 8088);
    }

    #[test]
    #[serial]
    fn test_legacy_env_and_toml_precedence() {
        std::env::set_var("WXLISTENER_LEGACY", "1");
        let mut args = bare_args();
        args.ip = None;
        let settings = args.gateway_settings().unwrap_err();
        assert!(settings.to_string().contains("Weather station IP"));

        args.ip = Some("10.1.1.1".to_string());
        let settings = args.gateway_settings().unwrap();
        assert!(settings.legacy);
        assert_eq!(settings.port, LEGACY_GATEWAY_PORT);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ip = \"10.2.2.2\"").unwrap();
        writeln!(temp_file, "legacy = false").unwrap();
        writeln!(temp_file, "rain_source = \"piezo\"").unwrap();
        args.ip = None;
        args.config = Some(temp_file.path().to_path_buf());
        let settings = args.gateway_settings().unwrap();
        assert!(!settings.legacy);
        assert_eq!(settings.port, HTTP_GATEWAY_PORT);
        assert_eq!(settings.rain_source, RainSource::Piezo);
        assert_eq!(settings.ip, "10.2.2.2");

        std::env::remove_var("WXLISTENER_LEGACY");
    }

    #[test]
    fn test_config_legacy_port_45000_is_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ip = \"10.3.3.3\"").unwrap();
        writeln!(temp_file, "port = 45000").unwrap();
        writeln!(temp_file, "legacy = true").unwrap();
        let mut args = bare_args();
        args.ip = None;
        args.config = Some(temp_file.path().to_path_buf());
        let settings = args.gateway_settings().unwrap();
        assert!(settings.legacy);
        assert_eq!(settings.port, 45000);
    }
}
