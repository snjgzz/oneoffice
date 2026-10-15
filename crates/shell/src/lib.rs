//! OneOffice 主窗口壳。
//!
//! 文档、表格、演示共用一个原生窗口。文字是 wordcraft 的 `WordApp`，表格是
//! gridcraft 的 `SheetApp`，演示是 deckcraft 的 `SlideApp`。

mod app;
mod chrome;
mod deck;
mod editor;
mod home;
mod kind;
mod platform;
mod recent;
mod sheet;
mod theme;
mod word;

pub use editor::{DocTab, DocumentEditor, Tab};
pub use egui;
pub use kind::DocKind;

/// 启动主窗口。`paths` 里扩展名为 doc / docx / xlsx / xlsm / xls / xlsb / ods / pptx 的现有文件会打开成标签，并写入最近列表。
pub fn run(paths: Vec<String>) -> eframe::Result {
    app::run(paths)
}
