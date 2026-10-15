//! 配置目录与「在文件夹中显示」。macOS 用 Application Support，其它 Unix 用 XDG。

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library/Application Support/OneOffice")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("OneOffice")
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("OneOffice")
    }
}

pub fn recent_file() -> PathBuf {
    config_dir().join("recent.json")
}

/// 菜单和搜索框里显示的修饰键。egui 的 `Modifiers::COMMAND` 在 macOS 上是 ⌘，其它平台是 Ctrl。
pub fn shortcut(key: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("⌘{key}")
    }
    #[cfg(not(target_os = "macos"))]
    {
        format!("Ctrl+{key}")
    }
}

pub fn reveal(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        if path.is_file() {
            cmd.arg("-R").arg(path);
        } else {
            cmd.arg(path);
        }
        let _ = cmd.spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        let _ = Command::new("xdg-open").arg(dir).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    }
}

pub fn display_location(path: &Path, home: Option<&Path>) -> String {
    let parent = path.parent().unwrap_or(path);
    if let Some(home) = home
        && let Ok(rel) = parent.strip_prefix(home)
    {
        if rel.as_os_str().is_empty() {
            return "~".to_owned();
        }
        return format!("~/{}", rel.display());
    }
    parent.display().to_string()
}
