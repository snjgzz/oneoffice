//! 最近打开的文件。只记录通过打开动作进来的 docx / xlsx / xlsm / xls / xlsb / ods / pptx。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Datelike, Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::kind::DocKind;
use crate::platform::{display_location, home_dir};

const MAX_RECENT: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: PathBuf,
    pub kind: DocKind,
    pub modified_ms: Option<i64>,
    pub opened_ms: i64,
}

impl RecentFile {
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }

    pub fn location(&self) -> String {
        display_location(&self.path, home_dir().as_deref())
    }

    pub fn modified_label(&self) -> String {
        let ms = mtime_ms(&self.path).or(self.modified_ms);
        let Some(ms) = ms else {
            return "未知".to_owned();
        };
        let Some(modified) = ms_to_local(ms) else {
            return "未知".to_owned();
        };
        format_time(modified.naive_local(), Local::now().naive_local())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentList {
    pub files: Vec<RecentFile>,
}

impl RecentList {
    pub fn load(path: &Path) -> Self {
        let Ok(bytes) = fs::read(path) else {
            return Self::default();
        };
        match serde_json::from_slice(&bytes) {
            Ok(list) => list,
            Err(err) => {
                eprintln!(
                    "oneoffice: 无法读取 {}（{err}），最近列表按空的处理",
                    path.display()
                );
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            eprintln!("oneoffice: 无法创建 {}（{err}）", parent.display());
            return;
        }
        let payload = match serde_json::to_vec_pretty(self) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("oneoffice: 无法序列化最近列表（{err}）");
                return;
            }
        };
        let tmp = path.with_extension("json.tmp");
        if let Err(err) = fs::write(&tmp, payload) {
            eprintln!("oneoffice: 无法写入 {}（{err}）", tmp.display());
            return;
        }
        if let Err(err) = fs::rename(&tmp, path) {
            eprintln!("oneoffice: 无法保存 {}（{err}）", path.display());
        }
    }

    /// 把 `path` 放到列表最前。扩展名不对或不是文件时返回 false。
    pub fn record_open(&mut self, path: PathBuf) -> bool {
        if !path.is_file() {
            return false;
        }
        let Some(kind) = DocKind::from_path(&path) else {
            return false;
        };
        let path = path.canonicalize().unwrap_or(path);
        self.files.retain(|file| file.path != path);
        self.files.insert(
            0,
            RecentFile {
                modified_ms: mtime_ms(&path),
                opened_ms: now_ms(),
                path,
                kind,
            },
        );
        if self.files.len() > MAX_RECENT {
            self.files.truncate(MAX_RECENT);
        }
        true
    }

    pub fn remove(&mut self, path: &Path) -> bool {
        let before = self.files.len();
        self.files.retain(|file| file.path != path);
        self.files.len() != before
    }
}

pub fn format_time(modified: NaiveDateTime, now: NaiveDateTime) -> String {
    let day = modified.date();
    let today = now.date();
    if day == today {
        format!("今天 {}", modified.format("%H:%M"))
    } else if today.pred_opt() == Some(day) {
        format!("昨天 {}", modified.format("%H:%M"))
    } else if modified.year() == now.year() {
        format!("{}月{}日", modified.month(), modified.day())
    } else {
        format!(
            "{}年{}月{}日",
            modified.year(),
            modified.month(),
            modified.day()
        )
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn mtime_ms(path: &Path) -> Option<i64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let dur = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(dur.as_millis() as i64)
}

fn ms_to_local(ms: i64) -> Option<DateTime<Local>> {
    let secs = ms.div_euclid(1000);
    let nanos = (ms.rem_euclid(1000) as u32) * 1_000_000;
    DateTime::from_timestamp(secs, nanos).map(|utc| utc.with_timezone(&Local))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::display_location;

    fn naive(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn format_time_buckets() {
        let now = naive("2026-10-15 12:00");
        assert_eq!(format_time(naive("2026-10-15 10:42"), now), "今天 10:42");
        assert_eq!(format_time(naive("2026-10-14 18:30"), now), "昨天 18:30");
        assert_eq!(format_time(naive("2026-10-12 09:00"), now), "10月12日");
        assert_eq!(format_time(naive("2025-10-08 09:00"), now), "2025年10月8日");
    }

    #[test]
    fn location_uses_tilde() {
        let home = Path::new("/Users/me");
        assert_eq!(
            display_location(Path::new("/Users/me/Documents/示例/a.docx"), Some(home)),
            "~/Documents/示例"
        );
        assert_eq!(
            display_location(Path::new("/Users/me/a.docx"), Some(home)),
            "~"
        );
        assert_eq!(
            display_location(Path::new("/tmp/a.docx"), Some(home)),
            "/tmp"
        );
    }

    #[test]
    fn record_dedupes_and_rejects_other_extensions() {
        let dir = std::env::temp_dir().join(format!("oneoffice-recent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let doc = dir.join("计划.docx");
        let sheet = dir.join("表.xlsx");
        let other = dir.join("笔记.txt");
        fs::write(&doc, b"x").unwrap();
        fs::write(&sheet, b"x").unwrap();
        fs::write(&other, b"x").unwrap();

        let mut list = RecentList::default();
        assert!(list.record_open(doc.clone()));
        assert!(list.record_open(sheet.clone()));
        assert!(list.record_open(doc.clone()));
        assert!(!list.record_open(other));
        assert_eq!(list.files.len(), 2);
        assert_eq!(list.files[0].path, doc.canonicalize().unwrap());
        assert_eq!(list.files[0].kind, DocKind::Word);
        assert_eq!(list.files[1].kind, DocKind::Sheet);

        let store = dir.join("recent.json");
        list.save(&store);
        let mut loaded = RecentList::load(&store);
        assert_eq!(loaded.files.len(), 2);
        assert!(loaded.remove(&list.files[0].path));
        assert_eq!(loaded.files.len(), 1);

        let mut many = RecentList::default();
        for i in 0..60 {
            let path = dir.join(format!("deck-{i}.pptx"));
            fs::write(&path, b"x").unwrap();
            assert!(many.record_open(path));
        }
        assert_eq!(many.files.len(), MAX_RECENT);
        assert!(many.files[0].name().starts_with("deck-59"));

        let _ = fs::remove_dir_all(&dir);
    }
}
