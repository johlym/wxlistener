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

    pub fn get_connection_info(&self) -> Result<(String, u16, String)> {
        if let Some(conn_str) = &self.connection_string {
            self.parse_connection_string(conn_str)
        } else if let Some(host) = &self.host {
            let port = self.port.unwrap_or(1883);
            let topic = self
                .topic
                .clone()
                .unwrap_or_else(|| "wx/live".to_string());
            Ok((host.clone(), port, topic))
        } else {
            anyhow::bail!(
                "MQTT broker must be specified via:\n\
                 - Connection string: mqtt://host:port/topic\n\
                 - Individual fields: host, port (optional), topic (optional)"
            );
        }
    }

    fn parse_connection_string(&self, conn_str: &str) -> Result<(String, u16, String)> {
        let url = url::Url::parse(conn_str)
            .context("Failed to parse MQTT connection string")?;

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
            self.topic
                .clone()
                .unwrap_or_else(|| "wx/live".to_string())
        };

        Ok((host, port, topic))
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
        let (host, port, topic) = config.get_connection_info()?;
        let client_id = config.get_client_id();

        let mut mqtt_options = MqttOptions::new(client_id, host, port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                        println!("Connected to MQTT broker");
                    }
                    Ok(Event::Incoming(Incoming::Disconnect)) => {
                        println!("Disconnected from MQTT broker");
                    }
                    Err(e) => {
                        eprintln!("MQTT connection error: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                    _ => {}
                }
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(Self { client, topic })
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

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
    }

    #[test]
    fn test_parse_connection_string_default_port() {
        let config = MqttConfig {
            connection_string: Some("mqtt://broker.example.com/weather/data".to_string()),
            ..Default::default()
        };

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "broker.example.com");
        assert_eq!(port, 1883);
        assert_eq!(topic, "weather/data");
    }

    #[test]
    fn test_parse_connection_string_no_topic() {
        let config = MqttConfig {
            connection_string: Some("mqtt://localhost:1883".to_string()),
            ..Default::default()
        };

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
    }

    #[test]
    fn test_parse_connection_string_with_config_topic() {
        let config = MqttConfig {
            connection_string: Some("mqtt://localhost:1883".to_string()),
            topic: Some("custom/topic".to_string()),
            ..Default::default()
        };

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);
        assert_eq!(topic, "custom/topic");
    }

    #[test]
    fn test_connection_from_fields() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            port: Some(8883),
            topic: Some("sensors/weather".to_string()),
            ..Default::default()
        };

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 8883);
        assert_eq!(topic, "sensors/weather");
    }

    #[test]
    fn test_connection_from_fields_default_port() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            topic: Some("weather".to_string()),
            ..Default::default()
        };

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 1883);
        assert_eq!(topic, "weather");
    }

    #[test]
    fn test_connection_from_fields_default_topic() {
        let config = MqttConfig {
            host: Some("mqtt.example.com".to_string()),
            ..Default::default()
        };

        let (host, port, topic) = config.get_connection_info().unwrap();
        assert_eq!(host, "mqtt.example.com");
        assert_eq!(port, 1883);
        assert_eq!(topic, "wx/live");
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
}
