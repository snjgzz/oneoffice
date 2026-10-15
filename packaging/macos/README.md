# macOS 应用图标

青绿到蓝的圆角底，白色粗体 O，后面是三份半透明文档。源文件在这个目录里：

| 文件 | 用途 |
| --- | --- |
| `OneOffice.svg` | 矢量源 |
| `OneOffice-1024.png` | 1024×1024 主图 |
| `OneOffice.iconset/` | `iconutil` 用的各档 PNG |
| `OneOffice.icns` | 已经生成的图标，打进 `.app` 时用这个 |

打未签名的 `.dmg`（需要本机的 `hdiutil`，并且已经 `cargo build --release -p oneoffice`）：

```sh
packaging/macos/package-dmg.sh v0.1.0
```

脚本会把 release 二进制放进 `OneOffice.app/Contents/MacOS/oneoffice`，把 `OneOffice.icns` 复制到 `Contents/Resources/`，并用 [`Info.plist`](Info.plist) 填版本号。`CFBundleIconFile` 写的是 `OneOffice`，不带 `.icns` 后缀。Finder 和程序坞读的是包里的这个文件。

产物是 `dist/OneOffice-<版本>-macos-<架构>.dmg`。GitHub 的 macOS runner 是 Apple Silicon，所以 Actions 打出来的是 `macos-aarch64`。这个包没有签名。

在 Mac 上要从 iconset 重新生成 icns（需要本机的 `iconutil`）：

```sh
cd packaging/macos
iconutil -c icns OneOffice.iconset -o OneOffice.icns
```
