use crate::{
    auth::{extract_session_token, StoredSession},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rustflow_collab_protocol::{
    DriveCreateFolderReq, DriveItem, DriveListResponse, DriveLoadDiagramResp, DriveSaveDiagramReq,
    DriveSaveDiagramResp, DriveUpdateItemReq,
};
use serde::Deserialize;
use std::sync::Arc;

const FOLDER_MIME: &str = "application/vnd.google-apps.folder";
const DIAGRAM_MIME: &str = "application/json";
const ROOT_FOLDER_NAME: &str = "Bohat Draw.io";

async fn require_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(String, StoredSession), Response> {
    let Some(tok) = extract_session_token(headers) else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Not signed in" })),
        )
            .into_response());
    };
    let Some(sess) = state.auth.get_valid_session(&tok).await else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Session expired, please sign in again" })),
        )
            .into_response());
    };
    Ok((tok, sess))
}

#[derive(Deserialize)]
struct DriveFileObj {
    id: String,
    name: String,
    #[serde(rename = "mimeType", default)]
    mime_type: String,
    #[serde(default)]
    parents: Option<Vec<String>>,
    #[serde(rename = "modifiedTime", default)]
    modified_time: Option<String>,
    #[serde(default)]
    size: Option<String>,
}

#[derive(Deserialize)]
struct DriveFilesList {
    #[serde(default)]
    files: Vec<DriveFileObj>,
}

async fn ensure_root_folder(
    state: &AppState,
    token: &str,
    sess: &StoredSession,
) -> Result<String, String> {
    if let Some(fid) = &sess.root_folder_id {
        return Ok(fid.clone());
    }
    let q = format!(
        "name = '{ROOT_FOLDER_NAME}' and mimeType = '{FOLDER_MIME}' and trashed = false"
    );
    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name)&pageSize=5",
        urlencoding::encode(&q)
    );
    let resp = state
        .auth
        .http
        .get(&url)
        .bearer_auth(&sess.access_token)
        .send()
        .await
        .map_err(|e| format!("Drive query error: {e}"))?;
    if !resp.status().is_success() {
        let st = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!("Drive list root failed ({st}): {txt}"));
    }
    let list = resp
        .json::<DriveFilesList>()
        .await
        .map_err(|e| format!("Drive parse error: {e}"))?;
    if let Some(existing) = list.files.first() {
        state
            .auth
            .set_root_folder_id(token, existing.id.clone())
            .await;
        return Ok(existing.id.clone());
    }

    let create_resp = state
        .auth
        .http
        .post("https://www.googleapis.com/drive/v3/files?fields=id,name")
        .bearer_auth(&sess.access_token)
        .json(&serde_json::json!({
            "name": ROOT_FOLDER_NAME,
            "mimeType": FOLDER_MIME,
        }))
        .send()
        .await
        .map_err(|e| format!("Drive create root error: {e}"))?;
    if !create_resp.status().is_success() {
        let st = create_resp.status();
        let txt = create_resp.text().await.unwrap_or_default();
        return Err(format!("Drive create root failed ({st}): {txt}"));
    }
    let created = create_resp
        .json::<DriveFileObj>()
        .await
        .map_err(|e| format!("Drive root parse error: {e}"))?;
    state
        .auth
        .set_root_folder_id(token, created.id.clone())
        .await;
    Ok(created.id)
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub folder_id: Option<String>,
}

pub async fn drive_list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Response {
    let (tok, sess) = match require_session(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let root_id = match ensure_root_folder(&state, &tok, &sess).await {
        Ok(id) => id,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": err })),
            )
                .into_response()
        }
    };
    let current_id = query
        .folder_id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| root_id.clone());
    let safe_id = current_id.replace('\'', "");
    let q = format!("'{safe_id}' in parents and trashed = false");
    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q={}&orderBy=folder,name&fields=files(id,name,mimeType,parents,modifiedTime,size)&pageSize=200",
        urlencoding::encode(&q)
    );
    let resp = match state
        .auth
        .http
        .get(&url)
        .bearer_auth(&sess.access_token)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            let st = r.status();
            let txt = r.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Drive list failed ({st}): {txt}") })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Drive list error: {e}") })),
            )
                .into_response()
        }
    };
    let parsed = match resp.json::<DriveFilesList>().await {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Drive parse error: {e}") })),
            )
                .into_response()
        }
    };

    let items: Vec<DriveItem> = parsed
        .files
        .into_iter()
        .map(|f| DriveItem {
            id: f.id,
            name: f.name,
            is_folder: f.mime_type == FOLDER_MIME,
            parent_id: f.parents.and_then(|p| p.into_iter().next()),
            modified_time: f.modified_time,
            size_bytes: f.size.and_then(|s| s.parse::<u64>().ok()),
        })
        .collect();

    Json(DriveListResponse {
        root_folder_id: root_id,
        current_folder_id: current_id,
        items,
    })
    .into_response()
}

pub async fn drive_create_folder(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DriveCreateFolderReq>,
) -> Response {
    let (tok, sess) = match require_session(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let name = req.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Folder name required" })),
        )
            .into_response();
    }
    let parent_id = match req.parent_id.filter(|s| !s.trim().is_empty()) {
        Some(pid) => pid,
        None => match ensure_root_folder(&state, &tok, &sess).await {
            Ok(id) => id,
            Err(err) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": err })),
                )
                    .into_response()
            }
        },
    };
    let resp = match state
        .auth
        .http
        .post("https://www.googleapis.com/drive/v3/files?fields=id,name,mimeType,parents,modifiedTime")
        .bearer_auth(&sess.access_token)
        .json(&serde_json::json!({
            "name": name,
            "mimeType": FOLDER_MIME,
            "parents": [parent_id],
        }))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            let st = r.status();
            let txt = r.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Create folder failed ({st}): {txt}") })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Create folder error: {e}") })),
            )
                .into_response()
        }
    };
    match resp.json::<DriveFileObj>().await {
        Ok(f) => Json(DriveItem {
            id: f.id,
            name: f.name,
            is_folder: true,
            parent_id: f.parents.and_then(|p| p.into_iter().next()),
            modified_time: f.modified_time,
            size_bytes: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Parse folder error: {e}") })),
        )
            .into_response(),
    }
}

pub async fn drive_save_diagram(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DriveSaveDiagramReq>,
) -> Response {
    let (tok, sess) = match require_session(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let raw_name = req.name.trim();
    if raw_name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Diagram name required" })),
        )
            .into_response();
    }
    let file_name = if raw_name.ends_with(".bohat.json") {
        raw_name.to_string()
    } else {
        format!("{raw_name}.bohat.json")
    };
    let parent_id = match req.parent_folder_id.filter(|s| !s.trim().is_empty()) {
        Some(pid) => pid,
        None => match ensure_root_folder(&state, &tok, &sess).await {
            Ok(id) => id,
            Err(err) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": err })),
                )
                    .into_response()
            }
        },
    };

    let boundary = "bohat_drive_multipart_7e3a91c";
    let metadata = if req.file_id.is_some() {
        serde_json::json!({ "name": file_name })
    } else {
        serde_json::json!({
            "name": file_name,
            "mimeType": DIAGRAM_MIME,
            "parents": [parent_id],
        })
    };
    let body = format!(
        "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: {DIAGRAM_MIME}\r\n\r\n{}\r\n--{boundary}--",
        metadata, req.scene_json
    );
    let content_type = format!("multipart/related; boundary={boundary}");

    let request = if let Some(fid) = req.file_id.filter(|s| !s.trim().is_empty()) {
        let url = format!(
            "https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=multipart&fields=id,name,parents",
            urlencoding::encode(&fid)
        );
        state.auth.http.patch(&url)
    } else {
        state.auth.http.post(
            "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id,name,parents",
        )
    };

    let resp = match request
        .bearer_auth(&sess.access_token)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            let st = r.status();
            let txt = r.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Save diagram failed ({st}): {txt}") })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Save diagram error: {e}") })),
            )
                .into_response()
        }
    };

    match resp.json::<DriveFileObj>().await {
        Ok(f) => Json(DriveSaveDiagramResp {
            id: f.id,
            name: f.name,
            parent_folder_id: f
                .parents
                .and_then(|p| p.into_iter().next())
                .unwrap_or(parent_id),
        })
        .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Parse saved file error: {e}") })),
        )
            .into_response(),
    }
}

pub async fn drive_load_diagram(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (_, sess) = match require_session(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let meta_url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?fields=id,name,parents",
        urlencoding::encode(&id)
    );
    let meta = match state
        .auth
        .http
        .get(&meta_url)
        .bearer_auth(&sess.access_token)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => match r.json::<DriveFileObj>().await {
            Ok(m) => m,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("Metadata parse error: {e}") })),
                )
                    .into_response()
            }
        },
        Ok(r) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Load metadata failed ({})", r.status()) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Load metadata error: {e}") })),
            )
                .into_response()
        }
    };

    let media_url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media",
        urlencoding::encode(&id)
    );
    let scene_json = match state
        .auth
        .http
        .get(&media_url)
        .bearer_auth(&sess.access_token)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
        Ok(r) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Download failed ({})", r.status()) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Download error: {e}") })),
            )
                .into_response()
        }
    };

    Json(DriveLoadDiagramResp {
        id: meta.id,
        name: meta.name,
        parent_folder_id: meta.parents.and_then(|p| p.into_iter().next()),
        scene_json,
    })
    .into_response()
}

pub async fn drive_update_item(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<DriveUpdateItemReq>,
) -> Response {
    let (_, sess) = match require_session(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?fields=id,name,mimeType,parents,modifiedTime",
        urlencoding::encode(&id)
    );
    if let (Some(new_p), Some(old_p)) = (&req.new_parent_id, &req.old_parent_id) {
        url.push_str(&format!(
            "&addParents={}&removeParents={}",
            urlencoding::encode(new_p),
            urlencoding::encode(old_p)
        ));
    }
    let mut body = serde_json::Map::new();
    if let Some(name) = req.name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        body.insert("name".to_string(), serde_json::Value::String(name));
    }
    let resp = match state
        .auth
        .http
        .patch(&url)
        .bearer_auth(&sess.access_token)
        .json(&serde_json::Value::Object(body))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            let st = r.status();
            let txt = r.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Update failed ({st}): {txt}") })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Update error: {e}") })),
            )
                .into_response()
        }
    };
    match resp.json::<DriveFileObj>().await {
        Ok(f) => Json(DriveItem {
            id: f.id,
            name: f.name,
            is_folder: f.mime_type == FOLDER_MIME,
            parent_id: f.parents.and_then(|p| p.into_iter().next()),
            modified_time: f.modified_time,
            size_bytes: None,
        })
        .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Parse update error: {e}") })),
        )
            .into_response(),
    }
}

pub async fn drive_delete_item(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (_, sess) = match require_session(&state, &headers).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}",
        urlencoding::encode(&id)
    );
    match state
        .auth
        .http
        .patch(&url)
        .bearer_auth(&sess.access_token)
        .json(&serde_json::json!({ "trashed": true }))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
        }
        Ok(r) => {
            let st = r.status();
            let txt = r.text().await.unwrap_or_default();
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Delete failed ({st}): {txt}") })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Delete error: {e}") })),
        )
            .into_response(),
    }
}
