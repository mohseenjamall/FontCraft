#![cfg_attr(not(test), windows_subsystem = "windows")]
mod app;
mod fonts;
mod samples;
mod scan;
mod store;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;

/// Minimum frames before a `--screenshot` picture; it also waits for the fonts to finish loading.
const SHOT_FRAMES: u32 = 120;
/// Give up waiting for fonts after this many frames and take the picture anyway.
const SHOT_MAX_FRAMES: u32 = 1500;

struct Shot {
    path: PathBuf,
    requested: bool,
    image: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
}

struct Runner {
    app: app::FontApp,
    shot: Option<Shot>,
}

impl eframe::App for Runner {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        let Some(shot) = &mut self.shot else { return };
        let ctx = ui.ctx().clone();
        ctx.request_repaint();
        let ready = self.app.frames >= SHOT_FRAMES && (self.app.settled() || self.app.frames >= SHOT_MAX_FRAMES);
        if ready && !shot.requested {
            shot.requested = true;
            let slot = shot.image.clone();
            ctx.request_screenshot(move |img| *slot.lock().unwrap() = Some(img));
        }
        if let Some(img) = shot.image.lock().unwrap().take() {
            if let Err(e) = save_png(&img, &shot.path) {
                eprintln!("screenshot failed: {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn save_png(img: &egui::ColorImage, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let [w, h] = img.size;
    let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
    let buf = image::RgbaImage::from_raw(w as u32, h as u32, bytes).ok_or("bad image size")?;
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    buf.save(path)?;
    Ok(())
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let shot_path = args.iter().position(|a| a == "--screenshot").and_then(|i| args.get(i + 1)).map(PathBuf::from);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("FontCraft")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([720.0, 480.0]);
    if shot_path.is_some() {
        // Don't steal focus from the person while a check runs.
        viewport = viewport.with_active(false);
        #[cfg(windows)]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }
    let options = eframe::NativeOptions { viewport, run_and_return: true, ..Default::default() };
    eframe::run_native(
        "FontCraft",
        options,
        Box::new(move |cc| {
            let shot = shot_path.map(|path| Shot { path, requested: false, image: Default::default() });
            Ok(Box::new(Runner { app: app::FontApp::new(&cc.egui_ctx), shot }))
        }),
    )
}
