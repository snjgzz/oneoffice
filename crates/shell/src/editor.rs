//! 标签，以及以后嵌入 wordcraft / gridcraft / deckcraft 的接口。
//!
//! 三个上游应用都在 `crates/ui-egui` 里提供 eframe 应用结构体，构造函数是
//! `WordApp::new(session, services)`、`SheetApp::new(session, services)`、
//! `SlideApp::new(session, services)`，并且都有 `fn ui(&mut self, ui: &mut egui::Ui)`。
//! gridcraft 的类型名是 `SheetApp`（设计稿里写的 GridApp 是同一层）。
//!
//! 它们现在假定自己独占窗口。嵌进来时包一层，实现 [`DocumentEditor`]，
//! 把 `ui()` 画到壳给的区域里，而不是再开一个原生窗口。

use std::path::{Path, PathBuf};

use crate::kind::DocKind;

pub trait DocumentEditor: Send {
    fn kind(&self) -> DocKind;
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// 一个文档标签。`editor` 现在是占位，以后换成对应的上游应用。
pub struct DocTab {
    id: u64,
    title: String,
    path: Option<PathBuf>,
    editor: Box<dyn DocumentEditor>,
}

impl DocTab {
    pub fn new(
        id: u64,
        title: impl Into<String>,
        path: Option<PathBuf>,
        editor: Box<dyn DocumentEditor>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            path,
            editor,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
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

    pub fn editor_ui(&mut self, ui: &mut egui::Ui) {
        if let Self::Word(doc) | Self::Sheet(doc) | Self::Deck(doc) = self {
            doc.editor_mut().ui(ui);
        }
    }
}
