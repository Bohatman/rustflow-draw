use crate::AppState;
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use base64::Engine;
use rand::RngCore;
use rustflow_collab_protocol::{AuthMeResponse, UserProfile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

pub const SESSION_COOKIE: &str = "bohat_session";

#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub public_base_url: String,
}

impl GoogleOAuthConfig {
    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").ok()?.trim().to_string();
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok()?.trim().to_string();
        if client_id.is_empty() || client_secret.is_empty() {
            return None;
        }
        let port = std::env::var("PORT").unwrap_or_else(|_| "6129".to_string());
        let public_base_url = std::env::var("PUBLIC_BASE_URL")
            .ok()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("http://localhost:{port}"));
        Some(Self {
            client_id,
            client_secret,
            public_base_url,
        })
    }

    pub fn redirect_uri(&self) -> String {
        format!("{}/api/auth/google/callback", self.public_base_url)
    }

    pub fn is_https(&self) -> bool {
        self.public_base_url.starts_with("https://")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub token: String,
    pub user: UserProfile,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
    pub root_folder_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PendingOAuthState {
    pub code_verifier: String,
    pub native_port: Option<u16>,
    pub created_at: u64,
}

pub struct AuthStore {
    pub oauth: Option<GoogleOAuthConfig>,
    pub http: reqwest::Client,
    pub sessions: RwLock<HashMap<String, StoredSession>>,
    pub pending: RwLock<HashMap<String, PendingOAuthState>>,
    pub store_path: PathBuf,
}

impl AuthStore {
    pub fn new() -> Self {
        let store_path = std::env::var("BOHAT_SESSION_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/sessions.json"));
        let loaded = std::fs::read_to_string(&store_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<HashMap<String, StoredSession>>(&raw).ok())
            .unwrap_or_default();
        Self {
            oauth: GoogleOAuthConfig::from_env(),
            http: reqwest::Client::new(),
            sessions: RwLock::new(loaded),
            pending: RwLock::new(HashMap::new()),
            store_path,
        }
    }

    pub async fn persist(&self) {
        let guard = self.sessions.read().await;
        if let Some(parent) = self.store_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(raw) = serde_json::to_string_pretty(&*guard) {
            let _ = std::fs::write(&self.store_path, raw);
        }
    }

    pub async fn get_valid_session(&self, token: &str) -> Option<StoredSession> {
        let current = {
            let guard = self.sessions.read().await;
            guard.get(token).cloned()?
        };
        let now = unix_now();
        if current.expires_at > now + 60 {
            return Some(current);
        }
        let refresh = current.refresh_token.clone()?;
        let cfg = self.oauth.as_ref()?;
        let refreshed = self.refresh_access_token(cfg, &refresh).await.ok()?;
        let updated = {
            let mut guard = self.sessions.write().await;
            let entry = guard.get_mut(token)?;
            entry.access_token = refreshed.access_token;
            entry.expires_at = unix_now() + refreshed.expires_in.unwrap_or(3500);
            if let Some(rt) = refreshed.refresh_token {
                entry.refresh_token = Some(rt);
            }
            entry.clone()
        };
        self.persist().await;
        Some(updated)
    }

    pub async fn set_root_folder_id(&self, token: &str, folder_id: String) {
        {
            let mut guard = self.sessions.write().await;
            if let Some(sess) = guard.get_mut(token) {
                sess.root_folder_id = Some(folder_id);
            }
        }
        self.persist().await;
    }

    async fn refresh_access_token(
        &self,
        cfg: &GoogleOAuthConfig,
        refresh_token: &str,
    ) -> Result<GoogleTokenResponse, String> {
        let resp = self
            .http
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", cfg.client_id.as_str()),
                ("client_secret", cfg.client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| format!("token refresh request failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("token refresh status {}", resp.status()));
        }
        resp.json::<GoogleTokenResponse>()
            .await
            .map_err(|e| format!("token refresh decode failed: {e}"))
    }
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn random_token_b64(num_bytes: usize) -> String {
    let mut bytes = vec![0u8; num_bytes];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn pkce_code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(tok) = auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("bearer ")) {
            let trimmed = tok.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    let cookie_hdr = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_hdr.split(';') {
        let kv = part.trim();
        if let Some(val) = kv.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[derive(Deserialize)]
pub struct LoginQuery {
    pub native_port: Option<u16>,
}

pub async fn google_login(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoginQuery>,
) -> Response {
    let Some(cfg) = state.auth.oauth.as_ref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Html(
                "<html><body style='font-family:sans-serif;padding:24px'>\
                 <h2>Google Sign-In Not Configured Yet</h2>\
                 <p>Set <code>GOOGLE_CLIENT_ID</code>, <code>GOOGLE_CLIENT_SECRET</code>, and <code>PUBLIC_BASE_URL</code> in <code>.env</code> and restart <code>bohat-drawio</code>.</p>\
                 <p><a href='/'>Return to Bohat Draw.io (Guest Mode)</a></p>\
                 </body></html>",
            ),
        )
            .into_response();
    };

    let state_tok = random_token_b64(24);
    let verifier = random_token_b64(32);
    let challenge = pkce_code_challenge(&verifier);

    {
        let mut pending = state.auth.pending.write().await;
        let cutoff = unix_now().saturating_sub(600);
        pending.retain(|_, v| v.created_at >= cutoff);
        pending.insert(
            state_tok.clone(),
            PendingOAuthState {
                code_verifier: verifier,
                native_port: query.native_port,
                created_at: unix_now(),
            },
        );
    }

    let scope = "openid email profile https://www.googleapis.com/auth/drive.file";
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&cfg.client_id),
        urlencoding::encode(&cfg.redirect_uri()),
        urlencoding::encode(scope),
        urlencoding::encode(&state_tok),
        urlencoding::encode(&challenge),
    );
    Redirect::temporary(&auth_url).into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    picture: Option<String>,
}

pub async fn google_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    if let Some(err) = query.error {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "<html><body style='font-family:sans-serif;padding:24px'><h2>Google Sign-In Cancelled</h2><p>{err}</p><p><a href='/'>Return to Bohat Draw.io</a></p></body></html>"
            )),
        )
            .into_response();
    }
    let Some(cfg) = state.auth.oauth.as_ref() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth not configured").into_response();
    };
    let (Some(code), Some(state_tok)) = (query.code, query.state) else {
        return (StatusCode::BAD_REQUEST, "Missing code or state").into_response();
    };
    let Some(pending) = state.auth.pending.write().await.remove(&state_tok) else {
        return (StatusCode::BAD_REQUEST, "Invalid or expired OAuth state").into_response();
    };

    let redirect_uri = cfg.redirect_uri();
    let tok_resp = match state
        .auth
        .http
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", cfg.client_id.as_str()),
            ("client_secret", cfg.client_secret.as_str()),
            ("code", code.as_str()),
            ("code_verifier", pending.code_verifier.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => match r.json::<GoogleTokenResponse>().await {
            Ok(parsed) => parsed,
            Err(e) => {
                return (StatusCode::BAD_GATEWAY, format!("Token decode error: {e}")).into_response()
            }
        },
        Ok(r) => {
            let status = r.status();
            let txt = r.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                format!("Google token exchange failed ({status}): {txt}"),
            )
                .into_response();
        }
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("Google token request error: {e}"))
                .into_response()
        }
    };

    let user_info = match state
        .auth
        .http
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(&tok_resp.access_token)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => match r.json::<GoogleUserInfo>().await {
            Ok(u) => u,
            Err(e) => {
                return (StatusCode::BAD_GATEWAY, format!("Userinfo decode error: {e}"))
                    .into_response()
            }
        },
        Ok(r) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Google userinfo failed ({})", r.status()),
            )
                .into_response()
        }
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("Userinfo request error: {e}"))
                .into_response()
        }
    };

    let display_name = if !user_info.name.trim().is_empty() {
        user_info.name.trim().to_string()
    } else if !user_info.email.is_empty() {
        user_info.email.clone()
    } else {
        "Google User".to_string()
    };

    let session_token = random_token_b64(32);
    let sess = StoredSession {
        token: session_token.clone(),
        user: UserProfile {
            id: user_info.sub,
            email: user_info.email,
            name: display_name,
            avatar_url: user_info.picture,
        },
        access_token: tok_resp.access_token,
        refresh_token: tok_resp.refresh_token,
        expires_at: unix_now() + tok_resp.expires_in.unwrap_or(3500),
        root_folder_id: None,
    };

    state
        .auth
        .sessions
        .write()
        .await
        .insert(session_token.clone(), sess);
    state.auth.persist().await;

    if let Some(native_port) = pending.native_port {
        let url = format!(
            "http://127.0.0.1:{native_port}/callback?token={}",
            urlencoding::encode(&session_token)
        );
        return Redirect::temporary(&url).into_response();
    }

    let secure_flag = if cfg.is_https() { "; Secure" } else { "" };
    let cookie = format!(
        "{SESSION_COOKIE}={session_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000{secure_flag}"
    );
    let mut resp = Redirect::temporary("/").into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(header::SET_COOKIE, val);
    }
    resp
}

pub async fn auth_me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let google_enabled = state.auth.oauth.is_some();
    let user = match extract_session_token(&headers) {
        Some(tok) => state
            .auth
            .get_valid_session(&tok)
            .await
            .map(|s| s.user),
        None => None,
    };
    let body = AuthMeResponse {
        authenticated: user.is_some(),
        google_enabled,
        user,
    };
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(body),
    )
}

pub async fn auth_logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(tok) = extract_session_token(&headers) {
        state.auth.sessions.write().await.remove(&tok);
        state.auth.persist().await;
    }
    let cookie = format!("{SESSION_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0");
    let mut resp = (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(header::SET_COOKIE, val);
    }
    resp
}
