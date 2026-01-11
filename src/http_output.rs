use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    /// HTTP endpoint URL to POST weather data to
    pub url: Option<String>,
    /// Request timeout in seconds (default: 10)
    pub timeout: Option<u64>,
    /// Optional authorization header value (e.g., "Bearer <token>")
    pub authorization: Option<String>,
}

impl HttpConfig {
    pub fn new() -> Self {
        Self {
            url: None,
            timeout: None,
            authorization: None,
        }
    }

    pub fn get_url(&self) -> Result<String> {
        if let Some(url) = &self.url {
            Ok(url.clone())
        } else if let Ok(url) = std::env::var("WXLISTENER_HTTP_URL") {
            Ok(url)
        } else {
            anyhow::bail!(
                "HTTP endpoint URL must be specified via:\n\
                 - Config file: [http] url = \"https://example.com/api/weather\"\n\
                 - Environment: WXLISTENER_HTTP_URL=<URL>"
            );
        }
    }

    pub fn get_timeout(&self) -> Duration {
        let secs = self.timeout.unwrap_or(10);
        Duration::from_secs(secs)
    }

    pub fn get_authorization(&self) -> Option<String> {
        self.authorization
            .clone()
            .or_else(|| std::env::var("WXLISTENER_HTTP_AUTH").ok())
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Weather measurement payload matching the required schema
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct WeatherPayload {
    pub weather_measurement: WeatherMeasurement,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeatherMeasurement {
    pub reading_date_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barometer_abs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barometer_rel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_max_wind: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dewpoint: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gust_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heatindex: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humidity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windchill: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain_day: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain_event: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uvi: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_dir: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f64>,
}

impl WeatherMeasurement {
    /// Create a WeatherMeasurement from raw sensor data
    pub fn from_data(data: &HashMap<String, f64>, timestamp: &DateTime<Utc>) -> Self {
        Self {
            reading_date_time: timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            barometer_abs: data.get("absbarometer").copied(),
            barometer_rel: data.get("relbarometer").copied(),
            day_max_wind: data.get("day_max_wind").copied(),
            dewpoint: data.get("dewpoint").copied(),
            gust_speed: data.get("gust_speed").copied(),
            heatindex: data.get("heatindex").copied(),
            humidity: data.get("outhumid").map(|v| *v as i32),
            light: data.get("light").copied(),
            windchill: data.get("windchill").copied(),
            rain_day: data.get("rain_day").copied(),
            rain_event: data.get("rain_event").copied(),
            rain_rate: data.get("rain_rate").copied(),
            temperature: data.get("outtemp").copied(),
            uv: data.get("uv").map(|v| *v as i32),
            uvi: data.get("uvi").map(|v| *v as i32),
            wind_dir: data.get("wind_dir").map(|v| *v as i32),
            wind_speed: data.get("wind_speed").copied(),
        }
    }
}

/// A queued payload waiting to be sent
#[derive(Debug, Clone, Serialize)]
struct QueuedPayload {
    weather_measurement: WeatherMeasurement,
}

pub struct HttpPublisher {
    client: Client,
    url: String,
    authorization: Option<String>,
    queue: Arc<Mutex<VecDeque<QueuedPayload>>>,
    is_draining: Arc<Mutex<bool>>,
}

impl HttpPublisher {
    pub async fn new(config: &HttpConfig) -> Result<Self> {
        let url = config.get_url()?;
        let timeout = config.get_timeout();
        let authorization = config.get_authorization();

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to create HTTP client")?;

        // Validate URL format
        reqwest::Url::parse(&url).context("Invalid HTTP endpoint URL")?;

        let publisher = Self {
            client,
            url,
            authorization,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            is_draining: Arc::new(Mutex::new(false)),
        };

        Ok(publisher)
    }

    /// Attempt to send a payload to the HTTP endpoint
    async fn try_send(&self, payload: &QueuedPayload) -> Result<()> {
        let mut request = self.client.post(&self.url).json(payload);

        if let Some(auth) = &self.authorization {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .context("Failed to send HTTP request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("HTTP request failed with status {}: {}", status, body);
        }

        Ok(())
    }

    /// Start the background queue drain task
    fn start_drain_task(&self) {
        let client = self.client.clone();
        let url = self.url.clone();
        let authorization = self.authorization.clone();
        let queue = self.queue.clone();
        let is_draining = self.is_draining.clone();

        tokio::spawn(async move {
            loop {
                // Check if there are items to drain
                let payload = {
                    let q = queue.lock().await;
                    if q.is_empty() {
                        *is_draining.lock().await = false;
                        break;
                    }
                    q.front().cloned()
                };

                if let Some(payload) = payload {
                    // Try to send
                    let mut request = client.post(&url).json(&payload);
                    if let Some(auth) = &authorization {
                        request = request.header("Authorization", auth);
                    }

                    match request.send().await {
                        Ok(response) if response.status().is_success() => {
                            // Success - remove from queue
                            let mut q = queue.lock().await;
                            q.pop_front();
                            let remaining = q.len();
                            drop(q);

                            if remaining > 0 {
                                println!(
                                    "  [OK] HTTP queue: sent 1 record ({} remaining)",
                                    remaining
                                );
                            } else {
                                println!("  [OK] HTTP queue: emptied (all records sent)");
                            }
                        }
                        Ok(response) => {
                            // Server error - wait and retry
                            eprintln!(
                                "  [WARN] HTTP queue: server returned {}, retrying in 1s...",
                                response.status()
                            );
                        }
                        Err(e) => {
                            // Connection error - wait and retry
                            eprintln!(
                                "  [WARN] HTTP queue: connection failed ({}), retrying in 1s...",
                                e
                            );
                        }
                    }
                }

                // Wait 1 second before next attempt
                time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    /// Publish weather data to the HTTP endpoint
    /// If the endpoint is unreachable, data is queued for later delivery
    pub async fn publish(&self, data: &HashMap<String, f64>, timestamp: &DateTime<Utc>) {
        let measurement = WeatherMeasurement::from_data(data, timestamp);
        let payload = QueuedPayload {
            weather_measurement: measurement,
        };

        // Check if we're currently draining the queue
        let is_draining = *self.is_draining.lock().await;

        if is_draining {
            // Queue is being drained, add to end of queue
            let mut q = self.queue.lock().await;
            q.push_back(payload);
            println!("  [QUEUE] HTTP: queued record ({} in queue)", q.len());
            return;
        }

        // Try to send directly
        match self.try_send(&payload).await {
            Ok(()) => {
                println!(
                    "  [OK] HTTP: sent record ({})",
                    timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            Err(e) => {
                // Failed - add to queue and start drain task
                eprintln!("  [WARN] HTTP publish failed: {}", e);
                let mut q = self.queue.lock().await;
                q.push_back(payload);
                let queue_len = q.len();
                drop(q);

                println!(
                    "  [QUEUE] HTTP: queued record ({} in queue), will retry...",
                    queue_len
                );

                // Start drain task if not already running
                *self.is_draining.lock().await = true;
                self.start_drain_task();
            }
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get current queue length
    #[allow(dead_code)]
    pub async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_new() {
        let config = HttpConfig::new();
        assert!(config.url.is_none());
        assert!(config.timeout.is_none());
        assert!(config.authorization.is_none());
    }

    #[test]
    fn test_http_config_get_timeout_default() {
        let config = HttpConfig::new();
        assert_eq!(config.get_timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_http_config_get_timeout_custom() {
        let config = HttpConfig {
            url: None,
            timeout: Some(30),
            authorization: None,
        };
        assert_eq!(config.get_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_http_config_get_url_from_config() {
        let config = HttpConfig {
            url: Some("https://example.com/api".to_string()),
            timeout: None,
            authorization: None,
        };
        assert_eq!(config.get_url().unwrap(), "https://example.com/api");
    }

    #[test]
    fn test_http_config_missing_url() {
        let config = HttpConfig::new();
        // Only fails if env var is also not set
        std::env::remove_var("WXLISTENER_HTTP_URL");
        assert!(config.get_url().is_err());
    }

    #[test]
    fn test_weather_measurement_from_data() {
        let mut data = HashMap::new();
        data.insert("outtemp".to_string(), 25.5);
        data.insert("outhumid".to_string(), 65.0);
        data.insert("absbarometer".to_string(), 1013.25);
        data.insert("relbarometer".to_string(), 1010.0);
        data.insert("wind_speed".to_string(), 5.5);
        data.insert("gust_speed".to_string(), 8.2);
        data.insert("wind_dir".to_string(), 180.0);
        data.insert("day_max_wind".to_string(), 12.0);
        data.insert("rain_day".to_string(), 2.5);
        data.insert("rain_event".to_string(), 1.0);
        data.insert("rain_rate".to_string(), 0.5);
        data.insert("light".to_string(), 50000.0);
        data.insert("uv".to_string(), 5.0);
        data.insert("uvi".to_string(), 3.0);

        let timestamp = Utc::now();
        let measurement = WeatherMeasurement::from_data(&data, &timestamp);

        assert_eq!(measurement.temperature, Some(25.5));
        assert_eq!(measurement.humidity, Some(65));
        assert_eq!(measurement.barometer_abs, Some(1013.25));
        assert_eq!(measurement.barometer_rel, Some(1010.0));
        assert_eq!(measurement.wind_speed, Some(5.5));
        assert_eq!(measurement.gust_speed, Some(8.2));
        assert_eq!(measurement.wind_dir, Some(180));
        assert_eq!(measurement.day_max_wind, Some(12.0));
        assert_eq!(measurement.rain_day, Some(2.5));
        assert_eq!(measurement.rain_event, Some(1.0));
        assert_eq!(measurement.rain_rate, Some(0.5));
        assert_eq!(measurement.light, Some(50000.0));
        assert_eq!(measurement.uv, Some(5));
        assert_eq!(measurement.uvi, Some(3));
    }

    #[test]
    fn test_weather_measurement_partial_data() {
        let mut data = HashMap::new();
        data.insert("outtemp".to_string(), 20.0);
        data.insert("outhumid".to_string(), 50.0);

        let timestamp = Utc::now();
        let measurement = WeatherMeasurement::from_data(&data, &timestamp);

        assert_eq!(measurement.temperature, Some(20.0));
        assert_eq!(measurement.humidity, Some(50));
        assert!(measurement.barometer_abs.is_none());
        assert!(measurement.wind_speed.is_none());
    }

    #[test]
    fn test_weather_payload_serialization() {
        let mut data = HashMap::new();
        data.insert("outtemp".to_string(), 22.5);
        data.insert("outhumid".to_string(), 55.0);

        let timestamp = Utc::now();
        let measurement = WeatherMeasurement::from_data(&data, &timestamp);
        let payload = WeatherPayload {
            weather_measurement: measurement,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("weather_measurement"));
        assert!(json.contains("temperature"));
        assert!(json.contains("humidity"));
        assert!(json.contains("reading_date_time"));
        // Should not contain null fields due to skip_serializing_if
        assert!(!json.contains("barometer_abs"));
    }
}
