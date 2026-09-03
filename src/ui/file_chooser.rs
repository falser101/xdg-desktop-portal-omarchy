use super::chrome::{
    self, body_text, caption_text, destructive_button, dim_overlay, hairline, labeled_toggle,
    primary_button, search_field, secondary_button, section_label, sheet_frame, sidebar_item,
    title_text, toolbar_glyph_button, trailing_actions, well_edit, BODY_PT, BUTTON_H, CAPTION_PT,
    ROW_H, ROW_ICON, SIDEBAR_W,
};
use super::glyphs::{self, Glyph};
use super::icons::IconCache;
use super::thumbs::ThumbCache;
use super::{cancelled, run_native_sized};
use crate::desktop::file_icon_names;
use crate::dict::Choice;
use crate::filters::FileFilter;
use crate::paths::{home_dir, places, recent_files, unique_path, RECENT_PLACE};
use crate::theme::OmarchyTheme;
use egui::{
    Align, CornerRadius, FontFamily, FontId, Frame, Key, Layout, Margin, Pos2, Rect, RichText,
    ScrollArea, Sense, Stroke, Vec2,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
    let _ = run_native_sized(
        title,
        [980.0, 660.0],
        [560.0, 400.0],
        ChooserUi {
            state: ui_state,
            token,
            icons: IconCache::default(),
            thumbs: ThumbCache::default(),
            theme: OmarchyTheme::load(),
        },
    );
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
    /// Swallow leftover clicks after entering a folder (KDE-style click-to-enter).
    ignore_list_until: Option<Instant>,
    editing_path: bool,
    last_query: String,
    search_focused: bool,
    path_focused: bool,
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
            ignore_list_until: None,
            editing_path: false,
            last_query: String::new(),
            search_focused: false,
            path_focused: false,
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
            self.selected = vec![path];
        }
    }

    fn toggle_select(&mut self, path: PathBuf) {
        if let Some(i) = self.selected.iter().position(|p| *p == path) {
            self.selected.remove(i);
        } else {
            self.selected.push(path);
        }
    }

    fn select_range(&mut self, path: PathBuf) {
        let Some(end) = self.entries.iter().position(|e| e.path == path) else {
            self.selected = vec![path];
            return;
        };
        let start = self
            .selected
            .last()
            .and_then(|p| self.entries.iter().position(|e| e.path == *p))
            .unwrap_or(end);
        let (a, b) = if start <= end { (start, end) } else { (end, start) };
        self.selected = self.entries[a..=b]
            .iter()
            .filter(|e| if self.req.directory { e.is_dir } else { !e.is_dir })
            .map(|e| e.path.clone())
            .collect();
        if self.selected.is_empty() {
            self.selected = vec![path];
        }
    }

    /// Click a directory to enter it. Click a file to select it (replacing the
    /// previous selection). Ctrl/Cmd+click toggles when multiple is allowed;
    /// Shift+click selects a range. Double-click accepts.
    fn on_list_click(&mut self, path: PathBuf, is_dir: bool, double: bool, modifiers: egui::Modifiers) {
        if is_dir {
            self.go_folder(path);
            self.ignore_list_until = Some(Instant::now() + Duration::from_millis(400));
            return;
        }
        if double {
            self.selected = vec![path];
            self.try_accept();
            return;
        }
        if self.req.multiple && modifiers.shift {
            self.select_range(path.clone());
        } else if self.req.multiple && modifiers.command {
            self.toggle_select(path.clone());
        } else {
            self.selected = vec![path.clone()];
        }
        if self.req.mode == FileMode::Save {
            if let Some(n) = path.file_name() {
                self.filename = n.to_string_lossy().into_owned();
            }
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
    icons: IconCache,
    thumbs: ThumbCache,
    theme: OmarchyTheme,
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
        self.thumbs.poll(ctx);

        let mut close = false;
        {
            let mut app = self.state.lock().unwrap();
            if app.outcome != Outcome::Pending {
                close = true;
            } else {
                draw(ctx, &mut app, &self.theme, &mut self.icons, &mut self.thumbs);
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    if app.overwrite.is_some() {
                        app.overwrite = None;
                    } else if app.new_folder.is_some() {
                        app.new_folder = None;
                    } else if !app.editing_path {
                        app.outcome = Outcome::Cancel;
                    }
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

fn draw(
    ctx: &egui::Context,
    app: &mut ChooserApp,
    theme: &OmarchyTheme,
    icons: &mut IconCache,
    thumbs: &mut ThumbCache,
) {
    if app.query != app.last_query {
        app.last_query = app.query.clone();
        app.refresh();
    }

    let sidebar_fill = ctx.style().visuals.faint_bg_color;
    let panel_fill = ctx.style().visuals.panel_fill;

    egui::TopBottomPanel::top("title")
        .frame(
            Frame::new()
                .fill(panel_fill)
                .inner_margin(Margin::symmetric(14, 10)),
        )
        .show(ctx, |ui| {
            ui.add(egui::Label::new(title_text(&app.req.title)).truncate());
            ui.add_space(10.0);
            draw_toolbar(ui, app);
        });

    egui::TopBottomPanel::bottom("actions")
        .frame(
            Frame::new()
                .fill(panel_fill)
                .inner_margin(Margin::symmetric(14, 10)),
        )
        .show(ctx, |ui| {
            draw_footer(ui, app, theme);
        });

    let win_w = ctx.screen_rect().width();
    let side_w = if win_w < 700.0 { 148.0 } else { SIDEBAR_W };
    egui::SidePanel::left("places")
        .resizable(true)
        .default_width(side_w)
        .width_range(132.0..=280.0)
        .frame(
            Frame::new()
                .fill(sidebar_fill)
                .inner_margin(Margin::symmetric(8, 10)),
        )
        .show(ctx, |ui| {
            section_label(ui, "Favorites");
            if sidebar_item(ui, Glyph::Recents, "Recents", app.recent_mode).clicked() {
                app.go_recent();
            }
            ui.add_space(6.0);
            section_label(ui, "Locations");
            for (label, path) in places() {
                let selected = !app.recent_mode && app.folder == path;
                if sidebar_item(ui, glyphs::for_place(&label), &label, selected).clicked() {
                    app.go_folder(path);
                }
            }
        });

    egui::CentralPanel::default()
        .frame(Frame::new().fill(panel_fill).inner_margin(Margin::symmetric(8, 6)))
        .show(ctx, |ui| {
            draw_column_header(ui, app);
            ui.add_space(2.0);
            hairline(ui);
            ui.add_space(2.0);
            let entries_len = app.entries.len();
            if entries_len == 0 {
                ui.add_space(48.0);
                ui.vertical_centered(|ui| {
                    ui.label(caption_text(
                        if app.query.is_empty() {
                            "No Items"
                        } else {
                            "No Matching Items"
                        },
                        chrome::muted_of(ui),
                    ));
                });
            } else {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, ROW_H, entries_len, |ui, range| {
                        let blocked = app.ignore_list_until.is_some_and(|t| Instant::now() < t);
                        let modifiers = ui.input(|i| i.modifiers);
                        let mut click = None;
                        for i in range {
                            let (name, path, is_dir, size, modified) = {
                                let e = &app.entries[i];
                                (e.name.clone(), e.path.clone(), e.is_dir, e.size, e.modified)
                            };
                            let selected = app.selected.contains(&path);
                            let row_resp = list_row(
                                ui, icons, thumbs, &name, &path, is_dir, size, modified, selected,
                            );
                            if blocked || click.is_some() {
                                continue;
                            }
                            if is_dir && row_resp.clicked() {
                                click = Some((path, true, false));
                            } else if row_resp.double_clicked() {
                                click = Some((path, is_dir, true));
                            } else if row_resp.clicked() {
                                click = Some((path, is_dir, false));
                            }
                        }
                        if let Some((path, is_dir, double)) = click {
                            app.on_list_click(path, is_dir, double, modifiers);
                        }
                    });
            }
        });

    if ctx.input(|i| i.key_pressed(Key::Backspace))
        && app.new_folder.is_none()
        && !app.search_focused
        && !app.path_focused
        && !app.editing_path
    {
        app.parent();
    }
    if ctx.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift)
        && app.overwrite.is_none()
        && app.new_folder.is_none()
        && !app.search_focused
        && !app.path_focused
        && !app.editing_path
    {
        app.try_accept();
    }
    if ctx.input(|i| i.key_pressed(Key::H) && i.modifiers.ctrl) {
        app.show_hidden = !app.show_hidden;
        app.refresh();
    }
    if ctx.input(|i| i.key_pressed(Key::L) && i.modifiers.ctrl) {
        app.editing_path = true;
        app.path_edit = if app.recent_mode {
            RECENT_PLACE.to_string()
        } else {
            app.folder.display().to_string()
        };
    }
    if ctx.input(|i| i.key_pressed(Key::F5)) {
        app.refresh();
    }

    if app.new_folder.is_some() || app.overwrite.is_some() {
        if dim_overlay(ctx).clicked() {
            app.new_folder = None;
            app.overwrite = None;
        }
    }

    if app.new_folder.is_some() {
        egui::Window::new("New Folder")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .frame(sheet_frame(ctx))
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.label(title_text("New Folder"));
                ui.add_space(6.0);
                ui.label(caption_text(
                    "Enter a name for the folder.",
                    chrome::muted_of(ui),
                ));
                ui.add_space(12.0);
                if let Some(name) = app.new_folder.as_mut() {
                    well_edit(ui, name, "Folder Name", ui.available_width());
                }
                ui.add_space(16.0);
                let (cancel, create) = trailing_actions(ui, "Cancel", "Create");
                if create {
                    app.create_folder();
                }
                if cancel {
                    app.new_folder = None;
                }
            });
    }

    if let Some(path) = app.overwrite.clone() {
        egui::Window::new("Replace File?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .frame(sheet_frame(ctx))
            .show(ctx, |ui| {
                ui.set_min_width(340.0);
                ui.label(title_text("Replace File?"));
                ui.add_space(8.0);
                ui.label(body_text(format!(
                    "“{}” already exists. Do you want to replace it?",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )));
                ui.add_space(16.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if destructive_button(ui, "Replace").clicked() {
                        app.confirm_overwrite();
                    }
                    ui.add_space(8.0);
                    if secondary_button(ui, "Cancel").clicked() {
                        app.overwrite = None;
                    }
                });
            });
    }
}

fn draw_toolbar(ui: &mut egui::Ui, app: &mut ChooserApp) {
    let total = ui.available_width();
    let stacked = total < 560.0;
    if stacked {
        draw_path_bar(ui, app);
        ui.add_space(8.0);
        let search = search_field(ui, &mut app.query, "Search", ui.available_width());
        app.search_focused = search.has_focus();
        return;
    }
    ui.horizontal(|ui| {
        ui.set_min_height(BUTTON_H);
        let search_w = (total * 0.30).clamp(160.0, 240.0);
        let path_w = (total - search_w - 10.0).max(200.0);
        ui.allocate_ui_with_layout(
            Vec2::new(path_w, BUTTON_H),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_max_width(path_w);
                draw_path_bar(ui, app);
            },
        );
        let search = search_field(ui, &mut app.query, "Search", search_w);
        app.search_focused = search.has_focus();
    });
}

fn draw_path_bar(ui: &mut egui::Ui, app: &mut ChooserApp) {
    let well = ui.visuals().extreme_bg_color;
    let stroke = Stroke::new(1.0_f32, ui.visuals().widgets.noninteractive.bg_stroke.color);
    Frame::new()
        .fill(well)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(4, 0))
        .show(ui, |ui| {
            ui.set_min_height(BUTTON_H);
            ui.set_max_height(BUTTON_H);
            ui.horizontal(|ui| {
                ui.set_min_height(BUTTON_H);
                if toolbar_glyph_button(ui, Glyph::ChevronLeft) {
                    app.parent();
                }
                let sep = ui.available_height().max(18.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, sep * 0.55), Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    0.0,
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                );
                if app.editing_path {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut app.path_edit)
                            .desired_width(ui.available_width().max(80.0))
                            .hint_text("Path")
                            .font(FontId::proportional(BODY_PT))
                            .frame(false),
                    );
                    app.path_focused = resp.has_focus();
                    if resp.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                        commit_path_edit(app);
                    }
                    if ui.input(|i| i.key_pressed(Key::Escape)) {
                        app.editing_path = false;
                        app.path_edit = if app.recent_mode {
                            RECENT_PLACE.to_string()
                        } else {
                            app.folder.display().to_string()
                        };
                    }
                } else {
                    app.path_focused = false;
                    draw_breadcrumbs(ui, app);
                }
            });
        });
}

fn draw_footer(ui: &mut egui::Ui, app: &mut ChooserApp, theme: &OmarchyTheme) {
    if let Some(err) = &app.error {
        ui.colored_label(super::visuals::rgb(theme.red), err);
        ui.add_space(6.0);
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
                    ui.label(caption_text(&choice.label, chrome::muted_of(ui)));
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
        ui.add_space(8.0);
    }

    ui.spacing_mut().interact_size.y = BUTTON_H;
    ui.spacing_mut().button_padding = egui::vec2(14.0, 8.0);
    let save = matches!(app.req.mode, FileMode::Save | FileMode::SaveFiles);
    let w = ui.available_width();
    let stacked_name = save && w < 860.0;
    let stacked_actions = w < 640.0;

    if stacked_name {
        well_edit(
            ui,
            &mut app.filename,
            "Save As",
            (w - 8.0).max(120.0),
        );
        ui.add_space(8.0);
    }

    if stacked_actions {
        draw_footer_tools(ui, app);
        ui.add_space(8.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.set_min_height(BUTTON_H);
            if primary_button(ui, &app.req.accept_label).clicked() {
                app.try_accept();
            }
            ui.add_space(8.0);
            if secondary_button(ui, "Cancel").clicked() {
                app.outcome = Outcome::Cancel;
            }
        });
        return;
    }

    ui.horizontal(|ui| {
        ui.set_min_height(BUTTON_H);
        draw_footer_tools(ui, app);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.set_min_height(BUTTON_H);
            if primary_button(ui, &app.req.accept_label).clicked() {
                app.try_accept();
            }
            ui.add_space(8.0);
            if secondary_button(ui, "Cancel").clicked() {
                app.outcome = Outcome::Cancel;
            }
            if save && !stacked_name {
                ui.add_space(10.0);
                let remain = (ui.available_width() - 10.0).clamp(140.0, 280.0);
                well_edit(ui, &mut app.filename, "Save As", remain);
            }
        });
    });
}

fn draw_footer_tools(ui: &mut egui::Ui, app: &mut ChooserApp) {
    if secondary_button(ui, "New Folder").clicked() {
        app.new_folder = Some(String::new());
    }
    ui.add_space(6.0);
    let mut hidden = app.show_hidden;
    if labeled_toggle(ui, &mut hidden, "Hidden") {
        app.show_hidden = hidden;
        app.refresh();
    }
    if app.req.filters.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.label(caption_text("Format", chrome::muted_of(ui)));
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
    let combo_w = if ui.available_width() < 180.0 { 110.0 } else { 148.0 };
    egui::ComboBox::from_id_salt("filter")
        .width(combo_w)
        .selected_text(RichText::new(label).size(BODY_PT))
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

fn commit_path_edit(app: &mut ChooserApp) {
    app.editing_path = false;
    let raw = app.path_edit.trim();
    if raw == RECENT_PLACE {
        app.go_recent();
    } else {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            app.go_folder(path);
        } else {
            app.path_edit = app.folder.display().to_string();
        }
    }
}

fn draw_breadcrumbs(ui: &mut egui::Ui, app: &mut ChooserApp) {
    let crumbs = if app.recent_mode {
        vec![("Recents".to_string(), PathBuf::new())]
    } else {
        breadcrumbs(&app.folder)
    };
    let visible = collapse_crumbs(&crumbs);
    let mut go = None;
    let mut edit = false;
    ui.spacing_mut().item_spacing.x = 2.0;
    for (i, (label, path, ellipsis)) in visible.iter().enumerate() {
        if i > 0 {
            ui.label(
                RichText::new("/")
                    .size(11.0)
                    .color(chrome::muted_of(ui)),
            );
        }
        let last = i + 1 == visible.len();
        let shown = truncate_crumb(label);
        let resp = crumb_chip(ui, &shown, last && !ellipsis);
        if resp.clicked() {
            if *ellipsis || last {
                edit = true;
            } else if !path.as_os_str().is_empty() {
                go = Some(path.clone());
            }
        }
    }
    if let Some(path) = go {
        app.go_folder(path);
    }
    if edit {
        app.editing_path = true;
        app.path_edit = if app.recent_mode {
            RECENT_PLACE.to_string()
        } else {
            app.folder.display().to_string()
        };
    }
}

fn truncate_crumb(label: &str) -> String {
    let n = label.chars().count();
    if n <= 22 {
        return label.to_string();
    }
    let head: String = label.chars().take(19).collect();
    format!("{head}…")
}

fn collapse_crumbs(crumbs: &[(String, PathBuf)]) -> Vec<(String, PathBuf, bool)> {
    if crumbs.len() <= 3 {
        return crumbs
            .iter()
            .map(|(l, p)| (l.clone(), p.clone(), false))
            .collect();
    }
    let first = crumbs.first().cloned().unwrap();
    let last = crumbs.last().cloned().unwrap();
    vec![
        (first.0, first.1, false),
        ("…".into(), PathBuf::new(), true),
        (last.0, last.1, false),
    ]
}

fn crumb_chip(ui: &mut egui::Ui, label: &str, current: bool) -> egui::Response {
    let font = if current {
        FontId::new(BODY_PT, chrome::semibold())
    } else {
        FontId::proportional(BODY_PT)
    };
    let color = if current {
        ui.visuals().text_color()
    } else {
        chrome::muted_of(ui)
    };
    let galley = ui.fonts(|f| f.layout_no_wrap(label.to_owned(), font, color));
    let pad_x = 8.0;
    let size = Vec2::new(galley.size().x + pad_x * 2.0, BUTTON_H - 8.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), ui.visuals().widgets.hovered.bg_fill);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let text_pos = Pos2::new(
        rect.left() + pad_x,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, galley, color);
    resp
}

fn breadcrumbs(folder: &Path) -> Vec<(String, PathBuf)> {
    let home = home_dir();
    let mut chain = Vec::new();
    let mut cur = folder.to_path_buf();
    loop {
        let label = if cur == home {
            "Home".to_string()
        } else if cur.as_os_str() == "/" {
            "Computer".to_string()
        } else {
            cur.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| cur.display().to_string())
        };
        chain.push((label, cur.clone()));
        if cur == home || cur.parent().is_none() {
            break;
        }
        let Some(parent) = cur.parent() else { break };
        if parent == cur {
            break;
        }
        cur = parent.to_path_buf();
    }
    chain.reverse();
    if chain.iter().any(|(_, p)| *p == home) {
        chain.retain(|(_, p)| *p == home || p.starts_with(&home));
    }
    chain
}

fn draw_column_header(ui: &mut egui::Ui, app: &mut ChooserApp) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), Sense::hover());
    let muted = chrome::muted_of(ui);
    let font = FontId::new(CAPTION_PT, FontFamily::Proportional);
    let date_w = 108.0;
    let size_w = 72.0;
    let pad = 10.0;
    let date_x = rect.right() - pad - date_w;
    let size_x = date_x - size_w;
    let name = sort_heading("Name", app.sort_key == SortKey::Name, app.sort_reversed);
    let size = sort_heading("Size", app.sort_key == SortKey::Size, app.sort_reversed);
    let date = sort_heading("Date", app.sort_key == SortKey::Time, app.sort_reversed);

    let name_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, rect.top()),
        Pos2::new(size_x - 8.0, rect.bottom()),
    );
    let size_rect = Rect::from_min_max(
        Pos2::new(size_x, rect.top()),
        Pos2::new(date_x - 8.0, rect.bottom()),
    );
    let date_rect = Rect::from_min_max(
        Pos2::new(date_x, rect.top()),
        Pos2::new(rect.right() - pad, rect.bottom()),
    );
    let name_r = ui.interact(name_rect, ui.id().with("sort-name"), Sense::click());
    let size_r = ui.interact(size_rect, ui.id().with("sort-size"), Sense::click());
    let date_r = ui.interact(date_rect, ui.id().with("sort-date"), Sense::click());
    ui.painter()
        .text(name_rect.left_center(), egui::Align2::LEFT_CENTER, name, font.clone(), muted);
    ui.painter()
        .text(size_rect.right_center(), egui::Align2::RIGHT_CENTER, size, font.clone(), muted);
    ui.painter()
        .text(date_rect.right_center(), egui::Align2::RIGHT_CENTER, date, font, muted);
    if name_r.clicked() {
        app.set_sort(SortKey::Name);
    }
    if size_r.clicked() {
        app.set_sort(SortKey::Size);
    }
    if date_r.clicked() {
        app.set_sort(SortKey::Time);
    }
}

/// Paint a file row with no child widgets so the row Sense::click is not stolen.
fn list_row(
    ui: &mut egui::Ui,
    icons: &mut IconCache,
    thumbs: &mut ThumbCache,
    name: &str,
    path: &Path,
    is_dir: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
    selected: bool,
) -> egui::Response {
    ui.push_id(path, |ui| list_row_inner(ui, icons, thumbs, name, path, is_dir, size, modified, selected))
        .inner
}

fn list_row_inner(
    ui: &mut egui::Ui,
    icons: &mut IconCache,
    thumbs: &mut ThumbCache,
    name: &str,
    path: &Path,
    is_dir: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
    selected: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_H), Sense::click());
    let vis = ui.visuals();
    let row = Rect::from_min_max(
        Pos2::new(rect.left() + 4.0, rect.top() + 1.0),
        Pos2::new(rect.right() - 4.0, rect.bottom() - 1.0),
    );
    if selected {
        ui.painter()
            .rect_filled(row, CornerRadius::same(8), vis.selection.bg_fill);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(row, CornerRadius::same(8), vis.widgets.hovered.bg_fill);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let pad = 10.0;
    let font = FontId::proportional(BODY_PT);
    let caption = FontId::proportional(chrome::CAPTION_PT);
    let fg = vis.text_color();
    let muted = vis.weak_text_color();
    let icon_rect = Rect::from_center_size(
        Pos2::new(row.left() + pad + ROW_ICON * 0.5, row.center().y),
        Vec2::splat(ROW_ICON),
    );
    if is_dir {
        glyphs::paint(ui, icon_rect, Glyph::Folder, fg);
    } else if !thumbs.paint_list(ui, path, false, icon_rect) {
        icons.paint_at(ui, &file_icon_names(false, path), icon_rect);
    }
    let date = format_time(modified);
    let date_galley = ui.fonts(|f| f.layout_no_wrap(date, caption.clone(), muted));
    let date_x = row.right() - pad - date_galley.size().x.max(96.0);
    let date_y = row.center().y - date_galley.size().y * 0.5;
    ui.painter()
        .galley(Pos2::new(date_x, date_y), date_galley, muted);

    let size_txt = if is_dir {
        String::new()
    } else {
        format_size(size)
    };
    let size_galley = ui.fonts(|f| f.layout_no_wrap(size_txt, caption, muted));
    let size_x = date_x - 16.0 - size_galley.size().x.max(48.0);
    let size_y = row.center().y - size_galley.size().y * 0.5;
    ui.painter()
        .galley(Pos2::new(size_x, size_y), size_galley, muted);

    let name_x = icon_rect.right() + 8.0;
    let name_w = (size_x - 12.0 - name_x).max(0.0);
    let name_galley = ui.fonts(|f| f.layout(name.to_owned(), font, fg, name_w));
    let name_y = row.center().y - name_galley.size().y * 0.5;
    ui.painter()
        .galley(Pos2::new(name_x, name_y), name_galley, fg);
    resp
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
    let now = chrono::Local::now();
    let today = now.date_naive();
    let day = dt.date_naive();
    if day == today {
        dt.format("%H:%M").to_string()
    } else if day == today - chrono::Duration::days(1) {
        "Yesterday".to_string()
    } else if dt.format("%Y").to_string() == now.format("%Y").to_string() {
        dt.format("%b %d").to_string()
    } else {
        dt.format("%b %d, %Y").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumbs_start_at_home() {
        let home = home_dir();
        let docs = home.join("Documents");
        let crumbs = breadcrumbs(&docs);
        assert_eq!(crumbs.first().map(|(l, _)| l.as_str()), Some("Home"));
        assert!(crumbs.iter().any(|(l, _)| l == "Documents" || docs.ends_with(l)));
    }

    #[test]
    fn collapse_keeps_short_paths() {
        let crumbs = vec![
            ("Home".into(), PathBuf::from("/home")),
            ("Downloads".into(), PathBuf::from("/home/Downloads")),
        ];
        assert_eq!(collapse_crumbs(&crumbs).len(), 2);
    }

    #[test]
    fn collapse_long_paths_to_ellipsis() {
        let crumbs: Vec<(String, PathBuf)> = (0..6)
            .map(|i| (format!("d{i}"), PathBuf::from(format!("/{i}"))))
            .collect();
        let vis = collapse_crumbs(&crumbs);
        assert_eq!(vis.len(), 3);
        assert!(vis[1].2);
        assert_eq!(vis[2].0, "d5");
    }

    #[test]
    fn today_formats_as_time() {
        let now = std::time::SystemTime::now();
        let s = format_time(Some(now));
        assert!(s.contains(':'), "today should be HH:MM, got {s}");
        assert!(!s.contains("Yesterday"));
    }
}
