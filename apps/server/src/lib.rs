pub mod auth;
pub mod drive;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use rustflow_collab_protocol::{ClientMsg, DuelInfo, PeerInfo, ServerMsg};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, RwLock};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

const DUEL_DURATION_MS: u64 = 5_000;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub static_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 6129,
            static_dir: PathBuf::from("dist-wasm"),
        }
    }
}

struct ClientConn {
    info: PeerInfo,
    tx: mpsc::UnboundedSender<String>,
}

struct ActiveDuel {
    info: DuelInfo,
    started_at: Instant,
}

#[derive(Default)]
struct RoomState {
    clients: HashMap<String, ClientConn>,
    snapshot: Option<String>,
    locks: HashMap<u64, String>,
    duels: HashMap<u64, ActiveDuel>,
}

impl RoomState {
    fn peer_list(&self) -> Vec<PeerInfo> {
        self.clients.values().map(|c| c.info.clone()).collect()
    }

    fn broadcast(&self, msg: &ServerMsg, exclude: Option<&str>) {
        if let Ok(text) = serde_json::to_string(msg) {
            for (cid, conn) in &self.clients {
                if exclude.is_some_and(|ex| ex == cid) {
                    continue;
                }
                let _ = conn.tx.send(text.clone());
            }
        }
    }

    fn send_to(&self, client_id: &str, msg: &ServerMsg) {
        if let Some(conn) = self.clients.get(client_id) {
            if let Ok(text) = serde_json::to_string(msg) {
                let _ = conn.tx.send(text);
            }
        }
    }
}

pub struct AppState {
    started_at: Instant,
    port: u16,
    next_client: AtomicU64,
    rooms: RwLock<HashMap<String, Arc<RwLock<RoomState>>>>,
    pub auth: auth::AuthStore,
}

impl AppState {
    pub fn new(port: u16) -> Arc<Self> {
        Arc::new(Self {
            started_at: Instant::now(),
            port,
            next_client: AtomicU64::new(1),
            rooms: RwLock::new(HashMap::new()),
            auth: auth::AuthStore::new(),
        })
    }

    async fn get_or_create_room(&self, room_id: &str) -> Arc<RwLock<RoomState>> {
        {
            let guard = self.rooms.read().await;
            if let Some(room) = guard.get(room_id) {
                return Arc::clone(room);
            }
        }
        let mut guard = self.rooms.write().await;
        Arc::clone(
            guard
                .entry(room_id.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(RoomState::default()))),
        )
    }
}

pub fn build_router(config: &ServerConfig) -> Router {
    let state = AppState::new(config.port);
    let index_file = config.static_dir.join("index.html");
    let static_service = ServeDir::new(&config.static_dir).fallback(ServeFile::new(index_file));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/version", get(healthz))
        .route("/api/auth/google/login", get(auth::google_login))
        .route("/api/auth/google/callback", get(auth::google_callback))
        .route("/api/auth/me", get(auth::auth_me))
        .route("/api/auth/logout", post(auth::auth_logout))
        .route("/api/drive/list", get(drive::drive_list))
        .route("/api/drive/browse-folders", get(drive::drive_browse_folders))
        .route("/api/drive/root", post(drive::drive_set_root))
        .route("/api/drive/folders", post(drive::drive_create_folder))
        .route("/api/drive/diagrams", post(drive::drive_save_diagram))
        .route("/api/drive/diagrams/{id}", get(drive::drive_load_diagram))
        .route("/api/drive/items/{id}", patch(drive::drive_update_item))
        .route("/api/drive/items/{id}", delete(drive::drive_delete_item))
        .route("/ws", get(ws_handler))
        .fallback_service(static_service)
        .with_state(state)
        .layer(axum::middleware::from_fn(static_cache_headers))
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("SAMEORIGIN"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
}

async fn static_cache_headers(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = req.uri().path().to_string();
    let mut res = next.run(req).await;
    if !res.headers().contains_key(header::CACHE_CONTROL) {
        if path.ends_with(".wasm") || path.ends_with(".js") {
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
        } else if path == "/" || path.ends_with(".html") {
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache"),
            );
        }
    }
    res
}

async fn healthz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let body = json!({
        "status": "ok",
        "service": "bohat-drawio",
        "engine": "rust-axum",
        "version": env!("CARGO_PKG_VERSION"),
        "port": state.port,
        "uptimeSec": state.started_at.elapsed().as_secs(),
    });
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(body),
    )
}

#[derive(serde::Deserialize)]
struct WsQuery {
    room: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let room_id = query
        .room
        .unwrap_or_else(|| "lobby".to_string())
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(48)
        .collect::<String>();
    let room_id = if room_id.is_empty() {
        "lobby".to_string()
    } else {
        room_id
    };
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: Arc<AppState>) {
    let seq = state.next_client.fetch_add(1, Ordering::Relaxed);
    let client_id = format!("p{seq}");
    let palette = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899"];
    let default_color = palette[(seq as usize) % palette.len()].to_string();

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let write_task = tokio::spawn(async move {
        while let Some(text) = rx.recv().await {
            if ws_tx.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    let room = state.get_or_create_room(&room_id).await;
    {
        let mut r = room.write().await;
        r.clients.insert(
            client_id.clone(),
            ClientConn {
                info: PeerInfo {
                    id: client_id.clone(),
                    name: format!("Guest-{seq}"),
                    color: default_color,
                    cursor: None,
                },
                tx,
            },
        );
        let welcome = ServerMsg::Welcome {
            client_id: client_id.clone(),
            room_id: room_id.clone(),
            peers: r.peer_list(),
            locks: r.locks.clone(),
            snapshot: r.snapshot.clone(),
        };
        r.send_to(&client_id, &welcome);
        r.broadcast(&ServerMsg::Peers { peers: r.peer_list() }, Some(&client_id));
    }

    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(parsed) = serde_json::from_str::<ClientMsg>(&text) {
                    handle_client_msg(&room, &client_id, parsed).await;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    {
        let mut r = room.write().await;
        r.clients.remove(&client_id);
        let mut locks_changed = false;
        r.locks.retain(|_, owner| {
            let keep = owner != &client_id;
            if !keep {
                locks_changed = true;
            }
            keep
        });
        r.broadcast(&ServerMsg::Peers { peers: r.peer_list() }, None);
        if locks_changed {
            r.broadcast(
                &ServerMsg::Locks {
                    locks: r.locks.clone(),
                },
                None,
            );
        }
    }
    write_task.abort();
}

async fn handle_client_msg(room: &Arc<RwLock<RoomState>>, client_id: &str, msg: ClientMsg) {
    match msg {
        ClientMsg::Join { name, color } => {
            let mut r = room.write().await;
            if let Some(c) = r.clients.get_mut(client_id) {
                let clean_name = name.trim();
                if !clean_name.is_empty() {
                    c.info.name = clean_name.chars().take(24).collect();
                }
                if !color.trim().is_empty() {
                    c.info.color = color.trim().chars().take(16).collect();
                }
            }
            r.broadcast(&ServerMsg::Peers { peers: r.peer_list() }, None);
        }
        ClientMsg::Cursor { x, y } => {
            let mut r = room.write().await;
            if let Some(c) = r.clients.get_mut(client_id) {
                c.info.cursor = Some([x, y]);
            }
            r.broadcast(
                &ServerMsg::Cursor {
                    client_id: client_id.to_string(),
                    x,
                    y,
                },
                Some(client_id),
            );
        }
        ClientMsg::Snapshot { scene_json } => {
            let mut r = room.write().await;
            r.snapshot = Some(scene_json.clone());
            r.broadcast(
                &ServerMsg::Snapshot {
                    sender_id: client_id.to_string(),
                    scene_json,
                },
                Some(client_id),
            );
        }
        ClientMsg::Lock { item_id } => {
            let mut r = room.write().await;
            let can_lock = match r.locks.get(&item_id) {
                None => true,
                Some(owner) => owner == client_id || !r.clients.contains_key(owner),
            };
            if can_lock {
                r.locks.insert(item_id, client_id.to_string());
                r.broadcast(
                    &ServerMsg::Locks {
                        locks: r.locks.clone(),
                    },
                    None,
                );
            }
        }
        ClientMsg::Unlock { item_id } => {
            let mut r = room.write().await;
            if r.locks.get(&item_id).is_some_and(|o| o == client_id) {
                r.locks.remove(&item_id);
                r.broadcast(
                    &ServerMsg::Locks {
                        locks: r.locks.clone(),
                    },
                    None,
                );
            }
        }
        ClientMsg::DuelStart {
            item_id,
            item_label,
        } => {
            let start_ticker = {
                let mut r = room.write().await;
                if r.duels.contains_key(&item_id) {
                    false
                } else if let Some(defender_id) = r.locks.get(&item_id).cloned() {
                    if defender_id == client_id {
                        false
                    } else {
                        let challenger = r.clients.get(client_id).map(|c| c.info.clone());
                        let defender = r.clients.get(&defender_id).map(|c| c.info.clone());
                        if let (Some(ch), Some(df)) = (challenger, defender) {
                            let duel_info = DuelInfo {
                                item_id,
                                item_label,
                                challenger_id: ch.id,
                                challenger_name: ch.name,
                                challenger_color: ch.color,
                                defender_id: df.id,
                                defender_name: df.name,
                                defender_color: df.color,
                                challenger_taps: 0,
                                defender_taps: 0,
                                remaining_ms: DUEL_DURATION_MS,
                            };
                            r.broadcast(
                                &ServerMsg::DuelStarted {
                                    duel: duel_info.clone(),
                                },
                                None,
                            );
                            r.duels.insert(
                                item_id,
                                ActiveDuel {
                                    info: duel_info,
                                    started_at: Instant::now(),
                                },
                            );
                            true
                        } else {
                            false
                        }
                    }
                } else {
                    false
                }
            };
            if start_ticker {
                let room_clone = Arc::clone(room);
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_millis(100));
                    loop {
                        interval.tick().await;
                        let mut r = room_clone.write().await;
                        let Some(duel) = r.duels.get_mut(&item_id) else {
                            break;
                        };
                        let elapsed = duel.started_at.elapsed().as_millis() as u64;
                        if elapsed >= DUEL_DURATION_MS {
                            let finished = r.duels.remove(&item_id).unwrap();
                            let d = finished.info;
                            let (winner_id, winner_name) =
                                if d.challenger_taps > d.defender_taps {
                                    (d.challenger_id.clone(), d.challenger_name.clone())
                                } else {
                                    (d.defender_id.clone(), d.defender_name.clone())
                                };
                            r.locks.insert(item_id, winner_id.clone());
                            r.broadcast(
                                &ServerMsg::Locks {
                                    locks: r.locks.clone(),
                                },
                                None,
                            );
                            r.broadcast(
                                &ServerMsg::DuelEnded {
                                    item_id,
                                    winner_id,
                                    winner_name,
                                    challenger_taps: d.challenger_taps,
                                    defender_taps: d.defender_taps,
                                },
                                None,
                            );
                            break;
                        } else {
                            duel.info.remaining_ms = DUEL_DURATION_MS - elapsed;
                            let snap = duel.info.clone();
                            r.broadcast(&ServerMsg::DuelTick { duel: snap }, None);
                        }
                    }
                });
            }
        }
        ClientMsg::DuelMash { item_id } => {
            let mut r = room.write().await;
            if let Some(duel) = r.duels.get_mut(&item_id) {
                if duel.info.challenger_id == client_id {
                    duel.info.challenger_taps += 1;
                } else if duel.info.defender_id == client_id {
                    duel.info.defender_taps += 1;
                }
                let snap = duel.info.clone();
                r.broadcast(&ServerMsg::DuelTick { duel: snap }, None);
            }
        }
    }
}

pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(&config);
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!(
        "Bohat Draw.io (100% Rust Server) listening on http://{addr} (static: {})",
        config.static_dir.display()
    );
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::connect_async;

    #[tokio::test]
    async fn ws_two_client_lock_and_duel_flow() {
        let cfg = ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
            static_dir: PathBuf::from("."),
        };
        let app = build_router(&cfg);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let url = format!("ws://{addr}/ws?room=test-duel");
        let (mut ws1, _) = connect_async(&url).await.unwrap();
        let (mut ws2, _) = connect_async(&url).await.unwrap();

        let _ = ws1.next().await;
        let _ = ws2.next().await;

        let lock_msg = serde_json::to_string(&ClientMsg::Lock { item_id: 99 }).unwrap();
        ws1.send(tokio_tungstenite::tungstenite::Message::Text(lock_msg.into()))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let duel_msg = serde_json::to_string(&ClientMsg::DuelStart {
            item_id: 99,
            item_label: "Core Node".into(),
        })
        .unwrap();
        ws2.send(tokio_tungstenite::tungstenite::Message::Text(duel_msg.into()))
            .await
            .unwrap();

        let mut saw_duel_started = false;
        for _ in 0..8 {
            if let Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt)))) =
                tokio::time::timeout(Duration::from_millis(400), ws1.next()).await
            {
                if let Ok(ServerMsg::DuelStarted { duel }) = serde_json::from_str(&txt) {
                    assert_eq!(duel.item_id, 99);
                    saw_duel_started = true;
                    break;
                }
            }
        }
        assert!(saw_duel_started, "expected DuelStarted broadcast");
        server_task.abort();
    }

    #[tokio::test]
    async fn auth_me_guest_and_drive_requires_auth() {
        let challenge = auth::pkce_code_challenge("test_verifier_12345");
        assert!(!challenge.is_empty());
        assert!(!challenge.contains('='));

        let cfg = ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
            static_dir: PathBuf::from("."),
        };
        let app = build_router(&cfg);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let client = reqwest::Client::new();
        let me_resp = client
            .get(format!("http://{addr}/api/auth/me"))
            .send()
            .await
            .unwrap();
        assert_eq!(me_resp.status(), StatusCode::OK);
        let me: rustflow_collab_protocol::AuthMeResponse = me_resp.json().await.unwrap();
        assert!(!me.authenticated);
        assert!(me.user.is_none());

        let drive_resp = client
            .get(format!("http://{addr}/api/drive/list"))
            .send()
            .await
            .unwrap();
        assert_eq!(drive_resp.status(), StatusCode::UNAUTHORIZED);
        server_task.abort();
    }
}
