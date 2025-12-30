# HTTP Endpoint Publishing

wxlistener can POST weather data to any HTTP endpoint in a structured JSON format. This is useful for custom APIs, cloud services, webhooks, or data collection endpoints.

## Table of Contents

- [Configuration](#configuration)
  - [Config File](#config-file)
  - [Environment Variables](#environment-variables)
  - [Configuration Options](#configuration-options)
- [Message Format](#message-format)
  - [Schema](#schema)
  - [Field Mapping](#field-mapping)
  - [Example Payload](#example-payload)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [Integration Examples](#integration-examples)
  - [Express.js Server](#expressjs-server)
  - [Python Flask Server](#python-flask-server)
  - [Webhook Services](#webhook-services)
- [Troubleshooting](#troubleshooting)

## Configuration

### Config File

Add an `[http]` section to your `wxlistener.toml`:

```toml
[http]
url = "https://example.com/api/weather"   # Required
timeout = 10                               # Optional, default: 10 seconds
authorization = "Bearer your-token-here"  # Optional
```

### Environment Variables

You can also configure HTTP output using environment variables:

```bash
export WXLISTENER_HTTP_URL="https://example.com/api/weather"
export WXLISTENER_HTTP_AUTH="Bearer your-token-here"  # Optional
```

Environment variables are used when:

- No config file is specified, or
- The config file doesn't have an `[http]` section

### Configuration Options

| Option          | Required | Default | Description                           |
| --------------- | -------- | ------- | ------------------------------------- |
| `url`           | Yes      | -       | The HTTP endpoint URL to POST data to |
| `timeout`       | No       | 10      | Request timeout in seconds            |
| `authorization` | No       | -       | Value for the `Authorization` header  |

## Message Format

### Schema

Data is POSTed as JSON with the following structure:

```json
{
  "weather_measurement": {
    "reading_date_time": "1970-01-01T00:00:00.000Z",
    "barometer_abs": float,
    "barometer_rel": float,
    "day_max_wind": float,
    "gust_speed": float,
    "humidity": integer,
    "light": float,
    "rain_day": float,
    "rain_event": float,
    "rain_rate": float,
    "temperature": float,
    "uv": integer,
    "uvi": integer,
    "wind_dir": integer,
    "wind_speed": float
  }
}
```

**Important**: Only raw numeric values are sent—no units of measure are included.

### Field Mapping

| HTTP Output Field   | Weather Station Field | Type    | Description                          |
| ------------------- | --------------------- | ------- | ------------------------------------ |
| `reading_date_time` | (timestamp)           | string  | ISO 8601 timestamp with milliseconds |
| `barometer_abs`     | `absbarometer`        | float   | Absolute barometric pressure (hPa)   |
| `barometer_rel`     | `relbarometer`        | float   | Relative barometric pressure (hPa)   |
| `day_max_wind`      | `day_max_wind`        | float   | Maximum wind speed today (m/s)       |
| `gust_speed`        | `gust_speed`          | float   | Current wind gust speed (m/s)        |
| `humidity`          | `outhumid`            | integer | Outdoor humidity (%)                 |
| `light`             | `light`               | float   | Light intensity (lux)                |
| `rain_day`          | `rain_day`            | float   | Rain today (mm)                      |
| `rain_event`        | `rain_event`          | float   | Rain since event started (mm)        |
| `rain_rate`         | `rain_rate`           | float   | Current rain rate (mm/h)             |
| `temperature`       | `outtemp`             | float   | Outdoor temperature (°C)             |
| `uv`                | `uv`                  | integer | UV radiation                         |
| `uvi`               | `uvi`                 | integer | UV index                             |
| `wind_dir`          | `wind_dir`            | integer | Wind direction (degrees)             |
| `wind_speed`        | `wind_speed`          | float   | Current wind speed (m/s)             |

**Note**: Fields are only included if the weather station provides them. Missing sensor data results in omitted fields (not null values).

### Example Payload

```json
{
  "weather_measurement": {
    "reading_date_time": "2025-12-10T15:30:45.123Z",
    "barometer_abs": 1013.25,
    "barometer_rel": 1010.0,
    "day_max_wind": 12.5,
    "gust_speed": 8.2,
    "humidity": 65,
    "light": 50000.0,
    "rain_day": 2.5,
    "rain_event": 1.0,
    "rain_rate": 0.5,
    "temperature": 22.5,
    "uv": 5,
    "uvi": 3,
    "wind_dir": 180,
    "wind_speed": 5.5
  }
}
```

## Authentication

The `authorization` config option sets the `Authorization` HTTP header. Common formats:

**Bearer Token**

```toml
[http]
url = "https://api.example.com/weather"
authorization = "Bearer eyJhbGciOiJIUzI1NiIs..."
```

**API Key**

```toml
[http]
url = "https://api.example.com/weather"
authorization = "ApiKey your-api-key-here"
```

**Basic Auth**

```toml
[http]
url = "https://api.example.com/weather"
authorization = "Basic dXNlcm5hbWU6cGFzc3dvcmQ="
```

## Error Handling

wxlistener will exit with an error if HTTP publishing fails. Common error scenarios:

| Error                                 | Cause                    | Solution                               |
| ------------------------------------- | ------------------------ | -------------------------------------- |
| "Invalid HTTP endpoint URL"           | Malformed URL            | Check URL format (must include scheme) |
| "Failed to send HTTP request"         | Network/connection error | Verify endpoint is reachable           |
| "HTTP request failed with status XXX" | Server returned error    | Check server logs, authentication      |
| "request timed out"                   | Endpoint too slow        | Increase `timeout` value               |

## Integration Examples

### Express.js Server

```javascript
const express = require("express");
const app = express();

app.use(express.json());

app.post("/api/weather", (req, res) => {
  const { weather_measurement } = req.body;

  console.log("Received weather data:", weather_measurement);
  console.log("Temperature:", weather_measurement.temperature);
  console.log("Humidity:", weather_measurement.humidity);
  console.log("Timestamp:", weather_measurement.reading_date_time);

  // Store in database, forward to another service, etc.

  res.status(200).json({ status: "ok" });
});

app.listen(3000, () => {
  console.log("Weather API listening on port 3000");
});
```

### Python Flask Server

```python
from flask import Flask, request, jsonify
from datetime import datetime

app = Flask(__name__)

@app.route('/api/weather', methods=['POST'])
def receive_weather():
    data = request.get_json()
    measurement = data.get('weather_measurement', {})

    print(f"Received weather data at {measurement.get('reading_date_time')}")
    print(f"Temperature: {measurement.get('temperature')}°C")
    print(f"Humidity: {measurement.get('humidity')}%")

    # Store in database, process, etc.

    return jsonify({'status': 'ok'}), 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000)
```

### Webhook Services

wxlistener works with webhook services like:

- **Zapier**: Create a Zap with "Webhooks by Zapier" trigger
- **IFTTT**: Use the Webhooks service as a trigger
- **n8n**: Use the Webhook node to receive data
- **Make (Integromat)**: Use the Webhooks module

Configure the webhook URL as your `[http].url` and add any required authentication.

## Troubleshooting

### "HTTP configuration failed"

**Cause**: Missing or invalid URL.

**Solution**: Ensure `url` is set in config or `WXLISTENER_HTTP_URL` environment variable.

### "Failed to send HTTP request"

**Cause**: Network connectivity issue or endpoint not reachable.

**Solutions**:

- Verify the endpoint URL is correct
- Check network connectivity: `curl -v <your-url>`
- Ensure firewall allows outbound connections
- Check if the endpoint requires VPN access

### "HTTP request failed with status 401"

**Cause**: Authentication required or invalid credentials.

**Solution**: Verify `authorization` header value matches what the endpoint expects.

### "HTTP request failed with status 4XX/5XX"

**Cause**: Server rejected the request.

**Solutions**:

- Check endpoint server logs for details
- Verify the endpoint accepts POST requests with JSON body
- Ensure Content-Type: application/json is accepted

### Request Timeouts

**Cause**: Endpoint is slow or unresponsive.

**Solution**: Increase timeout in config:

```toml
[http]
url = "https://example.com/api/weather"
timeout = 30  # Increase to 30 seconds
```

### Testing Your Endpoint

Use curl to test your endpoint with sample data:

```bash
curl -X POST https://your-endpoint.com/api/weather \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "weather_measurement": {
      "reading_date_time": "2025-12-10T15:30:45.123Z",
      "temperature": 22.5,
      "humidity": 65,
      "wind_speed": 5.5
    }
  }'
```
