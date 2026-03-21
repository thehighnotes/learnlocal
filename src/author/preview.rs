use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Deserialize;
use std::io::{Read, Write};
use std::sync::Arc;

use super::server::AppState;

#[derive(Deserialize)]
pub struct PreviewParams {
    pub lesson: Option<String>,
    #[allow(dead_code)]
    pub exercise: Option<String>,
}

pub async fn ws_preview(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<PreviewParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_preview(socket, state, params))
}

enum PtyInput {
    Data(Vec<u8>),
    Resize { cols: u16, rows: u16 },
}

async fn handle_preview(mut socket: WebSocket, state: Arc<AppState>, params: PreviewParams) {
    let course_path_opt = state.course_path.read().unwrap().clone();
    let course_path = match course_path_opt {
        Some(p) => p,
        None => {
            let _ = socket
                .send(Message::Text("No course loaded".to_string()))
                .await;
            return;
        }
    };

    // Spawn the pty in a blocking task
    let pty_result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let exe = std::env::current_exe().unwrap_or_else(|_| "learnlocal".into());
        let mut cmd = CommandBuilder::new(&exe);
        cmd.arg("start");

        let course_name = course_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        cmd.arg(&course_name);
        cmd.arg("--courses");
        cmd.arg(
            course_path
                .parent()
                .unwrap_or(&course_path)
                .to_string_lossy()
                .to_string(),
        );

        if let Some(ref lesson) = params.lesson {
            cmd.arg("--lesson");
            cmd.arg(lesson);
        }

        let _child = pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok((reader, writer, pair.master))
    })
    .await;

    let (reader, writer, master) = match pty_result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            log::error!("Failed to start preview pty: {}", e);
            return;
        }
        Err(e) => {
            log::error!("Preview task panicked: {}", e);
            return;
        }
    };

    // Channels to bridge sync pty I/O ↔ async WebSocket
    let (tx_to_ws, mut rx_to_ws) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    let (tx_to_pty, mut rx_to_pty) = tokio::sync::mpsc::channel::<PtyInput>(64);

    // Blocking thread: pty stdout → channel
    let reader = Arc::new(std::sync::Mutex::new(reader));
    let reader_clone = Arc::clone(&reader);
    tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 4096];
        loop {
            let n = match reader_clone.lock().unwrap().read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            if tx_to_ws.blocking_send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
    });

    // Blocking thread: channel → pty stdin + resize
    let writer = Arc::new(std::sync::Mutex::new(writer));
    let writer_clone = Arc::clone(&writer);
    let master = Arc::new(std::sync::Mutex::new(master));
    let master_clone = Arc::clone(&master);
    tokio::task::spawn_blocking(move || {
        while let Some(input) = rx_to_pty.blocking_recv() {
            match input {
                PtyInput::Data(data) => {
                    let _ = writer_clone.lock().unwrap().write_all(&data);
                }
                PtyInput::Resize { cols, rows } => {
                    let _ = master_clone.lock().unwrap().resize(PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    });
                }
            }
        }
    });

    // Async select loop: WebSocket ↔ channels
    loop {
        tokio::select! {
            Some(data) = rx_to_ws.recv() => {
                if socket.send(Message::Binary(data)).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if text.starts_with("{\"resize\":") {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(resize) = v.get("resize") {
                                    let cols = resize["cols"].as_u64().unwrap_or(80) as u16;
                                    let rows = resize["rows"].as_u64().unwrap_or(24) as u16;
                                    let _ = tx_to_pty.send(PtyInput::Resize { cols, rows }).await;
                                }
                            }
                            continue;
                        }
                        let _ = tx_to_pty.send(PtyInput::Data(text.into_bytes())).await;
                    }
                    Some(Ok(Message::Binary(data))) => {
                        let _ = tx_to_pty.send(PtyInput::Data(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}
