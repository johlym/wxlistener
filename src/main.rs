mod client;
mod config;
mod database;
mod debug_sensors;
mod decoder;
mod gateway;
mod http_gateway;
mod http_output;
mod mqtt;
mod output;
mod protocol;
mod web;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use std::time::Duration;

use config::Args;
use database::DatabaseWriter;
use debug_sensors::run_debug_sensors;
use gateway::{blocking_gateway, Gateway};
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
        println!("[OK] Table '{}' created successfully", db_config.table_name);
        return Ok(());
    }

    // Get connection info from args or config
    let settings = match args.gateway_settings() {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let ip = settings.ip.clone();
    let port = settings.port;
    let legacy = settings.legacy;

    let gateway = Gateway::connect(&settings)?;

    // One-shot sensor inventory probe (WN34S / WH34 TF validation)
    if args.debug_sensors {
        run_debug_sensors(&gateway)?;
        return Ok(());
    }

    // Initialize database writer if configured
    let db_writer = if let Some(db_config) = args.get_database_config()? {
        match DatabaseWriter::new(&db_config).await {
            Ok(writer) => {
                println!("[OK] Connected to database and table verified");
                Some(writer)
            }
            Err(e) => {
                eprintln!("[ERROR] Database connection failed: {}", e);
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
                println!(
                    "[OK] Connected to MQTT broker (topic: {})",
                    publisher.topic()
                );
                Some(publisher)
            }
            Err(e) => {
                eprintln!("[ERROR] MQTT connection failed: {}", e);
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
                println!("[OK] HTTP endpoint configured (url: {})", publisher.url());
                Some(publisher)
            }
            Err(e) => {
                eprintln!("[ERROR] HTTP configuration failed: {}", e);
                eprintln!("  Cannot continue with HTTP as it is currently configured.");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    println!("============================================================");
    println!("GW3000/Ecowitt Gateway Weather Station Listener");
    println!("============================================================");
    println!(
        "Transport: {}",
        if legacy { "legacy TCP" } else { "HTTP API" }
    );
    println!("Target device: {}:{}", ip, port);
    println!();

    // Get device info
    println!("--- Device Information ---");
    match blocking_gateway(gateway.clone(), |gateway| gateway.get_firmware_version()).await {
        Ok(version) => println!("[OK] Firmware Version: {}", version),
        Err(e) => println!("[ERROR] Failed to get firmware: {}", e),
    }

    match blocking_gateway(gateway.clone(), |gateway| gateway.get_mac_address()).await {
        Ok(mac) => println!("[OK] MAC Address: {}", mac),
        Err(e) => println!("[ERROR] Failed to get MAC: {}", e),
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
        run_web_server_background(web_config, gateway.clone());
        println!(
            "Web server: ENABLED (http://{}:{})",
            args.web_host, args.web_port
        );
    }

    println!("Press Ctrl+C to stop\n");

    loop {
        match blocking_gateway(gateway.clone(), |gateway| gateway.get_livedata()).await {
            Ok(data) => {
                let timestamp = Utc::now();

                // Write to database if configured
                if let Some(ref writer) = db_writer {
                    if let Err(e) = writer.insert_data(&data, &timestamp).await {
                        eprintln!("[ERROR] Database write error: {}", e);
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
                        eprintln!("[ERROR] MQTT publish error: {}", e);
                        eprintln!("  Cannot continue with MQTT configuration.");
                        std::process::exit(1);
                    }
                }

                // Publish to HTTP endpoint if configured
                if let Some(ref publisher) = http_publisher {
                    publisher.publish(&data, &timestamp).await;
                }

                // Always print each measurement, regardless of configured sinks
                if args.format == "json" {
                    println!("{}", serde_json::to_string_pretty(&data)?);
                } else {
                    print_livedata(&data, &timestamp);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(args.continuous)).await;
    }
}
