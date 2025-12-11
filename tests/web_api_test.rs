use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower::util::ServiceExt;

// Import the web module functions
use wxlistener::web::*;

#[tokio::test]
async fn test_api_current_json_endpoint_exists() {
    // Create a broadcast channel for testing
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // Build the router
    let app = axum::Router::new()
        .route(
            "/api/v1/current.json",
            axum::routing::get(api_current_handler),
        )
        .with_state(tx);

    // Make a request to the API endpoint
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/current.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 200 OK (even if timeout, it returns JSON with error)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_current_json_returns_json() {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // Send some test data to the channel
    let test_data = serde_json::json!({
        "timestamp": "2025-12-10 12:00:00 UTC",
        "data": {
            "outtemp": "15.5°C",
            "intemp": "22.0°C",
            "outhumid": "65%"
        }
    });
    let _ = tx.send(test_data.to_string());

    let app = axum::Router::new()
        .route(
            "/api/v1/current.json",
            axum::routing::get(api_current_handler),
        )
        .with_state(tx);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/current.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check content type is JSON
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[tokio::test]
async fn test_api_current_json_with_data() {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx_clone = Arc::new(tx);

    // Spawn a task to send data after a short delay
    let tx_for_task = tx_clone.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let test_data = serde_json::json!({
            "timestamp": "2025-12-10 12:00:00 UTC",
            "data": {
                "outtemp": "15.5°C",
                "intemp": "22.0°C"
            }
        });
        let _ = tx_for_task.send(test_data.to_string());
    });

    let app = axum::Router::new()
        .route(
            "/api/v1/current.json",
            axum::routing::get(api_current_handler),
        )
        .with_state(tx_clone);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/current.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the JSON structure
    assert!(json.get("timestamp").is_some());
    assert!(json.get("data").is_some());
}

#[tokio::test]
async fn test_api_current_json_timeout() {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // Don't send any data - should timeout

    let app = axum::Router::new()
        .route(
            "/api/v1/current.json",
            axum::routing::get(api_current_handler),
        )
        .with_state(tx);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/current.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should have an error field
    assert!(json.get("error").is_some());
    assert!(json["error"].as_str().unwrap().contains("Timeout"));
}
