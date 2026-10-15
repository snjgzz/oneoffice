//! OneOffice 主窗口壳。
//!
//! 文档、表格、演示共用一个原生窗口。文字 / 表格 / 演示页目前是占位，
//! 以后换成 wordcraft 的 `WordApp`、gridcraft 的 `SheetApp`、deckcraft 的 `SlideApp`。

mod app;
mod chrome;
mod document;
mod editor;
mod home;
mod kind;
mod platform;
mod recent;
mod theme;

pub use editor::{DocTab, DocumentEditor, Tab};
pub use egui;
pub use kind::DocKind;

/// 启动主窗口。`paths` 里扩展名为 docx / xlsx / pptx 的现有文件会打开成标签，并写入最近列表。
pub fn run(paths: Vec<String>) -> eframe::Result {
    app::run(paths)
}
