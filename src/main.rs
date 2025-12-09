mod client;
mod config;
mod decoder;
mod output;
mod protocol;
mod web;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use std::time::Duration;

use client::GW1000Client;
use config::Args;
use output::print_livedata;
use web::{run_web_server, WebServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Get connection info from args or config
    let (ip, port) = args.get_connection_info()?;

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
    println!("Press Ctrl+C to stop\n");

    loop {
        match client.get_livedata() {
            Ok(data) => {
                let timestamp = Utc::now();
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
