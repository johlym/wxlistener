# MQTT Integration Guide

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
  - [Connection String Format](#connection-string-format)
  - [Individual Fields](#individual-fields)
  - [Configuration Examples](#configuration-examples)
- [Message Format](#message-format)
  - [Payload Structure](#payload-structure)
  - [Example Messages](#example-messages)
- [Topics](#topics)
  - [Default Topic](#default-topic)
  - [Custom Topics](#custom-topics)
- [Authentication](#authentication)
- [Quality of Service (QoS)](#quality-of-service-qos)
- [Usage Examples](#usage-examples)
  - [Basic Setup](#basic-setup)
  - [With Authentication](#with-authentication)
  - [Custom Topic](#custom-topic)
  - [Multiple Subscribers](#multiple-subscribers)
- [Integration Examples](#integration-examples)
  - [Home Assistant](#home-assistant)
  - [Node-RED](#node-red)
  - [Python Subscriber](#python-subscriber)
  - [Mosquitto CLI](#mosquitto-cli)
- [Troubleshooting](#troubleshooting)
  - [Connection Issues](#connection-issues)
  - [Authentication Failures](#authentication-failures)
  - [Message Not Received](#message-not-received)
- [Best Practices](#best-practices)
  - [Broker Selection](#broker-selection)
  - [Topic Design](#topic-design)
  - [Monitoring](#monitoring)
- [Performance](#performance)
- [Security](#security)
- [Advanced Configuration](#advanced-configuration)
  - [TLS/SSL Support](#tlsssl-support)
  - [Retained Messages](#retained-messages)
  - [Last Will and Testament](#last-will-and-testament)

## Overview

wxlistener includes built-in MQTT support for publishing weather data to an MQTT broker. Every time new data is received from the weather station, it's automatically published to the configured MQTT topic in JSON format.

**Key Features:**

- Automatic publishing on every data update
- Flexible configuration (connection string or individual fields)
- Configurable topics with sensible defaults
- Authentication support
- Automatic reconnection on connection loss
- QoS 1 (at least once delivery)

## Features

- ✅ **Automatic Publishing** - Data published on every update
- ✅ **Flexible Configuration** - Connection string or individual fields
- ✅ **Custom Topics** - Configure your own topic hierarchy
- ✅ **Authentication** - Username/password support
- ✅ **Auto-Reconnect** - Handles connection drops gracefully
- ✅ **JSON Format** - Standard, parseable message format
- ✅ **QoS 1** - At least once delivery guarantee

## Quick Start

Add MQTT configuration to your `wxlistener.toml`:

```toml
[mqtt]
connection_string = "mqtt://localhost:1883/wx/live"
```

Or use individual fields:

```toml
[mqtt]
host = "localhost"
port = 1883
topic = "wx/live"
```

Run wxlistener:

```bash
wxlistener --config wxlistener.toml
```

Subscribe to messages:

```bash
mosquitto_sub -h localhost -t "wx/live"
```

## Configuration

### Connection String Format

The connection string format is:

```
mqtt://[username:password@]host[:port][/topic]
```

**Components:**

- `mqtt://` - Required protocol prefix
- `username:password@` - Optional authentication
- `host` - MQTT broker hostname or IP (required)
- `port` - Broker port (optional, default: 1883)
- `topic` - MQTT topic (optional, default: wx/live)

**Examples:**

```toml
# Basic
connection_string = "mqtt://localhost:1883/wx/live"

# With authentication
connection_string = "mqtt://user:pass@broker.example.com:1883/weather/station1"

# Default port
connection_string = "mqtt://192.168.1.100/sensors/weather"

# Just host (uses all defaults)
connection_string = "mqtt://mqtt.local"
```

### Individual Fields

Configure MQTT using individual fields:

```toml
[mqtt]
host = "mqtt.example.com"      # Required
port = 1883                     # Optional, default: 1883
topic = "weather/outdoor"       # Optional, default: wx/live
client_id = "wxlistener-main"   # Optional, auto-generated
username = "mqtt_user"          # Optional
password = "mqtt_password"      # Optional
```

### Configuration Examples

**Local Mosquitto Broker:**

```toml
[mqtt]
connection_string = "mqtt://localhost:1883/wx/live"
```

**Cloud MQTT Service:**

```toml
[mqtt]
host = "mqtt.cloudmqtt.com"
port = 18883
topic = "home/weather/outdoor"
username = "your_username"
password = "your_password"
```

**Home Assistant MQTT:**

```toml
[mqtt]
host = "homeassistant.local"
port = 1883
topic = "homeassistant/sensor/weather/state"
username = "homeassistant"
password = "your_ha_password"
```

**Multiple Locations:**

```toml
[mqtt]
connection_string = "mqtt://mqtt.local:1883/weather/backyard"
```

## Message Format

### Payload Structure

Messages are published in JSON format:

```json
{
  "timestamp": "2025-12-10 15:30:45 UTC",
  "data": {
    "outtemp": "15.5°C",
    "intemp": "22.0°C",
    "outhumid": "65%",
    "inhumid": "45%",
    "absbarometer": "1013.2 hPa",
    "relbarometer": "1010.5 hPa",
    "wind_speed": "3.5 m/s",
    "wind_dir": "180.0°",
    "gust_speed": "8.2 m/s",
    "rain_rate": "0.0 mm/h",
    "rain_day": "12.7 mm",
    "light": "45000.0 lux",
    "uv": "250",
    "uvi": "3"
  }
}
```

### Example Messages

**Sunny Day:**

```json
{
  "timestamp": "2025-06-15 14:00:00 UTC",
  "data": {
    "outtemp": "28.5°C",
    "outhumid": "45%",
    "wind_speed": "5.2 m/s",
    "light": "85000.0 lux",
    "uvi": "8"
  }
}
```

**Rainy Weather:**

```json
{
  "timestamp": "2025-11-20 09:30:00 UTC",
  "data": {
    "outtemp": "12.3°C",
    "outhumid": "95%",
    "rain_rate": "15.2 mm/h",
    "rain_day": "45.6 mm",
    "wind_speed": "12.5 m/s"
  }
}
```

## Topics

### Default Topic

If no topic is specified, wxlistener uses: `wx/live`

### Custom Topics

You can customize the topic to fit your needs:

**Hierarchical Topics:**

```toml
# By location
topic = "home/outdoor/weather"

# By device
topic = "sensors/gw1000/data"

# By function
topic = "weather/realtime"
```

**Topic Best Practices:**

- Use forward slashes `/` for hierarchy
- Keep topics descriptive but concise
- Use lowercase for consistency
- Avoid special characters
- Consider future expansion

**Examples:**

```
weather/outdoor
home/backyard/weather
sensors/weather/live
station1/data
```

## Authentication

Configure username and password for authenticated brokers:

**In connection string:**

```toml
[mqtt]
connection_string = "mqtt://username:password@broker.example.com:1883/wx/live"
```

**As separate fields:**

```toml
[mqtt]
host = "broker.example.com"
username = "mqtt_user"
password = "secure_password"
topic = "wx/live"
```

## Quality of Service (QoS)

wxlistener publishes messages with **QoS 1** (at least once delivery):

- Messages are guaranteed to be delivered at least once
- Broker acknowledges receipt
- Suitable for most weather monitoring applications
- Balance between reliability and performance

## Usage Examples

### Basic Setup

**Configuration:**

```toml
ip = "192.168.1.50"
port = 45000

[mqtt]
connection_string = "mqtt://localhost:1883/wx/live"
```

**Run:**

```bash
wxlistener --config wxlistener.toml
```

**Output:**

```
✓ Connected to MQTT broker (topic: wx/live)
============================================================
GW1000/Ecowitt Gateway Weather Station Listener
============================================================
Target device: 192.168.1.50:45000

--- Continuous Mode (every 5 seconds) ---
MQTT publishing: ENABLED
Press Ctrl+C to stop
```

### With Authentication

```toml
[mqtt]
host = "mqtt.example.com"
port = 1883
topic = "weather/outdoor"
username = "weather_user"
password = "secret123"
```

### Custom Topic

```toml
[mqtt]
connection_string = "mqtt://localhost:1883/home/backyard/weather"
```

### Multiple Subscribers

Multiple clients can subscribe to the same topic:

**Terminal 1 - Publisher:**

```bash
wxlistener --config wxlistener.toml
```

**Terminal 2 - Subscriber 1:**

```bash
mosquitto_sub -h localhost -t "wx/live"
```

**Terminal 3 - Subscriber 2:**

```bash
mosquitto_sub -h localhost -t "wx/live" -F "%t: %p"
```

## Integration Examples

### Home Assistant

**MQTT Sensor Configuration:**

```yaml
mqtt:
  sensor:
    - name: "Outdoor Temperature"
      state_topic: "wx/live"
      value_template: "{{ value_json.data.outtemp | replace('°C', '') }}"
      unit_of_measurement: "°C"
      device_class: temperature

    - name: "Outdoor Humidity"
      state_topic: "wx/live"
      value_template: "{{ value_json.data.outhumid | replace('%', '') }}"
      unit_of_measurement: "%"
      device_class: humidity

    - name: "Wind Speed"
      state_topic: "wx/live"
      value_template: "{{ value_json.data.wind_speed | replace(' m/s', '') }}"
      unit_of_measurement: "m/s"

    - name: "Rain Today"
      state_topic: "wx/live"
      value_template: "{{ value_json.data.rain_day | replace(' mm', '') }}"
      unit_of_measurement: "mm"
```

### Node-RED

**MQTT In Node Configuration:**

```json
{
  "name": "Weather Data",
  "topic": "wx/live",
  "qos": "1",
  "broker": "localhost"
}
```

**Function Node to Parse:**

```javascript
// Parse weather data
const data = JSON.parse(msg.payload);

msg.temperature = parseFloat(data.data.outtemp);
msg.humidity = parseFloat(data.data.outhumid);
msg.timestamp = data.timestamp;

return msg;
```

### Python Subscriber

```python
import paho.mqtt.client as mqtt
import json

def on_connect(client, userdata, flags, rc):
    print(f"Connected with result code {rc}")
    client.subscribe("wx/live")

def on_message(client, userdata, msg):
    data = json.loads(msg.payload.decode())
    print(f"Timestamp: {data['timestamp']}")
    print(f"Temperature: {data['data']['outtemp']}")
    print(f"Humidity: {data['data']['outhumid']}")
    print("---")

client = mqtt.Client()
client.on_connect = on_connect
client.on_message = on_message

client.connect("localhost", 1883, 60)
client.loop_forever()
```

### Mosquitto CLI

**Subscribe to all messages:**

```bash
mosquitto_sub -h localhost -t "wx/live" -v
```

**Subscribe with formatted output:**

```bash
mosquitto_sub -h localhost -t "wx/live" -F "%I %t %p" | jq
```

**Subscribe with authentication:**

```bash
mosquitto_sub -h broker.example.com -t "wx/live" -u username -P password
```

## Troubleshooting

### Connection Issues

**Problem:** Cannot connect to MQTT broker

**Solutions:**

1. Verify broker is running:

   ```bash
   mosquitto -v
   ```

2. Check network connectivity:

   ```bash
   ping mqtt.local
   telnet mqtt.local 1883
   ```

3. Verify configuration:

   ```bash
   cat wxlistener.toml | grep -A 5 "\[mqtt\]"
   ```

4. Check wxlistener output for errors:
   ```
   ✗ MQTT error: Connection refused
   ```

### Authentication Failures

**Problem:** Authentication failed

**Solutions:**

1. Verify credentials are correct
2. Check broker logs:

   ```bash
   tail -f /var/log/mosquitto/mosquitto.log
   ```

3. Test with mosquitto_pub:
   ```bash
   mosquitto_pub -h localhost -t test -m "test" -u username -P password
   ```

### Message Not Received

**Problem:** Subscriber not receiving messages

**Solutions:**

1. Verify topic matches exactly:

   ```bash
   mosquitto_sub -h localhost -t "wx/live" -v
   ```

2. Check QoS settings
3. Verify wxlistener is publishing:

   ```
   MQTT publishing: ENABLED
   ```

4. Test with wildcard subscription:
   ```bash
   mosquitto_sub -h localhost -t "#" -v
   ```

## Best Practices

### Broker Selection

**Local Broker (Recommended for home use):**

- Mosquitto on Raspberry Pi
- Home Assistant built-in broker
- Low latency, high reliability

**Cloud Broker:**

- CloudMQTT
- HiveMQ Cloud
- AWS IoT Core
- Good for remote access

### Topic Design

**Good:**

- `weather/outdoor`
- `home/backyard/weather`
- `sensors/gw1000/live`

**Avoid:**

- `data` (too generic)
- `weather/outdoor/temperature/celsius/current` (too deep)
- `Weather-Data!` (special characters)

### Monitoring

1. **Monitor connection status** in wxlistener output
2. **Set up alerts** for connection failures
3. **Log MQTT errors** for debugging
4. **Use retained messages** for last known state

## Performance

**Publishing Rate:**

- Matches wxlistener polling interval (default: 5 seconds)
- Configurable via `--continuous` flag
- No additional overhead

**Message Size:**

- Typical: 400-800 bytes
- Depends on available sensors
- Efficient JSON format

**Network Usage:**

- ~10 KB/minute at 5-second intervals
- Minimal bandwidth requirements
- Suitable for low-bandwidth connections

## Security

**Recommendations:**

1. **Use authentication** on production brokers
2. **Enable TLS/SSL** for remote connections
3. **Restrict topic permissions** via ACLs
4. **Use strong passwords**
5. **Keep broker updated**

**Mosquitto ACL Example:**

```
# /etc/mosquitto/acl
user wxlistener
topic write wx/#

user homeassistant
topic read wx/#
```

## Advanced Configuration

### TLS/SSL Support

For secure connections, use `mqtts://` (currently requires broker configuration):

```toml
[mqtt]
connection_string = "mqtts://broker.example.com:8883/wx/live"
username = "secure_user"
password = "secure_password"
```

### Retained Messages

Retained messages are not currently supported but may be added in future versions.

### Last Will and Testament

LWT is not currently configured but could be added for connection monitoring.

## Future Enhancements

Potential future features:

- Retained message support
- Last Will and Testament (LWT)
- TLS/SSL certificate configuration
- Configurable QoS levels
- Message batching
- Compression support

## Support

For issues or questions:

- Check the main [README](../README.md)
- Review [Configuration examples](../wxlistener.example.toml)
- File an issue on GitHub
