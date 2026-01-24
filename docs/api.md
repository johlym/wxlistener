# API Documentation

## Table of Contents

- [Overview](#overview)
- [Base URL](#base-url)
- [Authentication](#authentication)
- [Endpoints](#endpoints)
  - [GET /api/v1/current.json](#get-apiv1currentjson)
- [Response Format](#response-format)
  - [Success Response](#success-response)
  - [Error Response](#error-response)
- [Data Fields](#data-fields)
- [Usage Examples](#usage-examples)
  - [cURL](#curl)
  - [JavaScript/Fetch](#javascriptfetch)
  - [Python](#python)
  - [Rust](#rust)
- [Rate Limiting](#rate-limiting)
- [CORS](#cors)
- [Troubleshooting](#troubleshooting)

## Overview

The wxlistener web server provides a REST API for accessing current weather station data in JSON format. The API is designed to be simple and lightweight, perfect for integrating weather data into your applications.

## Base URL

When running the web server, the API is available at:

```
http://<host>:<port>/api/v1/
```

Default: `http://localhost:18888/api/v1/`

## Authentication

No authentication is required. The API is designed for local network use.

> **Security Note**: The API does not implement authentication. If you need to expose it publicly, consider using a reverse proxy with authentication (e.g., nginx with basic auth).

## Endpoints

### GET /api/v1/current.json

Returns the most recent weather station data.

**URL**: `/api/v1/current.json`

**Method**: `GET`

**URL Parameters**: None

**Success Response**:

- **Code**: 200 OK
- **Content-Type**: `application/json`

**Example Response**:

```json
{
  "timestamp": "2025-12-10 15:30:45 UTC",
  "data": {
    "absbarometer": "996.0 hPa",
    "day_max_wind": "6.6 m/s",
    "gust_speed": "0.5 m/s",
    "inhumid": "35%",
    "intemp": "29.3°C",
    "light": "0.0 lux",
    "outhumid": "99%",
    "outtemp": "12.2°C",
    "rain_day": "57.9 mm",
    "rain_event": "77.9 mm",
    "rain_month": "106.6 mm",
    "rain_rate": "7.2 mm",
    "rain_week": "77.9 mm",
    "rain_year": "882.4 mm",
    "relbarometer": "993.3 hPa",
    "uv": "0",
    "uvi": "0",
    "wind_dir": "109.0°",
    "wind_speed": "0.1 m/s"
  }
}
```

**Error Responses**:

- **Timeout** (no data available within 16 seconds):

  ```json
  {
    "error": "Timeout waiting for data"
  }
  ```

- **No Data Available**:

  ```json
  {
    "error": "No data available"
  }
  ```

- **Parse Error**:
  ```json
  {
    "error": "Failed to parse weather data"
  }
  ```

## Response Format

### Success Response

| Field       | Type   | Description                                                                   |
| ----------- | ------ | ----------------------------------------------------------------------------- |
| `timestamp` | string | UTC timestamp when the data was collected (format: `YYYY-MM-DD HH:MM:SS UTC`) |
| `data`      | object | Weather measurements with formatted values                                    |

### Error Response

| Field   | Type   | Description                              |
| ------- | ------ | ---------------------------------------- |
| `error` | string | Error message describing what went wrong |

## Data Fields

The `data` object contains weather measurements. Available fields depend on your weather station configuration:

| Field          | Description                  | Example Value |
| -------------- | ---------------------------- | ------------- |
| `intemp`       | Indoor temperature           | `22.5°C`      |
| `outtemp`      | Outdoor temperature          | `15.3°C`      |
| `inhumid`      | Indoor humidity              | `45%`         |
| `outhumid`     | Outdoor humidity             | `68%`         |
| `absbarometer` | Absolute barometric pressure | `1013.2 hPa`  |
| `relbarometer` | Relative barometric pressure | `1010.5 hPa`  |
| `wind_speed`   | Current wind speed           | `3.5 m/s`     |
| `wind_dir`     | Wind direction               | `180.0°`      |
| `gust_speed`   | Wind gust speed              | `8.2 m/s`     |
| `day_max_wind` | Maximum wind speed today     | `12.5 m/s`    |
| `rain_rate`    | Current rain rate            | `5.0 mm/h`    |
| `rain_event`   | Rain since event started     | `25.4 mm`     |
| `rain_day`     | Rain today                   | `12.7 mm`     |
| `rain_week`    | Rain this week               | `45.2 mm`     |
| `rain_month`   | Rain this month              | `125.6 mm`    |
| `rain_year`    | Rain this year               | `850.3 mm`    |
| `light`        | Light intensity              | `45000.0 lux` |
| `uv`           | UV radiation                 | `250`         |
| `uvi`          | UV index                     | `3`           |

**Note**: Not all fields may be present. Available fields depend on the sensors connected to your weather station.

## Usage Examples

### cURL

```bash
# Get current weather data
curl http://localhost:18888/api/v1/current.json

# Pretty print with jq
curl -s http://localhost:18888/api/v1/current.json | jq .

# Get just the outdoor temperature
curl -s http://localhost:18888/api/v1/current.json | jq -r '.data.outtemp'
```

### JavaScript/Fetch

```javascript
// Fetch current weather data
async function getCurrentWeather() {
  try {
    const response = await fetch("http://localhost:18888/api/v1/current.json");
    const data = await response.json();

    if (data.error) {
      console.error("Error:", data.error);
      return null;
    }

    console.log("Temperature:", data.data.outtemp);
    console.log("Humidity:", data.data.outhumid);
    return data;
  } catch (error) {
    console.error("Failed to fetch weather data:", error);
    return null;
  }
}

// Poll for updates every 30 seconds
setInterval(getCurrentWeather, 30000);
```

### Python

```python
import requests
import time

def get_current_weather():
    """Fetch current weather data from wxlistener API"""
    try:
        response = requests.get('http://localhost:18888/api/v1/current.json', timeout=10)
        response.raise_for_status()
        data = response.json()

        if 'error' in data:
            print(f"Error: {data['error']}")
            return None

        return data
    except requests.exceptions.RequestException as e:
        print(f"Failed to fetch weather data: {e}")
        return None

# Get current data
weather = get_current_weather()
if weather:
    print(f"Timestamp: {weather['timestamp']}")
    print(f"Temperature: {weather['data']['outtemp']}")
    print(f"Humidity: {weather['data']['outhumid']}")

# Poll every 60 seconds
while True:
    weather = get_current_weather()
    if weather:
        print(f"Updated: {weather['timestamp']}")
    time.sleep(60)
```

### Rust

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct WeatherResponse {
    timestamp: String,
    data: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:18888/api/v1/current.json")
        .send()
        .await?;

    let text = response.text().await?;

    // Try to parse as success response
    if let Ok(weather) = serde_json::from_str::<WeatherResponse>(&text) {
        println!("Timestamp: {}", weather.timestamp);
        if let Some(temp) = weather.data.get("outtemp") {
            println!("Temperature: {}", temp);
        }
    } else if let Ok(error) = serde_json::from_str::<ErrorResponse>(&text) {
        eprintln!("Error: {}", error.error);
    }

    Ok(())
}
```

## Rate Limiting

There is no explicit rate limiting. However:

- The API waits up to 16 seconds for new data
- Data is updated based on your configured polling interval (default: 16 seconds)
- Excessive requests won't get fresher data, just the same cached value

**Recommendation**: Poll at the same interval as your wxlistener configuration (default: 16 seconds) or slower.

## CORS

CORS (Cross-Origin Resource Sharing) is not currently enabled. If you need to access the API from a web browser on a different origin, you'll need to:

1. Use a reverse proxy (e.g., nginx) with CORS headers
2. Run your web application on the same origin as wxlistener
3. Make requests from a server-side application instead

## Troubleshooting

### "Timeout waiting for data"

**Cause**: No weather data has been received within 16 seconds.

**Solutions**:

- Check that wxlistener is successfully connecting to your weather station
- Verify the weather station is powered on and accessible
- Check the wxlistener console output for errors

### "No data available"

**Cause**: The broadcast channel has no subscribers or data.

**Solutions**:

- Ensure wxlistener web server is running in continuous mode
- Wait a few seconds for the first data update
- Check wxlistener logs for connection issues

### Connection Refused

**Cause**: Web server is not running or wrong host/port.

**Solutions**:

```bash
# Check if wxlistener is running
ps aux | grep wxlistener

# Verify the correct port
netstat -an | grep 18888

# Start wxlistener with web mode
wxlistener --ip YOUR_DEVICE_IP --web
```

### Empty Response

**Cause**: Network issue or server crash.

**Solutions**:

- Check wxlistener process is still running
- Review wxlistener logs for errors
- Restart wxlistener if necessary

## Integration Tips

1. **Error Handling**: Always check for the `error` field in responses
2. **Polling**: Match your polling interval to wxlistener's update interval
3. **Timeouts**: Set reasonable HTTP timeouts (10-15 seconds recommended)
4. **Caching**: Consider caching responses client-side to reduce load
5. **Monitoring**: Log failed requests to detect connectivity issues

## Future Enhancements

Potential future API additions:

- Historical data endpoints (`/api/v1/history`)
- Statistics endpoints (`/api/v1/stats/daily`, `/api/v1/stats/monthly`)
- WebSocket API for real-time streaming
- Filtering/field selection (`?fields=outtemp,outhumid`)
- Multiple output formats (`/api/v1/current.xml`, `/api/v1/current.csv`)

## Support

For issues or questions:

- Check the main [README](../README.md)
- Review [Testing documentation](testing.md)
- File an issue on GitHub
