# Proxmox LXC Deployment Guide

> [!WARNING]
> This script is not yet production-ready. Use at your own risk. I honestly have no idea if it works, yet.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Automated Setup](#automated-setup)
  - [Basic Usage](#basic-usage)
  - [Custom Configuration](#custom-configuration)
  - [Command Options](#command-options)
- [Manual Setup](#manual-setup)
  - [1. Create LXC Container](#1-create-lxc-container)
  - [2. Install Dependencies](#2-install-dependencies)
  - [3. Install Rust](#3-install-rust)
  - [4. Build wxlistener](#4-build-wxlistener)
  - [5. Configure Service](#5-configure-service)
- [Configuration](#configuration)
  - [Weather Station Settings](#weather-station-settings)
  - [Database Configuration](#database-configuration)
  - [Web Server Settings](#web-server-settings)
- [Service Management](#service-management)
  - [Starting the Service](#starting-the-service)
  - [Stopping the Service](#stopping-the-service)
  - [Viewing Logs](#viewing-logs)
  - [Restarting After Config Changes](#restarting-after-config-changes)
- [Resource Management](#resource-management)
  - [Memory Usage](#memory-usage)
  - [CPU Usage](#cpu-usage)
  - [Disk Space](#disk-space)
- [Networking](#networking)
  - [Static IP Configuration](#static-ip-configuration)
  - [DHCP Configuration](#dhcp-configuration)
  - [Firewall Rules](#firewall-rules)
  - [Port Forwarding](#port-forwarding)
- [Best Practices](#best-practices)
  - [Security](#security)
  - [Backups](#backups)
  - [Monitoring](#monitoring)
  - [Updates](#updates)
- [Troubleshooting](#troubleshooting)
  - [Container Won't Start](#container-wont-start)
  - [Build Failures](#build-failures)
  - [Network Issues](#network-issues)
  - [Service Won't Start](#service-wont-start)
- [Upgrading](#upgrading)
  - [Upgrading wxlistener](#upgrading-wxlistener)
  - [Upgrading Container OS](#upgrading-container-os)
- [Advanced Configuration](#advanced-configuration)
  - [Resource Limits](#resource-limits)
  - [Privileged vs Unprivileged](#privileged-vs-unprivileged)
  - [Nested Containers](#nested-containers)
- [Performance Tuning](#performance-tuning)
- [Migration](#migration)
  - [Backing Up Container](#backing-up-container)
  - [Restoring Container](#restoring-container)
  - [Moving Between Hosts](#moving-between-hosts)

## Overview

wxlistener can be deployed on Proxmox VE using LXC (Linux Containers) for efficient, lightweight virtualization. This guide covers both automated and manual setup methods.

**Container Specifications:**

- **OS**: Ubuntu 22.04 LTS (latest)
- **Memory**: 512MB RAM
- **CPU**: 1 core
- **Disk**: 4GB
- **Type**: Unprivileged container

## Quick Start

On your Proxmox host, run:

```bash
# Option 1: Download and run
wget https://raw.githubusercontent.com/johlym/wxlistener/main/bin/proxmox-lxc-setup
chmod +x proxmox-lxc-setup
./proxmox-lxc-setup

# Option 2: Run directly with bash -c (no download needed)
bash -c "$(wget -qO- https://raw.githubusercontent.com/johlym/wxlistener/main/bin/proxmox-lxc-setup)"

# With static IP (Option 1)
./proxmox-lxc-setup --ip 192.168.1.100/24 --gateway 192.168.1.1

# With static IP (Option 2 - bash -c with arguments)
bash -c "$(wget -qO- https://raw.githubusercontent.com/johlym/wxlistener/main/bin/proxmox-lxc-setup)" -- --ip 192.168.1.100/24 --gateway 192.168.1.1
```

The script will:

1. Create an LXC container with optimal settings
2. Install all dependencies
3. Build wxlistener from source
4. Configure systemd service
5. Set up configuration files

## Prerequisites

- Proxmox VE 7.0 or later
- Ubuntu 22.04 LTS template downloaded
- Sufficient storage space (minimum 4GB)
- Network connectivity for the container

**Download Ubuntu template:**

```bash
# On Proxmox host
pveam update
pveam download local ubuntu-22.04-standard_22.04-1_amd64.tar.zst
```

## Automated Setup

### Basic Usage

```bash
# Use defaults (auto CTID, DHCP, local-lvm storage)
./bin/proxmox-lxc-setup

# Specify container ID
./bin/proxmox-lxc-setup --ctid 200

# Custom hostname
./bin/proxmox-lxc-setup --hostname weather-station
```

### Custom Configuration

```bash
# Full custom setup
./bin/proxmox-lxc-setup \
  --ctid 200 \
  --hostname wxlistener \
  --storage local-lvm \
  --bridge vmbr0 \
  --ip 192.168.1.100/24 \
  --gateway 192.168.1.1
```

### Command Options

| Option                | Description        | Default                 |
| --------------------- | ------------------ | ----------------------- |
| `--ctid <ID>`         | Container ID       | Next available          |
| `--hostname <NAME>`   | Container hostname | `wxlistener`            |
| `--storage <STORAGE>` | Storage location   | `local-lvm`             |
| `--bridge <BRIDGE>`   | Network bridge     | `vmbr0`                 |
| `--ip <IP/CIDR>`      | Static IP address  | DHCP                    |
| `--gateway <IP>`      | Gateway IP         | Required with static IP |
| `--help`              | Show help message  | -                       |

## Manual Setup

### 1. Create LXC Container

```bash
# Create container
pct create 200 local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst \
  --hostname wxlistener \
  --memory 512 \
  --cores 1 \
  --rootfs local-lvm:4 \
  --net0 name=eth0,bridge=vmbr0,ip=dhcp \
  --unprivileged 1 \
  --features nesting=1 \
  --onboot 1 \
  --description "wxlistener - Weather Station Data Collector"

# Start container
pct start 200
```

### 2. Install Dependencies

```bash
# Enter container
pct enter 200

# Update system
apt-get update
apt-get upgrade -y

# Install build dependencies
apt-get install -y \
  curl \
  build-essential \
  pkg-config \
  libssl-dev \
  git \
  ca-certificates
```

### 3. Install Rust

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Add to bashrc for future sessions
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

### 4. Build wxlistener

```bash
# Clone repository
cd /opt
git clone https://github.com/johlym/wxlistener.git
cd wxlistener

# Build release binary
cargo build --release

# Install binary
cp target/release/wxlistener /usr/local/bin/
chmod +x /usr/local/bin/wxlistener
```

### 5. Configure Service

```bash
# Create config directory
mkdir -p /etc/wxlistener

# Copy example config
cp wxlistener.example.toml /etc/wxlistener/wxlistener.toml

# Edit configuration
nano /etc/wxlistener/wxlistener.toml

# Create systemd service
cat > /etc/systemd/system/wxlistener.service << 'EOF'
[Unit]
Description=wxlistener - Weather Station Data Collector
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/wxlistener
ExecStart=/usr/local/bin/wxlistener --config /etc/wxlistener/wxlistener.toml --web
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/etc/wxlistener

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable wxlistener
systemctl start wxlistener
```

## Configuration

### Weather Station Settings

Edit `/etc/wxlistener/wxlistener.toml`:

```toml
# Required: Your weather station IP
ip = "192.168.1.50"

# Optional: Custom port (default: 45000)
port = 45000
```

### Database Configuration

```toml
[database]
# PostgreSQL
connection_string = "postgres://wxlistener:password@192.168.1.10:5432/weather"

# Or MySQL
# connection_string = "mysql://wxlistener:password@192.168.1.10:3306/weather"

# Optional: Custom table name
table_name = "wx_records"
```

### Web Server Settings

The web server runs on port 18888 by default. To customize:

```bash
# Edit service file
systemctl edit wxlistener

# Add custom arguments
[Service]
ExecStart=
ExecStart=/usr/local/bin/wxlistener \
  --config /etc/wxlistener/wxlistener.toml \
  --web \
  --web-port 8080 \
  --web-host 0.0.0.0
```

## Service Management

### Starting the Service

```bash
systemctl start wxlistener
```

### Stopping the Service

```bash
systemctl stop wxlistener
```

### Viewing Logs

```bash
# Follow logs in real-time
journalctl -u wxlistener -f

# View last 100 lines
journalctl -u wxlistener -n 100

# View logs since boot
journalctl -u wxlistener -b
```

### Restarting After Config Changes

```bash
systemctl restart wxlistener
```

## Resource Management

### Memory Usage

The container is configured with 512MB RAM, which is sufficient for wxlistener:

```bash
# Check memory usage
pct exec 200 -- free -h

# Monitor in real-time
pct exec 200 -- top
```

**Typical memory usage**: 50-100MB

### CPU Usage

With 1 core allocated, wxlistener runs efficiently:

```bash
# Check CPU usage
pct exec 200 -- top -bn1 | grep wxlistener
```

**Typical CPU usage**: <5% during normal operation

### Disk Space

4GB is more than sufficient:

```bash
# Check disk usage
pct exec 200 -- df -h

# Check wxlistener directory size
pct exec 200 -- du -sh /opt/wxlistener
```

**Typical disk usage**: 1-2GB (including build artifacts)

## Networking

### Static IP Configuration

Set during container creation:

```bash
pct set 200 -net0 name=eth0,bridge=vmbr0,ip=192.168.1.100/24,gw=192.168.1.1
```

Or edit `/etc/pve/lxc/200.conf`:

```
net0: name=eth0,bridge=vmbr0,hwaddr=XX:XX:XX:XX:XX:XX,ip=192.168.1.100/24,gw=192.168.1.1,type=veth
```

### DHCP Configuration

```bash
pct set 200 -net0 name=eth0,bridge=vmbr0,ip=dhcp
```

### Firewall Rules

**On Proxmox host**, allow access to web interface:

```bash
# Allow port 18888 from specific network
iptables -A FORWARD -p tcp -d <container-ip> --dport 18888 -s 192.168.1.0/24 -j ACCEPT

# Or allow from anywhere (less secure)
iptables -A FORWARD -p tcp -d <container-ip> --dport 18888 -j ACCEPT
```

**Inside container** (if using ufw):

```bash
pct exec 200 -- ufw allow 18888/tcp
pct exec 200 -- ufw enable
```

### Port Forwarding

Forward Proxmox host port to container:

```bash
# Forward host port 8888 to container port 18888
iptables -t nat -A PREROUTING -p tcp --dport 8888 -j DNAT --to <container-ip>:18888
iptables -t nat -A POSTROUTING -j MASQUERADE
```

## Best Practices

### Security

1. **Use unprivileged containers** (default in our setup)

   - Reduces attack surface
   - Better isolation from host

2. **Limit network access**

   ```bash
   # Only allow necessary ports
   pct exec 200 -- ufw default deny incoming
   pct exec 200 -- ufw allow 18888/tcp
   pct exec 200 -- ufw enable
   ```

3. **Regular updates**

   ```bash
   pct exec 200 -- apt-get update
   pct exec 200 -- apt-get upgrade -y
   ```

4. **Use strong passwords** for database connections

5. **Enable automatic security updates**
   ```bash
   pct exec 200 -- apt-get install -y unattended-upgrades
   pct exec 200 -- dpkg-reconfigure -plow unattended-upgrades
   ```

### Backups

**Automated Proxmox backups:**

```bash
# Create backup schedule in Proxmox web UI:
# Datacenter → Backup → Add

# Or via CLI
vzdump 200 --compress zstd --mode snapshot --storage backup-storage
```

**Manual config backup:**

```bash
# Backup configuration
pct exec 200 -- tar -czf /tmp/wxlistener-config.tar.gz /etc/wxlistener

# Copy to host
pct pull 200 /tmp/wxlistener-config.tar.gz ./wxlistener-config-backup.tar.gz
```

### Monitoring

1. **Check service status regularly**

   ```bash
   pct exec 200 -- systemctl status wxlistener
   ```

2. **Monitor resource usage**

   ```bash
   # CPU and memory
   pct status 200

   # Detailed stats
   pct exec 200 -- htop
   ```

3. **Set up alerts** in Proxmox for:
   - High CPU usage (>80%)
   - High memory usage (>90%)
   - Service failures

### Updates

**Update wxlistener:**

```bash
pct exec 200 -- bash -c "
  cd /opt/wxlistener
  git pull
  source /root/.cargo/env
  cargo build --release
  cp target/release/wxlistener /usr/local/bin/
  systemctl restart wxlistener
"
```

**Update container OS:**

```bash
pct exec 200 -- apt-get update
pct exec 200 -- apt-get upgrade -y
pct exec 200 -- apt-get autoremove -y
```

## Troubleshooting

### Container Won't Start

```bash
# Check container status
pct status 200

# View container logs
pct enter 200
journalctl -xe

# Check Proxmox logs
tail -f /var/log/pve/tasks/active
```

### Build Failures

```bash
# Check Rust installation
pct exec 200 -- bash -c "source /root/.cargo/env && rustc --version"

# Clean and rebuild
pct exec 200 -- bash -c "
  cd /opt/wxlistener
  source /root/.cargo/env
  cargo clean
  cargo build --release
"

# Check disk space
pct exec 200 -- df -h
```

### Network Issues

```bash
# Check network configuration
pct config 200 | grep net0

# Test connectivity from container
pct exec 200 -- ping -c 3 8.8.8.8

# Check if container can reach weather station
pct exec 200 -- ping -c 3 <weather-station-ip>

# Verify DNS
pct exec 200 -- nslookup google.com
```

### Service Won't Start

```bash
# Check service status
pct exec 200 -- systemctl status wxlistener

# View detailed logs
pct exec 200 -- journalctl -u wxlistener -n 50

# Test binary manually
pct exec 200 -- /usr/local/bin/wxlistener --help

# Check configuration
pct exec 200 -- cat /etc/wxlistener/wxlistener.toml
```

## Upgrading

### Upgrading wxlistener

```bash
# Stop service
pct exec 200 -- systemctl stop wxlistener

# Pull latest code
pct exec 200 -- bash -c "cd /opt/wxlistener && git pull"

# Rebuild
pct exec 200 -- bash -c "
  cd /opt/wxlistener
  source /root/.cargo/env
  cargo build --release
  cp target/release/wxlistener /usr/local/bin/
"

# Start service
pct exec 200 -- systemctl start wxlistener
```

### Upgrading Container OS

```bash
# Update package lists
pct exec 200 -- apt-get update

# Upgrade packages
pct exec 200 -- apt-get upgrade -y

# Upgrade to new Ubuntu LTS (if available)
pct exec 200 -- do-release-upgrade
```

## Advanced Configuration

### Resource Limits

Adjust container resources:

```bash
# Increase memory to 1GB
pct set 200 -memory 1024

# Add another CPU core
pct set 200 -cores 2

# Increase disk size
pct resize 200 rootfs +2G
```

### Privileged vs Unprivileged

Our setup uses **unprivileged containers** (recommended):

**Advantages:**

- Better security isolation
- Reduced attack surface
- Safer for production

**Disadvantages:**

- Some features may not work
- More complex permission handling

To create a privileged container (not recommended):

```bash
pct create 200 ... --unprivileged 0
```

### Nested Containers

Nesting is enabled by default in our setup:

```bash
pct set 200 -features nesting=1
```

This allows running Docker inside the LXC container if needed.

## Performance Tuning

### Optimize for Low Resource Usage

```bash
# Limit systemd journal size
pct exec 200 -- bash -c "
  echo 'SystemMaxUse=50M' >> /etc/systemd/journald.conf
  systemctl restart systemd-journald
"

# Disable unnecessary services
pct exec 200 -- systemctl disable snapd
pct exec 200 -- systemctl disable unattended-upgrades
```

### Optimize Rust Build

```bash
# Use faster linker
pct exec 200 -- bash -c "
  apt-get install -y lld
  echo '[target.x86_64-unknown-linux-gnu]
linker = \"clang\"
rustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]' > /root/.cargo/config.toml
"
```

## Migration

### Backing Up Container

```bash
# Create backup
vzdump 200 --compress zstd --mode snapshot --storage local

# Backup location
ls -lh /var/lib/vz/dump/
```

### Restoring Container

```bash
# Restore from backup
pct restore 201 /var/lib/vz/dump/vzdump-lxc-200-*.tar.zst --storage local-lvm

# Start restored container
pct start 201
```

### Moving Between Hosts

```bash
# On source host - create backup
vzdump 200 --compress zstd --dumpdir /tmp

# Copy to destination host
scp /tmp/vzdump-lxc-200-*.tar.zst root@destination-host:/var/lib/vz/dump/

# On destination host - restore
pct restore 200 /var/lib/vz/dump/vzdump-lxc-200-*.tar.zst --storage local-lvm
pct start 200
```

## Support

For issues or questions:

- Check the main [README](../README.md)
- Review [Docker documentation](docker.md) for containerization concepts
- Review [API documentation](api.md) for web interface details
- File an issue on GitHub
