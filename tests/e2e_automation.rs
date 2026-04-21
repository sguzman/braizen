use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::json;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

fn write_test_config(path: &std::path::Path) {
    let toml = format!(
        r#"
[features]
automation_server = true

[automation]
enabled = true
bind = "ws://127.0.0.1:0/ws"
require_auth = false

[logging]
console_filter = "info"
file_filter = "off"
"#
    );
    std::fs::write(path, toml).expect("write config");
}

fn spawn_brazen(config_path: &std::path::Path, endpoint_file: &std::path::Path) -> Child {
    let exe = env!("CARGO_BIN_EXE_brazen");
    Command::new(exe)
        .arg("--config")
        .arg(config_path)
        .env("BRAZEN_AUTOMATION_ENDPOINT_FILE", endpoint_file)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn brazen")
}

async fn wait_for_endpoint_file(path: &std::path::Path) -> Url {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if Instant::now() > deadline {
            panic!("timed out waiting for automation endpoint file: {}", path.display());
        }
        if let Ok(text) = std::fs::read_to_string(path) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Url::parse(trimmed).expect("endpoint file contains url");
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn ws_roundtrip(url: &Url, payload: serde_json::Value) -> serde_json::Value {
    let (ws, _) = connect_async(url.as_str()).await.expect("connect ws");
    let (mut write, mut read) = ws.split();
    write
        .send(Message::Text(payload.to_string().into()))
        .await
        .expect("send");
    let msg = tokio::time::timeout(Duration::from_secs(5), read.next())
        .await
        .expect("recv timeout")
        .expect("recv")
        .expect("recv ok");
    let Message::Text(text) = msg else {
        panic!("expected text response");
    };
    serde_json::from_str(&text).expect("json response")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_boot_connect_logs_and_shutdown() {
    if std::env::var("BRAZEN_E2E").ok().as_deref() != Some("1") {
        return;
    }
    if std::env::var("DISPLAY").ok().is_none() && std::env::var("WAYLAND_DISPLAY").ok().is_none()
    {
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("brazen.toml");
    let endpoint_file = tmp.path().join("endpoint.txt");
    write_test_config(&config_path);

    let mut child = spawn_brazen(&config_path, &endpoint_file);
    let url = wait_for_endpoint_file(&endpoint_file).await;

    // Subscribe to logs (smoke)
    let response = ws_roundtrip(
        &url,
        json!({"id":"t1","type":"log-subscribe"}),
    )
    .await;
    assert!(response["ok"].as_bool().unwrap_or(false), "log subscribe failed: {response}");

    // Tab list should return an array (even if empty early).
    let response = ws_roundtrip(&url, json!({"id":"t2","type":"tab-list"})).await;
    assert!(response["ok"].as_bool().unwrap_or(false), "tab list failed: {response}");

    // Request shutdown and ensure process exits.
    let response = ws_roundtrip(&url, json!({"id":"t3","type":"shutdown"})).await;
    assert!(response["ok"].as_bool().unwrap_or(false), "shutdown failed: {response}");

    let status = tokio::task::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                return status;
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                return child.wait().expect("wait after kill");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    })
    .await
    .expect("join");
    assert!(status.success(), "brazen did not exit cleanly: {status}");
}
