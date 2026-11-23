//! The window: dark header with preview text and size, dark sidebar with font sources,
//! light list of font families previewed in their own typeface, and a detail panel.
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::{self, Color32, FontId, RichText};

use crate::activate;
use crate::collections::Collection;
use crate::fonts::FontLoader;
use crate::grid;
use crate::scan::{self, Face};
use crate::theme::{self, color};
use crate::widgets;

// Slider limits read from FontBase's style.css / app.js (see MECHANICS.md).
const SIZE_MIN: f32 = 6.0;
const SIZE_MAX: f32 = 180.0;
const SIZE_STEP: f64 = 2.0;
const SIZE_DEFAULT: f32 = 42.0;
const WATERFALL: [f32; 4] = [72.0, 48.0, 24.0, 14.0];

pub struct Family {
    pub name: String,
    pub faces: Vec<Face>,
    /// Scripts any face supports, whether any face is variable, and how many different weights it has.
    pub scripts: u16,
    pub variable: bool,
    pub weight_count: usize,
}

impl Family {
    pub fn new(name: String, faces: Vec<Face>) -> Self {
        let mut weights: Vec<u16> = faces.iter().map(|f| f.weight).collect();
        weights.sort_unstable();
        weights.dedup();
        Self {
            name,
            scripts: faces.iter().fold(0, |m, f| m | f.scripts),
            variable: faces.iter().any(|f| f.variable),
            weight_count: weights.len(),
            faces,
        }
    }

    /// The face shown in the list: closest to Regular (400, upright).
    fn main_face(&self) -> &Face {
        self.faces
            .iter()
            .min_by_key(|f| (f.italic, (f.weight as i32 - 400).abs()))
            .expect("a family always has a face")
    }

    /// The distinct font files behind this family, sorted. A .ttc can hold other families too.
    fn files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = self.faces.iter().map(|f| f.path.clone()).collect();
        files.sort();
        files.dedup();
        files
    }
}

/// On for every app only when every one of its files is on (`active` holds `activate::norm` keys).
fn all_active(active: &HashSet<PathBuf>, disabled: &HashSet<PathBuf>, fam: &Family) -> bool {
    let user_fonts = if cfg!(unix) { crate::platform::user_fonts_dir().unwrap_or_default() } else { PathBuf::new() };
    all_active_in(active, disabled, &user_fonts, Path::new(&activate::key()), fam)
}

/// `all_active` with the folders given: a file put straight in `user_fonts` (outside `links`) is
/// on by default until turned off, because the system loads it without any link. An empty
/// `user_fonts` turns that off (Windows keeps its fonts in the registry instead).
fn all_active_in(active: &HashSet<PathBuf>, disabled: &HashSet<PathBuf>, user_fonts: &Path, links: &Path, fam: &Family) -> bool {
    fam.faces.iter().all(|f| {
        let key = activate::norm(&f.path);
        active.contains(&key)
            || (!user_fonts.as_os_str().is_empty() && f.path.starts_with(user_fonts) && !f.path.starts_with(links) && !disabled.contains(&key))
    })
}

/// Files FontBase turned on: from its registry values, and from values FontCraft saved when
/// turning them off (`norm` keys).
fn fontbase_files(store: &std::path::Path) -> HashSet<PathBuf> {
    let saved = crate::disabled::Saved::load(store);
    let turned_off = saved.entries.iter().filter(|e| e.name.contains("(FontBase)")).filter_map(|e| e.path());
    // On macOS/Linux that list is FontCraft's own links, not FontBase's.
    let theirs = if cfg!(windows) { scan::fontbase_activated() } else { Vec::new() };
    theirs.into_iter().chain(turned_off).map(|p| activate::norm(&p)).collect()
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
    /// A Library or Collections filter; exclusive with `source`.
    collection: Option<Collection>,
    /// Families per [`Collection::ALL`] entry, refreshed when families or the on/off state change.
    counts: [usize; 5],
    /// Whether the families show as a list or as a grid of cards.
    view: grid::View,
    selected: Option<String>,
    selected_face: usize,
    /// Files on for this user through the registry (installed, FontCraft or FontBase), `norm` keys.
    active: HashSet<PathBuf>,
    /// Files whose registry values FontCraft saved when turning them off, `norm` keys.
    disabled: HashSet<PathBuf>,
    /// Files FontBase turned on (now or before FontCraft turned them off), `norm` keys.
    fontbase_files: HashSet<PathBuf>,
    /// The saved list of turned-off values.
    store: PathBuf,
    /// Last activation error, shown under the toggle until clicked away.
    notice: Option<String>,
    /// A newer release, once the background check finds one; shown as a banner that stays.
    update: Option<crate::update::Update>,
    updates: Option<Receiver<crate::update::Update>>,
    /// Why the download page couldn't be opened, shown in the banner.
    update_error: Option<String>,
    /// Set while "Update now" downloads and swaps the program.
    installing: Option<Receiver<Result<(), String>>>,
    /// The support-card picture, uploaded once (None in tests).
    art: Option<egui::TextureHandle>,
    pub frames: u32,
}

impl FontApp {
    pub fn new(ctx: &egui::Context) -> Self {
        crate::theme::apply(ctx);
        crate::fonts::add_system_fallback(ctx);
        let mut sources = scan::system_folders();
        let activated = sources.len();
        // Fonts FontBase activated from other folders (scanned file by file).
        let label = if cfg!(windows) { "From FontBase" } else { "Turned on by FontCraft" };
        sources.push((label.into(), PathBuf::new()));
        sources.extend(crate::store::load_folders().into_iter().map(|p| (folder_label(&p), p)));
        let mut jobs: Vec<(usize, PathBuf)> =
            sources.iter().cloned().enumerate().filter(|(i, _)| *i != activated).map(|(i, (_, p))| (i, p)).collect();
        let store = crate::store::disabled_list();
        // FontBase's files and every turned-off file, so a font stays listed (and can be turned
        // back on) even when it lives outside the scanned folders.
        let saved = crate::disabled::Saved::load(&store).entries.into_iter().filter_map(|e| e.path());
        jobs.extend(scan::fontbase_activated().into_iter().chain(saved).filter(|p| p.is_file()).map(|p| (activated, p)));
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
            collection: None,
            counts: [0; 5],
            view: crate::store::load_view(),
            selected: None,
            selected_face: 0,
            active: activate::read_active(&activate::key()),
            disabled: activate::read_disabled(&store),
            fontbase_files: fontbase_files(&store),
            store,
            notice: None,
            update: None,
            updates: None,
            update_error: None,
            installing: None,
            art: Some(ctx.load_texture("app-art", crate::support::art_image(), egui::TextureOptions::LINEAR)),
            frames: 0,
        }
    }

    /// Starts the once-per-start check for a newer release (in the background).
    pub fn check_updates(&mut self, ctx: &egui::Context) {
        self.updates = Some(crate::update::spawn_check(ctx.clone()));
    }


    /// Shows one family: its details panel open and the list searched down to it.
    pub fn open_family(&mut self, name: String) {
        self.search = name.clone();
        self.selected = Some(name);
    }

    /// Why this family can't be turned on or off (its first locked file), or `None`.
    fn lock(fam: &Family) -> Option<&'static str> {
        fam.faces.iter().find_map(|f| activate::lock_reason(&f.path))
    }

    fn can_toggle(&self, fam: &Family) -> bool {
        Self::lock(fam).is_none()
    }

    fn toggle(&mut self, name: &str) {
        let store = self.store.clone();
        self.toggle_in(name, &activate::key(), &store);
    }

    /// Turns the family's files off when all are on, otherwise on, under `key` with the saved
    /// list `store`, then re-reads what's on and off.
    fn toggle_in(&mut self, name: &str, key: &str, store: &std::path::Path) {
        let Some(fam) = self.families.iter().find(|f| f.name == name) else { return };
        if !self.can_toggle(fam) {
            return;
        }
        let files = fam.files();
        let result = if all_active(&self.active, &self.disabled, fam) {
            activate::turn_off_in(key, store, &files)
        } else {
            activate::turn_on_in(key, store, &files)
        };
        // Re-read even on error: a batch can fail after some files already changed.
        self.active = activate::read_active(key);
        self.disabled = activate::read_disabled(store);
        self.notice = result.err();
        self.refresh_counts();
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    /// Shows the families as a list or as a grid of cards.
    pub fn set_view(&mut self, v: grid::View) {
        self.view = v;
    }

    /// Picks a Library or Collections filter (or clears it); it replaces any chosen folder.
    pub fn set_collection(&mut self, c: Option<Collection>) {
        self.collection = c;
        if c.is_some() {
            self.source = None;
        }
    }

    /// Picks a folder source (or clears it); it replaces any chosen collection.
    pub fn set_source(&mut self, s: Option<usize>) {
        self.source = s;
        if s.is_some() {
            self.collection = None;
        }
    }

    /// Families per collection, as of the last scan result or on/off change.
    pub fn collection_counts(&self) -> [usize; 5] {
        self.counts
    }

    fn refresh_counts(&mut self) {
        let mut counts = [0; 5];
        for fam in &self.families {
            let on = all_active(&self.active, &self.disabled, fam);
            for c in Collection::ALL {
                counts[c.index()] += usize::from(c.contains(fam, on));
            }
        }
        self.counts = counts;
    }

    /// Scripts the typed preview text needs a font to support (list view only; the grid ignores the text).
    pub fn text_need(&self) -> u16 {
        if self.view == grid::View::List { crate::samples::scripts_in(&self.text) } else { 0 }
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
            self.refresh_counts();
        }
    }

    /// Forgets an added folder: the saved list, its fonts and its row. Files on disk and the on/off
    /// state of fonts are never touched. Built-in sources can't be removed.
    fn remove_folder(&mut self, source: usize) {
        if self.drop_source(source) {
            crate::store::save_folders(&self.sources.iter().skip(self.fixed_sources).map(|(_, p)| p.clone()).collect::<Vec<_>>());
        }
    }

    /// The in-memory part of [`FontApp::remove_folder`]: drops the source and its faces, shifts later
    /// sources down by one and repairs the chosen folder and the selection.
    fn drop_source(&mut self, source: usize) -> bool {
        if source < self.fixed_sources || source >= self.sources.len() {
            return false;
        }
        self.sources.remove(source);
        self.faces.retain(|f| f.source != source);
        for f in &mut self.faces {
            if f.source > source {
                f.source -= 1;
            }
        }
        self.source = match self.source {
            Some(s) if s == source => None,
            Some(s) if s > source => Some(s - 1),
            other => other,
        };
        self.families = group(&self.faces);
        self.refresh_counts();
        if self.selected.as_ref().is_some_and(|name| !self.families.iter().any(|f| &f.name == name)) {
            self.selected = None;
            self.selected_face = 0;
        }
        true
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
        self.set_source(Some(index));
    }

    fn visible(&self) -> Vec<usize> {
        let needle = self.search.to_lowercase();
        let need = self.text_need();
        (0..self.families.len())
            .filter(|&i| {
                let f = &self.families[i];
                f.name.to_lowercase().contains(&needle)
                    && self.source.is_none_or(|s| f.faces.iter().any(|x| x.source == s))
                    && self.collection.is_none_or(|c| {
                        c.contains(f, c == Collection::TurnedOn && all_active(&self.active, &self.disabled, f))
                    })
                    && f.scripts & need == need
            })
            .collect()
    }
}

/// How to draw a sample: in the face itself, or dimmed in the fallback font when the face lacks a letter.
struct Drawn {
    font: FontId,
    color: Color32,
    has_all: bool,
}

fn pick_font(ctx: &egui::Context, family: egui::FontFamily, sample: &str, size: f32, ink: Color32) -> Drawn {
    let own = FontId::new(size, family);
    if ctx.fonts_mut(|f| f.has_glyphs(&own, sample)) {
        Drawn { font: own, color: ink, has_all: true }
    } else {
        Drawn { font: FontId::proportional(size), color: color::INK_MUTED, has_all: false }
    }
}

fn folder_label(p: &std::path::Path) -> String {
    p.file_name().map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into_owned())
}

/// A family name laid out to at most two lines inside a card width, cut with an ellipsis when
/// it would need more.
fn name_galley(ctx: &egui::Context, name: &str) -> std::sync::Arc<egui::Galley> {
    let font = FontId::proportional(theme::size::SECONDARY);
    let layout = |s: String| ctx.fonts_mut(|f| f.layout(s, font.clone(), color::INK_DIM, grid::CARD_SIZE));
    let galley = layout(name.to_owned());
    if galley.rows.len() <= 2 {
        return galley;
    }
    let chars: Vec<char> = name.chars().collect();
    for cut in (0..chars.len()).rev() {
        let mut s: String = chars[..cut].iter().collect();
        s.push('…');
        let short = layout(s);
        if short.rows.len() <= 2 {
            return short;
        }
    }
    layout("…".to_owned())
}

/// A sidebar row: label left, count right in mono, 32 high with a quiet selection.
fn source_row(ui: &mut egui::Ui, label: &str, count: usize, selected: bool) -> egui::Response {
    sidebar_row(ui, label, count, selected, false).0
}

/// A sidebar row; when `removable`, hovering it swaps the count for a remove control.
/// Returns the row's response and whether the remove control was used (click, or Delete while focused).
fn sidebar_row(ui: &mut egui::Ui, label: &str, count: usize, selected: bool, removable: bool) -> (egui::Response, bool) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), theme::ROW_HEIGHT), egui::Sense::click());
    let over = removable && ui.rect_contains_pointer(rect);
    let painter = ui.painter();
    if selected {
        painter.rect(
            rect,
            theme::radius::CONTROL,
            color::CHROME_RAISED,
            egui::Stroke::new(1.0, color::CHROME_LINE),
            egui::StrokeKind::Inside,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, theme::radius::CONTROL, color::CHROME_HOVER);
    }
    let text_color = if selected { color::CHROME_TEXT } else { color::CHROME_TEXT_DIM };
    // shortcut: a long label is clipped before the count, no ellipsis; upgrade if names get truncated often.
    let label_area = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right() - 40.0, rect.max.y));
    painter.with_clip_rect(label_area).text(
        egui::pos2(rect.left() + theme::space::S, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(theme::size::BODY),
        text_color,
    );
    let mut removed = removable && response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Delete));
    if over {
        let target = egui::Rect::from_center_size(egui::pos2(rect.right() - 16.0, rect.center().y), egui::vec2(24.0, 24.0));
        let button = ui.interact(target, response.id.with("remove"), egui::Sense::click());
        let tint = if button.hovered() { color::CHROME_TEXT } else { color::CHROME_TEXT_MUTED };
        ui.painter().text(target.center(), egui::Align2::CENTER_CENTER, "×", egui::FontId::proportional(theme::size::TITLE), tint);
        removed |= button.on_hover_text("Remove folder").clicked();
    } else {
        ui.painter().text(
            egui::pos2(rect.right() - theme::space::S, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            count.to_string(),
            egui::FontId::new(theme::size::META, theme::mono()),
            color::CHROME_TEXT_MUTED,
        );
    }
    (response.on_hover_cursor(egui::CursorIcon::PointingHand), removed)
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
    let mut map: std::collections::BTreeMap<String, (String, Vec<Face>)> = Default::default();
    for f in faces {
        map.entry(f.family.to_lowercase()).or_insert_with(|| (f.family.clone(), Vec::new())).1.push(f.clone());
    }
    map.into_values()
        .map(|(name, mut faces)| {
            faces.sort_by_key(|f| (f.weight, f.italic));
            faces.dedup_by(|a, b| a.style == b.style && a.weight == b.weight && a.italic == b.italic);
            Family::new(name, faces)
        })
        .collect()
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
        if let Some(u) = self.updates.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.update = Some(u);
        }
        self.update_banner(ui);
        self.header(ui);
        self.sidebar(ui);
        self.details(ui);
        self.list(ui);
    }
}

impl FontApp {
    /// Topmost bar while a newer release exists; it can't be closed, only acted on.
    fn update_banner(&mut self, ui: &mut egui::Ui) {
        let Some(update) = self.update.clone() else { return };
        let (text, url) = (format!("Update required: FontCraft {} is available. Please update now.", update.version), update.url.clone());
        let ctx = ui.ctx().clone();
        if let Some(done) = self.installing.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.installing = None;
            match done.and_then(|()| crate::update::restart()) {
                Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                Err(e) => self.update_error = Some(e),
            }
        }
        egui::Panel::top("update")
            .frame(egui::Frame::NONE.fill(color::COBALT).inner_margin(egui::Margin::symmetric(16, 8)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new(text).color(Color32::WHITE));
                    if self.installing.is_some() {
                        ui.spinner();
                        ui.label(RichText::new("Downloading the update…").color(Color32::WHITE));
                    } else {
                        let in_place = crate::update::installs_in_place();
                        let label = if in_place { "Update now" } else { "Download update" };
                        let button = egui::Button::new(
                            RichText::new(label)
                                .font(FontId::new(theme::size::BODY, theme::medium()))
                                .color(color::COBALT),
                        )
                        .fill(Color32::WHITE)
                        .corner_radius(theme::radius::CONTROL)
                        .min_size(egui::vec2(0.0, 28.0));
                        if ui.add(button).clicked() {
                            self.update_error = None;
                            if in_place {
                                self.installing = Some(crate::update::spawn_install(update.clone(), ctx.clone()));
                            } else {
                                // macOS: the signed .app can't swap its own program; the user opens the new .dmg.
                                self.update_error = crate::update::open(&crate::update::download_url(&update)).err();
                            }
                        }
                        let page = egui::Button::new(RichText::new("Release page").color(Color32::WHITE))
                            .fill(Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, Color32::WHITE))
                            .corner_radius(theme::radius::CONTROL)
                            .min_size(egui::vec2(0.0, 28.0));
                        if ui.add(page).clicked() {
                            self.update_error = crate::update::open(&url).err();
                        }
                    }
                    if let Some(e) = &self.update_error {
                        ui.label(RichText::new(e).color(Color32::WHITE));
                    }
                });
            });
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("header")
            .exact_size(theme::HEADER_HEIGHT)
            .frame(egui::Frame::NONE.fill(color::HEADER).inner_margin(egui::Margin::symmetric(16, 8)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    *ui.visuals_mut() = theme::chrome_visuals();
                    let in_list = self.view == grid::View::List;
                    // Narrow windows shrink the controls and drop the "Size" label so nothing is pushed out of the header.
                    let avail = ui.available_width();
                    let (slider_w, search_w, show_label) = if avail < 800.0 {
                        (64.0, 120.0, false)
                    } else if avail < 900.0 {
                        (64.0, 120.0, true)
                    } else {
                        (100.0, 180.0, true)
                    };
                    let gaps = 4.0 * theme::space::M;
                    let label_w = if show_label { 40.0 } else { 0.0 };
                    let width = (avail - label_w - slider_w - 70.0 - 100.0 - search_w - gaps).max(100.0);
                    // The preview text and the size only drive the list, so the grid hides them.
                    if in_list {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.text)
                                .hint_text("Type something to preview…")
                                .desired_width(width)
                                .min_size(egui::vec2(0.0, 28.0))
                                .align(egui::Align2::LEFT_CENTER)
                                .background_color(color::CHROME_RAISED),
                        );
                        ui.add_space(theme::space::M);
                        ui.scope(|ui| {
                            ui.spacing_mut().slider_width = slider_w;
                            if show_label {
                                ui.label(RichText::new("Size").color(color::CHROME_TEXT_DIM));
                            }
                            widgets::ruler_slider(ui, &mut self.size, SIZE_MIN..=SIZE_MAX, SIZE_STEP);
                        });
                        ui.add_space(theme::space::M);
                    }
                    if let Some(i) = widgets::segmented(ui, &["List", "Grid"], if in_list { 0 } else { 1 }) {
                        let view = if i == 1 { grid::View::Grid } else { grid::View::List };
                        self.set_view(view);
                        crate::store::save_view(view);
                    }
                    ui.add_space(theme::space::M);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Search fonts")
                            .desired_width(search_w)
                            .min_size(egui::vec2(0.0, 28.0))
                            .align(egui::Align2::LEFT_CENTER)
                            .background_color(color::CHROME_RAISED),
                    );
                });
            });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let counts = self.collection_counts();
        egui::Panel::left("sources")
            .exact_size(if ui.available_width() < 800.0 { 200.0 } else { 220.0 })
            .frame(egui::Frame::NONE.fill(color::SIDEBAR).inner_margin(egui::Margin::same(16)))
            .show(ui, |ui| {
                *ui.visuals_mut() = theme::chrome_visuals();
                // The three groups scroll; the footer below them (counts and the support card) stays in view.
                let footer = 104.0;
                egui::ScrollArea::vertical()
                    .id_salt("sidebar_groups")
                    .auto_shrink([false, false])
                    .max_height((ui.available_height() - footer).max(60.0))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        widgets::section_label(ui, "Library");
                        ui.add_space(theme::space::S);
                        let all = self.source.is_none() && self.collection.is_none();
                        if source_row(ui, "All fonts", self.families.len(), all).clicked() {
                            self.set_source(None);
                            self.set_collection(None);
                        }
                        self.collection_row(ui, Collection::TurnedOn, counts);
                        ui.add_space(theme::space::M);
                        widgets::section_label(ui, "Collections");
                        ui.add_space(theme::space::S);
                        for c in &Collection::ALL[1..] {
                            self.collection_row(ui, *c, counts);
                        }
                        ui.add_space(theme::space::M);
                        widgets::section_label(ui, "Folders");
                        ui.add_space(theme::space::S);
                        let mut remove = None;
                        for i in 0..self.sources.len() {
                            let count = self.families.iter().filter(|f| f.faces.iter().any(|x| x.source == i)).count();
                            // Added folders can be removed once the scan is over (the scan indexes folders by position).
                            let removable = i >= self.fixed_sources && self.scan_done;
                            let (row, removed) = sidebar_row(ui, &self.sources[i].0, count, self.source == Some(i), removable);
                            if removed {
                                remove = Some(i);
                            } else if row.clicked() {
                                self.set_source(Some(i));
                            }
                        }
                        if let Some(i) = remove {
                            self.remove_folder(i);
                        }
                        ui.add_space(theme::space::M);
                        if widgets::primary_button(ui, "Add folder").clicked()
                            && let Some(dir) = rfd::FileDialog::new().pick_folder()
                        {
                            self.add_folder(dir);
                        }
                    });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    // First in a bottom-up layout = lowest on screen.
                    self.support_card(ui);
                    ui.add_space(theme::space::S);
                    let status = format!("{} families · {} styles", self.families.len(), self.faces.len());
                    let status = if self.scan_done { status } else { format!("Scanning… {status}") };
                    ui.label(RichText::new(status).font(FontId::new(theme::size::META, theme::mono())).color(color::CHROME_TEXT_MUTED));
                });
            });
    }

    /// One Library or Collections row; clicking the chosen one clears the filter.
    fn collection_row(&mut self, ui: &mut egui::Ui, c: Collection, counts: [usize; 5]) {
        let chosen = self.collection == Some(c);
        let response = source_row(ui, c.label(), counts[c.index()], chosen).on_hover_text(c.hint());
        if response.clicked() {
            self.set_collection(if chosen { None } else { Some(c) });
        }
    }

    /// While the preview text filters the list: says so, and offers to show everything again.
    fn filter_notice(&mut self, ui: &mut egui::Ui, need: u16) {
        ui.horizontal(|ui| {
            let names = crate::samples::script_names(need);
            ui.label(RichText::new(format!("Showing fonts that support {names}")).size(theme::size::SECONDARY).color(color::INK_DIM));
            if widgets::link(ui, "Show all fonts").clicked() {
                self.text.clear();
            }
        });
        ui.add_space(theme::space::S);
    }

    /// Why the list is empty and what to do about it; the first rule that fits wins.
    fn empty_text(&self) -> String {
        if !self.scan_done && self.families.is_empty() {
            return "Scanning your fonts…".into();
        }
        let need = self.text_need();
        if need != 0 {
            return format!("No fonts support {}. Show all fonts to see everything.", crate::samples::script_names(need));
        }
        if !self.search.is_empty() {
            return format!("No fonts match \u{201C}{}\u{201D}. Clear the search or pick another source.", self.search);
        }
        match self.collection {
            Some(Collection::TurnedOn) => "Nothing is turned on yet. Turn a font on from its details panel.".into(),
            Some(Collection::Variable) => "No variable fonts found. Fonts with adjustable weight or width appear here.".into(),
            Some(Collection::ManyWeights) => "No families with more than 3 weights here. Try All fonts.".into(),
            Some(c) => format!("No {} fonts here. Try All fonts.", c.label()),
            None => "No fonts match.".into(),
        }
    }

    /// Small card at the bottom of the side panel: picture, thanks line and the coffee link.
    fn support_card(&mut self, ui: &mut egui::Ui) {
        // Fixed height (48px picture + margins): the sidebar's bottom-up layout would otherwise stretch the frame.
        let size = egui::vec2(ui.available_width(), 64.0);
        ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::LEFT), |ui| {
            egui::Frame::NONE
                .fill(color::CHROME_RAISED)
                .corner_radius(theme::radius::CARD)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        // The picture goes first when the sidebar is narrow so the button keeps its full label.
                        if ui.available_width() > 160.0
                            && let Some(art) = &self.art
                        {
                            ui.add(egui::Image::new((art.id(), egui::vec2(48.0, 48.0))).corner_radius(4.0));
                        }
                        ui.add_space(theme::space::XS);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Support FontCraft").size(theme::size::SECONDARY).color(color::CHROME_TEXT));
                            ui.add_space(2.0);
                            if widgets::ghost_button(ui, "Buy me a coffee").clicked() {
                                self.notice = crate::support::open_support().err();
                            }
                        });
                    });
                });
        });
    }

    fn list(&mut self, ui: &mut egui::Ui) {
        if self.view == grid::View::Grid {
            return self.grid(ui);
        }
        let visible = self.visible();
        let row_h = self.size * 1.35 + 34.0;
        let need = self.text_need();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(color::PAPER).inner_margin(egui::Margin::symmetric(theme::space::XL as i8, 8)))
            .show(ui, |ui| {
                if need != 0 {
                    self.filter_notice(ui, need);
                }
                if visible.is_empty() {
                    ui.centered_and_justified(|ui| {
                        let text = self.empty_text();
                        ui.label(RichText::new(text).size(theme::size::BODY).color(color::INK_MUTED));
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
        let on = all_active(&self.active, &self.disabled, fam);
        let family = self.loader.family(&face);
        let sample = self.sample(&face);
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_h), egui::Sense::click());
        let painter = ui.painter_at(rect);
        if selected || resp.hovered() {
            let inner = rect.shrink2(egui::vec2(0.0, 2.0));
            if selected {
                painter.rect(inner, theme::radius::CONTROL, color::SELECTED, egui::Stroke::new(1.0, color::CANVAS_LINE), egui::StrokeKind::Inside);
            } else {
                painter.rect_filled(inner, theme::radius::CONTROL, color::CANVAS_HOVER);
            }
        }
        let drawn = family.map(|f| pick_font(ui.ctx(), f, &sample, self.size, color::INK));
        let lacks = drawn.as_ref().is_some_and(|d| !d.has_all);
        let mut meta = format!("{name}   ·   {styles} style{}", if styles == 1 { "" } else { "s" });
        if lacks {
            meta.push_str("   ·   no glyphs for this text");
        }
        painter.text(rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, meta, FontId::proportional(theme::size::SECONDARY), color::INK_DIM);
        if on {
            let status = egui::Rect::from_min_size(rect.right_top() + egui::vec2(-theme::space::M, 8.0), egui::vec2(0.0, 16.0));
            widgets::status_dot(&painter, status);
        }
        let pos = rect.left_top() + egui::vec2(8.0, 28.0);
        match drawn {
            Some(d) => {
                painter.text(pos, egui::Align2::LEFT_TOP, sample, d.font, d.color);
            }
            None => {
                painter.text(pos, egui::Align2::LEFT_TOP, sample, FontId::proportional(self.size), color::CANVAS_LINE);
            }
        }
        if resp.clicked() {
            self.selected = if selected { None } else { Some(name) };
            self.selected_face = 0;
        }
    }

    /// The same families as square cards, virtualized by row: only the visible cards are drawn
    /// and only their fonts load.
    fn grid(&mut self, ui: &mut egui::Ui) {
        let visible = self.visible();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(color::PAPER).inner_margin(egui::Margin::symmetric(theme::space::XL as i8, 8)))
            .show(ui, |ui| {
                if visible.is_empty() {
                    ui.centered_and_justified(|ui| {
                        let text = self.empty_text();
                        ui.label(RichText::new(text).size(theme::size::BODY).color(color::INK_MUTED));
                    });
                    return;
                }
                let columns = grid::columns(ui.available_width());
                let rows = grid::row_count(visible.len(), columns);
                egui::ScrollArea::vertical().auto_shrink(false).show_rows(ui, grid::row_height(), rows, |ui, range| {
                    let drawn = grid::visible_range(range.start, range.end, columns, visible.len());
                    // One allocation per row; the cards sit left to right inside it.
                    let mut row: Option<(usize, egui::Rect)> = None;
                    for i in drawn {
                        let r = i / columns;
                        let row_rect = match row {
                            Some((same, rect)) if same == r => rect,
                            _ => {
                                let (rect, _) =
                                    ui.allocate_exact_size(egui::vec2(ui.available_width(), grid::row_height()), egui::Sense::hover());
                                row = Some((r, rect));
                                rect
                            }
                        };
                        // Leftover width is split evenly on both sides so the grid stays centred as the window changes.
                        let used = columns as f32 * grid::CARD_SIZE + (columns - 1) as f32 * theme::space::L;
                        let margin = ((row_rect.width() - used) / 2.0).max(0.0);
                        let x = row_rect.left() + margin + (i % columns) as f32 * (grid::CARD_SIZE + theme::space::L);
                        let card = egui::Rect::from_min_size(egui::pos2(x, row_rect.top()), egui::vec2(grid::CARD_SIZE, grid::CARD_SIZE));
                        self.card(ui, visible[i], card);
                    }
                });
            });
    }

    /// One card: the script glyph in the family's own face, its style count, the On dot and the
    /// name underneath. A click opens the same details as a list row.
    fn card(&mut self, ui: &mut egui::Ui, i: usize, card: egui::Rect) {
        let fam = &self.families[i];
        let face = fam.main_face().clone();
        let selected = self.selected.as_deref() == Some(fam.name.as_str());
        let (name, styles) = (fam.name.clone(), fam.faces.len());
        let on = all_active(&self.active, &self.disabled, fam);
        let family = self.loader.family(&face);
        let glyph = crate::samples::card_glyph(face.script);
        let name_rect = egui::Rect::from_min_size(
            egui::pos2(card.left(), card.bottom() + theme::space::S),
            egui::vec2(grid::CARD_SIZE, grid::NAME_H),
        );
        let resp = ui.interact(card.union(name_rect), ui.scope_id().with(("grid card", i)), egui::Sense::click());
        let painter = ui.painter_at(card.union(name_rect));
        let fill = if selected {
            color::SELECTED
        } else if resp.hovered() {
            color::CANVAS_HOVER
        } else {
            color::CARD
        };
        painter.rect(card, theme::radius::CARD, fill, egui::Stroke::new(1.0, color::CANVAS_LINE), egui::StrokeKind::Inside);
        const GLYPH_SIZE: f32 = 44.0;
        match family.map(|f| pick_font(ui.ctx(), f, glyph, GLYPH_SIZE, color::INK)) {
            Some(d) => {
                painter.text(card.center(), egui::Align2::CENTER_CENTER, glyph, d.font, d.color);
            }
            None => {
                painter.text(card.center(), egui::Align2::CENTER_CENTER, glyph, FontId::proportional(GLYPH_SIZE), color::CANVAS_LINE);
            }
        }
        painter.text(
            egui::pos2(card.left() + theme::space::S, card.bottom() - theme::space::S),
            egui::Align2::LEFT_BOTTOM,
            format!("{styles} style{}", if styles == 1 { "" } else { "s" }),
            FontId::proportional(theme::size::META),
            color::INK_MUTED,
        );
        if on {
            let status =
                egui::Rect::from_min_size(card.right_top() + egui::vec2(-theme::space::S, theme::space::S), egui::vec2(0.0, 16.0));
            widgets::status_dot(&painter, status);
        }
        let galley = name_galley(ui.ctx(), &name);
        painter.galley(name_rect.left_top(), galley, color::INK_DIM);
        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            self.selected = if selected { None } else { Some(name) };
            self.selected_face = 0;
        }
    }

    fn details(&mut self, ui: &mut egui::Ui) {
        let Some(name) = self.selected.clone() else { return };
        let Some(fam) = self.families.iter().find(|f| f.name == name) else { return };
        let faces = fam.faces.clone();
        let files = fam.files();
        let on = all_active(&self.active, &self.disabled, fam);
        let saved_off = files.iter().any(|f| self.disabled.contains(&activate::norm(f)));
        let by_fontbase = files.iter().any(|f| self.fontbase_files.contains(&activate::norm(f)));
        let lock = Self::lock(fam);
        let can = lock.is_none();
        let shares: Vec<String> = self
            .families
            .iter()
            .filter(|f| f.name != name && f.faces.iter().any(|x| files.contains(&x.path)))
            .map(|f| f.name.clone())
            .collect();
        let mut clicked = false;
        egui::Panel::right("details")
            .default_size(420.0)
            .max_size(ui.available_width() * 0.6)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(color::CARD).inner_margin(egui::Margin::same(16)))
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(color::INK);
                ui.visuals_mut().selection.bg_fill = color::SELECTED;
                ui.visuals_mut().selection.stroke.color = color::INK;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&name).size(theme::size::DETAIL).color(color::INK));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::paper_button(ui, "×").on_hover_text("Close").clicked() {
                            self.selected = None;
                        }
                    });
                });
                ui.label(RichText::new(faces[0].path.display().to_string()).font(FontId::new(theme::size::META, theme::mono())).color(color::INK_MUTED));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let text = match (can, on) {
                        (false, _) => if cfg!(windows) { "System font: needs administrator" } else { "Locked" },
                        (true, true) => "Turn off",
                        (true, false) => "Turn on",
                    };
                    // A locked font gets a plain label (a faded disabled button would fall under 4.5:1).
                    if can {
                        let resp = if on { widgets::paper_button(ui, text) } else { widgets::primary_button(ui, text) };
                        clicked = resp.clicked();
                    } else {
                        ui.label(RichText::new(text).size(theme::size::SECONDARY).color(color::INK_DIM))
                            .on_hover_text(lock.unwrap_or_default());
                    }
                    if on && can {
                        ui.label(RichText::new("On for all apps").size(theme::size::SECONDARY).color(color::COBALT));
                    } else if saved_off {
                        ui.label(RichText::new("Off: its settings are saved").size(theme::size::SECONDARY).color(color::INK_MUTED));
                    }
                });
                if by_fontbase && can {
                    ui.label(RichText::new("FontBase قد يعرض هذا الخط كمفعّل حتى تفتحه فيه").size(11.0).color(color::INK_MUTED));
                }
                if !shares.is_empty() && can {
                    ui.label(
                        RichText::new(format!("This file also holds: {} (they change together)", shares.join(", ")))
                            .size(11.0)
                            .color(color::INK_MUTED),
                    );
                }
                if let Some(n) = &self.notice {
                    let notice = egui::Label::new(RichText::new(n).size(12.0).color(color::DANGER))
                        .sense(egui::Sense::click());
                    if ui.add(notice).on_hover_text("Click to dismiss").clicked() {
                        self.notice = None;
                    }
                }
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    for (k, face) in faces.iter().enumerate() {
                        let fam = self.loader.family(face);
                        let label = RichText::new(&face.style).size(22.0);
                        let label = match fam { Some(f) => label.family(f), None => label.color(color::CANVAS_LINE) };
                        if ui.selectable_label(self.selected_face == k, label).clicked() {
                            self.selected_face = k;
                        }
                    }
                    ui.add_space(16.0);
                    widgets::paper_section_label(ui, "Waterfall");
                    let face = &faces[self.selected_face.min(faces.len() - 1)];
                    let sample = self.sample(face);
                    if let Some(f) = self.loader.family(face) {
                        for size in WATERFALL {
                            let d = pick_font(ui.ctx(), f.clone(), &sample, size, color::INK);
                            ui.label(RichText::new(format!("{size} px")).font(FontId::new(theme::size::META, theme::mono())).color(color::INK_MUTED));
                            ui.add(egui::Label::new(RichText::new(&sample).font(d.font).color(d.color)).wrap());
                        }
                    }
                });
            });
        if clicked {
            self.toggle(&name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(family: &str, style: &str, weight: u16, italic: bool) -> Face {
        Face { path: PathBuf::from(format!("{family}{style}.ttf")), index: 0, family: family.into(), style: style.into(), weight, italic, script: 0, scripts: crate::samples::LATIN, variable: false, source: 0 }
    }

    fn sourced(path: &std::path::Path, family: &str, source: usize) -> Face {
        Face { path: path.to_path_buf(), source, ..face(family, "Regular", 400, false) }
    }

    impl FontApp {
        /// An app with no scan running, `fixed_sources` built-in sources and these faces.
        fn for_test(fixed_sources: usize, faces: Vec<Face>) -> Self {
            Self {
                sources: Vec::new(),
                fixed_sources,
                scan: Scans::new(),
                scan_done: true,
                families: group(&faces),
                faces,
                loader: FontLoader::new(),
                text: String::new(),
                size: SIZE_DEFAULT,
                search: String::new(),
                source: None,
                collection: None,
                counts: [0; 5],
                view: grid::View::List,
                selected: None,
                selected_face: 0,
                active: HashSet::new(),
                disabled: HashSet::new(),
                fontbase_files: HashSet::new(),
                store: PathBuf::new(),
                notice: None,
                update: None,
                updates: None,
                update_error: None,
                installing: None,
                art: None,
                frames: 0,
            }
        }
    }

    #[test]
    fn family_files_are_distinct_and_sorted() {
        let fam = Family::new(
            "X".into(),
            vec![sourced("b.ttc".as_ref(), "X", 3), sourced("a.ttf".as_ref(), "X", 3), sourced("b.ttc".as_ref(), "X", 3)],
        );
        assert_eq!(fam.files(), vec![PathBuf::from("a.ttf"), PathBuf::from("b.ttc")]);
    }

    #[test]
    fn family_aggregates_scripts_variable_and_weights() {
        let mut a = face("Mixed", "Regular", 400, false);
        a.scripts = crate::samples::LATIN | crate::samples::ARABIC;
        let mut b = face("Mixed", "Bold", 700, false);
        b.variable = true;
        let c = face("Mixed", "Italic", 400, true);
        let fam = &group(&[a, b, c])[0];
        assert_eq!(fam.scripts, crate::samples::LATIN | crate::samples::ARABIC);
        assert!(fam.variable);
        assert_eq!(fam.weight_count, 2); // 400 and 700; the italic repeats 400
    }

    #[cfg(windows)]
    fn env_path(var: &str, rest: &str) -> PathBuf {
        PathBuf::from(std::env::var_os(var).unwrap()).join(rest)
    }

    #[cfg(windows)]
    #[test]
    fn can_toggle_false_for_built_in_sources() {
        let app = FontApp::for_test(3, vec![sourced(&env_path("WINDIR", r"Fonts\arial.ttf"), "X", 0)]);
        assert!(!app.can_toggle(&app.families[0]));
    }

    #[test]
    fn can_toggle_true_for_added_folder() {
        let app = FontApp::for_test(3, vec![sourced("a.ttf".as_ref(), "X", 3)]);
        assert!(app.can_toggle(&app.families[0]));
    }

    #[cfg(windows)]
    #[test]
    fn can_toggle_for_the_four_font_sources() {
        let faces = vec![
            sourced(&env_path("WINDIR", r"Fonts\arial.ttf"), "System", 0),
            sourced(&env_path("LOCALAPPDATA", r"Microsoft\Windows\Fonts\user.ttf"), "User", 1),
            sourced(&env_path("LOCALAPPDATA", r"Microsoft\Windows\Fonts\FontBase\fb.otf"), "FontBase", 2),
            sourced(r"D:\My fonts\added.ttf".as_ref(), "Added", 3),
        ];
        let app = FontApp::for_test(3, faces);
        let can = |name: &str| app.can_toggle(app.families.iter().find(|f| f.name == name).unwrap());
        assert!(!can("System"));
        assert!(can("User"));
        assert!(can("FontBase"));
        assert!(can("Added"));
    }

    #[test]
    fn user_font_is_on_until_rejected() {
        let user = Path::new("/home/u/.local/share/fonts");
        let links = user.join("FontCraft");
        let fam = |p: &str| Family::new("X".into(), vec![sourced(p.as_ref(), "X", 1)]);
        let (none, off) = (HashSet::new(), HashSet::from([PathBuf::from("/home/u/.local/share/fonts/a.ttf")]));
        let own = fam("/home/u/.local/share/fonts/a.ttf");
        assert!(all_active_in(&none, &none, user, &links, &own));
        assert!(!all_active_in(&none, &off, user, &links, &own));
        // Elsewhere, or inside the links folder, a font is on only when linked.
        assert!(!all_active_in(&none, &none, user, &links, &fam("/data/a.ttf")));
        assert!(!all_active_in(&none, &none, user, &links, &fam("/home/u/.local/share/fonts/FontCraft/a-1.ttf")));
        assert!(!all_active_in(&none, &none, Path::new(""), &links, &own));
    }

    #[cfg(unix)]
    #[test]
    fn unix_system_font_cannot_toggle() {
        let system = crate::platform::system_font_dirs()[0].join("a.ttf");
        let app = FontApp::for_test(3, vec![sourced(&system, "X", 0)]);
        assert!(!app.can_toggle(&app.families[0]));
    }

    #[cfg(unix)]
    #[test]
    fn unix_toggle_links_font_from_added_folder() {
        let dir = std::env::temp_dir().join(format!("fontcraft_app_toggle_{}", std::process::id()));
        let (links, font) = (dir.join("links"), dir.join("added").join("a.ttf"));
        std::fs::create_dir_all(font.parent().unwrap()).unwrap();
        std::fs::write(&font, b"x").unwrap();
        let mut app = FontApp::for_test(3, vec![sourced(&font, "X", 3)]);
        app.toggle_in("X", links.to_str().unwrap(), &dir.join("fonts.conf"));
        let on = app.active.contains(&font);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(app.notice, None);
        assert!(on);
    }

    #[test]
    fn is_active_requires_all_files_active() {
        let fam = Family::new("X".into(), vec![sourced("a.ttf".as_ref(), "X", 3), sourced("b.ttf".as_ref(), "X", 3)]);
        let mut active = HashSet::from([PathBuf::from("a.ttf")]);
        assert!(!all_active(&active, &HashSet::new(), &fam));
        active.insert(PathBuf::from("b.ttf"));
        assert!(all_active(&active, &HashSet::new(), &fam));
    }

    #[cfg(windows)]
    #[test]
    fn toggle_turns_family_on_then_off() {
        let s = crate::activate::tests::Scratch::new("apptoggle");
        let f = s.font("toggle.ttf");
        let mut app = FontApp::for_test(3, vec![sourced(&f, "X", 3)]);
        app.toggle_in("X", &s.key, &s.store());
        assert_eq!(app.notice, None);
        assert!(all_active(&app.active, &app.disabled, &app.families[0]));
        app.toggle_in("X", &s.key, &s.store());
        assert!(!all_active(&app.active, &app.disabled, &app.families[0]));
        // A system family is never touched.
        let mut system = FontApp::for_test(3, vec![sourced(&env_path("WINDIR", r"Fonts\arial.ttf"), "Y", 0)]);
        system.toggle_in("Y", &s.key, &s.store());
        assert!(system.active.is_empty() && system.notice.is_none());
    }

    #[cfg(windows)]
    #[test]
    fn toggle_user_installed_family_off_then_on() {
        let s = crate::activate::tests::Scratch::new("appuser");
        let f = s.font("installed.ttf");
        s.install("Installed Regular (TrueType)", &f);
        let mut app = FontApp::for_test(3, vec![sourced(&f, "X", 1)]);
        app.active = crate::activate::read_active(&s.key);
        app.toggle_in("X", &s.key, &s.store());
        assert_eq!(app.notice, None);
        assert!(!all_active(&app.active, &app.disabled, &app.families[0]));
        assert!(app.disabled.contains(&crate::activate::norm(&f)));
        app.toggle_in("X", &s.key, &s.store());
        assert_eq!(app.notice, None);
        assert!(all_active(&app.active, &app.disabled, &app.families[0]));
        assert!(app.disabled.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn toggle_missing_file_shows_notice_and_keeps_state() {
        let s = crate::activate::tests::Scratch::new("appmissing");
        let gone = s.dir.join("gone.ttf");
        let mut app = FontApp::for_test(3, vec![sourced(&gone, "X", 3)]);
        app.toggle_in("X", &s.key, &s.store());
        assert!(app.notice.as_deref().is_some_and(|n| n.contains("gone.ttf")));
        assert!(app.active.is_empty());
    }

    #[test]
    fn groups_case_insensitively_and_picks_regular() {
        let fams = group(&[face("Inter", "Bold", 700, false), face("inter", "Regular", 400, false), face("Inter", "Italic", 400, true), face("Abc", "Regular", 400, false)]);
        assert_eq!(fams.len(), 2);
        assert_eq!(fams[0].name, "Abc");
        assert_eq!(fams[1].faces.len(), 3);
        assert_eq!(fams[1].main_face().style, "Regular");
    }

    #[test]
    fn grid_drawn_for_visible_rows_only() {
        let faces: Vec<Face> = (0..50).map(|i| face(&format!("Fam{i:02}"), "Regular", 400, false)).collect();
        let app = FontApp::for_test(3, faces);
        let visible = app.visible();
        assert_eq!(visible.len(), 50);
        let cols = grid::columns(grid::CARD_SIZE * 4.0 + theme::space::L * 3.0);
        assert_eq!(cols, 4);
        // What `show_rows` reports for three rows starting at row 3: exactly those families.
        let drawn = grid::visible_range(3, 6, cols, visible.len());
        assert_eq!(drawn, 12..24);
        assert_eq!(&visible[drawn], &visible[12..24]);
        // The last row stops at the last family, and nothing ever reaches past it.
        assert_eq!(grid::visible_range(11, 14, cols, visible.len()), 44..50);
        assert_eq!(grid::visible_range(0, 999, cols, visible.len()), 0..50);
        assert_eq!(grid::visible_range(0, 2, cols, 0), 0..0);
    }

    fn app_with_scripts() -> FontApp {
        use crate::samples::{ARABIC, LATIN};
        let mut ara = face("Ara", "Regular", 400, false);
        ara.scripts = ARABIC;
        let mut lat = face("Lat", "Regular", 400, false);
        lat.scripts = LATIN;
        let mut mix = face("Mix", "Regular", 400, false);
        mix.scripts = LATIN | ARABIC;
        FontApp::for_test(3, vec![ara, lat, mix])
    }

    fn names(app: &FontApp) -> Vec<String> {
        app.visible().into_iter().map(|i| app.families[i].name.clone()).collect()
    }

    #[test]
    fn visible_obeys_collection_text_and_search() {
        let mut app = app_with_scripts();
        app.refresh_counts();
        assert_eq!(names(&app), ["Ara", "Lat", "Mix"]);
        assert_eq!(app.collection_counts()[Collection::Arabic.index()], 2);
        assert_eq!(app.collection_counts()[Collection::English.index()], 2);
        app.text = "عبد".into();
        assert_eq!(names(&app), ["Ara", "Mix"]);
        app.text = "Hello".into();
        assert_eq!(names(&app), ["Lat", "Mix"]);
        app.text = "Hello عبد".into();
        assert_eq!(names(&app), ["Mix"]);
        app.text = "123 !?".into();
        assert_eq!(names(&app).len(), 3);
        app.text.clear();
        app.set_collection(Some(Collection::Arabic));
        assert_eq!(names(&app), ["Ara", "Mix"]);
        app.search = "mi".into();
        assert_eq!(names(&app), ["Mix"]);
        app.search.clear();
        app.set_collection(None);
        app.text = "عبد".into();
        app.set_view(grid::View::Grid);
        assert_eq!(names(&app).len(), 3); // the grid ignores the preview text
    }

    #[test]
    fn collection_and_source_exclude_each_other() {
        let mut app = app_with_scripts();
        app.set_collection(Some(Collection::Arabic));
        app.set_source(Some(1));
        assert_eq!((app.collection, app.source), (None, Some(1)));
        app.set_collection(Some(Collection::English));
        assert_eq!((app.collection, app.source), (Some(Collection::English), None));
    }

    #[test]
    fn turned_on_follows_the_active_set() {
        let mut app = app_with_scripts();
        app.refresh_counts();
        assert_eq!(app.collection_counts()[Collection::TurnedOn.index()], 0);
        app.set_collection(Some(Collection::TurnedOn));
        assert!(names(&app).is_empty());
        let path = app.families[1].faces[0].path.clone();
        app.active.insert(activate::norm(&path));
        app.refresh_counts();
        assert_eq!(app.collection_counts()[Collection::TurnedOn.index()], 1);
        assert_eq!(names(&app), ["Lat"]);
        app.active.clear();
        app.refresh_counts();
        assert!(names(&app).is_empty());
    }

    #[test]
    fn filters_do_not_clear_the_selection() {
        let mut app = app_with_scripts();
        app.selected = Some("Lat".into());
        app.text = "عبد".into();
        assert!(!names(&app).contains(&"Lat".to_string()));
        assert_eq!(app.selected.as_deref(), Some("Lat"));
    }

    #[test]
    fn dropping_a_folder_shifts_the_rest_and_repairs_state() {
        let dir = std::env::temp_dir().join(format!("fontcraft-drop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let kept_file = dir.join("keep.ttf");
        std::fs::write(&kept_file, b"x").unwrap();
        let mut app = FontApp::for_test(
            3,
            vec![
                sourced("a.ttf".as_ref(), "OnlyA", 3),
                sourced(&kept_file, "OnlyB", 4),
                sourced("shared-a.ttf".as_ref(), "Shared", 3),
                sourced("shared-b.ttf".as_ref(), "Shared", 4),
            ],
        );
        app.sources = ["System", "User", "FontBase", "A", "B"].map(|n| (n.to_string(), PathBuf::from(n))).to_vec();
        app.source = Some(4);
        app.selected = Some("OnlyA".into());
        assert!(app.drop_source(3));
        assert_eq!(app.sources.len(), 4);
        assert_eq!(app.sources[3].0, "B");
        assert_eq!(app.source, Some(3)); // B moved from 4 to 3 and stays chosen
        let names: Vec<&str> = app.families.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["OnlyB", "Shared"]);
        assert!(app.faces.iter().all(|f| f.source == 3)); // everything left lives in the shifted folder
        assert_eq!(app.selected, None); // its only family went away
        assert!(kept_file.is_file()); // files on disk are never touched
        // Built-in sources and unknown indexes are refused.
        assert!(!app.drop_source(1));
        assert!(!app.drop_source(9));
        assert_eq!(app.sources.len(), 4);
        // Removing the chosen folder clears the choice; a kept selection stays.
        app.selected = Some("Shared".into());
        assert!(app.drop_source(3));
        assert_eq!(app.source, None);
        assert!(app.families.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn set_view_keeps_selection() {
        let mut app = FontApp::for_test(3, vec![face("Arial", "Regular", 400, false)]);
        app.selected = Some("Arial".into());
        app.set_view(grid::View::Grid);
        assert_eq!(app.view, grid::View::Grid);
        assert_eq!(app.selected.as_deref(), Some("Arial"));
        app.set_view(grid::View::List);
        assert_eq!(app.selected.as_deref(), Some("Arial"));
    }
}
