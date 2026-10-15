# Windows 安装包

目标是 64 位 Windows（x86_64）。安装包是 NSIS 打的 `.exe`，没有代码签名。

它把 `oneoffice.exe` 装到 `C:\Program Files\OneOffice`，在开始菜单放一个快捷方式，并写一条卸载项。不创建桌面图标，也不写开机启动。

先编 release，再打包（需要本机的 `makensis`）：

```powershell
cargo build --release -p oneoffice
packaging\windows\package.ps1 v0.1.0
```

产物是 `dist\OneOffice-<版本>-windows-x86_64-setup.exe`。GitHub Actions 在 `windows-latest` 上做同样的事，见 [`.github/workflows/release.yml`](../../.github/workflows/release.yml)。

安装时会请求管理员权限。SmartScreen 可能提示未知发布者，因为安装包未签名。
