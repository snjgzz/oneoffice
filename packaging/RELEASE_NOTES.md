这些文件没有代码签名。

**macOS（Apple Silicon）**

- `OneOffice-<版本>-macos-aarch64.dmg` 里是 `OneOffice.app`。图标是仓库里的 `packaging/macos/OneOffice.icns`。
- 第一次打开时，Gatekeeper 通常会拦住。在 Finder 里按住 Control 点按 OneOffice（或右键），选择「打开」，再确认一次。

**Windows（x86_64）**

- `OneOffice-<版本>-windows-x86_64-setup.exe` 是 NSIS 安装程序。它把 `oneoffice.exe` 装到 `C:\Program Files\OneOffice`，并在开始菜单放快捷方式。
- 没有开机启动项。安装时会请求管理员权限。
- SmartScreen 可能提示未知发布者。这是未签名安装包的系统提示。

安装包只包含这次编出的 OneOffice 程序。格式兼容和其它限制见仓库 README 的「已知限制」。
