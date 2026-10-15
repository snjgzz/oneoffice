//! 壳的状态：标签、最近文件、快捷键。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{Modifiers, Pos2, ViewportCommand};

use crate::chrome;
use crate::document::PlaceholderEditor;
use crate::editor::{DocTab, Tab};
use crate::kind::DocKind;
use crate::platform;
use crate::recent::RecentList;
use crate::sheet::{self, SheetEditor};
use crate::theme::{self, BG};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HomeView {
    Home,
    Recent,
    Settings,
}

pub(crate) struct Shell {
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active: usize,
    pub(crate) home_view: HomeView,
    pub(crate) search: String,
    pub(crate) focus_search: bool,
    pub(crate) recent: RecentList,
    pub(crate) recent_path: PathBuf,
    next_id: u64,
    untitled: [u32; 3],
    pub(crate) menu: Option<Pos2>,
    pub(crate) menu_just_opened: bool,
    sent_title: String,
    sheet_opens: Arc<Mutex<Vec<PathBuf>>>,
    pending_close: Option<usize>,
    logic_tab: usize,
}

pub(crate) fn run(paths: Vec<String>) -> eframe::Result {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let options = eframe::NativeOptions {
        viewport: viewport(),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "OneOffice",
        options,
        Box::new(move |cc| {
            theme::install(&cc.egui_ctx);
            sheet::register_cjk_font();
            Ok(Box::new(Shell::new(paths)) as Box<dyn eframe::App>)
        }),
    )
}

fn viewport() -> egui::ViewportBuilder {
    let viewport = egui::ViewportBuilder::default()
        .with_title("OneOffice")
        .with_app_id("OneOffice")
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([1024.0, 680.0]);
    if cfg!(target_os = "macos") {
        viewport
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
    } else {
        // fullsize content view 只在 macOS 上保留系统红绿灯。Linux 去掉系统标题栏，自己画三颗按钮。
        viewport.with_decorations(false).with_resizable(true)
    }
}

impl Shell {
    fn new(paths: Vec<PathBuf>) -> Self {
        let recent_path = platform::recent_file();
        let mut shell = Self {
            tabs: vec![Tab::Home],
            active: 0,
            home_view: HomeView::Home,
            search: String::new(),
            focus_search: false,
            recent: RecentList::load(&recent_path),
            recent_path,
            next_id: 1,
            untitled: [0; 3],
            menu: None,
            menu_just_opened: false,
            sent_title: String::new(),
            sheet_opens: Arc::new(Mutex::new(Vec::new())),
            pending_close: None,
            logic_tab: 0,
        };
        for path in paths {
            shell.open_existing(path);
        }
        shell
    }

    pub(crate) fn toggle_menu(&mut self, pos: Pos2) {
        if self.menu.is_some() && !self.menu_just_opened {
            self.menu = None;
        } else {
            self.menu = Some(pos);
            self.menu_just_opened = true;
        }
    }

    pub(crate) fn new_untitled(&mut self, kind: DocKind) {
        self.untitled[kind.index()] += 1;
        let n = self.untitled[kind.index()];
        let title = if n == 1 {
            format!("未命名{}.{}", kind.label(), kind.extension())
        } else {
            format!("未命名{} {}.{}", kind.label(), n, kind.extension())
        };
        self.push_tab(kind, title, None);
    }

    pub(crate) fn open_dialog(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("打开文件")
            .add_filter(
                "Office 文档",
                &["docx", "xlsx", "xlsm", "xls", "xlsb", "ods", "pptx"],
            )
            .add_filter("表格", &["xlsx", "xlsm", "xls", "xlsb", "ods"])
            .add_filter("文字", &["docx"])
            .add_filter("演示", &["pptx"])
            .pick_file();
        if let Some(path) = picked {
            self.open_existing(path);
        }
    }

    pub(crate) fn open_existing(&mut self, path: PathBuf) {
        if !self.recent.record_open(path.clone()) {
            return;
        }
        self.recent.save(&self.recent_path);
        let stored = self.recent.files.first().map(|file| file.path.clone());
        let Some(path) = stored else {
            return;
        };
        let Some(kind) = DocKind::from_path(&path) else {
            return;
        };
        let title = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.push_tab(kind, title, Some(path));
    }

    pub(crate) fn remove_recent(&mut self, path: &std::path::Path) {
        if self.recent.remove(path) {
            self.recent.save(&self.recent_path);
        }
    }

    pub(crate) fn close_tab(&mut self, index: usize) {
        if index == 0 || index >= self.tabs.len() {
            return;
        }
        if self.tabs[index].dirty() {
            self.pending_close = Some(index);
            return;
        }
        self.remove_tab(index);
    }

    fn remove_tab(&mut self, index: usize) {
        if index == 0 || index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.active > index {
            self.active -= 1;
        } else if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        if self.logic_tab > index {
            self.logic_tab -= 1;
        } else if self.logic_tab >= self.tabs.len() {
            self.logic_tab = self.tabs.len() - 1;
        }
    }

    fn confirm_save_and_close(&mut self) {
        let Some(index) = self.pending_close else {
            return;
        };
        let saved = self.tabs.get_mut(index).is_some_and(Tab::save);
        if saved {
            self.pending_close = None;
            self.remove_tab(index);
        }
    }

    fn push_tab(&mut self, kind: DocKind, title: String, path: Option<PathBuf>) {
        if let Some(path) = &path
            && let Some(index) = self
                .tabs
                .iter()
                .position(|tab| tab.path() == Some(path.as_path()))
        {
            self.active = index;
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        let editor: Box<dyn crate::editor::DocumentEditor> = match kind {
            DocKind::Sheet => {
                let sheet = match &path {
                    Some(path) => SheetEditor::open(id, path, Arc::clone(&self.sheet_opens)),
                    None => SheetEditor::blank(id, title, Arc::clone(&self.sheet_opens)),
                };
                Box::new(sheet)
            }
            DocKind::Word | DocKind::Deck => Box::new(PlaceholderEditor::new(id, kind, title)),
        };
        let doc = DocTab::new(id, path, editor);
        let tab = match kind {
            DocKind::Word => Tab::Word(doc),
            DocKind::Sheet => Tab::Sheet(doc),
            DocKind::Deck => Tab::Deck(doc),
        };
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
    }

    fn close_active(&mut self) {
        self.close_tab(self.active);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.pending_close.is_some() {
            let mut cancel = false;
            ctx.input_mut(|input| {
                cancel = input.consume_key(Modifiers::NONE, egui::Key::Escape);
                input.events.retain(|event| {
                    !matches!(event, egui::Event::Key { .. } | egui::Event::Text(_))
                });
            });
            if cancel {
                self.pending_close = None;
            }
            return;
        }
        let command = Modifiers::COMMAND;
        let mut home = false;
        let mut close = false;
        let mut open = false;
        let mut search = false;
        let mut tab_index = None;
        ctx.input_mut(|input| {
            if input.consume_key(command, egui::Key::T) {
                home = true;
            } else if input.consume_key(command, egui::Key::W) {
                close = true;
            } else if input.consume_key(command, egui::Key::O) {
                open = true;
            } else if input.consume_key(command, egui::Key::K) {
                search = true;
            } else {
                const KEYS: [egui::Key; 9] = [
                    egui::Key::Num1,
                    egui::Key::Num2,
                    egui::Key::Num3,
                    egui::Key::Num4,
                    egui::Key::Num5,
                    egui::Key::Num6,
                    egui::Key::Num7,
                    egui::Key::Num8,
                    egui::Key::Num9,
                ];
                for (index, key) in KEYS.into_iter().enumerate() {
                    if input.consume_key(command, key) {
                        tab_index = Some(index);
                        break;
                    }
                }
            }
        });
        if home {
            self.active = 0;
            self.home_view = HomeView::Home;
        }
        if search {
            self.active = 0;
            if self.home_view == HomeView::Settings {
                self.home_view = HomeView::Home;
            }
            self.focus_search = true;
        }
        if close {
            self.close_active();
        }
        if let Some(index) = tab_index
            && index < self.tabs.len()
        {
            self.active = index;
        }
        if open {
            self.open_dialog();
        }
    }

    fn sync_title(&mut self, ctx: &egui::Context) {
        let title = match self.tabs.get(self.active) {
            Some(Tab::Home) | None => "OneOffice".to_owned(),
            Some(tab) => format!("{} - OneOffice", tab.title()),
        };
        if title != self.sent_title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.sent_title = title;
        }
    }

    fn drain_sheet_opens(&mut self) {
        let queued = std::mem::take(
            &mut *self
                .sheet_opens
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for path in queued {
            self.open_existing(path);
        }
    }

    fn note_saved_path(&mut self) {
        let Some(path) = self.tabs.get(self.active).and_then(Tab::editor_path) else {
            return;
        };
        if self.tabs.get(self.active).and_then(Tab::path) == Some(path.as_path()) {
            return;
        }
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.set_path(Some(path.clone()));
        }
        if self.recent.record_open(path) {
            self.recent.save(&self.recent_path);
        }
    }

    fn show_close_prompt(&mut self, ctx: &egui::Context) {
        let Some(index) = self.pending_close else {
            return;
        };
        let title = self
            .tabs
            .get(index)
            .map(|tab| tab.title().to_owned())
            .unwrap_or_default();
        let mut action = None;
        egui::Modal::new(egui::Id::new("oneoffice-close")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(format!("「{title}」有未保存的更改。"));
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("保存").clicked() {
                    action = Some(1);
                }
                if ui.button("不保存").clicked() {
                    action = Some(2);
                }
                if ui.button("取消").clicked() {
                    action = Some(0);
                }
            });
        });
        if action == Some(1) {
            self.confirm_save_and_close();
        } else if action == Some(2) {
            if let Some(index) = self.pending_close.take() {
                self.remove_tab(index);
            }
        } else if action == Some(0) {
            self.pending_close = None;
        }
    }
}

impl eframe::App for Shell {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG.to_normalized_gamma_f32()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_sheet_opens();
        if self.active != self.logic_tab {
            egui::Popup::close_all(ctx);
            let focused = ctx.memory(|memory| memory.focused());
            if let Some(id) = focused {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
            self.logic_tab = self.active;
        }
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.logic(ctx);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ui.ctx());

        egui::Panel::top("tabbar")
            .exact_size(theme::TAB_BAR_H)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::TITLEBAR)
                    .inner_margin(egui::Margin::same(0)),
            )
            .show(ui, |ui| chrome::show_tab_bar(ui, self));

        let home = self.tabs.get(self.active).is_some_and(Tab::is_home);
        if home {
            egui::Panel::left("sidebar")
                .exact_size(theme::SIDEBAR_W)
                .resizable(false)
                .show_separator_line(true)
                .frame(
                    egui::Frame::new()
                        .fill(theme::SIDE)
                        .inner_margin(egui::Margin {
                            left: 12,
                            right: 12,
                            top: 16,
                            bottom: 12,
                        }),
                )
                .show(ui, |ui| self.show_sidebar(ui));
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(theme::BG)
                        .inner_margin(egui::Margin {
                            left: 36,
                            right: 36,
                            top: 24,
                            bottom: 24,
                        }),
                )
                .show(ui, |ui| self.show_home_main(ui));
        } else {
            egui::CentralPanel::default()
                .frame(egui::Frame::new().inner_margin(egui::Margin::same(0)))
                .show(ui, |ui| {
                    if let Some(tab) = self.tabs.get_mut(self.active) {
                        tab.editor_ui(ui);
                    }
                });
        }

        self.note_saved_path();
        self.show_new_menu(ui.ctx());
        self.show_close_prompt(ui.ctx());
        self.sync_title(ui.ctx());
    }
}
