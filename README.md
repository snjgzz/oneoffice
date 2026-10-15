# OneOffice

OneOffice 是一个类似 WPS 的集成办公套件外壳：文字、表格、演示共用一个原生窗口。这一版是主窗口壳，文档页还是占位；后面会把 [wordcraft](https://github.com/storytold/wordcraft)、[gridcraft](https://github.com/storytold/gridcraft)、[deckcraft](https://github.com/storytold/deckcraft) 嵌进标签。

界面是中文。侧栏只有「首页」和「最近」，「设置」钉在底部。最近文件记在本机配置目录的 `recent.json` 里，只收通过打开动作进来的 `.docx` / `.xlsx` / `.pptx`。

## English

OneOffice is the shared native window for a WPS-style office suite. This repository is the shell: a macOS-style tab bar, a home page, and placeholder document tabs. Word, spreadsheet, and presentation editing will later be the embedded egui apps from storytold/wordcraft (`WordApp`), gridcraft (`SheetApp`), and deckcraft (`SlideApp`). The UI is Chinese. Recent files are a real local JSON list of docx/xlsx/pptx opened from the native file dialog.

## 在 Mac 上构建和运行

需要 Rust 1.95 或更新（edition 2024）。[rustup](https://rustup.rs/) 的 stable 即可。Apple Silicon 是主要目标，Intel Mac 同一套代码也能编。

```sh
git clone https://github.com/snjgzz/oneoffice
cd oneoffice
cargo run -p oneoffice
```

打开已有文件：

```sh
cargo run -p oneoffice -- ~/Documents/计划.docx
```

扩展名必须是 docx、xlsx 或 pptx。文件会成为新标签，并写进最近列表。

最近列表的位置：

- macOS：`~/Library/Application Support/OneOffice/recent.json`
- Linux：`$XDG_CONFIG_HOME/OneOffice/recent.json`，未设置时是 `~/.config/OneOffice/recent.json`

## Linux

能编过、能跑。系统标题栏在 Linux 上会关掉，窗口按钮画在标签栏左侧，因为 macOS 的 full-size content view 和红绿灯是平台能力。编译前安装：

```sh
sudo apt-get install -y pkg-config libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-0 libssl-dev libwayland-dev
cargo run -p oneoffice
```

## 快捷键

修饰键在 macOS 上是 ⌘，在 Linux 上是 Ctrl。

| 按键 | 作用 |
| --- | --- |
| ⌘T | 回到首页标签 |
| ⌘W | 关闭当前文档标签（首页关不掉） |
| ⌘1–9 | 切换标签，1 是首页 |
| ⌘O | 打开文件 |
| ⌘K | 聚焦搜索 |

首页上双击最近文件会再打开它。右键可以打开所在文件夹，或从列表里移除。

## 仓库结构

```
crates/shell    可执行文件 oneoffice，以及同名库
```

库导出 `DocKind`、`Tab` 和 `DocumentEditor`。以后的集成 crate 实现这个 trait，再放进 `Tab::Word` / `Tab::Sheet` / `Tab::Deck`。workspace 现在只有 shell，就是给那三个 crate 留的位置。

`DocumentEditor::ui` 对齐上游已经公开的形状：`WordApp::ui`、`SheetApp::ui`、`SlideApp::ui`。上游目前假定自己独占窗口；嵌进来之前要改成往壳给的 `Ui` 里画，不再自己管标题栏和窗口循环。

## 路线图

1. 主窗口壳（当前）。
2. 让 wordcraft、gridcraft、deckcraft 的 `ui()` 能画进父级 `Ui`，不再各开一个 eframe 窗口。
3. 增加集成 crate：构造 `WordApp::new(session, services)`、`SheetApp::new(session, services)`、`SlideApp::new(session, services)`，套上 `DocumentEditor`。
4. 「新建」和「打开」接到真正的 session，而不是占位页。
5. 未保存标记，以及关闭未保存标签时的确认。

## 许可证

代码以 MIT OR Apache-2.0 双许可发布，见 `LICENSE-MIT` 和 `LICENSE-APACHE`。

界面字体是 Noto Sans SC Regular 2.004 的子集（GB2312 加常用符号），字体单独使用 SIL Open Font License 1.1。版权归 Adobe，保留字体名 Source。许可证全文在 `crates/shell/assets/fonts/OFL.txt`。子集可以用 `tools/subset_noto.py` 重新生成。GB2312 以外的汉字会缺字形。

## 归属

OneOffice 不隶属于 WPS、Microsoft 或 storytold。

嵌入 wordcraft、gridcraft、deckcraft 时，保留它们的版权声明和 MIT OR Apache-2.0 许可证，不删文件头。仓库会加一份 `NOTICE`，写明这三个上游项目的名称、仓库地址和许可证。不使用它们的商标做背书。

Noto Sans SC 只作为界面字体打包，按 OFL 随附许可证；不把保留名 Source 用在衍生字体上。
