mod app;
mod canvas;
mod collab;
mod inventory;
mod panels;
mod platform;
mod theme;

use app::BohatDrawApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--server") {
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(6129);
        let static_dir = std::env::var("BOHAT_STATIC_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let exe_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let candidate1 = exe_dir.join("dist-wasm");
                let candidate2 = exe_dir.join("../libexec/dist-wasm");
                if candidate1.exists() {
                    candidate1
                } else if candidate2.exists() {
                    candidate2
                } else {
                    std::path::PathBuf::from("apps/studio/dist")
                }
            });

        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        if let Err(err) = rt.block_on(rustflow_server::run_server(rustflow_server::ServerConfig {
            host: "0.0.0.0".to_string(),
            port,
            static_dir,
        })) {
            eprintln!("server error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Bohat Draw.io — 100% Rust")
            .with_inner_size([1360.0, 840.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "Bohat Draw.io",
        options,
        Box::new(|cc| Ok(Box::new(BohatDrawApp::new(cc)))),
    ) {
        eprintln!("desktop app error: {err}");
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id("bohat_canvas")
            .expect("missing #bohat_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element is not HtmlCanvasElement");
        let _ = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(BohatDrawApp::new(cc)))),
            )
            .await;
    });
}
