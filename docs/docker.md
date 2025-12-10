# Docker Guide

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
  - [1. Build the Image](#1-build-the-image)
  - [2. Configure with Environment Variables](#2-configure-with-environment-variables)
  - [3. Run (Continuous Mode by Default)](#3-run-continuous-mode-by-default)
- [Docker Images](#docker-images)
  - [Production Image](#production-image)
  - [Development Image](#development-image)
- [Usage Examples](#usage-examples)
  - [Single Read](#single-read)
  - [Continuous Monitoring](#continuous-monitoring)
  - [JSON Output](#json-output)
  - [Direct IP (no config file)](#direct-ip-no-config-file)
- [Docker Compose](#docker-compose)
  - [Basic Setup](#basic-setup)
  - [Continuous Mode](#continuous-mode)
  - [Development Mode](#development-mode)
- [Network Configuration](#network-configuration)
  - [Host Network (Recommended)](#host-network-recommended)
  - [Bridge Network](#bridge-network)
- [Volume Mounts](#volume-mounts)
  - [Config File (Read-Only)](#config-file-read-only)
  - [Data Directory (for logging)](#data-directory-for-logging)
  - [Source Code (Development)](#source-code-development)
- [Environment Variables](#environment-variables)
  - [Configuration Variables](#configuration-variables)
  - [Usage Examples](#usage-examples-1)
  - [Using .env File](#using-env-file)
- [Multi-Architecture Builds](#multi-architecture-builds)
  - [Build for Multiple Platforms](#build-for-multiple-platforms)
  - [Supported Architectures](#supported-architectures)
- [Optimization Tips](#optimization-tips)
  - [Reduce Image Size](#reduce-image-size)
  - [Cache Dependencies](#cache-dependencies)
  - [Use BuildKit](#use-buildkit)
- [Troubleshooting](#troubleshooting)
  - ["Cannot connect to device"](#cannot-connect-to-device)
  - ["Permission denied" on config file](#permission-denied-on-config-file)
  - ["Image too large"](#image-too-large)
  - [Development: "Cargo cache not persisting"](#development-cargo-cache-not-persisting)
- [CI/CD Integration](#cicd-integration)
  - [GitHub Actions](#github-actions)
  - [GitLab CI](#gitlab-ci)
- [Production Deployment](#production-deployment)
  - [Docker Swarm](#docker-swarm)
  - [Kubernetes](#kubernetes)
- [Health Checks](#health-checks)
- [Security Best Practices](#security-best-practices)
- [Resources](#resources)

## Overview

wxlistener can run in Docker for easy deployment and isolation. We provide optimized multi-stage builds for minimal image sizes.

## Quick Start

### 1. Build the Image

```bash
# Using our script (recommended)
bin/docker-build

# Or manually
docker build -t wxlistener:latest .
```

### 2. Configure with Environment Variables

```bash
# Copy example environment file
cp .env.example .env

# Edit with your device IP
vim .env
```

Or set directly:

```bash
export WXLISTENER_IP=10.31.100.42
export WXLISTENER_PORT=45000
export WXLISTENER_INTERVAL=60
```

### 3. Run (Continuous Mode by Default)

```bash
# Using our script (continuous mode by default)
bin/docker-run

# Or with docker run
docker run --rm \
  --network host \
  -e WXLISTENER_IP=10.31.100.42 \
  wxlistener:latest

# Or with docker-compose
docker-compose up
```

**The container runs in continuous mode by default** - no additional configuration needed!

## Docker Images

### Production Image

**Size**: ~80MB (multi-stage build)

**Features:**

- Minimal Debian base
- Only runtime dependencies
- Non-root user
- Optimized binary

**Build:**

```bash
docker build -t wxlistener:latest .
```

### Development Image

**Size**: ~2GB (includes build tools)

**Features:**

- Full Rust toolchain
- cargo-watch for hot reload
- Mounted source code
- Cached dependencies

**Build:**

```bash
docker build -f Dockerfile.dev -t wxlistener:dev .
```

## Usage Examples

### Single Read

```bash
docker run --rm \
  --network host \
  -v $(pwd)/wxlistener.toml:/home/wxlistener/wxlistener.toml:ro \
  wxlistener:latest \
  --config wxlistener.toml
```

### Continuous Monitoring

```bash
docker run --rm \
  --network host \
  -v $(pwd)/wxlistener.toml:/home/wxlistener/wxlistener.toml:ro \
  wxlistener:latest \
  --config wxlistener.toml \
  --continuous 60
```

### JSON Output

```bash
docker run --rm \
  --network host \
  -v $(pwd)/wxlistener.toml:/home/wxlistener/wxlistener.toml:ro \
  wxlistener:latest \
  --config wxlistener.toml \
  --format json
```

### Direct IP (no config file)

```bash
docker run --rm \
  --network host \
  wxlistener:latest \
  --ip 10.31.100.42 \
  --port 45000
```

## Docker Compose

### Basic Setup

```yaml
# docker-compose.yml
version: "3.8"

services:
  wxlistener:
    image: wxlistener:latest
    network_mode: host
    volumes:
      - ./wxlistener.toml:/home/wxlistener/wxlistener.toml:ro
    command: ["--config", "wxlistener.toml"]
```

**Run:**

```bash
docker-compose up
```

### Continuous Mode

```bash
# Start continuous monitoring
docker-compose --profile continuous up wxlistener-continuous

# Run in background
docker-compose --profile continuous up -d wxlistener-continuous

# View logs
docker-compose logs -f wxlistener-continuous

# Stop
docker-compose down
```

### Development Mode

```bash
# Start development environment
docker-compose --profile dev up wxlistener-dev

# Run tests
docker-compose --profile dev run --rm wxlistener-dev cargo test

# Build
docker-compose --profile dev run --rm wxlistener-dev cargo build --release
```

## Network Configuration

### Host Network (Recommended)

```bash
docker run --network host wxlistener:latest --ip 10.31.100.42
```

**Pros:**

- Direct access to local network
- No port mapping needed
- Best for local weather stations

**Cons:**

- Less isolation
- Not available on Docker Desktop for Mac/Windows

### Bridge Network

```bash
docker run -p 45000:45000 wxlistener:latest --ip 10.31.100.42
```

**Pros:**

- Better isolation
- Works on all platforms

**Cons:**

- Requires port mapping
- May not work with UDP discovery

## Volume Mounts

### Config File (Read-Only)

```bash
-v $(pwd)/wxlistener.toml:/home/wxlistener/wxlistener.toml:ro
```

### Data Directory (for logging)

```bash
-v $(pwd)/data:/home/wxlistener/data
```

### Source Code (Development)

```bash
-v $(pwd):/app
```

## Environment Variables

### Configuration Variables

| Variable              | Description                      | Default | Required |
| --------------------- | -------------------------------- | ------- | -------- |
| `WXLISTENER_IP`       | Weather station IP address       | -       | ✅ Yes   |
| `WXLISTENER_PORT`     | Weather station port             | `45000` | No       |
| `WXLISTENER_INTERVAL` | Polling interval (seconds)       | `60`    | No       |
| `WXLISTENER_FORMAT`   | Output format (`text` or `json`) | `text`  | No       |
| `RUST_LOG`            | Logging level                    | `info`  | No       |
| `TZ`                  | Timezone                         | `UTC`   | No       |

### Usage Examples

```bash
# Minimal (just IP required)
docker run -e WXLISTENER_IP=10.31.100.42 wxlistener:latest

# Custom interval (poll every 30 seconds)
docker run \
  -e WXLISTENER_IP=10.31.100.42 \
  -e WXLISTENER_INTERVAL=30 \
  wxlistener:latest

# JSON output
docker run \
  -e WXLISTENER_IP=10.31.100.42 \
  -e WXLISTENER_FORMAT=json \
  wxlistener:latest

# Debug logging
docker run \
  -e WXLISTENER_IP=10.31.100.42 \
  -e RUST_LOG=debug \
  wxlistener:latest

# Custom timezone
docker run \
  -e WXLISTENER_IP=10.31.100.42 \
  -e TZ=America/New_York \
  wxlistener:latest
```

### Using .env File

Create a `.env` file:

```bash
WXLISTENER_IP=10.31.100.42
WXLISTENER_PORT=45000
WXLISTENER_INTERVAL=60
WXLISTENER_FORMAT=text
RUST_LOG=info
```

Use with docker-compose:

```bash
docker-compose up
```

Or with docker run:

```bash
docker run --env-file .env wxlistener:latest
```

## Multi-Architecture Builds

### Build for Multiple Platforms

```bash
# Enable buildx
docker buildx create --use

# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  -t wxlistener:latest \
  --push \
  .
```

### Supported Architectures

- `linux/amd64` - x86_64 (Intel/AMD)
- `linux/arm64` - ARM 64-bit (Raspberry Pi 4, Apple Silicon)
- `linux/arm/v7` - ARM 32-bit (Raspberry Pi 3)

## Optimization Tips

### Reduce Image Size

Already optimized with multi-stage build:

- Builder stage: ~2GB
- Final image: ~80MB

### Cache Dependencies

```dockerfile
# Copy only Cargo files first
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Then copy source
COPY src ./src
RUN cargo build --release
```

### Use BuildKit

```bash
DOCKER_BUILDKIT=1 docker build -t wxlistener:latest .
```

## Troubleshooting

### "Cannot connect to device"

**Problem:** Container can't reach weather station

**Solutions:**

```bash
# Use host network
docker run --network host ...

# Or ensure device is accessible
ping 10.31.100.42
```

### "Permission denied" on config file

**Problem:** Volume mount permissions

**Solutions:**

```bash
# Make config readable
chmod 644 wxlistener.toml

# Or run as root (not recommended)
docker run --user root ...
```

### "Image too large"

**Problem:** Using wrong Dockerfile

**Solution:**

```bash
# Use production Dockerfile (not .dev)
docker build -f Dockerfile -t wxlistener:latest .
```

### Development: "Cargo cache not persisting"

**Problem:** Dependencies rebuild every time

**Solution:**

```bash
# Use named volumes in docker-compose
volumes:
  - cargo-cache:/usr/local/cargo/registry
  - target-cache:/app/target
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Docker

on:
  push:
    branches: [main]
    tags: ["v*"]

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            username/wxlistener:latest
            username/wxlistener:${{ github.sha }}
```

### GitLab CI

```yaml
docker:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
```

## Production Deployment

### Docker Swarm

```yaml
version: "3.8"

services:
  wxlistener:
    image: wxlistener:latest
    deploy:
      replicas: 1
      restart_policy:
        condition: on-failure
    networks:
      - host
    configs:
      - source: wxlistener_config
        target: /home/wxlistener/wxlistener.toml

configs:
  wxlistener_config:
    file: ./wxlistener.toml
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wxlistener
spec:
  replicas: 1
  selector:
    matchLabels:
      app: wxlistener
  template:
    metadata:
      labels:
        app: wxlistener
    spec:
      hostNetwork: true
      containers:
        - name: wxlistener
          image: wxlistener:latest
          args:
            - "--config"
            - "/config/wxlistener.toml"
            - "--continuous"
            - "60"
          volumeMounts:
            - name: config
              mountPath: /config
      volumes:
        - name: config
          configMap:
            name: wxlistener-config
```

## Health Checks

```dockerfile
# Add to Dockerfile
HEALTHCHECK --interval=60s --timeout=10s --start-period=5s --retries=3 \
  CMD wxlistener --ip ${DEVICE_IP} || exit 1
```

## Security Best Practices

1. **Run as non-root** ✅ (already implemented)
2. **Read-only config** ✅ (use `:ro` flag)
3. **Minimal base image** ✅ (Debian slim)
4. **No secrets in image** ✅ (use volumes)
5. **Scan for vulnerabilities**:
   ```bash
   docker scan wxlistener:latest
   ```

## Resources

- [Dockerfile reference](https://docs.docker.com/engine/reference/builder/)
- [Docker Compose reference](https://docs.docker.com/compose/compose-file/)
- [Multi-stage builds](https://docs.docker.com/build/building/multi-stage/)
