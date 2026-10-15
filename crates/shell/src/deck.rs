//! deckcraft `SlideApp` 适配器。每个演示标签一份 `Session` 和一份 `SlideApp`。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use deckcraft_engine::Session;
use deckcraft_ui_egui::{Services, SlideApp};
use serde_json::json;

use crate::editor::{DocumentEditor, HostEvent};
use crate::kind::DocKind;
use crate::theme;

pub(crate) struct DeckEditor {
    app: SlideApp,
    title: String,
    salt: u64,
    host: Arc<Mutex<Vec<HostEvent>>>,
}

impl DeckEditor {
    pub(crate) fn blank(salt: u64, title: String, host: Arc<Mutex<Vec<HostEvent>>>) -> Self {
        let mut session = Session::new();
        let _ = session.execute("file.new", &json!({}));
        if let Ok(doc) = session.doc_mut() {
            doc.title = stem(&title);
        }
        let mut editor = Self::from_session(salt, session, host);
        editor.refresh();
        editor
    }

    pub(crate) fn open(salt: u64, path: &Path, host: Arc<Mutex<Vec<HostEvent>>>) -> Self {
        let mut editor = Self::from_session(salt, Session::new(), host);
        match path.to_str() {
            Some(path) => {
                if let Err(err) = editor.app.open_path(path) {
                    editor.app.set_status(err);
                }
            }
            None => editor.app.set_status("文件路径不是 UTF-8，无法打开。"),
        }
        editor.refresh();
        editor
    }

    fn from_session(salt: u64, session: Session, host: Arc<Mutex<Vec<HostEvent>>>) -> Self {
        let mut app = SlideApp::new(session, services(Arc::clone(&host)));
        app.embed_salt = Some(salt);
        app.ui.brightness = deckcraft_ui_egui::theme::Brightness::Light;
        Self {
            app,
            title: "演示".into(),
            salt,
            host,
        }
    }

    fn refresh(&mut self) {
        if let Some(title) = self.app.session.active().map(|doc| doc.title()) {
            self.title = title;
        }
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

impl DocumentEditor for DeckEditor {
    fn kind(&self) -> DocKind {
        DocKind::Deck
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
        let has_path = self
            .app
            .session
            .active()
            .is_some_and(|doc| doc.path.is_some());
        if has_path {
            if let Err(err) = self.app.session.execute("file.save", &json!({})) {
                self.app.set_status(err.to_string());
            }
        } else {
            self.app.save();
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

/// 把壳的中文字体交给 deckcraft。要在第一帧建字体之前调用，只生效一次。
pub(crate) fn register_cjk_font() {
    deckcraft_ui_egui::theme::set_host_fallback_fonts(vec![(
        "oneoffice-noto-sc".to_owned(),
        egui::FontData::from_static(theme::cjk_font_bytes()),
    )]);
}

fn stem(title: &str) -> String {
    title.strip_suffix(".pptx").unwrap_or(title).to_owned()
}

fn services(host: Arc<Mutex<Vec<HostEvent>>>) -> Services {
    Services {
        pick_open: Some(Box::new(move |purpose| {
            let presentation = purpose == "open";
            let (title, label, exts): (&str, &str, &[&str]) = if presentation {
                ("打开演示", "PowerPoint 演示文稿", &["pptx"])
            } else if purpose == "picture" {
                (
                    "插入图片",
                    "图片",
                    &["png", "jpg", "jpeg", "gif", "webp", "bmp"],
                )
            } else if purpose == "audio" {
                ("插入音频", "音频", &["mp3", "m4a", "wav", "flac"])
            } else if purpose == "video" {
                ("插入视频", "视频", &["mp4", "mov", "mkv", "webm"])
            } else {
                ("打开文件", "所有文件", &["*"])
            };
            let picked = rfd::FileDialog::new()
                .set_title(title)
                .add_filter(label, exts)
                .pick_file();
            if presentation {
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
                .unwrap_or("未命名演示.pptx");
            let name = if name.ends_with(".pptx") {
                name.to_owned()
            } else {
                format!("{name}.pptx")
            };
            rfd::FileDialog::new()
                .set_title("保存演示")
                .set_file_name(&name)
                .add_filter("PowerPoint 演示文稿", &["pptx"])
                .save_file()
                .and_then(|path| path.to_str().map(str::to_string))
        })),
        ..Default::default()
    }
}

/// 在标题占位符里打字，存成 pptx，再打开。两个 session 各改各的。
#[cfg(test)]
mod tests {
    use super::*;

    fn slide_text(session: &Session) -> String {
        let Some(doc) = session.active() else {
            return String::new();
        };
        let mut out = String::new();
        for slide in &doc.doc.slides {
            for shape in &slide.shapes {
                let Some(body) = &shape.text else { continue };
                for paragraph in &body.paragraphs {
                    for run in &paragraph.runs {
                        out.push_str(&run.text);
                    }
                }
            }
        }
        out
    }

    #[test]
    fn pptx_roundtrip_and_separate_sessions() {
        let dir = std::env::temp_dir().join(format!("oneoffice-deck-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("演示.pptx");

        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        let id = session
            .active()
            .and_then(|doc| doc.doc.slides.first())
            .and_then(|slide| slide.shapes.first())
            .map(|shape| shape.id)
            .expect("title placeholder");
        session.execute("text.edit", &json!({"id": id})).unwrap();
        session
            .execute("text.insert", &json!({"text": "你好 OneOffice"}))
            .unwrap();
        session
            .execute(
                "file.saveAs",
                &json!({"path": path.to_str().expect("utf8 temp path")}),
            )
            .unwrap();
        assert!(session.active().is_some_and(|doc| !doc.is_dirty()));

        let mut again = Session::new();
        again
            .execute(
                "file.open",
                &json!({"path": path.to_str().expect("utf8 temp path")}),
            )
            .unwrap();
        assert!(slide_text(&again).contains("你好 OneOffice"));
        assert!(again.active().is_some_and(|doc| !doc.is_dirty()));

        let mut other = Session::new();
        other.execute("file.new", &json!({})).unwrap();
        let other_id = other
            .active()
            .and_then(|doc| doc.doc.slides.first())
            .and_then(|slide| slide.shapes.first())
            .map(|shape| shape.id)
            .expect("other title");
        other
            .execute("text.edit", &json!({"id": other_id}))
            .unwrap();
        other
            .execute("text.insert", &json!({"text": "另一份"}))
            .unwrap();
        assert!(slide_text(&again).contains("你好 OneOffice"));
        assert!(slide_text(&other).contains("另一份"));
        assert!(!slide_text(&again).contains("另一份"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
