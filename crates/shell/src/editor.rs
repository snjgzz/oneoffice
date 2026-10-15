//! 标签，以及嵌入 wordcraft / gridcraft / deckcraft 的接口。
//!
//! 三个上游应用都在 `crates/ui-egui` 里提供 eframe 应用结构体，构造函数是
//! `WordApp::new(session, services)`、`SheetApp::new(session, services)`、
//! `SlideApp::new(session, services)`，并且都有 `fn ui(&mut self, ui: &mut egui::Ui)`。
//! gridcraft 的类型名是 `SheetApp`（设计稿里写的 GridApp 是同一层）。
//!
//! 编辑器只在界面线程上用，不要求 `Send`：文件对话框回调不是 `Send`。

use std::path::{Path, PathBuf};

/// 嵌入应用请壳做的事：另开一个标签，或关掉当前标签。
pub(crate) enum HostEvent {
    Open(PathBuf),
    Close(u64),
}

use crate::kind::DocKind;

pub trait DocumentEditor {
    fn kind(&self) -> DocKind;
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut egui::Ui);
    fn dirty(&self) -> bool {
        false
    }
    fn save(&mut self) -> bool {
        true
    }
    fn logic(&mut self, _ctx: &egui::Context) {}
    /// 切回这个标签时重新装上它的字体。三个应用共用一个 egui 字体表，后调用的 `set_fonts` 会清掉前一个的具名族（例如 deckcraft 的 `bold`）。
    fn prepare_embed_fonts(&mut self, _ctx: &egui::Context) {}
    fn path(&self) -> Option<PathBuf> {
        None
    }
}

/// 一个文档标签。文字、表格、演示各是一份上游应用。
pub struct DocTab {
    id: u64,
    path: Option<PathBuf>,
    editor: Box<dyn DocumentEditor>,
}

impl DocTab {
    pub fn new(id: u64, path: Option<PathBuf>, editor: Box<dyn DocumentEditor>) -> Self {
        Self { id, path, editor }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title(&self) -> &str {
        self.editor.title()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.path = path;
    }

    pub fn dirty(&self) -> bool {
        self.editor.dirty()
    }

    pub fn save(&mut self) -> bool {
        self.editor.save()
    }

    pub fn logic(&mut self, ctx: &egui::Context) {
        self.editor.logic(ctx);
    }

    pub fn editor_path(&self) -> Option<PathBuf> {
        self.editor.path()
    }

    pub fn editor_mut(&mut self) -> &mut dyn DocumentEditor {
        self.editor.as_mut()
    }
}

/// 首页固定在最左，不能关。其余变体各对应一种以后要嵌进来的应用。
pub enum Tab {
    Home,
    Word(DocTab),
    Sheet(DocTab),
    Deck(DocTab),
}

impl Tab {
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Home => "首页",
            Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) => doc.title(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Home => None,
            Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) => doc.path(),
        }
    }

    pub fn kind(&self) -> Option<DocKind> {
        match self {
            Self::Home => None,
            Self::Word(_) => Some(DocKind::Word),
            Self::Sheet(_) => Some(DocKind::Sheet),
            Self::Deck(_) => Some(DocKind::Deck),
        }
    }

    pub fn widget_id(&self) -> u64 {
        match self {
            Self::Home => 0,
            Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) => doc.id(),
        }
    }

    pub fn dirty(&self) -> bool {
        match self {
            Self::Home => false,
            Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) => doc.dirty(),
        }
    }

    pub fn save(&mut self) -> bool {
        match self {
            Self::Home => true,
            Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) => doc.save(),
        }
    }

    pub fn logic(&mut self, ctx: &egui::Context) {
        if let Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) = self {
            doc.logic(ctx);
        }
    }

    pub fn prepare_embed_fonts(&mut self, ctx: &egui::Context) {
        if let Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) = self {
            doc.editor_mut().prepare_embed_fonts(ctx);
        }
    }

    pub fn editor_path(&self) -> Option<PathBuf> {
        match self {
            Self::Home => None,
            Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) => doc.editor_path(),
        }
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        if let Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) = self {
            doc.set_path(path);
        }
    }

    pub fn editor_ui(&mut self, ui: &mut egui::Ui) {
        if let Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) = self {
            let id = doc.id();
            // 面板和默认控件的 id 跟着标签走。对话框仍用各应用的 `scope`。
            ui.push_id(("oneoffice-tab", id), |ui| doc.editor_mut().ui(ui));
        }
    }
}
