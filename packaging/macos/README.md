# macOS 应用图标

青绿到蓝的圆角底，白色粗体 O，后面是三份半透明文档。源文件在这个目录里：

| 文件 | 用途 |
| --- | --- |
| `OneOffice.svg` | 矢量源 |
| `OneOffice-1024.png` | 1024×1024 主图 |
| `OneOffice.iconset/` | `iconutil` 用的各档 PNG |
| `OneOffice.icns` | 已经生成的图标，打进 `.app` 时用这个 |

仓库还没有签名过的安装包，也没有自动打包脚本。自己组装 `.app` 时，把图标放到包内的资源目录，文件名保持 `OneOffice.icns`：

```sh
mkdir -p OneOffice.app/Contents/Resources
cp packaging/macos/OneOffice.icns OneOffice.app/Contents/Resources/OneOffice.icns
```

`Contents/Info.plist` 里对应写：

```xml
<key>CFBundleIconFile</key>
<string>OneOffice</string>
```

`CFBundleIconFile` 不写 `.icns` 后缀。Finder 和程序坞读的是包里的这个文件，不是仓库里的路径。

在 Mac 上要从 iconset 重新生成 icns（需要本机的 `iconutil`）：

```sh
cd packaging/macos
iconutil -c icns OneOffice.iconset -o OneOffice.icns
```
