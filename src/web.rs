use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::client::GW1000Client;
use crate::output::format_value;

const HTML_PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Weather Station Live Data</title>
    <style>
        body {
            font-family: 'Courier New', monospace;
            background-color: #1e1e1e;
            color: #d4d4d4;
            padding: 20px;
            margin: 0;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
        h1 {
            color: #4ec9b0;
            border-bottom: 2px solid #4ec9b0;
            padding-bottom: 10px;
        }
        .status {
            padding: 10px;
            margin: 10px 0;
            border-radius: 4px;
            background-color: #2d2d30;
        }
        .status.connected {
            border-left: 4px solid #4ec9b0;
        }
        .status.disconnected {
            border-left: 4px solid #f48771;
        }
        .data-container {
            background-color: #252526;
            padding: 20px;
            border-radius: 4px;
            margin-top: 20px;
        }
        .data-header {
            color: #569cd6;
            font-weight: bold;
            margin-bottom: 15px;
            font-size: 1.1em;
        }
        .data-row {
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid #3e3e42;
        }
        .data-row:last-child {
            border-bottom: none;
        }
        .data-key {
            color: #9cdcfe;
        }
        .data-value {
            color: #ce9178;
            font-weight: bold;
        }
        .timestamp {
            color: #6a9955;
            font-size: 0.9em;
            margin-top: 10px;
        }
        pre {
            background-color: #1e1e1e;
            padding: 15px;
            border-radius: 4px;
            overflow-x: auto;
            border: 1px solid #3e3e42;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌤️ Weather Station Live Data</h1>
        <div id="status" class="status disconnected">
            Status: <span id="status-text">Connecting...</span>
        </div>
        <div class="data-container">
            <div class="data-header">Live Weather Data</div>
            <pre id="data">Waiting for data...</pre>
            <div class="timestamp" id="timestamp"></div>
        </div>
    </div>

    <script>
        let ws;
        const statusEl = document.getElementById('status');
        const statusTextEl = document.getElementById('status-text');
        const dataEl = document.getElementById('data');
        const timestampEl = document.getElementById('timestamp');

        function connect() {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

            ws.onopen = () => {
                statusEl.className = 'status connected';
                statusTextEl.textContent = 'Connected';
                console.log('WebSocket connected');
            };

            ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    displayData(data);
                } catch (e) {
                    console.error('Failed to parse data:', e);
                }
            };

            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                statusEl.className = 'status disconnected';
                statusTextEl.textContent = 'Error';
            };

            ws.onclose = () => {
                statusEl.className = 'status disconnected';
                statusTextEl.textContent = 'Disconnected - Reconnecting...';
                console.log('WebSocket closed, reconnecting in 2s...');
                setTimeout(connect, 2000);
            };
        }

        function displayData(data) {
            if (data.error) {
                dataEl.textContent = `Error: ${data.error}`;
                return;
            }

            let output = '';
            const keys = Object.keys(data.data).sort();
            
            for (const key of keys) {
                const value = data.data[key];
                output += `${key.padEnd(20)} : ${value}\n`;
            }

            dataEl.textContent = output;
            timestampEl.textContent = `Last update: ${data.timestamp}`;
        }

        connect();
    </script>
</body>
</html>
"#;

pub struct WebServerConfig {
    pub ip: String,
    pub port: u16,
    pub interval: u64,
}

/// Spawns the web server as a background task
pub fn run_web_server_background(config: WebServerConfig, gw_ip: String, gw_port: u16) {
    tokio::spawn(async move {
        if let Err(e) = run_web_server(config, gw_ip, gw_port).await {
            eprintln!("[ERROR] Web server error: {}", e);
        }
    });
}

pub async fn run_web_server(
    config: WebServerConfig,
    gw_ip: String,
    gw_port: u16,
) -> anyhow::Result<()> {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // Spawn background task to fetch weather data
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let client = GW1000Client::new(gw_ip, gw_port);
        let mut interval = time::interval(Duration::from_secs(config.interval));

        loop {
            interval.tick().await;

            match client.get_livedata() {
                Ok(data) => {
                    let timestamp = Utc::now();
                    let mut formatted_data = std::collections::HashMap::new();

                    for (key, value) in data.iter() {
                        formatted_data.insert(key.clone(), format_value(key, *value));
                    }

                    let message = serde_json::json!({
                        "timestamp": timestamp.to_rfc3339(),
                        "data": formatted_data,
                    });

                    if let Ok(json) = serde_json::to_string(&message) {
                        let _ = tx_clone.send(json);
                    }
                }
                Err(e) => {
                    let error_msg = serde_json::json!({
                        "error": format!("Failed to fetch data: {}", e),
                        "timestamp": Utc::now().to_rfc3339(),
                    });

                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = tx_clone.send(json);
                    }
                }
            }
        }
    });

    // Build the router with logging
    let tx_for_ws = tx.clone();
    let app = Router::new()
        .route("/", get(index_handler))
        .route(
            "/ws",
            get(move |ws, addr| websocket_handler(ws, tx_for_ws.clone(), addr)),
        )
        .route("/api/v1/current.json", get(api_current_handler))
        .with_state(tx)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO)),
        );

    let addr = format!("{}:{}", config.ip, config.port);
    println!("============================================================");
    println!("Web server starting on http://{}", addr);
    println!("Press Ctrl+C to stop");
    println!("============================================================\n");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn index_handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    println!("[{}] GET / - 200 OK", addr);
    Html(HTML_PAGE)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    tx: Arc<broadcast::Sender<String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("[{}] WebSocket connection established", addr);
    ws.on_upgrade(move |socket| handle_socket(socket, tx, addr))
}

async fn handle_socket(socket: WebSocket, tx: Arc<broadcast::Sender<String>>, addr: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    // Spawn a task to send messages from the broadcast channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task to receive messages from the WebSocket (for connection management)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    println!("[{}] WebSocket connection closed", addr);
}

pub async fn api_current_handler(
    State(tx): State<Arc<broadcast::Sender<String>>>,
    addr: Option<ConnectInfo<SocketAddr>>,
) -> impl IntoResponse {
    if let Some(ConnectInfo(addr)) = addr {
        println!("[{}] GET /api/v1/current.json", addr);
    }

    // Subscribe to the broadcast channel to get the latest data
    let mut rx = tx.subscribe();

    // Try to receive the latest message with a timeout
    match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
        Ok(Ok(data)) => {
            // Parse the JSON string and return it
            match serde_json::from_str::<serde_json::Value>(&data) {
                Ok(json) => Json(json),
                Err(_) => Json(serde_json::json!({
                    "error": "Failed to parse weather data"
                })),
            }
        }
        Ok(Err(_)) => Json(serde_json::json!({
            "error": "No data available"
        })),
        Err(_) => Json(serde_json::json!({
            "error": "Timeout waiting for data"
        })),
    }
}
