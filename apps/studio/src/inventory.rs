use rustflow_collab_protocol::{
    AuthMeResponse, DriveCreateFolderReq, DriveFolderBrowseResponse, DriveItem, DriveListResponse,
    DriveLoadDiagramResp, DriveSaveDiagramReq, DriveSaveDiagramResp, DriveSetRootReq,
    DriveUpdateItemReq, UserProfile,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum InventoryEvent {
    AuthLoaded(AuthMeResponse),
    #[allow(dead_code)]
    NativeTokenReceived(String),
    LoggedOut,
    ListLoaded(DriveListResponse),
    PickerFoldersLoaded(DriveFolderBrowseResponse),
    RootFolderChanged(String, String),
    DiagramLoaded(DriveLoadDiagramResp),
    PreviewLoaded(String, String),
    DiagramSaved(DriveSaveDiagramResp),
    Mutated(String),
    Error(String),
}

pub struct InventoryState {
    pub user: Option<UserProfile>,
    pub google_enabled: bool,
    pub auth_checked: bool,
    pub bearer_token: Option<String>,
    pub server_base: String,
    pub root_folder_id: Option<String>,
    pub root_folder_name: String,
    pub folder_stack: Vec<(String, String)>,
    pub items: Vec<DriveItem>,
    pub preview_textures: HashMap<String, egui::TextureHandle>,
    pub preview_requested: HashSet<String>,
    pub show_folder_picker: bool,
    pub picker_stack: Vec<(String, String)>,
    pub picker_folders: Vec<DriveItem>,
    pub picker_search: String,
    pub active_file_id: Option<String>,
    pub active_file_name: String,
    pub active_folder_id: Option<String>,
    pub show_modal: bool,
    pub show_setup_help: bool,
    pub list_view: bool,
    pub search: String,
    pub new_folder_name: String,
    pub show_new_folder_bar: bool,
    pub save_as_name: String,
    pub selected_item_id: Option<String>,
    pub renaming_item: Option<(String, String)>,
    pub loading: bool,
    pub last_error: Option<String>,
    events: Arc<Mutex<Vec<InventoryEvent>>>,
}

impl InventoryState {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let server_base = String::new();
        #[cfg(not(target_arch = "wasm32"))]
        let server_base = std::env::var("BOHAT_SERVER_URL")
            .ok()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "http://127.0.0.1:6129".to_string());

        Self {
            user: None,
            google_enabled: false,
            auth_checked: false,
            bearer_token: None,
            server_base,
            root_folder_id: None,
            root_folder_name: "Bohat Draw.io".to_string(),
            folder_stack: Vec::new(),
            items: Vec::new(),
            preview_textures: HashMap::new(),
            preview_requested: HashSet::new(),
            show_folder_picker: false,
            picker_stack: vec![("root".to_string(), "My Drive".to_string())],
            picker_folders: Vec::new(),
            picker_search: String::new(),
            active_file_id: None,
            active_file_name: "Untitled Diagram".to_string(),
            active_folder_id: None,
            show_modal: false,
            show_setup_help: false,
            list_view: false,
            search: String::new(),
            new_folder_name: String::new(),
            show_new_folder_bar: false,
            save_as_name: "Untitled Diagram".to_string(),
            selected_item_id: None,
            renaming_item: None,
            loading: false,
            last_error: None,
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn current_folder_id(&self) -> Option<String> {
        self.folder_stack
            .last()
            .map(|(id, _)| id.clone())
            .or_else(|| self.root_folder_id.clone())
    }

    pub fn drain_events(&self) -> Vec<InventoryEvent> {
        if let Ok(mut guard) = self.events.lock() {
            std::mem::take(&mut *guard)
        } else {
            Vec::new()
        }
    }

    pub fn fetch_auth_me(&mut self, ctx: egui::Context) {
        let url = format!("{}/api/auth/me", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("GET", url, None, token, ctx, move |res| {
            let ev = match res {
                Ok(body) => match serde_json::from_str::<AuthMeResponse>(&body) {
                    Ok(me) => InventoryEvent::AuthLoaded(me),
                    Err(e) => InventoryEvent::Error(format!("Auth parse error: {e}")),
                },
                Err(_) => InventoryEvent::AuthLoaded(AuthMeResponse {
                    authenticated: false,
                    google_enabled: false,
                    user: None,
                }),
            };
            push_event(&events, ev);
        });
    }

    pub fn start_google_login(&mut self, ctx: egui::Context) {
        if !self.google_enabled {
            self.show_setup_help = true;
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = ctx;
            if let Some(win) = web_sys::window() {
                let _ = win.location().set_href("/api/auth/google/login");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let base = self.server_base.clone();
            let events = Arc::clone(&self.events);
            std::thread::spawn(move || {
                use std::io::{Read, Write};
                let Ok(listener) = std::net::TcpListener::bind("127.0.0.1:0") else {
                    push_event(
                        &events,
                        InventoryEvent::Error("Could not bind local OAuth callback port".into()),
                    );
                    ctx.request_repaint();
                    return;
                };
                let Ok(addr) = listener.local_addr() else { return };
                let login_url = format!("{base}/api/auth/google/login?native_port={}", addr.port());
                open_browser_url(&login_url);
                let _ = listener.set_nonblocking(false);
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 4096];
                    if let Ok(n) = stream.read(&mut buf) {
                        let req_str = String::from_utf8_lossy(&buf[..n]);
                        if let Some(first_line) = req_str.lines().next() {
                            if let Some(path) = first_line.split_whitespace().nth(1) {
                                if let Some(q) = path.split('?').nth(1) {
                                    for pair in q.split('&') {
                                        if let Some(tok) = pair.strip_prefix("token=") {
                                            push_event(
                                                &events,
                                                InventoryEvent::NativeTokenReceived(tok.to_string()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let html = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body style='font-family:sans-serif;padding:28px'><h2>Signed in to Bohat Draw.io!</h2><p>You can close this tab and return to the desktop app.</p></body></html>";
                    let _ = stream.write_all(html.as_bytes());
                    ctx.request_repaint();
                }
            });
        }
    }

    pub fn logout(&mut self, ctx: egui::Context) {
        let url = format!("{}/api/auth/logout", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        self.user = None;
        self.bearer_token = None;
        self.items.clear();
        self.preview_textures.clear();
        self.preview_requested.clear();
        self.folder_stack.clear();
        self.root_folder_id = None;
        self.active_file_id = None;
        self.show_modal = false;
        spawn_http_json("POST", url, Some("{}".to_string()), token, ctx, move |_| {
            push_event(&events, InventoryEvent::LoggedOut);
        });
    }

    pub fn fetch_folder(&mut self, folder_id: Option<String>, ctx: egui::Context) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let url = match folder_id {
            Some(ref fid) if !fid.is_empty() => {
                format!("{}/api/drive/list?folder_id={fid}", self.server_base)
            }
            _ => format!("{}/api/drive/list", self.server_base),
        };
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("GET", url, None, token, ctx, move |res| {
            let ev = match res {
                Ok(body) => match serde_json::from_str::<DriveListResponse>(&body) {
                    Ok(list) => InventoryEvent::ListLoaded(list),
                    Err(e) => InventoryEvent::Error(format!("Drive list parse error: {e}")),
                },
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn browse_drive_folders(&mut self, parent_id: Option<String>, ctx: egui::Context) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let pid = parent_id.unwrap_or_else(|| "root".to_string());
        let url = format!("{}/api/drive/browse-folders?parent_id={pid}", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("GET", url, None, token, ctx, move |res| {
            let ev = match res {
                Ok(body) => match serde_json::from_str::<DriveFolderBrowseResponse>(&body) {
                    Ok(parsed) => InventoryEvent::PickerFoldersLoaded(parsed),
                    Err(e) => InventoryEvent::Error(format!("Folder browse parse error: {e}")),
                },
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn select_root_folder(
        &mut self,
        folder_id: Option<String>,
        folder_name: Option<String>,
        ctx: egui::Context,
    ) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let req = DriveSetRootReq {
            folder_id,
            folder_name,
        };
        let Ok(body) = serde_json::to_string(&req) else {
            return;
        };
        let url = format!("{}/api/drive/root", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("POST", url, Some(body), token, ctx, move |res| {
            let ev = match res {
                Ok(txt) => match serde_json::from_str::<serde_json::Value>(&txt) {
                    Ok(v) => {
                        let id = v
                            .get("root_folder_id")
                            .and_then(|x| x.as_str())
                            .unwrap_or("root")
                            .to_string();
                        let name = v
                            .get("root_folder_name")
                            .and_then(|x| x.as_str())
                            .unwrap_or("Bohat Draw.io")
                            .to_string();
                        InventoryEvent::RootFolderChanged(id, name)
                    }
                    Err(e) => InventoryEvent::Error(format!("Set root parse error: {e}")),
                },
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn create_folder(&mut self, name: String, ctx: egui::Context) {
        let trimmed = name.trim().to_string();
        if trimmed.is_empty() || self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let req = DriveCreateFolderReq {
            name: trimmed.clone(),
            parent_id: self.current_folder_id(),
        };
        let Ok(body) = serde_json::to_string(&req) else {
            return;
        };
        let url = format!("{}/api/drive/folders", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("POST", url, Some(body), token, ctx, move |res| {
            let ev = match res {
                Ok(_) => InventoryEvent::Mutated(format!("Created folder '{trimmed}'")),
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn save_diagram(
        &mut self,
        file_id: Option<String>,
        parent_folder_id: Option<String>,
        name: String,
        scene_json: String,
        ctx: egui::Context,
    ) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let req = DriveSaveDiagramReq {
            file_id,
            parent_folder_id: parent_folder_id.or_else(|| self.current_folder_id()),
            name,
            scene_json,
        };
        let Ok(body) = serde_json::to_string(&req) else {
            return;
        };
        let url = format!("{}/api/drive/diagrams", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("POST", url, Some(body), token, ctx, move |res| {
            let ev = match res {
                Ok(txt) => match serde_json::from_str::<DriveSaveDiagramResp>(&txt) {
                    Ok(saved) => InventoryEvent::DiagramSaved(saved),
                    Err(e) => InventoryEvent::Error(format!("Save parse error: {e}")),
                },
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn load_diagram(&mut self, file_id: String, ctx: egui::Context) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let url = format!("{}/api/drive/diagrams/{file_id}", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("GET", url, None, token, ctx, move |res| {
            let ev = match res {
                Ok(txt) => match serde_json::from_str::<DriveLoadDiagramResp>(&txt) {
                    Ok(loaded) => InventoryEvent::DiagramLoaded(loaded),
                    Err(e) => InventoryEvent::Error(format!("Load parse error: {e}")),
                },
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn request_diagram_preview(&mut self, file_id: String, cache_key: String, ctx: egui::Context) {
        if self.user.is_none()
            || self.preview_textures.contains_key(&cache_key)
            || !self.preview_requested.insert(cache_key.clone())
        {
            return;
        }
        let url = format!("{}/api/drive/diagrams/{file_id}", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("GET", url, None, token, ctx, move |res| {
            if let Ok(txt) = res {
                if let Ok(loaded) = serde_json::from_str::<DriveLoadDiagramResp>(&txt) {
                    push_event(&events, InventoryEvent::PreviewLoaded(cache_key, loaded.scene_json));
                }
            }
        });
    }

    pub fn rename_item(&mut self, item_id: String, new_name: String, ctx: egui::Context) {
        let trimmed = new_name.trim().to_string();
        if trimmed.is_empty() || self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let req = DriveUpdateItemReq {
            name: Some(trimmed.clone()),
            new_parent_id: None,
            old_parent_id: None,
        };
        let Ok(body) = serde_json::to_string(&req) else {
            return;
        };
        let url = format!("{}/api/drive/items/{item_id}", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("PATCH", url, Some(body), token, ctx, move |res| {
            let ev = match res {
                Ok(_) => InventoryEvent::Mutated(format!("Renamed to '{trimmed}'")),
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn move_item_to_folder(
        &mut self,
        item_id: String,
        old_parent: String,
        new_parent: String,
        ctx: egui::Context,
    ) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let req = DriveUpdateItemReq {
            name: None,
            new_parent_id: Some(new_parent),
            old_parent_id: Some(old_parent),
        };
        let Ok(body) = serde_json::to_string(&req) else {
            return;
        };
        let url = format!("{}/api/drive/items/{item_id}", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("PATCH", url, Some(body), token, ctx, move |res| {
            let ev = match res {
                Ok(_) => InventoryEvent::Mutated("Moved item".to_string()),
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }

    pub fn delete_item(&mut self, item_id: String, ctx: egui::Context) {
        if self.user.is_none() {
            return;
        }
        self.loading = true;
        self.last_error = None;
        let url = format!("{}/api/drive/items/{item_id}", self.server_base);
        let token = self.bearer_token.clone();
        let events = Arc::clone(&self.events);
        spawn_http_json("DELETE", url, None, token, ctx, move |res| {
            let ev = match res {
                Ok(_) => InventoryEvent::Mutated("Moved to Google Drive Trash".to_string()),
                Err(err) => InventoryEvent::Error(err),
            };
            push_event(&events, ev);
        });
    }
}

fn format_http_error(status: u16, text: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(msg) = v.get("error").and_then(|e| e.as_str()) {
            return msg.to_string();
        }
    }
    if text.trim_start().starts_with('<') {
        return format!("HTTP {status} (server/gateway error — please retry or Sign Out & Sign In again)");
    }
    format!("HTTP {status}: {}", text.chars().take(180).collect::<String>())
}

fn push_event(events: &Arc<Mutex<Vec<InventoryEvent>>>, ev: InventoryEvent) {
    if let Ok(mut guard) = events.lock() {
        guard.push(ev);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_browser_url(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_http_json(
    method: &'static str,
    url: String,
    body: Option<String>,
    bearer: Option<String>,
    ctx: egui::Context,
    on_done: impl FnOnce(Result<String, String>) + Send + 'static,
) {
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut req = match method {
            "POST" => client.post(&url),
            "PATCH" => client.patch(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };
        if let Some(tok) = bearer {
            req = req.bearer_auth(tok);
        }
        if let Some(b) = body {
            req = req
                .header("Content-Type", "application/json")
                .body(b);
        }
        let res = match req.send() {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                if status.is_success() {
                    Ok(text)
                } else {
                    Err(format_http_error(status.as_u16(), &text))
                }
            }
            Err(e) => Err(format!("Network error: {e}")),
        };
        on_done(res);
        ctx.request_repaint();
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_http_json(
    method: &'static str,
    url: String,
    body: Option<String>,
    _bearer: Option<String>,
    ctx: egui::Context,
    on_done: impl FnOnce(Result<String, String>) + 'static,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    wasm_bindgen_futures::spawn_local(async move {
        let result = async {
            let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
            let opts = web_sys::RequestInit::new();
            opts.set_method(method);
            opts.set_credentials(web_sys::RequestCredentials::Include);
            opts.set_mode(web_sys::RequestMode::SameOrigin);
            if let Some(b) = body {
                opts.set_body(&wasm_bindgen::JsValue::from_str(&b));
                let headers = web_sys::Headers::new().map_err(|_| "headers failed".to_string())?;
                headers
                    .set("Content-Type", "application/json")
                    .map_err(|_| "set header failed".to_string())?;
                opts.set_headers(&headers);
            }
            let request = web_sys::Request::new_with_str_and_init(&url, &opts)
                .map_err(|_| "request init failed".to_string())?;
            let resp_val = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|_| "fetch failed".to_string())?;
            let resp: web_sys::Response = resp_val
                .dyn_into()
                .map_err(|_| "response cast failed".to_string())?;
            let text_promise = resp.text().map_err(|_| "resp.text failed".to_string())?;
            let text_val = JsFuture::from(text_promise)
                .await
                .map_err(|_| "text await failed".to_string())?;
            let text = text_val.as_string().unwrap_or_default();
            if resp.ok() {
                Ok(text)
            } else {
                Err(format_http_error(resp.status(), &text))
            }
        }
        .await;
        on_done(result);
        ctx.request_repaint();
    });
}
