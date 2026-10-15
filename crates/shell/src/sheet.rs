//! gridcraft `SheetApp` 适配器。每个表格标签一份 `Session` 和一份 `SheetApp`。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gridcraft_engine::Session;
use gridcraft_ui_egui::{Services, SheetApp};
use serde_json::json;

use crate::editor::DocumentEditor;
use crate::kind::DocKind;
use crate::theme;

const SHEET_EXTENSIONS: &[&str] = &["xlsx", "xlsm", "xls", "xlsb", "ods"];

pub(crate) struct SheetEditor {
    app: SheetApp,
    title: String,
}

impl SheetEditor {
    pub(crate) fn blank(salt: u64, title: String, opens: Arc<Mutex<Vec<PathBuf>>>) -> Self {
        let mut session = Session::new();
        session.new_workbook();
        if let Some(doc) = session.active_mut() {
            doc.title.clone_from(&title);
        }
        let mut editor = Self::from_session(salt, session, opens);
        editor.title = title;
        editor
    }

    pub(crate) fn open(salt: u64, path: &Path, opens: Arc<Mutex<Vec<PathBuf>>>) -> Self {
        let mut editor = Self::from_session(salt, Session::new(), opens);
        match path.to_str() {
            Some(path) => editor.app.open_path(path),
            None => {
                editor.app.message =
                    Some(("OneOffice".into(), "文件路径不是 UTF-8，无法打开。".into()));
            }
        }
        editor.refresh();
        editor
    }

    fn from_session(salt: u64, session: Session, opens: Arc<Mutex<Vec<PathBuf>>>) -> Self {
        let mut app = SheetApp::new(session, services(opens));
        app.embed_salt = Some(salt);
        app.ui.language = gridcraft_ui_egui::i18n::Language::Zh;
        app.ui.system_theme = false;
        app.ui.dark = false;
        Self {
            app,
            title: "表格".into(),
        }
    }

    fn refresh(&mut self) {
        if let Some(title) = self.app.session.active().map(|doc| doc.display_title()) {
            self.title = title;
        }
    }
}

impl DocumentEditor for SheetEditor {
    fn kind(&self) -> DocKind {
        DocKind::Sheet
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn dirty(&self) -> bool {
        self.app.session.active().is_some_and(|doc| doc.is_dirty())
    }

    fn path(&self) -> Option<PathBuf> {
        self.app
            .session
            .active()
            .and_then(|doc| doc.path.clone())
            .map(PathBuf::from)
    }

    fn save(&mut self) -> bool {
        if !self.app.commit_edit(0, 0, false, false) {
            return false;
        }
        self.app.store_view_state();
        let has_path = self
            .app
            .session
            .active()
            .is_some_and(|doc| doc.path.is_some());
        if has_path {
            if let Err(err) = self.app.run("file.save", json!({})) {
                self.app.message = Some(("OneOffice".into(), err));
            }
        } else {
            self.app.open_dialog("saveAs", json!({}));
        }
        self.refresh();
        !self.dirty()
    }

    fn logic(&mut self, ctx: &egui::Context) {
        self.app.logic(ctx);
        self.refresh();
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        self.app.ui(ui);
        self.refresh();
    }
}

/// 把壳的中文字体交给 gridcraft。要在第一帧建字体之前调用，只生效一次。
pub(crate) fn register_cjk_font() {
    gridcraft_ui_egui::theme::set_host_fallback_fonts(vec![(
        "oneoffice-noto-sc".to_owned(),
        egui::FontData::from_static(theme::cjk_font_bytes()),
    )]);
}

fn services(opens: Arc<Mutex<Vec<PathBuf>>>) -> Services {
    let open_queue = opens;
    Services {
        pick_open: Some(Box::new(move || {
            let picked = rfd::FileDialog::new()
                .set_title("打开表格")
                .add_filter("表格", SHEET_EXTENSIONS)
                .pick_file();
            if let Some(path) = picked {
                open_queue
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(path);
            }
            // 返回 None，让壳另开一个标签，而不是塞进当前 session。
            None
        })),
        pick_save: Some(Box::new(|suggested: &str| {
            let name = Path::new(suggested)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("未命名表格.xlsx");
            rfd::FileDialog::new()
                .set_title("保存表格")
                .set_file_name(name)
                .add_filter("Excel 工作簿", &["xlsx"])
                .save_file()
                .and_then(|path| path.to_str().map(str::to_string))
        })),
        ..Default::default()
    }
}

/// 公式计算和保存走引擎，不经过界面。两个 session 各算各的。
#[cfg(test)]
mod tests {
    use super::*;

    fn cell_value(session: &mut Session, cell: &str) -> serde_json::Value {
        session
            .execute("cell.get", json!({"cell": cell}))
            .expect("cell.get")["value"]
            .clone()
    }

    #[test]
    fn formula_roundtrip_and_separate_sessions() {
        let dir = std::env::temp_dir().join(format!("oneoffice-sheet-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("公式.xlsx");

        let mut session = Session::new();
        session.new_workbook();
        session
            .execute("cell.set", json!({"cell": "A1", "input": "2"}))
            .unwrap();
        session
            .execute("cell.set", json!({"cell": "A2", "input": "3"}))
            .unwrap();
        session
            .execute("cell.set", json!({"cell": "A3", "input": "=SUM(A1:A2)"}))
            .unwrap();
        assert_eq!(cell_value(&mut session, "A3"), serde_json::json!(5.0));
        session
            .execute(
                "file.save",
                json!({"path": path.to_str().expect("utf8 temp path")}),
            )
            .unwrap();

        let mut again = Session::new();
        again
            .execute(
                "file.open",
                json!({"path": path.to_str().expect("utf8 temp path")}),
            )
            .unwrap();
        assert_eq!(cell_value(&mut again, "A3"), serde_json::json!(5.0));
        assert!(!again.active().expect("doc").is_dirty());

        let mut other = Session::new();
        other.new_workbook();
        other
            .execute("cell.set", json!({"cell": "A1", "input": "9"}))
            .unwrap();
        assert_eq!(cell_value(&mut again, "A1"), serde_json::json!(2.0));
        assert_eq!(cell_value(&mut other, "A1"), serde_json::json!(9.0));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
