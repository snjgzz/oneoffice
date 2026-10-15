//! wordcraft `WordApp` 适配器。每个文字标签一份 `Session` 和一份 `WordApp`。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;
use wordcraft_engine::Session;
use wordcraft_ui_egui::{Services, WordApp};

use crate::editor::{DocumentEditor, HostEvent};
use crate::kind::DocKind;
use crate::theme;

const WORD_EXTENSIONS: &[&str] = &["docx", "doc"];

pub(crate) struct WordEditor {
    app: WordApp,
    title: String,
    salt: u64,
    host: Arc<Mutex<Vec<HostEvent>>>,
}

impl WordEditor {
    pub(crate) fn blank(salt: u64, title: String, host: Arc<Mutex<Vec<HostEvent>>>) -> Self {
        let mut session = Session::new(wordcraft_engine::doc::Document::new());
        session.dirty = false;
        let _ = session.run("file.new", &json!({}));
        session.file_name = Some(stem(&title));
        session.dirty = false;
        let mut editor = Self::from_session(salt, session, host);
        editor.title = title;
        editor
    }

    pub(crate) fn open(salt: u64, path: &Path, host: Arc<Mutex<Vec<HostEvent>>>) -> Self {
        let mut session = Session::new(wordcraft_engine::doc::Document::new());
        session.dirty = false;
        let mut editor = Self::from_session(salt, session, host);
        match path.to_str() {
            Some(path) => {
                if let Err(err) = editor.app.run("file.open", json!({"path": path})) {
                    editor.app.status(err);
                }
            }
            None => editor.app.status("文件路径不是 UTF-8，无法打开。"),
        }
        editor.refresh();
        editor
    }

    fn from_session(salt: u64, session: Session, host: Arc<Mutex<Vec<HostEvent>>>) -> Self {
        let mut app = WordApp::new(session, services(Arc::clone(&host)));
        app.embed_salt = Some(salt);
        app.ui.language = "zh-hans".into();
        app.ui.theme = wordcraft_ui_egui::theme::Appearance::Light;
        Self {
            app,
            title: "文字".into(),
            salt,
            host,
        }
    }

    fn refresh(&mut self) {
        self.title = self.app.title_stem();
    }

    fn note_close(&self) {
        if self.app.embed_closed {
            self.host
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(HostEvent::Close(self.salt));
        }
    }
}

impl DocumentEditor for WordEditor {
    fn kind(&self) -> DocKind {
        DocKind::Word
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn dirty(&self) -> bool {
        self.app.session.dirty
    }

    fn path(&self) -> Option<PathBuf> {
        self.app.session.path.clone()
    }

    fn save(&mut self) -> bool {
        if let Err(err) = self.app.run("file.save", json!({})) {
            self.app.status(err);
        }
        self.refresh();
        !self.dirty()
    }

    fn logic(&mut self, ctx: &egui::Context) {
        self.app.logic(ctx);
        self.note_close();
        self.refresh();
    }

    fn prepare_embed_fonts(&mut self, ctx: &egui::Context) {
        self.app.prepare_embed_fonts(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        self.app.ui(ui);
        self.note_close();
        self.refresh();
    }
}

/// 把壳的中文字体交给 wordcraft。要在第一帧建字体之前调用，只生效一次。
pub(crate) fn register_cjk_font() {
    wordcraft_ui_egui::theme::set_host_fallback_fonts(vec![(
        "oneoffice-noto-sc".to_owned(),
        egui::FontData::from_static(theme::cjk_font_bytes()),
    )]);
}

fn stem(title: &str) -> String {
    title
        .strip_suffix(".docx")
        .or_else(|| title.strip_suffix(".doc"))
        .unwrap_or(title)
        .to_owned()
}

fn services(host: Arc<Mutex<Vec<HostEvent>>>) -> Services {
    Services {
        pick_open: Some(Box::new(move |purpose| {
            let document = purpose == "document" || purpose == "open";
            let (title, filters): (&str, &[(&str, &[&str])]) = if document {
                ("打开文字", &[("Word 文档", WORD_EXTENSIONS)])
            } else if purpose == "picture" {
                (
                    "插入图片",
                    &[("图片", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])],
                )
            } else {
                ("打开文件", &[("所有文件", &["*"])])
            };
            let mut dialog = rfd::FileDialog::new().set_title(title);
            for (label, exts) in filters {
                dialog = dialog.add_filter(*label, exts);
            }
            let picked = dialog.pick_file();
            if document {
                if let Some(path) = picked {
                    host.lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(HostEvent::Open(path));
                }
                None
            } else {
                picked.and_then(|path| path.to_str().map(str::to_string))
            }
        })),
        pick_save: Some(Box::new(|suggested: &str| {
            let name = Path::new(suggested)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("未命名文字.docx");
            rfd::FileDialog::new()
                .set_title("保存文字")
                .set_file_name(name)
                .add_filter("Word 文档", &["docx"])
                .save_file()
                .and_then(|path| path.to_str().map(str::to_string))
        })),
        ..Default::default()
    }
}

/// 打字、保存、再打开。两个 session 各改各的。.doc 由引擎的 docbin 读取。
#[cfg(test)]
mod tests {
    use super::*;

    fn body(session: &mut Session) -> String {
        session
            .run("document.text", &json!({}))
            .expect("document.text")["text"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    }

    #[test]
    fn docx_roundtrip_and_separate_sessions() {
        let dir = std::env::temp_dir().join(format!("oneoffice-word-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("文字.docx");

        let mut session = Session::new(wordcraft_engine::doc::Document::new());
        session.run("file.new", &json!({})).unwrap();
        session.dirty = false;
        session
            .run("text.insert", &json!({"text": "你好 OneOffice"}))
            .unwrap();
        session
            .run(
                "file.save",
                &json!({"path": path.to_str().expect("utf8 temp path")}),
            )
            .unwrap();
        assert!(!session.dirty);

        let mut again = Session::new(wordcraft_engine::doc::Document::new());
        again
            .run(
                "file.open",
                &json!({"path": path.to_str().expect("utf8 temp path")}),
            )
            .unwrap();
        assert!(body(&mut again).contains("你好 OneOffice"));
        assert!(!again.dirty);

        let mut other = Session::new(wordcraft_engine::doc::Document::new());
        other.run("file.new", &json!({})).unwrap();
        other
            .run("text.insert", &json!({"text": "另一份"}))
            .unwrap();
        assert!(body(&mut again).contains("你好 OneOffice"));
        assert!(body(&mut other).contains("另一份"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
