//! The window: dark header with preview text and size, dark sidebar with font sources,
//! light list of font families previewed in their own typeface, and a detail panel.
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::{self, Color32, FontId, RichText};

use crate::fonts::FontLoader;
use crate::scan::{self, Face};

// Theme colours and slider limits read from FontBase's style.css / app.js (see MECHANICS.md).
const HEADER_BG: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const SIDE_BG: Color32 = Color32::from_rgb(0x18, 0x18, 0x18);
const MAIN_BG: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xFA);
const TEXT_LIGHT: Color32 = Color32::from_rgb(0xE7, 0xE7, 0xE7);
const TEXT_MAIN: Color32 = Color32::from_rgb(0x22, 0x22, 0x22);
const ACTION: Color32 = Color32::from_rgb(0x52, 0x00, 0xEA);
const SIZE_MIN: f32 = 6.0;
const SIZE_MAX: f32 = 180.0;
const SIZE_STEP: f64 = 2.0;
const SIZE_DEFAULT: f32 = 42.0;
const WATERFALL: [f32; 4] = [72.0, 48.0, 24.0, 14.0];

pub struct Family {
    pub name: String,
    pub faces: Vec<Face>,
}

impl Family {
    /// The face shown in the list: closest to Regular (400, upright).
    fn main_face(&self) -> &Face {
        self.faces
            .iter()
            .min_by_key(|f| (f.italic, (f.weight as i32 - 400).abs()))
            .expect("a family always has a face")
    }
}

pub struct FontApp {
    sources: Vec<(String, PathBuf)>,
    /// Built-in sources before the person's own folders (not saved).
    fixed_sources: usize,
    scan: Scans,
    scan_done: bool,
    faces: Vec<Face>,
    families: Vec<Family>,
    loader: FontLoader,
    text: String,
    size: f32,
    search: String,
    source: Option<usize>,
    selected: Option<String>,
    selected_face: usize,
    pub frames: u32,
}

impl FontApp {
    pub fn new(ctx: &egui::Context) -> Self {
        ctx.set_visuals(egui::Visuals::light());
        let mut sources = scan::system_folders();
        let activated = sources.len();
        // Fonts FontBase activated from other folders (scanned file by file).
        sources.push(("Activated by FontBase".into(), PathBuf::new()));
        sources.extend(crate::store::load_folders().into_iter().map(|p| (folder_label(&p), p)));
        let mut jobs: Vec<(usize, PathBuf)> =
            sources.iter().cloned().enumerate().filter(|(i, _)| *i != activated).map(|(i, (_, p))| (i, p)).collect();
        jobs.extend(scan::fontbase_activated().into_iter().map(|p| (activated, p)));
        let mut scan = Scans::new();
        scan.start(jobs);
        Self {
            fixed_sources: activated + 1,
            sources,
            scan,
            scan_done: false,
            faces: Vec::new(),
            families: Vec::new(),
            loader: FontLoader::new(),
            text: String::new(),
            size: SIZE_DEFAULT,
            search: String::new(),
            source: None,
            selected: None,
            selected_face: 0,
            frames: 0,
        }
    }

    /// True once every font is listed and every visible preview has its typeface.
    pub fn settled(&self) -> bool {
        self.scan_done && !self.loader.waiting()
    }

    /// The person's text, or the font's own writing-system sample when they typed nothing.
    fn sample(&self, face: &Face) -> String {
        if self.text.trim().is_empty() {
            crate::samples::SCRIPTS[face.script as usize].1.to_owned()
        } else {
            self.text.clone()
        }
    }

    fn receive_faces(&mut self) {
        let (new, done) = self.scan.poll();
        self.scan_done = done;
        if !new.is_empty() {
            self.faces.extend(new);
            self.families = group(&self.faces);
        }
    }

    fn add_folder(&mut self, dir: PathBuf) {
        if self.sources.iter().any(|(_, p)| *p == dir) {
            return;
        }
        let index = self.sources.len();
        self.sources.push((folder_label(&dir), dir.clone()));
        crate::store::save_folders(&self.sources.iter().skip(self.fixed_sources).map(|(_, p)| p.clone()).collect::<Vec<_>>());
        self.scan.start(vec![(index, dir)]);
        self.scan_done = false;
        self.source = Some(index);
    }

    fn visible(&self) -> Vec<usize> {
        let needle = self.search.to_lowercase();
        (0..self.families.len())
            .filter(|&i| {
                let f = &self.families[i];
                f.name.to_lowercase().contains(&needle)
                    && self.source.is_none_or(|s| f.faces.iter().any(|x| x.source == s))
            })
            .collect()
    }
}

/// Dark controls for the header and sidebar, with FontBase's accent for selection.
fn dark_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(TEXT_LIGHT);
    v.extreme_bg_color = Color32::from_rgb(0x21, 0x21, 0x21);
    v.selection.bg_fill = ACTION;
    v.selection.stroke.color = Color32::WHITE;
    v
}

fn folder_label(p: &std::path::Path) -> String {
    p.file_name().map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into_owned())
}

/// Background scans feeding one stream of faces; `pending` counts scans still running.
struct Scans {
    tx: Sender<Face>,
    rx: Receiver<Face>,
    done_tx: Sender<()>,
    done_rx: Receiver<()>,
    pending: usize,
}

impl Scans {
    fn new() -> Self {
        let (tx, rx) = channel();
        let (done_tx, done_rx) = channel();
        Self { tx, rx, done_tx, done_rx, pending: 0 }
    }

    fn start(&mut self, folders: Vec<(usize, PathBuf)>) {
        self.pending += 1;
        let (tx, done) = (self.tx.clone(), self.done_tx.clone());
        std::thread::spawn(move || {
            scan::scan(folders, tx);
            let _ = done.send(());
        });
    }

    /// New faces since last call, and whether every scan has finished.
    fn poll(&mut self) -> (Vec<Face>, bool) {
        // Read "done" first: a finished scan's faces are then already queued.
        self.pending -= self.done_rx.try_iter().count();
        let faces: Vec<Face> = self.rx.try_iter().collect();
        (faces, self.pending == 0)
    }
}

/// Groups faces into families, sorted by name; styles ordered upright-first by weight.
pub fn group(faces: &[Face]) -> Vec<Family> {
    let mut map: std::collections::BTreeMap<String, Family> = Default::default();
    for f in faces {
        map.entry(f.family.to_lowercase())
            .or_insert_with(|| Family { name: f.family.clone(), faces: Vec::new() })
            .faces
            .push(f.clone());
    }
    let mut out: Vec<Family> = map.into_values().collect();
    for fam in &mut out {
        fam.faces.sort_by_key(|f| (f.weight, f.italic));
        fam.faces.dedup_by(|a, b| a.style == b.style && a.weight == b.weight && a.italic == b.italic);
    }
    out
}

impl eframe::App for FontApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frames += 1;
        self.loader.pump(&ctx);
        self.receive_faces();
        if !self.scan_done {
            ctx.request_repaint();
        }
        self.header(ui);
        self.sidebar(ui);
        self.details(ui);
        self.list(ui);
    }
}

impl FontApp {
    fn header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("header")
            .frame(egui::Frame::NONE.fill(HEADER_BG).inner_margin(egui::Margin::symmetric(16, 10)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    *ui.visuals_mut() = dark_visuals();
                    ui.add(
                        egui::TextEdit::singleline(&mut self.text)
                            .hint_text("Type something to preview…")
                            .desired_width(ui.available_width() * 0.5),
                    );
                    ui.add_space(12.0);
                    ui.label("Size");
                    ui.add(
                        egui::Slider::new(&mut self.size, SIZE_MIN..=SIZE_MAX)
                            .step_by(SIZE_STEP)
                            .suffix(" px"),
                    );
                    ui.add_space(12.0);
                    ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search fonts").desired_width(180.0));
                });
            });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("sources")
            .exact_size(220.0)
            .frame(egui::Frame::NONE.fill(SIDE_BG).inner_margin(egui::Margin::symmetric(12, 14)))
            .show(ui, |ui| {
                *ui.visuals_mut() = dark_visuals();
                ui.label(RichText::new("FONTS").size(11.0).color(Color32::GRAY));
                ui.add_space(6.0);
                let all = format!("All fonts  {}", self.families.len());
                if ui.selectable_label(self.source.is_none(), all).clicked() {
                    self.source = None;
                }
                for i in 0..self.sources.len() {
                    let count = self.families.iter().filter(|f| f.faces.iter().any(|x| x.source == i)).count();
                    let label = format!("{}  {}", self.sources[i].0, count);
                    if ui.selectable_label(self.source == Some(i), label).clicked() {
                        self.source = Some(i);
                    }
                }
                ui.add_space(12.0);
                if ui.add(egui::Button::new(RichText::new("+ Add folder").color(Color32::WHITE)).fill(ACTION)).clicked()
                    && let Some(dir) = rfd::FileDialog::new().pick_folder()
                {
                    self.add_folder(dir);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    let status = if self.scan_done {
                        format!("{} styles", self.faces.len())
                    } else {
                        format!("Scanning… {} styles", self.faces.len())
                    };
                    ui.label(RichText::new(status).size(11.0).color(Color32::GRAY));
                });
            });
    }

    fn list(&mut self, ui: &mut egui::Ui) {
        let visible = self.visible();
        let row_h = self.size * 1.35 + 34.0;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(MAIN_BG).inner_margin(egui::Margin::symmetric(20, 8)))
            .show(ui, |ui| {
                if visible.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(if self.scan_done { "No fonts match." } else { "Looking for fonts…" });
                    });
                    return;
                }
                egui::ScrollArea::vertical().auto_shrink(false).show_rows(ui, row_h, visible.len(), |ui, range| {
                    for &i in &visible[range] {
                        self.row(ui, i, row_h);
                    }
                });
            });
    }

    fn row(&mut self, ui: &mut egui::Ui, i: usize, row_h: f32) {
        let fam = &self.families[i];
        let face = fam.main_face().clone();
        let selected = self.selected.as_deref() == Some(fam.name.as_str());
        let (name, styles) = (fam.name.clone(), fam.faces.len());
        let family = self.loader.family(&face);
        let sample = self.sample(&face);
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_h), egui::Sense::click());
        let painter = ui.painter_at(rect);
        if selected || resp.hovered() {
            painter.rect_filled(rect, 6.0, if selected { Color32::from_rgb(0xEC, 0xE6, 0xFD) } else { Color32::from_gray(0xF0) });
        }
        let meta = format!("{name}   ·   {styles} style{}", if styles == 1 { "" } else { "s" });
        painter.text(rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, meta, FontId::proportional(12.0), Color32::from_gray(0x80));
        let pos = rect.left_top() + egui::vec2(8.0, 28.0);
        match family {
            Some(f) => {
                painter.text(pos, egui::Align2::LEFT_TOP, sample, FontId::new(self.size, f), TEXT_MAIN);
            }
            None => {
                painter.text(pos, egui::Align2::LEFT_TOP, sample, FontId::proportional(self.size), Color32::from_gray(0xD0));
            }
        }
        if resp.clicked() {
            self.selected = if selected { None } else { Some(name) };
            self.selected_face = 0;
        }
    }

    fn details(&mut self, ui: &mut egui::Ui) {
        let Some(name) = self.selected.clone() else { return };
        let Some(fam) = self.families.iter().find(|f| f.name == name) else { return };
        let faces = fam.faces.clone();
        egui::Panel::right("details")
            .default_size(420.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(Color32::WHITE).inner_margin(egui::Margin::same(16)))
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(TEXT_MAIN);
                ui.horizontal(|ui| {
                    ui.heading(&name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            self.selected = None;
                        }
                    });
                });
                ui.label(RichText::new(faces[0].path.display().to_string()).size(11.0).color(Color32::GRAY));
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    for (k, face) in faces.iter().enumerate() {
                        let fam = self.loader.family(face);
                        let label = RichText::new(&face.style).size(22.0);
                        let label = match fam { Some(f) => label.family(f), None => label.color(Color32::LIGHT_GRAY) };
                        if ui.selectable_label(self.selected_face == k, label).clicked() {
                            self.selected_face = k;
                        }
                    }
                    ui.add_space(16.0);
                    ui.label(RichText::new("WATERFALL").size(11.0).color(Color32::GRAY));
                    let face = &faces[self.selected_face.min(faces.len() - 1)];
                    let sample = self.sample(face);
                    if let Some(f) = self.loader.family(face) {
                        for size in WATERFALL {
                            ui.label(RichText::new(format!("{size} px")).size(11.0).color(Color32::GRAY));
                            ui.add(egui::Label::new(RichText::new(&sample).font(FontId::new(size, f.clone()))).wrap());
                        }
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(family: &str, style: &str, weight: u16, italic: bool) -> Face {
        Face { path: PathBuf::from(format!("{family}{style}.ttf")), index: 0, family: family.into(), style: style.into(), weight, italic, script: 0, source: 0 }
    }

    #[test]
    fn groups_case_insensitively_and_picks_regular() {
        let fams = group(&[face("Inter", "Bold", 700, false), face("inter", "Regular", 400, false), face("Inter", "Italic", 400, true), face("Abc", "Regular", 400, false)]);
        assert_eq!(fams.len(), 2);
        assert_eq!(fams[0].name, "Abc");
        assert_eq!(fams[1].faces.len(), 3);
        assert_eq!(fams[1].main_face().style, "Regular");
    }
}
