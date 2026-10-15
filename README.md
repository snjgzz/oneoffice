# OneOffice

文字、表格和演示放在同一个原生窗口里。界面以中文为主，用 Rust 和 egui 写成，文件留在你自己的电脑上。

<p align="center">
  <a href="LICENSE-MIT"><img alt="许可证：MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2F6FED"></a>
  <img alt="需要 Rust 1.95 或更新" src="https://img.shields.io/badge/rust-1.95%2B-1FA463?logo=rust&logoColor=white">
  <a href="https://github.com/snjgzz/oneoffice/actions/workflows/ci.yml"><img alt="持续集成" src="https://github.com/snjgzz/oneoffice/actions/workflows/ci.yml/badge.svg"></a>
</p>

<p align="center">
  <a href="#功能">功能</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#系统要求">系统要求</a> ·
  <a href="#已知限制">已知限制</a> ·
  <a href="#参与贡献">贡献</a> ·
  <a href="#许可证与归属">许可证</a>
</p>

In short: one native window for a word processor, a spreadsheet and slides. The interface is mainly Chinese. It is not a LibreOffice-level office suite, and it is not an official WPS, Microsoft or storytold product.

<p align="center">
  <img src="docs/images/app-icon.png" width="420" alt="OneOffice 图标：青绿到蓝的圆角底，白色粗体 O，后面是三份半透明文档。">
  <br>
  <sub>应用图标。源文件和打进 <code>.app</code> 的方法在 <a href="packaging/macos/README.md"><code>packaging/macos</code></a>。</sub>
</p>

<p align="center">
  <img src="docs/images/three-tabs.png" width="920" alt="OneOffice 同时开着文字、表格和演示。当前标签是一张中文表格，金额列有材料和人工，合计是公式。">
  <br>
  <sub>Linux 上从源码跑起来的窗口：文字、表格、演示三个标签同时开着。</sub>
</p>

## 功能

- **三种文档，一个窗口。** 文字、表格、演示各自一个标签，可以一起开着，用 ⌘1–9 切换。每种都能打开、编辑、保存。
- **中文界面。** 壳、文字和表格的功能区是中文。演示的功能区目前仍是英文，幻灯片里的中文可以显示。
- **常见 Office 文件。** 打开 `.doc`、`.docx`、`.xlsx`、`.xlsm`、`.xls`、`.xlsb`、`.ods`、`.pptx`。旧格式和新格式的保存方式不一样，见 [已知限制](#已知限制)。
- **最近文件在本机。** 从打开对话框进来的文件记在配置目录的 `recent.json` 里，没有账号。

<table>
<tr>
<td width="50%" valign="top">
<img src="docs/images/word.png" alt="文字标签：中文功能区，页面上是一段中文和英文。">
<p align="center"><sub>文字。功能区是中文。</sub></p>
</td>
<td width="50%" valign="top">
<img src="docs/images/slides.png" alt="演示标签：英文功能区，幻灯片标题是中文。">
<p align="center"><sub>演示。功能区仍是英文。</sub></p>
</td>
</tr>
</table>

编辑能力来自三个独立项目，嵌在对应标签里：[wordcraft](https://github.com/storytold/wordcraft)、[gridcraft](https://github.com/storytold/gridcraft)、[deckcraft](https://github.com/storytold/deckcraft)。OneOffice 提供窗口、标签和最近文件，不替代这三家自己的仓库。

## 快速开始

需要 Rust 1.95 或更新。[rustup](https://rustup.rs/) 的 stable 即可。

```sh
git clone --recurse-submodules https://github.com/snjgzz/oneoffice
cd oneoffice
tools/apply-gridcraft-patch.sh
tools/apply-wordcraft-patch.sh
tools/apply-deckcraft-patch.sh
cargo run -p oneoffice
```

克隆时如果没带上子模块，先补一次再打补丁：

```sh
git submodule update --init --recursive
tools/apply-gridcraft-patch.sh
tools/apply-wordcraft-patch.sh
tools/apply-deckcraft-patch.sh
```

三个脚本会改 `vendor/` 里的工作区，让上游编辑器画在这个窗口里。不打补丁，工程编不过。脚本可以重复执行。

打开一个已有文件：

```sh
cargo run -p oneoffice -- ~/Documents/预算.xlsx
```

要一个去掉调试信息的二进制：

```sh
cargo build --release -p oneoffice
```

产物在 `target/release/oneoffice`。演示用的音视频解码器会一起编进去，这个文件大约 130 MB。

### 在 Mac 上打开未签名的包

仓库里没有签过名的安装包。你自己编出的应用，或者自己打的未签名 `.app` / `.dmg`，第一次打开时 Gatekeeper 通常会拦住。在 Finder 里按住 Control 点按该应用（或右键），选择「打开」，再确认一次。

自己组装 `.app` 时，把 [`packaging/macos/OneOffice.icns`](packaging/macos/OneOffice.icns) 复制到 `OneOffice.app/Contents/Resources/OneOffice.icns`。`Info.plist` 的 `CFBundleIconFile` 写 `OneOffice`。步骤在 [`packaging/macos/README.md`](packaging/macos/README.md)。

## 系统要求

- **Rust** 1.95 或更新（edition 2024）。仓库里的 `rust-toolchain.toml` 用的是 stable。
- **macOS**：主要在 Apple Silicon 上开发和打包。Intel Mac 用同一套代码可以编，但不是平时验证的机器。
- **Linux** 能编过、能跑。系统标题栏会关掉，窗口按钮画在标签栏左侧。编译前安装：

```sh
sudo apt-get install -y pkg-config libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-0 libssl-dev libwayland-dev
```

Windows 还没有当作目标验证过。

## 快捷键

修饰键在 macOS 上是 ⌘，在 Linux 上是 Ctrl。⌘S / Ctrl+S 留给当前文档保存，壳不拦截。

| 按键 | 作用 |
| --- | --- |
| ⌘T | 回到首页 |
| ⌘W | 关闭当前文档标签（首页关不掉） |
| ⌘1–9 | 切换标签，1 是首页 |
| ⌘O | 打开文件 |
| ⌘K | 聚焦搜索 |

首页上双击最近文件会再打开它。右键可以打开所在文件夹，或从列表里移除。

最近列表的位置：

- macOS：`~/Library/Application Support/OneOffice/recent.json`
- Linux：`$XDG_CONFIG_HOME/OneOffice/recent.json`，未设置时是 `~/.config/OneOffice/recent.json`

## 已知限制

- 格式兼容远不到 LibreOffice 或桌面 Office。复杂排版、宏、修订和一部分对象会丢，或者直接打不开。
- `.doc` 可以读，保存时写成 `.docx`。`.xls`、`.xlsb`、`.ods` 可以导入，保存时写成 `.xlsx`。
- 演示的功能区、菜单和状态栏仍是英文。
- 界面字体是 Noto Sans SC 的 GB2312 子集，子集以外的汉字会缺字形。
- 同一类文档开着多个标签时，少数对话框或浮层可能互相干扰。
- 没有代码签名，也没有现成的安装包。

## 仓库结构

```
crates/shell       可执行文件 oneoffice，以及同名库
vendor/wordcraft   storytold/wordcraft，固定在某一提交
vendor/gridcraft   storytold/gridcraft，固定在某一提交
vendor/deckcraft   storytold/deckcraft，固定在某一提交
patches/           把三个编辑器嵌进壳的补丁
tools/             打补丁的脚本，以及字体子集工具
packaging/macos/   应用图标：SVG、iconset 和 OneOffice.icns
docs/images/       本页用的界面截图和图标预览
```

三个上游仓库不加入本仓库的 workspace。怎么嵌、怎么跟着上游改，写在 [`patches/README.md`](patches/README.md)。

## 参与贡献

问题和改动请开 [Issue](https://github.com/snjgzz/oneoffice/issues) 或 Pull Request。

改标签里的编辑器之前，先看 [`patches/README.md`](patches/README.md)。补丁打在子模块的工作区上，不要把打过补丁的子模块提交成上游仓库里不存在的提交。

## 路线图

已经有的：主窗口，文字、表格、演示同时开着，以及上面列出的那些 Office 文件的打开和保存。

接着会做的，不排期：

- 演示界面改成中文。
- 如果发布体积碍事，再把音视频解码器做成可选功能。
- 用更多真实文件试打开和保存，把丢内容、打不开的情况记下来，而不是先宣称兼容某一整套格式。

## 许可证与归属

代码以 MIT OR Apache-2.0 双许可发布，见 [`LICENSE-MIT`](LICENSE-MIT) 和 [`LICENSE-APACHE`](LICENSE-APACHE)。

OneOffice 不隶属于 WPS、Microsoft 或 storytold，也不使用它们或 ArtCraft 的名称做背书。

文字、表格和演示的编辑来自上面三个子模块，版权和 MIT OR Apache-2.0 许可证留在各自仓库。固定的提交和补丁名写在 [`NOTICE`](NOTICE)。

界面字体是 Noto Sans SC Regular 2.004 的子集，单独使用 SIL Open Font License 1.1。版权归 Adobe，保留字体名 Source。许可证全文在 [`crates/shell/assets/fonts/OFL.txt`](crates/shell/assets/fonts/OFL.txt)。
