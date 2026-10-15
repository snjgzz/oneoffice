//! 文档类型：文字、表格、演示。颜色与设计规范一致。

use egui::Color32;

use crate::theme::{BLUE, GREEN, NEUTRAL, ORANGE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocKind {
    Word,
    Sheet,
    Deck,
}

impl DocKind {
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "doc" | "docx" => Some(Self::Word),
            "xlsx" | "xlsm" | "xls" | "xlsb" | "ods" => Some(Self::Sheet),
            "pptx" => Some(Self::Deck),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Word => 0,
            Self::Sheet => 1,
            Self::Deck => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Word => "文字",
            Self::Sheet => "表格",
            Self::Deck => "演示",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Word => "W",
            Self::Sheet => "S",
            Self::Deck => "P",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Word => "docx",
            Self::Sheet => "xlsx",
            Self::Deck => "pptx",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::Word => BLUE,
            Self::Sheet => GREEN,
            Self::Deck => ORANGE,
        }
    }

    /// 以后嵌进这个标签的上游应用。
    pub fn embedded_app(self) -> &'static str {
        match self {
            Self::Word => "wordcraft",
            Self::Sheet => "gridcraft",
            Self::Deck => "deckcraft",
        }
    }

    pub fn menus(self) -> &'static [&'static str] {
        match self {
            Self::Word => &["文件", "开始", "插入", "页面布局", "引用", "审阅", "视图"],
            Self::Sheet => &[
                "文件",
                "开始",
                "插入",
                "页面布局",
                "公式",
                "数据",
                "审阅",
                "视图",
            ],
            Self::Deck => &[
                "文件",
                "开始",
                "插入",
                "设计",
                "切换",
                "动画",
                "幻灯片放映",
                "审阅",
                "视图",
            ],
        }
    }

    pub fn ribbon(self) -> &'static [(&'static str, &'static [f32])] {
        match self {
            Self::Word => &[
                ("剪贴板", &[26.0, 26.0, 26.0]),
                ("字体", &[110.0, 50.0, 26.0, 26.0, 26.0]),
                ("段落", &[26.0, 26.0, 26.0, 26.0, 26.0]),
                ("样式", &[60.0, 60.0, 60.0]),
            ],
            Self::Sheet => &[
                ("剪贴板", &[26.0, 26.0, 26.0]),
                ("字体", &[110.0, 26.0, 26.0, 26.0]),
                ("对齐", &[26.0, 26.0, 26.0, 26.0, 26.0, 26.0]),
                ("数字", &[90.0, 26.0, 26.0]),
            ],
            Self::Deck => &[
                ("剪贴板", &[26.0, 26.0, 26.0]),
                ("幻灯片", &[26.0, 26.0, 72.0]),
                ("字体", &[110.0, 50.0, 26.0, 26.0]),
                ("绘图", &[26.0, 26.0, 26.0, 26.0]),
            ],
        }
    }

    pub fn status_left(self) -> &'static [&'static str] {
        match self {
            Self::Word => &["第 1 页，共 1 页", "字数：0"],
            Self::Sheet => &["就绪"],
            Self::Deck => &["幻灯片 1 / 1"],
        }
    }

    /// 打开文件卡片上的副标题。
    pub fn card_subtitle(self) -> &'static str {
        match self {
            Self::Word => "空白文档 · .docx",
            Self::Sheet => "空白工作簿 · .xlsx",
            Self::Deck => "空白演示 · .pptx",
        }
    }

    pub fn new_label(self) -> &'static str {
        match self {
            Self::Word => "新建文字",
            Self::Sheet => "新建表格",
            Self::Deck => "新建演示",
        }
    }
}

/// 中性的「打开文件」卡片，不是一种文档类型。
pub fn open_card_color() -> Color32 {
    NEUTRAL
}
