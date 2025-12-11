use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{MySqlPool, PgPool};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Connection string (e.g., "postgres://user:pass@localhost/db" or "mysql://user:pass@localhost/db")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_string: Option<String>,

    /// Database type: "postgres" or "mysql"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_type: Option<String>,

    /// Database host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Database port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Database username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Database password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Database name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,

    /// Table name (default: "wx_records")
    #[serde(default = "default_table_name")]
    pub table_name: String,

    /// Path to CA certificate file for TLS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert: Option<String>,

    /// Path to client certificate file for TLS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,

    /// Path to client key file for TLS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,

    /// Whether to require TLS (default: false)
    #[serde(default)]
    pub require_tls: bool,

    /// Skip SSL certificate verification (default: false)
    #[serde(default)]
    pub skip_ssl_verify: bool,
}

fn default_table_name() -> String {
    "wx_records".to_string()
}

impl DatabaseConfig {
    /// Build a connection string from individual fields
    pub fn build_connection_string(&self) -> Result<String> {
        if let Some(ref conn_str) = self.connection_string {
            return Ok(conn_str.clone());
        }

        let db_type = self
            .db_type
            .as_ref()
            .context("Database type must be specified")?;
        let host = self
            .host
            .as_ref()
            .context("Database host must be specified")?;
        let username = self
            .username
            .as_ref()
            .context("Database username must be specified")?;
        let password = self
            .password
            .as_ref()
            .context("Database password must be specified")?;
        let database = self
            .database
            .as_ref()
            .context("Database name must be specified")?;

        let port = self.port.unwrap_or(match db_type.as_str() {
            "postgres" => 5432,
            "mysql" => 3306,
            _ => 5432,
        });

        let mut conn_str = format!(
            "{}://{}:{}@{}:{}/{}",
            db_type, username, password, host, port, database
        );

        // Append TLS parameters if configured
        let mut params = Vec::new();

        if self.require_tls {
            params.push("sslmode=require".to_string());
        }

        if self.skip_ssl_verify {
            // PostgreSQL uses sslmode=require with verify disabled
            // MySQL uses ssl-mode=REQUIRED with verify disabled
            if db_type == "postgres" {
                params.push("sslmode=require".to_string());
            }
            params.push("ssl-verify=false".to_string());
        }

        if let Some(ca_cert) = &self.ca_cert {
            params.push(format!("sslrootcert={}", ca_cert));
        }

        if let Some(client_cert) = &self.client_cert {
            params.push(format!("sslcert={}", client_cert));
        }

        if let Some(client_key) = &self.client_key {
            params.push(format!("sslkey={}", client_key));
        }

        if !params.is_empty() {
            conn_str.push('?');
            conn_str.push_str(&params.join("&"));
        }

        Ok(conn_str)
    }
}

pub enum DatabasePool {
    Postgres(PgPool),
    MySql(MySqlPool),
}

pub struct DatabaseWriter {
    pool: DatabasePool,
    table_name: String,
}

impl DatabaseWriter {
    /// Create a new database writer from configuration
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let connection_string = config.build_connection_string()?;

        // Determine database type and create appropriate pool
        let pool = if connection_string.starts_with("postgres://") {
            let pg_pool = PgPool::connect(&connection_string)
                .await
                .context("Failed to connect to PostgreSQL database")?;
            DatabasePool::Postgres(pg_pool)
        } else if connection_string.starts_with("mysql://") {
            let mysql_pool = MySqlPool::connect(&connection_string)
                .await
                .context("Failed to connect to MySQL database")?;
            DatabasePool::MySql(mysql_pool)
        } else {
            anyhow::bail!("Unsupported database type. Use postgres:// or mysql://");
        };

        let writer = Self {
            pool,
            table_name: config.table_name.clone(),
        };

        // Check if table exists, prompt to create if not
        if !writer.table_exists().await? {
            println!(
                "Table '{}' does not exist in the database.",
                writer.table_name
            );
            print!("Would you like to create it now? (Y/n): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input.is_empty() || input == "y" || input == "yes" {
                println!("Creating table '{}'...", writer.table_name);
                writer.create_table().await?;
                println!("✓ Table '{}' created successfully", writer.table_name);
            } else {
                anyhow::bail!(
                    "Table '{}' does not exist. Cannot proceed without it. \
                    Run with --db-create-table to create it non-interactively.",
                    writer.table_name
                );
            }
        }

        Ok(writer)
    }

    /// Check if the table exists in the database
    async fn table_exists(&self) -> Result<bool> {
        let exists = match &self.pool {
            DatabasePool::Postgres(pool) => {
                let query = "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_name = $1
                )";
                let row: (bool,) = sqlx::query_as(query)
                    .bind(&self.table_name)
                    .fetch_one(pool)
                    .await
                    .context("Failed to check if table exists")?;
                row.0
            }
            DatabasePool::MySql(pool) => {
                let query = "SELECT COUNT(*) > 0 FROM information_schema.tables 
                    WHERE table_name = ? AND table_schema = DATABASE()";
                let row: (i64,) = sqlx::query_as(query)
                    .bind(&self.table_name)
                    .fetch_one(pool)
                    .await
                    .context("Failed to check if table exists")?;
                row.0 > 0
            }
        };

        Ok(exists)
    }

    /// Create the weather data table if it doesn't exist
    pub async fn create_table(&self) -> Result<()> {
        let create_table_sql = match &self.pool {
            DatabasePool::Postgres(_) => format!(
                r#"
                CREATE TABLE IF NOT EXISTS {} (
                    id SERIAL PRIMARY KEY,
                    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                    intemp DOUBLE PRECISION,
                    outtemp DOUBLE PRECISION,
                    inhumid DOUBLE PRECISION,
                    outhumid DOUBLE PRECISION,
                    absbarometer DOUBLE PRECISION,
                    relbarometer DOUBLE PRECISION,
                    wind_dir DOUBLE PRECISION,
                    wind_speed DOUBLE PRECISION,
                    gust_speed DOUBLE PRECISION,
                    rain_event DOUBLE PRECISION,
                    rain_rate DOUBLE PRECISION,
                    rain_day DOUBLE PRECISION,
                    rain_week DOUBLE PRECISION,
                    rain_month DOUBLE PRECISION,
                    rain_year DOUBLE PRECISION,
                    light DOUBLE PRECISION,
                    uv DOUBLE PRECISION,
                    uvi DOUBLE PRECISION,
                    day_max_wind DOUBLE PRECISION
                )
                "#,
                self.table_name
            ),
            DatabasePool::MySql(_) => format!(
                r#"
                CREATE TABLE IF NOT EXISTS {} (
                    id INT AUTO_INCREMENT PRIMARY KEY,
                    timestamp TIMESTAMP NOT NULL,
                    intemp DOUBLE,
                    outtemp DOUBLE,
                    inhumid DOUBLE,
                    outhumid DOUBLE,
                    absbarometer DOUBLE,
                    relbarometer DOUBLE,
                    wind_dir DOUBLE,
                    wind_speed DOUBLE,
                    gust_speed DOUBLE,
                    rain_event DOUBLE,
                    rain_rate DOUBLE,
                    rain_day DOUBLE,
                    rain_week DOUBLE,
                    rain_month DOUBLE,
                    rain_year DOUBLE,
                    light DOUBLE,
                    uv DOUBLE,
                    uvi DOUBLE,
                    day_max_wind DOUBLE
                )
                "#,
                self.table_name
            ),
        };

        match &self.pool {
            DatabasePool::Postgres(pool) => {
                sqlx::query(&create_table_sql)
                    .execute(pool)
                    .await
                    .context("Failed to create table")?;
            }
            DatabasePool::MySql(pool) => {
                sqlx::query(&create_table_sql)
                    .execute(pool)
                    .await
                    .context("Failed to create table")?;
            }
        }

        Ok(())
    }

    /// Insert weather data into the database
    pub async fn insert_data(
        &self,
        data: &HashMap<String, f64>,
        timestamp: &DateTime<Utc>,
    ) -> Result<()> {
        // Filter out heap_free as requested
        let filtered_data: HashMap<String, f64> = data
            .iter()
            .filter(|(key, _)| *key != "heap_free")
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Build column names and placeholders
        let mut columns = vec!["timestamp".to_string()];

        // Add data columns
        for key in filtered_data.keys() {
            columns.push(key.clone());
        }

        match &self.pool {
            DatabasePool::Postgres(pool) => {
                // PostgreSQL uses $1, $2, etc.
                let placeholders = (1..=columns.len())
                    .map(|i| format!("${}", i))
                    .collect::<Vec<_>>()
                    .join(", ");

                let insert_sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    self.table_name,
                    columns.join(", "),
                    placeholders
                );

                let mut query = sqlx::query(&insert_sql);
                query = query.bind(timestamp);

                for key in filtered_data.keys() {
                    if let Some(value) = filtered_data.get(key) {
                        query = query.bind(value);
                    }
                }

                query.execute(pool).await.context("Failed to insert data")?;
            }
            DatabasePool::MySql(pool) => {
                // MySQL uses ?
                let placeholders = vec!["?"; columns.len()].join(", ");

                let insert_sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    self.table_name,
                    columns.join(", "),
                    placeholders
                );

                let mut query = sqlx::query(&insert_sql);
                query = query.bind(timestamp);

                for key in filtered_data.keys() {
                    if let Some(value) = filtered_data.get(key) {
                        query = query.bind(value);
                    }
                }

                query.execute(pool).await.context("Failed to insert data")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_table_name() {
        assert_eq!(default_table_name(), "wx_records");
    }

    #[test]
    fn test_build_connection_string_from_string() {
        let config = DatabaseConfig {
            connection_string: Some("postgres://user:pass@localhost/mydb".to_string()),
            db_type: None,
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: false,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert_eq!(conn_str, "postgres://user:pass@localhost/mydb");
    }

    #[test]
    fn test_build_connection_string_from_fields_postgres() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: Some("postgres".to_string()),
            host: Some("localhost".to_string()),
            port: Some(5432),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            database: Some("mydb".to_string()),
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: false,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert_eq!(conn_str, "postgres://user:pass@localhost:5432/mydb");
    }

    #[test]
    fn test_build_connection_string_from_fields_mysql() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: Some("mysql".to_string()),
            host: Some("localhost".to_string()),
            port: Some(3306),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            database: Some("mydb".to_string()),
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: false,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert_eq!(conn_str, "mysql://user:pass@localhost:3306/mydb");
    }

    #[test]
    fn test_build_connection_string_default_port_postgres() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: Some("postgres".to_string()),
            host: Some("localhost".to_string()),
            port: None,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            database: Some("mydb".to_string()),
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: false,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert_eq!(conn_str, "postgres://user:pass@localhost:5432/mydb");
    }

    #[test]
    fn test_build_connection_string_default_port_mysql() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: Some("mysql".to_string()),
            host: Some("localhost".to_string()),
            port: None,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            database: Some("mydb".to_string()),
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: false,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert_eq!(conn_str, "mysql://user:pass@localhost:3306/mydb");
    }

    #[test]
    fn test_build_connection_string_missing_fields() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: None,
            host: Some("localhost".to_string()),
            port: None,
            username: None,
            password: None,
            database: None,
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: false,
        };

        let result = config.build_connection_string();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Database type must be specified"));
    }

    #[test]
    fn test_build_connection_string_with_skip_ssl_verify_postgres() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: Some("postgres".to_string()),
            host: Some("localhost".to_string()),
            port: Some(5432),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            database: Some("mydb".to_string()),
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: true,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert!(conn_str.contains("sslmode=require"));
        assert!(conn_str.contains("ssl-verify=false"));
    }

    #[test]
    fn test_build_connection_string_with_skip_ssl_verify_mysql() {
        let config = DatabaseConfig {
            connection_string: None,
            db_type: Some("mysql".to_string()),
            host: Some("localhost".to_string()),
            port: Some(3306),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            database: Some("mydb".to_string()),
            table_name: "wx_records".to_string(),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            require_tls: false,
            skip_ssl_verify: true,
        };

        let conn_str = config.build_connection_string().unwrap();
        assert!(conn_str.contains("ssl-verify=false"));
        assert!(!conn_str.contains("sslmode=require")); // MySQL doesn't use sslmode
    }
}
