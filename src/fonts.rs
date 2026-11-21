//! Loads font files into egui on demand, off the UI thread, so only the fonts on screen cost memory.
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::{self, FontData, FontFamily, FontInsert, InsertFontFamily, epaint::text::FontPriority};

use crate::scan::Face;

pub struct FontLoader {
    requests: Sender<(String, PathBuf, u32)>,
    loaded: Receiver<FontInsert>,
    requested: HashSet<String>,
    /// Added to egui last frame; usable from this frame on.
    arriving: Vec<String>,
    ready: HashSet<String>,
}

impl FontLoader {
    pub fn new() -> Self {
        let (requests, req_rx) = channel::<(String, PathBuf, u32)>();
        let (done_tx, loaded) = channel();
        std::thread::spawn(move || {
            // .ttc files hold several faces: read each file once.
            let mut files: HashMap<PathBuf, egui::epaint::text::Blob> = HashMap::new();
            for (key, path, index) in req_rx {
                let blob = match files.get(&path) {
                    Some(b) => b.clone(),
                    None => match std::fs::read(&path) {
                        Ok(bytes) => {
                            let b: egui::epaint::text::Blob = Arc::new(bytes);
                            files.insert(path.clone(), b.clone());
                            b
                        }
                        Err(_) => continue, // file vanished: the row keeps its placeholder
                    },
                };
                let family = InsertFontFamily { family: FontFamily::Name(key.as_str().into()), priority: FontPriority::Highest };
                let insert = FontInsert::new(&key, FontData::from_blob(blob, index), vec![family]);
                if done_tx.send(insert).is_err() {
                    return;
                }
            }
        });
        Self { requests, loaded, requested: HashSet::new(), arriving: Vec::new(), ready: HashSet::new() }
    }

    /// Call once at the start of every frame.
    pub fn pump(&mut self, ctx: &egui::Context) {
        self.ready.extend(self.arriving.drain(..));
        while let Ok(insert) = self.loaded.try_recv() {
            self.arriving.push(insert.name.clone());
            ctx.add_font(insert);
        }
        if !self.arriving.is_empty() || self.waiting() {
            ctx.request_repaint();
        }
    }

    /// The egui family for this face, or None while it is still loading (and asks for it).
    pub fn family(&mut self, face: &Face) -> Option<FontFamily> {
        let key = face.key();
        if self.ready.contains(&key) {
            return Some(FontFamily::Name(key.into()));
        }
        if self.requested.insert(key.clone()) {
            let _ = self.requests.send((key, face.path.clone(), face.index));
        }
        None
    }

    pub fn waiting(&self) -> bool {
        self.requested.len() > self.ready.len()
    }
}

/// Text with Arabic or Hebrew letters is drawn in this family (Segoe UI / Tahoma only). egui cuts a line into
/// runs by font and lays the runs out left to right, so if the spaces came from another font the words would swap places.
pub fn rtl_family() -> FontFamily {
    FontFamily::Name("rtl".into())
}

/// egui's built-in fonts have no Arabic or Hebrew; borrow them from Windows for any text that needs them.
pub fn add_system_fallback(ctx: &egui::Context) {
    let Some(win) = std::env::var_os("WINDIR") else { return };
    for name in ["segoeui.ttf", "tahoma.ttf"] {
        let Ok(bytes) = std::fs::read(PathBuf::from(&win).join("Fonts").join(name)) else { continue };
        let families = vec![
            InsertFontFamily { family: FontFamily::Proportional, priority: FontPriority::Lowest },
            InsertFontFamily { family: rtl_family(), priority: FontPriority::Highest },
        ];
        ctx.add_font(FontInsert::new(name, FontData::from_blob(Arc::new(bytes), 0), families));
    }
}
