#[cfg(not(target_arch = "wasm32"))]
pub fn copy_png_to_clipboard(png_bytes: &[u8]) -> Result<String, String> {
    copy_png_to_system_clipboard(png_bytes)?;
    Ok("Copied 2x PNG picture to clipboard (ready to paste into PowerPoint / Slides)".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_png_to_system_clipboard(png_bytes: &[u8]) -> Result<(), String> {
    let decoder = png::Decoder::new(png_bytes);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("PNG decode failed: {e}"))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("PNG frame failed: {e}"))?;
    let bytes = &buf[..info.buffer_size()];
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
    clipboard
        .set_image(arboard::ImageData {
            width: info.width as usize,
            height: info.height as usize,
            bytes: std::borrow::Cow::Borrowed(bytes),
        })
        .map_err(|e| format!("Clipboard set_image failed: {e}"))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_animated_gif_or_save(gif_bytes: &[u8]) -> Result<String, String> {
    let path = std::env::temp_dir().join("bohat-draw-animation.gif");
    std::fs::write(&path, gif_bytes).map_err(|e| format!("Write GIF failed: {e}"))?;
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let html = format!(
            "<img src=\"data:image/gif;base64,{}\" alt=\"Bohat Draw Animation\"/>",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, gif_bytes)
        );
        let _ = clipboard.set_html(html, Some(path.display().to_string()));
    }
    Ok(format!("Copied animated GIF & saved to {}", path.display()))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_bytes_dialog(default_name: &str, bytes: &[u8]) -> Result<String, String> {
    let Some(path) = rfd::FileDialog::new().set_file_name(default_name).save_file() else {
        return Err("Save cancelled".to_string());
    };
    std::fs::write(&path, bytes).map_err(|e| format!("Save failed: {e}"))?;
    Ok(format!("Saved {}", path.display()))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_image_file_sync() -> Option<(String, Vec<u8>)> {
    let path = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .pick_file()?;
    let bytes = std::fs::read(&path).ok()?;
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Image".to_string());
    Some((name, bytes))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn trigger_wasm_image_picker(
    _target_node: Option<u64>,
    _slot: std::rc::Rc<std::cell::RefCell<Option<(Option<u64>, String, Vec<u8>)>>>,
    _ctx: egui::Context,
) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_text_dialog() -> Option<(String, String)> {
    let path = rfd::FileDialog::new()
        .add_filter("Diagram", &["json", "drawio", "xml"])
        .pick_file()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "diagram".to_string());
    Some((name, content))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_ws_url(room: &str) -> String {
    format!("ws://127.0.0.1:6129/ws?room={room}")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn share_room_url(room: &str) -> String {
    format!("http://127.0.0.1:6129/?room={room}")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn generate_unique_room_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9e37_79b9_7f4a_7c15);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id() as u64;
    let mut x = nanos ^ (pid << 21) ^ seq.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;
    format!("r-{:08x}{:02x}", (x & 0xffff_ffff) as u32, ((x >> 32) & 0xff) as u8)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn initial_room_from_url() -> Option<String> {
    std::env::var("BOHAT_ROOM").ok().filter(|s| !s.is_empty())
}

#[cfg(target_arch = "wasm32")]
fn trigger_browser_download(filename: &str, mime: &str, bytes: &[u8]) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let u8_array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&u8_array.buffer());
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts)
        .map_err(|_| "blob failed")?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|_| "object url failed")?;
    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| "create anchor failed")?
        .dyn_into()
        .map_err(|_| "cast anchor failed")?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn copy_png_to_system_clipboard(png_bytes: &[u8]) -> Result<(), String> {
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png_bytes);
    let data_url = format!("data:image/png;base64,{b64}");
    let u8_array = js_sys::Uint8Array::from(png_bytes);
    let data_url_js = wasm_bindgen::JsValue::from_str(&data_url);
    let func = js_sys::Function::new_with_args(
        "bytes, dataUrl",
        "try {
            const html = '<img src=\"' + dataUrl + '\" alt=\"Bohat Draw Diagram\"/>';
            window.__bohatLastCopiedHtml = html;
            if (!window.__bohatCopyHooked) {
                window.__bohatCopyHooked = true;
                window.addEventListener('copy', function(e) {
                    if (window.__bohatLastCopiedHtml && e.clipboardData) {
                        e.clipboardData.setData('text/html', window.__bohatLastCopiedHtml);
                        e.preventDefault();
                    }
                }, true);
            }
            if (navigator.clipboard && window.ClipboardItem) {
                const blob = new Blob([bytes], { type: 'image/png' });
                const htmlBlob = new Blob([html], { type: 'text/html' });
                navigator.clipboard.write([
                    new ClipboardItem({ 'image/png': blob, 'text/html': htmlBlob })
                ]).catch(function() {
                    navigator.clipboard.write([
                        new ClipboardItem({ 'image/png': blob })
                    ]).catch(function() {});
                });
            } else {
                const holder = document.createElement('div');
                holder.contentEditable = 'true';
                holder.style.position = 'fixed';
                holder.style.left = '-9999px';
                holder.style.top = '0';
                holder.innerHTML = html;
                document.body.appendChild(holder);
                const range = document.createRange();
                range.selectNodeContents(holder);
                const sel = window.getSelection();
                if (sel) {
                    sel.removeAllRanges();
                    sel.addRange(range);
                    document.execCommand('copy');
                    sel.removeAllRanges();
                }
                document.body.removeChild(holder);
            }
            return true;
        } catch (_) {
            return false;
        }",
    );
    let _ = func.call2(&wasm_bindgen::JsValue::NULL, &u8_array, &data_url_js);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn copy_png_to_clipboard(png_bytes: &[u8]) -> Result<String, String> {
    let _ = copy_png_to_system_clipboard(png_bytes);
    Ok("Copied 2x PNG picture to clipboard (ready to paste into PowerPoint / Slides)".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn copy_animated_gif_or_save(gif_bytes: &[u8]) -> Result<String, String> {
    trigger_browser_download("bohat-draw-animated.gif", "image/gif", gif_bytes)?;
    Ok("Exported Animated GIF".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn save_bytes_dialog(default_name: &str, bytes: &[u8]) -> Result<String, String> {
    let mime = if default_name.ends_with(".svg") {
        "image/svg+xml"
    } else if default_name.ends_with(".png") {
        "image/png"
    } else if default_name.ends_with(".gif") {
        "image/gif"
    } else if default_name.ends_with(".drawio") || default_name.ends_with(".xml") {
        "application/xml"
    } else {
        "application/json"
    };
    trigger_browser_download(default_name, mime, bytes)?;
    Ok(format!("Downloaded {default_name}"))
}

#[cfg(target_arch = "wasm32")]
pub fn pick_image_file_sync() -> Option<(String, Vec<u8>)> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_wasm_image_picker(
    target_node: Option<u64>,
    slot: std::rc::Rc<std::cell::RefCell<Option<(Option<u64>, String, Vec<u8>)>>>,
    ctx: egui::Context,
) {
    use wasm_bindgen::{closure::Closure, JsCast};
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let Ok(el) = document.create_element("input") else { return };
    let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() else { return };
    input.set_type("file");
    input.set_accept("image/png,image/jpeg,image/gif,image/webp,image/*");
    let input_clone = input.clone();
    let onchange = Closure::<dyn FnMut()>::new(move || {
        let Some(files) = input_clone.files() else { return };
        let Some(file) = files.get(0) else { return };
        let name = file.name();
        let Ok(reader) = web_sys::FileReader::new() else { return };
        let reader_clone = reader.clone();
        let slot_clone = slot.clone();
        let ctx_clone = ctx.clone();
        let onload = Closure::<dyn FnMut()>::new(move || {
            if let Ok(res) = reader_clone.result() {
                let arr = js_sys::Uint8Array::new(&res);
                let mut bytes = vec![0u8; arr.length() as usize];
                arr.copy_to(&mut bytes);
                *slot_clone.borrow_mut() = Some((target_node, name.clone(), bytes));
                ctx_clone.request_repaint();
            }
        });
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        let _ = reader.read_as_array_buffer(&file);
    });
    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();
    input.click();
}

#[cfg(target_arch = "wasm32")]
pub fn open_text_dialog() -> Option<(String, String)> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn default_ws_url(room: &str) -> String {
    if let Some(win) = web_sys::window() {
        let loc = win.location();
        let proto = loc.protocol().unwrap_or_else(|_| "http:".into());
        let host = loc.host().unwrap_or_else(|_| "127.0.0.1:6129".into());
        let ws_proto = if proto == "https:" { "wss" } else { "ws" };
        return format!("{ws_proto}://{host}/ws?room={room}");
    }
    format!("ws://127.0.0.1:6129/ws?room={room}")
}

#[cfg(target_arch = "wasm32")]
pub fn share_room_url(room: &str) -> String {
    if let Some(win) = web_sys::window() {
        let loc = win.location();
        let proto = loc.protocol().unwrap_or_else(|_| "http:".into());
        let host = loc.host().unwrap_or_else(|_| "127.0.0.1:6129".into());
        return format!("{proto}//{host}/?room={room}");
    }
    format!("http://127.0.0.1:6129/?room={room}")
}

#[cfg(target_arch = "wasm32")]
pub fn generate_unique_room_id() -> String {
    let a = (js_sys::Math::random() * 4_294_967_295.0) as u32;
    let b = ((js_sys::Date::now() as u64) ^ ((js_sys::Math::random() * 65535.0) as u64)) & 0xffff;
    format!("r-{a:08x}{b:04x}")
}

#[cfg(target_arch = "wasm32")]
pub fn initial_room_from_url() -> Option<String> {
    let win = web_sys::window()?;
    let search = win.location().search().ok()?;
    for pair in search.trim_start_matches('?').split('&') {
        if let Some(val) = pair.strip_prefix("room=") {
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}
