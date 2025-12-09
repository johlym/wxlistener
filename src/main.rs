mod client;
mod config;
mod decoder;
mod output;
mod protocol;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use std::time::Duration;

use client::GW1000Client;
use config::Args;
use output::print_livedata;

fn main() -> Result<()> {
    let args = Args::parse();

    // Get connection info from args or config
    let (ip, port) = args.get_connection_info()?;

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

    // Continuous or single read
    if let Some(interval) = args.continuous {
        println!("\n--- Continuous Mode (every {} seconds) ---", interval);
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
            std::thread::sleep(Duration::from_secs(interval));
        }
    } else {
        println!("\n--- Live Data ---");
        let data = client.get_livedata()?;
        let timestamp = Utc::now();
        
        if args.format == "json" {
            println!("{}", serde_json::to_string_pretty(&data)?);
        } else {
            print_livedata(&data, &timestamp);
        }
    }

    Ok(())
}
