use anyhow::{Context, Result};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub connection_string: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub topic: Option<String>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl MqttConfig {
    pub fn new() -> Self {
        Self {
            connection_string: None,
            host: None,
            port: None,
            topic: None,
            client_id: None,
            username: None,
            password: None,
        }
    }

    pub fn get_connection_info(
        &self,
    ) -> Result<(String, u16, String, Option<String>, Option<String>)> {
        if let Some(conn_str) = &self.connection_string {
            self.parse_connection_string(conn_str)
        } else if let Some(host) = &self.host {
            let port = self.port.unwrap_or(1883);
            let topic = self.topic.clone().unwrap_or_else(|| "wx/live".to_string());
            Ok((
                host.clone(),
                port,
                topic,
                self.username.clone(),
                self.password.clone(),
            ))
        } else {
            anyhow::bail!(
                "MQTT broker must be specified via:\n\
                 - Connection string: mqtt://[username:password@]host:port/topic\n\
                 - Individual fields: host, port (optional), topic (optional)"
            );
        }
    }

    fn parse_connection_string(
        &self,
        conn_str: &str,
    ) -> Result<(String, u16, String, Option<String>, Option<String>)> {
        let url = url::Url::parse(conn_str).context("Failed to parse MQTT connection string")?;

        if url.scheme() != "mqtt" && url.scheme() != "mqtts" {
            anyhow::bail!("MQTT connection string must start with mqtt:// or mqtts://");
        }

        let host = url
            .host_str()
            .context("MQTT connection string must include a host")?
            .to_string();

        let port = url.port().unwrap_or(1883);

        let topic = if !url.path().is_empty() && url.path() != "/" {
            url.path().trim_start_matches('/').to_string()
        } else {
            self.topic.clone().unwrap_or_else(|| "wx/live".to_string())
        };

        // Extract username and password from URL if present
        let username = if !url.username().is_empty() {
            Some(url.username().to_string())
        } else {
            self.username.clone()
        };

        let password = url
            .password()
            .map(|p| p.to_string())
            .or_else(|| self.password.clone());

        Ok((host, port, topic, username, password))
    }

    pub fn get_client_id(&self) -> String {
        self.client_id
            .clone()
            .unwrap_or_else(|| format!("wxlistener-{}", std::process::id()))
    }
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MqttPublisher {
    client: AsyncClient,
    topic: String,
}

impl MqttPublisher {
    pub async fn new(config: &MqttConfig) -> Result<Self> {
        let (host, port, topic, username, password) = config.get_connection_info()?;
        let client_id = config.get_client_id();

        let mut mqtt_options = MqttOptions::new(client_id, host, port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), Some(password)) = (username, password) {
            mqtt_options.set_credentials(username, password);
        }

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        // Wait for initial connection confirmation
        let mut connection_confirmed = false;
        let timeout = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::ConnAck(connack))) => {
                        if connack.code == rumqttc::ConnectReturnCode::Success {
                            connection_confirmed = true;
                            break Ok(());
                        } else {
                            break Err(anyhow::anyhow!(
                                "MQTT connection refused: {:?}",
                                connack.code
                            ));
                        }
                    }
                    Err(e) => {
                        break Err(anyhow::anyhow!("MQTT connection error: {}", e));
                    }
                    _ => {}
                }
            }
        })
        .await;

        match timeout {
            Ok(Ok(())) => {
                // Connection successful, spawn background task to handle events
                tokio::spawn(async move {
                    loop {
                        match eventloop.poll().await {
                            Ok(Event::Incoming(Incoming::Disconnect)) => {
                                eprintln!("MQTT broker disconnected");
                            }
                            Err(e) => {
                                eprintln!("MQTT connection error: {}", e);
                                tokio::time::sleep(Duration::from_secs(5)).await;
                            }
                            _ => {}
                        }
                    }
                });
                Ok(Self { client, topic })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("MQTT connection timeout after 5 seconds")),
        }
    }

    pub async fn publish(&self, payload: &str) -> Result<()> {
        self.client
            .publish(&self.topic, QoS::AtLeastOnce, false, payload)
            .await
            .context("Failed to publish MQTT message")?;
        Ok(())
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_config_new() {
        let config = MqttConfig::new();
        assert!(config.connection_string.is_none());
        assert!(config.host.is_none());
        assert!(config.port.is_none());
        assert!(config.topic.is_none());
    }

    #[test]
    fn test_parse_connection_string_basic() {
        let config = MqttConfig {
            connection_string: Some("mqtt://localhost:1883/wx/live".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_parse_connection_string_default_port() {
        let config = MqttConfig {
            connection_string: Some("mqtt://broker.example.com/weather/data".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "broker.example.com");
        assert_eq!(port, 1883);
        assert_eq!(topic, "weather/data");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_parse_connection_string_no_topic() {
        let config = MqttConfig {
            connection_string: Some("mqtt://localhost:1883".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_parse_connection_string_with_config_topic() {
        let config = MqttConfig {
            connection_string: Some("mqtt://localhost:1883".to_string()),
            topic: Some("custom/topic".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "custom/topic");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_connection_from_fields() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            port: Some(8883),
            topic: Some("sensors/weather".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 8883);
        assert_eq!(topic, "sensors/weather");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_connection_from_fields_default_port() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            topic: Some("weather".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 1883);
        assert_eq!(topic, "weather");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_connection_from_fields_default_topic() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
        assert!(username.is_none());
        assert!(password.is_none());
    }

    #[test]
    fn test_get_client_id_default() {
        let config = MqttConfig::new();
        let client_id = config.get_client_id();
        assert!(client_id.starts_with("wxlistener-"));
    }

    #[test]
    fn test_get_client_id_custom() {
        let config = MqttConfig {
            client_id: Some("my-custom-client".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_client_id(), "my-custom-client");
    }

    #[test]
    fn test_missing_connection_info() {
        let config = MqttConfig::new();
        assert!(config.get_connection_info().is_err());
    }

    #[test]
    fn test_invalid_scheme() {
        let config = MqttConfig {
            connection_string: Some("http://localhost:1883".to_string()),
            ..Default::default()
        };
        assert!(config.get_connection_info().is_err());
    }

    #[test]
    fn test_parse_connection_string_with_credentials() {
        let config = MqttConfig {
            connection_string: Some("mqtt://myuser:mypass@localhost:1883/wx/live".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
        assert_eq!(username, Some("myuser".to_string()));
        assert_eq!(password, Some("mypass".to_string()));
    }

    #[test]
    fn test_parse_connection_string_username_only() {
        let config = MqttConfig {
            connection_string: Some("mqtt://myuser@localhost:1883/wx/live".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
        assert_eq!(username, Some("myuser".to_string()));
        assert!(password.is_none());
    }

    #[test]
    fn test_connection_string_overrides_config_credentials() {
        let config = MqttConfig {
            connection_string: Some("mqtt://urluser:urlpass@localhost:1883/wx/live".to_string()),
            username: Some("configuser".to_string()),
            password: Some("configpass".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
        assert_eq!(username, Some("urluser".to_string()));
        assert_eq!(password, Some("urlpass".to_string()));
    }

    #[test]
    fn test_connection_from_fields_with_credentials() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            port: Some(8883),
            topic: Some("sensors/weather".to_string()),
            username: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            ..Default::default()
        };

        let (host, port, topic, username, password) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 8883);
        assert_eq!(topic, "sensors/weather");
        assert_eq!(username, Some("testuser".to_string()));
        assert_eq!(password, Some("testpass".to_string()));
    }
}
