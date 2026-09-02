use super::{cancelled, run_native};
use crate::dict::Choice;
use crate::filters::FileFilter;
use crate::paths::{home_dir, places, recent_files, unique_path, RECENT_PLACE};
use crate::theme::OmarchyTheme;
use egui::{Align, Key, Layout, RichText, ScrollArea, Sense};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileMode {
    Open,
    Save,
    SaveFiles,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileChooserRequest {
    pub title: String,
    pub accept_label: String,
    pub mode: FileMode,
    pub multiple: bool,
    pub directory: bool,
    pub filters: Vec<FileFilter>,
    pub current_filter: Option<usize>,
    pub choices: Vec<Choice>,
    pub current_folder: PathBuf,
    /// Host path of `current_file` after document-portal restore (Save preselect).
    #[serde(default)]
    pub current_file: Option<PathBuf>,
    pub current_name: String,
    pub save_names: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileChooserResult {
    pub paths: Vec<PathBuf>,
    pub choices: Vec<(String, String)>,
    pub current_filter: Option<(String, Vec<(u32, String)>)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pending,
    Accept,
    Cancel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Name,
    Size,
    Time,
}

struct Entry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
}

pub fn run_file_chooser(
    req: FileChooserRequest,
    token: CancellationToken,
) -> Option<FileChooserResult> {
    let state = Arc::new(Mutex::new(ChooserApp::new(req, token.clone())));
    let title = state.lock().unwrap().req.title.clone();
    let ui_state = Arc::clone(&state);
    let _ = run_native(title, [960.0, 640.0], ChooserUi { state: ui_state, token });
    let app = state.lock().unwrap();
    match app.outcome {
        Outcome::Accept => Some(app.build_result()),
        _ => None,
    }
}

struct ChooserApp {
    req: FileChooserRequest,
    folder: PathBuf,
    entries: Vec<Entry>,
    selected: Vec<PathBuf>,
    filter_index: Option<usize>,
    filename: String,
    query: String,
    path_edit: String,
    show_hidden: bool,
    error: Option<String>,
    new_folder: Option<String>,
    overwrite: Option<PathBuf>,
    choice_values: Vec<String>,
    outcome: Outcome,
    sort_key: SortKey,
    sort_reversed: bool,
    recent_mode: bool,
}

impl ChooserApp {
    fn new(req: FileChooserRequest, _token: CancellationToken) -> Self {
        let folder = if req.current_folder.is_dir() {
            req.current_folder.clone()
        } else {
            req.current_folder
                .parent()
                .filter(|p| p.is_dir())
                .map(Path::to_path_buf)
                .unwrap_or_else(home_dir)
        };
        let filename = req.current_name.clone();
        let filter_index = req
            .current_filter
            .or_else(|| if req.filters.is_empty() { None } else { Some(0) });
        let choice_values = req.choices.iter().map(|c| c.selected.clone()).collect();
        let preselect = req
            .current_file
            .clone()
            .filter(|p| p.parent().is_some_and(|parent| parent == folder) && p.exists());
        let mut app = Self {
            req,
            folder: folder.clone(),
            entries: Vec::new(),
            selected: preselect.into_iter().collect(),
            filter_index,
            filename,
            query: String::new(),
            path_edit: folder.display().to_string(),
            show_hidden: false,
            error: None,
            new_folder: None,
            overwrite: None,
            choice_values,
            outcome: Outcome::Pending,
            sort_key: SortKey::Name,
            sort_reversed: false,
            recent_mode: false,
        };
        app.refresh();
        app
    }

    fn set_sort(&mut self, key: SortKey) {
        if self.sort_key == key {
            self.sort_reversed = !self.sort_reversed;
        } else {
            self.sort_key = key;
            self.sort_reversed = false;
        }
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        let key = self.sort_key;
        let reversed = self.sort_reversed;
        self.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let mut ord = match key {
                    SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortKey::Size => a.size.cmp(&b.size),
                    SortKey::Time => a.modified.cmp(&b.modified),
                };
                if reversed {
                    ord = ord.reverse();
                }
                ord
            }
        });
    }

    fn go_recent(&mut self) {
        self.recent_mode = true;
        self.selected.clear();
        self.refresh();
    }

    fn go_folder(&mut self, path: PathBuf) {
        self.recent_mode = false;
        self.folder = path;
        self.selected.clear();
        self.refresh();
    }

    fn refresh(&mut self) {
        self.error = None;
        self.entries.clear();
        if self.recent_mode {
            for f in recent_files(24) {
                if !self.show_hidden && f.label.starts_with('.') {
                    continue;
                }
                if !self.query.is_empty()
                    && !f.label.to_lowercase().contains(&self.query.to_lowercase())
                {
                    continue;
                }
                self.entries.push(Entry {
                    name: f.label,
                    path: f.path,
                    is_dir: f.is_dir,
                    size: f.size,
                    modified: if f.modified > 0 {
                        std::time::UNIX_EPOCH
                            .checked_add(std::time::Duration::from_secs(f.modified as u64))
                    } else {
                        None
                    },
                });
            }
            self.path_edit = RECENT_PLACE.to_string();
            self.apply_sort();
            return;
        }
        let read = match std::fs::read_dir(&self.folder) {
            Ok(rd) => rd,
            Err(err) => {
                self.error = Some(err.to_string());
                return;
            }
        };
        for ent in read.flatten() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }
            if !self.query.is_empty()
                && !name.to_lowercase().contains(&self.query.to_lowercase())
            {
                continue;
            }
            let path = ent.path();
            let meta = ent.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(path.is_dir());
            if !is_dir && self.req.directory && self.req.mode == FileMode::Open {
                continue;
            }
            if !is_dir {
                if let Some(idx) = self.filter_index {
                    if let Some(filter) = self.req.filters.get(idx) {
                        if !filter.matches(&path) {
                            continue;
                        }
                    }
                }
            }
            self.entries.push(Entry {
                name,
                path,
                is_dir,
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: meta.and_then(|m| m.modified().ok()),
            });
        }
        self.apply_sort();
        self.path_edit = self.folder.display().to_string();
        self.selected.retain(|p| p.parent() == Some(self.folder.as_path()));
    }

    fn enter(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.go_folder(path);
        } else if self.req.mode == FileMode::Save {
            if let Some(name) = path.file_name() {
                self.filename = name.to_string_lossy().into_owned();
            }
            self.selected = vec![path];
        } else if !self.req.directory {
            self.toggle_select(path);
        }
    }

    fn toggle_select(&mut self, path: PathBuf) {
        if self.req.multiple {
            if let Some(i) = self.selected.iter().position(|p| *p == path) {
                self.selected.remove(i);
            } else {
                self.selected.push(path);
            }
        } else {
            self.selected = vec![path];
        }
    }

    fn parent(&mut self) {
        if self.recent_mode {
            self.go_folder(home_dir());
            return;
        }
        if let Some(parent) = self.folder.parent() {
            self.go_folder(parent.to_path_buf());
        }
    }

    fn try_accept(&mut self) {
        match self.req.mode {
            FileMode::Open => {
                if self.req.directory {
                    let path = self.selected.first().cloned().unwrap_or_else(|| self.folder.clone());
                    if path.is_dir() {
                        self.selected = vec![path];
                        self.outcome = Outcome::Accept;
                    }
                } else if !self.selected.is_empty() && self.selected.iter().all(|p| p.is_file() || p.is_dir()) {
                    if !self.req.multiple && self.selected[0].is_dir() {
                        self.enter(self.selected[0].clone());
                        return;
                    }
                    self.outcome = Outcome::Accept;
                } else {
                    self.error = Some("Select a file first".into());
                }
            }
            FileMode::Save => {
                if self.filename.trim().is_empty() {
                    self.error = Some("Enter a file name".into());
                    return;
                }
                let path = self.folder.join(self.filename.trim());
                if path.exists() {
                    self.overwrite = Some(path);
                    return;
                }
                self.selected = vec![path];
                self.outcome = Outcome::Accept;
            }
            FileMode::SaveFiles => {
                if !self.folder.is_dir() {
                    self.error = Some("Select a folder".into());
                    return;
                }
                let names = if self.req.save_names.is_empty() {
                    vec![self.filename.clone()].into_iter().filter(|s| !s.is_empty()).collect()
                } else {
                    self.req.save_names.clone()
                };
                self.selected = names
                    .iter()
                    .map(|name| unique_path(&self.folder, name))
                    .collect();
                self.outcome = Outcome::Accept;
            }
        }
    }

    fn confirm_overwrite(&mut self) {
        if let Some(path) = self.overwrite.take() {
            self.selected = vec![path];
            self.outcome = Outcome::Accept;
        }
    }

    fn create_folder(&mut self) {
        if let Some(name) = self.new_folder.take() {
            let name = name.trim();
            if name.is_empty() {
                return;
            }
            let path = self.folder.join(name);
            if let Err(err) = std::fs::create_dir(&path) {
                self.error = Some(err.to_string());
            } else {
                self.go_folder(path);
            }
        }
    }

    fn build_result(&self) -> FileChooserResult {
        FileChooserResult {
            paths: self.selected.clone(),
            choices: self
                .req
                .choices
                .iter()
                .zip(self.choice_values.iter())
                .map(|(c, v)| (c.id.clone(), v.clone()))
                .collect(),
            current_filter: self
                .filter_index
                .and_then(|i| self.req.filters.get(i))
                .map(|f| f.to_portal()),
        }
    }
}

struct ChooserUi {
    state: Arc<Mutex<ChooserApp>>,
    token: CancellationToken,
}

impl eframe::App for ChooserUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if cancelled(&self.token) {
            if let Ok(mut app) = self.state.lock() {
                app.outcome = Outcome::Cancel;
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(120));

        let theme = OmarchyTheme::load();
        let mut close = false;
        {
            let mut app = self.state.lock().unwrap();
            if app.outcome != Outcome::Pending {
                close = true;
            } else {
                draw(ctx, &mut app, &theme);
                if ctx.input(|i| i.key_pressed(Key::Escape)) && app.overwrite.is_none() && app.new_folder.is_none()
                {
                    app.outcome = Outcome::Cancel;
                }
                if app.outcome != Outcome::Pending {
                    close = true;
                }
            }
        }
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn draw(ctx: &egui::Context, app: &mut ChooserApp, theme: &OmarchyTheme) {
    egui::TopBottomPanel::top("title").show(ctx, |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(&app.req.title).size(18.0).strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button(super::fonts::up_glyph()).clicked() {
                app.parent();
            }
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.path_edit)
                    .desired_width(ui.available_width() - 220.0),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                let raw = app.path_edit.trim();
                if raw == RECENT_PLACE {
                    app.go_recent();
                } else {
                    let path = PathBuf::from(raw);
                    if path.is_dir() {
                        app.go_folder(path);
                    }
                }
            }
            ui.add(
                egui::TextEdit::singleline(&mut app.query)
                    .hint_text("Search")
                    .desired_width(180.0),
            );
            if ui.input(|i| i.key_released(Key::Enter)) && !app.query.is_empty() {
                app.refresh();
            }
        });
        ui.add_space(4.0);
    });

    egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
        if let Some(err) = &app.error {
            ui.colored_label(super::visuals::rgb(theme.red), err);
        }
        if !app.req.choices.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for (i, choice) in app.req.choices.iter().enumerate() {
                    if choice.options.is_empty() {
                        let mut on = app.choice_values[i] == "true";
                        if ui.checkbox(&mut on, &choice.label).changed() {
                            app.choice_values[i] = if on { "true" } else { "false" }.into();
                        }
                    } else {
                        ui.label(&choice.label);
                        egui::ComboBox::from_id_salt(format!("choice-{i}"))
                            .selected_text(&app.choice_values[i])
                            .show_ui(ui, |ui| {
                                for (id, label) in &choice.options {
                                    ui.selectable_value(&mut app.choice_values[i], id.clone(), label);
                                }
                            });
                    }
                }
            });
        }
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                app.outcome = Outcome::Cancel;
            }
            if ui.button("New folder").clicked() {
                app.new_folder = Some(String::new());
            }
            let mut hidden = app.show_hidden;
            if ui.checkbox(&mut hidden, "Hidden").changed() {
                app.show_hidden = hidden;
                app.refresh();
            }
            if !app.req.filters.is_empty() {
                let label = app
                    .filter_index
                    .and_then(|i| app.req.filters.get(i))
                    .map(|f| f.label.clone())
                    .unwrap_or_else(|| "All files".into());
                let filter_labels: Vec<(usize, String)> = app
                    .req
                    .filters
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (i, f.label.clone()))
                    .collect();
                let selected = app.filter_index;
                let mut chosen = None;
                egui::ComboBox::from_id_salt("filter")
                    .selected_text(label)
                    .show_ui(ui, |ui| {
                        for (i, name) in &filter_labels {
                            if ui.selectable_label(selected == Some(*i), name).clicked() {
                                chosen = Some(*i);
                            }
                        }
                    });
                if let Some(i) = chosen {
                    app.filter_index = Some(i);
                    app.refresh();
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(RichText::new(&app.req.accept_label).strong()).clicked() {
                    app.try_accept();
                }
                if app.req.mode == FileMode::Save || app.req.mode == FileMode::SaveFiles {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.filename)
                            .hint_text("File name")
                            .desired_width(260.0),
                    );
                }
            });
        });
    });

    egui::SidePanel::left("places")
        .resizable(true)
        .default_width(180.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Places").small().color(super::visuals::rgb(theme.muted)));
            if ui.selectable_label(app.recent_mode, "Recent").clicked() {
                app.go_recent();
            }
            for (label, path) in places() {
                let selected = !app.recent_mode && app.folder == path;
                if ui.selectable_label(selected, label).clicked() {
                    app.go_folder(path);
                }
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        let header = ui.horizontal(|ui| {
            let name = sort_heading("Name", app.sort_key == SortKey::Name, app.sort_reversed);
            if ui.button(RichText::new(name).strong()).clicked() {
                app.set_sort(SortKey::Name);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let date = sort_heading("Date", app.sort_key == SortKey::Time, app.sort_reversed);
                if ui.button(RichText::new(date).strong()).clicked() {
                    app.set_sort(SortKey::Time);
                }
                ui.add_space(80.0);
                let size = sort_heading("Size", app.sort_key == SortKey::Size, app.sort_reversed);
                if ui.button(RichText::new(size).strong()).clicked() {
                    app.set_sort(SortKey::Size);
                }
            });
        });
        ui.separator();
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let entries_len = app.entries.len();
                for i in 0..entries_len {
                    let (name, path, is_dir, size, modified) = {
                        let e = &app.entries[i];
                        (e.name.clone(), e.path.clone(), e.is_dir, e.size, e.modified)
                    };
                    let selected = app.selected.contains(&path);
                    let mut activate = false;
                    let mut toggle = false;
                    ui.horizontal(|ui| {
                        let glyph = super::fonts::file_glyph(is_dir, &name);
                        let label = format!("{glyph}  {name}");
                        let resp = ui.selectable_label(selected, label);
                        // egui reports clicked() on the double-click frame too;
                        // only treat it as a single click when it is not a double-click.
                        if resp.double_clicked() {
                            activate = true;
                        } else if resp.clicked() {
                            toggle = true;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format_time(modified));
                            ui.add_space(24.0);
                            ui.label(if is_dir {
                                String::new()
                            } else {
                                format_size(size)
                            });
                        });
                    });
                    if activate {
                        if is_dir {
                            app.enter(path);
                        } else {
                            app.selected = vec![path];
                            app.try_accept();
                        }
                    } else if toggle {
                        app.toggle_select(path.clone());
                        if !is_dir && app.req.mode == FileMode::Save {
                            if let Some(n) = path.file_name() {
                                app.filename = n.to_string_lossy().into_owned();
                            }
                        }
                    }
                }
            });
        let _ = header;
    });

    if ctx.input(|i| i.key_pressed(Key::Backspace)) && app.new_folder.is_none() {
        app.parent();
    }
    if ctx.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift) && app.overwrite.is_none() {
        app.try_accept();
    }
    if ctx.input(|i| i.key_pressed(Key::H) && i.modifiers.ctrl) {
        app.show_hidden = !app.show_hidden;
        app.refresh();
    }
    if ctx.input(|i| i.key_pressed(Key::L) && i.modifiers.ctrl) {
        // path bar is already editable
    }
    if ctx.input(|i| i.key_pressed(Key::F5)) {
        app.refresh();
    }

    if app.new_folder.is_some() {
        egui::Window::new("New folder")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(name) = app.new_folder.as_mut() {
                    ui.add(egui::TextEdit::singleline(name).hint_text("Folder name"));
                }
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        app.create_folder();
                    }
                    if ui.button("Cancel").clicked() {
                        app.new_folder = None;
                    }
                });
            });
    }

    if let Some(path) = app.overwrite.clone() {
        egui::Window::new("Replace file?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} already exists. Replace it?",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Replace").clicked() {
                        app.confirm_overwrite();
                    }
                    if ui.button("Cancel").clicked() {
                        app.overwrite = None;
                    }
                });
            });
    }

    let _ = Sense::click();
}

fn sort_heading(label: &str, active: bool, reversed: bool) -> String {
    if !active {
        return label.to_string();
    }
    format!("{label} {}", if reversed { "↓" } else { "↑" })
}

fn format_size(n: u64) -> String {
    const KB: f64 = 1024.0;
    let n = n as f64;
    if n < KB {
        format!("{n:.0} B")
    } else if n < KB * KB {
        format!("{:.1} KB", n / KB)
    } else if n < KB * KB * KB {
        format!("{:.1} MB", n / (KB * KB))
    } else {
        format!("{:.1} GB", n / (KB * KB * KB))
    }
}

fn format_time(ts: Option<std::time::SystemTime>) -> String {
    let Some(ts) = ts else {
        return String::new();
    };
    let dt = chrono::DateTime::<chrono::Local>::from(ts);
    dt.format("%Y-%m-%d %H:%M").to_string()
}
