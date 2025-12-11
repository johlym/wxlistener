mod client;
mod config;
mod database;
mod decoder;
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
use mqtt::MqttPublisher;
use output::print_livedata;
use web::{run_web_server, WebServerConfig};

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
        Err(_) => {
            // Print help and exit if required arguments are missing
            Args::parse_from(["wxlistener", "--help"]);
            unreachable!()
        }
    };

    // Check if web mode is enabled
    if args.web {
        let web_config = WebServerConfig {
            ip: args.web_host.clone(),
            port: args.web_port,
            interval: args.continuous,
        };
        return run_web_server(web_config, ip, port).await;
    }

    let client = GW1000Client::new(ip.clone(), port);

    // Initialize database writer if configured
    let db_writer = if let Some(db_config) = args.get_database_config()? {
        match DatabaseWriter::new(&db_config).await {
            Ok(writer) => {
                println!("✓ Connected to database and table verified");
                Some(writer)
            }
            Err(e) => {
                eprintln!("✗ Database error: {}", e);
                eprintln!("  Continuing without database support");
                None
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
                eprintln!("✗ MQTT error: {}", e);
                eprintln!("  Continuing without MQTT support");
                None
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
    println!("Press Ctrl+C to stop\n");

    loop {
        match client.get_livedata() {
            Ok(data) => {
                let timestamp = Utc::now();

                // Write to database if configured
                if let Some(ref writer) = db_writer {
                    if let Err(e) = writer.insert_data(&data, &timestamp).await {
                        eprintln!("Database write error: {}", e);
                    }
                }

                // Display output
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
