mod client;
mod config;
mod database;
mod decoder;
mod http_output;
mod mqtt;
mod output;
mod protocol;
mod web;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use std::time::Duration;

use client::GW1000Client;
use config::Args;
use database::DatabaseWriter;
use http_output::HttpPublisher;
use mqtt::MqttPublisher;
use output::print_livedata;
use web::{run_web_server_background, WebServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle database table creation mode
    if args.db_create_table {
        let db_config = args.get_database_config()?.ok_or_else(|| {
            anyhow::anyhow!(
                "Database configuration required. Add [database] section to config file."
            )
        })?;

        println!("Creating database table...");
        let writer = DatabaseWriter::new(&db_config).await?;
        writer.create_table().await?;
        println!("✓ Table '{}' created successfully", db_config.table_name);
        return Ok(());
    }

    // Get connection info from args or config
    let (ip, port) = match args.get_connection_info() {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let client = GW1000Client::new(ip.clone(), port);

    // Initialize database writer if configured
    let db_writer = if let Some(db_config) = args.get_database_config()? {
        match DatabaseWriter::new(&db_config).await {
            Ok(writer) => {
                println!("✓ Connected to database and table verified");
                Some(writer)
            }
            Err(e) => {
                eprintln!("✗ Database connection failed: {}", e);
                eprintln!("  Cannot continue with database configuration.");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // Initialize MQTT publisher if configured
    let mqtt_publisher = if let Some(mqtt_config) = args.get_mqtt_config()? {
        match MqttPublisher::new(&mqtt_config).await {
            Ok(publisher) => {
                println!("✓ Connected to MQTT broker (topic: {})", publisher.topic());
                Some(publisher)
            }
            Err(e) => {
                eprintln!("✗ MQTT connection failed: {}", e);
                eprintln!("  Cannot continue with MQTT as it is currently configured.");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // Initialize HTTP publisher if configured
    let http_publisher = if let Some(http_config) = args.get_http_config()? {
        match HttpPublisher::new(&http_config).await {
            Ok(publisher) => {
                println!("✓ HTTP endpoint configured (url: {})", publisher.url());
                Some(publisher)
            }
            Err(e) => {
                eprintln!("✗ HTTP configuration failed: {}", e);
                eprintln!("  Cannot continue with HTTP as it is currently configured.");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    println!("============================================================");
    println!("GW1000/Ecowitt Gateway Weather Station Listener");
    println!("============================================================");
    println!("Target device: {}:{}", ip, port);
    println!();

    // Get device info
    println!("--- Device Information ---");
    match client.get_firmware_version() {
        Ok(version) => println!("✓ Firmware Version: {}", version),
        Err(e) => println!("✗ Failed to get firmware: {}", e),
    }

    match client.get_mac_address() {
        Ok(mac) => println!("✓ MAC Address: {}", mac),
        Err(e) => println!("✗ Failed to get MAC: {}", e),
    }

    // Continuous mode (default)
    println!(
        "\n--- Continuous Mode (every {} seconds) ---",
        args.continuous
    );
    if db_writer.is_some() {
        println!("Database logging: ENABLED");
    }
    if mqtt_publisher.is_some() {
        println!("MQTT publishing: ENABLED");
    }
    if http_publisher.is_some() {
        println!("HTTP publishing: ENABLED");
    }

    // Start web server in background if enabled
    if args.web {
        let web_config = WebServerConfig {
            ip: args.web_host.clone(),
            port: args.web_port,
            interval: args.continuous,
        };
        run_web_server_background(web_config, ip.clone(), port);
        println!("Web server: ENABLED (http://{}:{})", args.web_host, args.web_port);
    }

    println!("Press Ctrl+C to stop\n");

    loop {
        match client.get_livedata() {
            Ok(data) => {
                let timestamp = Utc::now();

                // Write to database if configured
                if let Some(ref writer) = db_writer {
                    if let Err(e) = writer.insert_data(&data, &timestamp).await {
                        eprintln!("✗ Database write error: {}", e);
                        eprintln!("  Cannot continue with database configuration.");
                        std::process::exit(1);
                    }
                }

                // Publish to MQTT if configured
                if let Some(ref publisher) = mqtt_publisher {
                    let json_data = serde_json::json!({
                        "timestamp": timestamp.to_rfc3339(),
                        "data": data
                    });
                    if let Err(e) = publisher.publish(&json_data.to_string()).await {
                        eprintln!("✗ MQTT publish error: {}", e);
                        eprintln!("  Cannot continue with MQTT configuration.");
                        std::process::exit(1);
                    }
                }

                // Publish to HTTP endpoint if configured
                if let Some(ref publisher) = http_publisher {
                    publisher.publish(&data, &timestamp).await;
                }

                // Display output only if no output sink is configured
                if db_writer.is_none() && mqtt_publisher.is_none() && http_publisher.is_none() {
                    if args.format == "json" {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    } else {
                        print_livedata(&data, &timestamp);
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(args.continuous)).await;
    }
}
