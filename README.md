# OneOffice

OneOffice 是一个类似 WPS 的集成办公套件外壳：文字、表格、演示共用一个原生窗口。表格标签嵌的是 [gridcraft](https://github.com/storytold/gridcraft) 的 `SheetApp`（功能区、编辑栏、网格、公式）。文字和演示还是占位，后面接 [wordcraft](https://github.com/storytold/wordcraft) 和 [deckcraft](https://github.com/storytold/deckcraft)。

界面是中文。侧栏只有「首页」和「最近」，「设置」钉在底部。最近文件记在本机配置目录的 `recent.json` 里，收通过打开动作进来的 `.docx` / `.xlsx` / `.xlsm` / `.xls` / `.xlsb` / `.ods` / `.pptx`。`.xls`、`.xlsb`、`.ods` 由 gridcraft 导入，保存时写成 `.xlsx`。

## English

OneOffice is the shared native window for a WPS-style office suite. Spreadsheet tabs embed storytold/gridcraft (`SheetApp`): ribbon, formula bar, grid, and formulas. Word and presentation tabs are still placeholders for wordcraft (`WordApp`) and deckcraft (`SlideApp`). The UI is Chinese. Recent files are a local JSON list of documents opened from the native file dialog.

## 在 Mac 上构建和运行

需要 Rust 1.95 或更新（edition 2024）。[rustup](https://rustup.rs/) 的 stable 即可。Apple Silicon 是主要目标，Intel Mac 同一套代码也能编。

```sh
git clone --recurse-submodules https://github.com/snjgzz/oneoffice
cd oneoffice
tools/apply-gridcraft-patch.sh
cargo run -p oneoffice
```

已经克隆过、子模块是空的：

```sh
git submodule update --init --recursive
tools/apply-gridcraft-patch.sh
```

打开已有文件：

```sh
cargo run -p oneoffice -- ~/Documents/预算.xlsx
```

扩展名是 docx、xlsx、xlsm、xls、xlsb、ods 或 pptx。文件会成为新标签，并写进最近列表。表格可以编辑、计算公式，用 Ctrl+S / ⌘S 或功能区「文件」保存。没有路径的新工作簿会弹出另存为。

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
crates/shell          可执行文件 oneoffice，以及同名库
vendor/gridcraft      storytold/gridcraft 子模块，固定提交
patches/              嵌进壳所需的 gridcraft 补丁
```

库导出 `DocKind`、`Tab` 和 `DocumentEditor`。`Tab::Sheet` 使用 `SheetEditor`，里面每个标签一份 `SheetApp`。文字和演示仍是 `PlaceholderEditor`。gridcraft 不加入本仓库的 workspace members，它自己的 `Cargo.toml` 继续解析 `.workspace = true`。

嵌入补丁让 `SheetApp` 在 `embed_salt` 有值时去掉自己的标题栏、窗口标题和主题/缩放接管，并把面板和编辑器的 egui id 按标签隔开。补丁内容和更新方法在 `patches/README.md`。

## 路线图

1. 主窗口壳。
2. gridcraft 嵌进表格标签（当前）：打开、编辑、公式、保存、多个表格标签。
3. 用同样的方式嵌 wordcraft（`WordApp`）和 deckcraft（`SlideApp`）。
4. 关闭未保存的文字/演示标签时的确认（表格已经有）。

## 许可证

代码以 MIT OR Apache-2.0 双许可发布，见 `LICENSE-MIT` 和 `LICENSE-APACHE`。

界面字体是 Noto Sans SC Regular 2.004 的子集（GB2312 加常用符号），字体单独使用 SIL Open Font License 1.1。版权归 Adobe，保留字体名 Source。许可证全文在 `crates/shell/assets/fonts/OFL.txt`。子集可以用 `tools/subset_noto.py` 重新生成。GB2312 以外的汉字会缺字形。

## 归属

OneOffice 不隶属于 WPS、Microsoft 或 storytold。

gridcraft 以子模块形式放在 `vendor/gridcraft`，版权和 MIT OR Apache-2.0 许可证保留在它自己的仓库里。`NOTICE` 写了项目名、仓库地址、固定的提交和许可证。不使用 GridCraft / ArtCraft 的商标做背书。wordcraft 和 deckcraft 接进来时同样处理。

Noto Sans SC 只作为界面字体打包，按 OFL 随附许可证；不把保留名 Source 用在衍生字体上。
