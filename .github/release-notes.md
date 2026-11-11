请选择与你的系统和处理器对应的安装包：

| 系统 | 文件后缀 | 使用方式 |
| --- | --- | --- |
| Windows x64 | `windows-x64-setup.exe` | 双击安装后从开始菜单启动 Purge |
| Linux x64 | `linux-x64.tar.gz` | 解压后运行 `bin/purge` |
| Linux ARM64 | `linux-arm64.tar.gz` | 解压后运行 `bin/purge` |
| macOS Intel | `macos-x64.zip` | 解压后将 `Purge.app` 放入 Applications |
| macOS Apple Silicon | `macos-arm64.zip` | 解压后将 `Purge.app` 放入 Applications |

Windows 安装器默认按当前用户安装，支持覆盖安装、开始菜单入口和可选桌面快捷方式；卸载会保留用户设置与清理历史。安装器尚未进行代码签名。

Linux 包基于 Ubuntu 24.04 构建，需要兼容的系统库和 Vulkan 驱动，同时支持原生 Wayland 与 X11。桌面入口与图标位于包内的 `share/` 目录。

macOS 应用采用本地 ad-hoc 签名，尚未进行 Apple Developer ID 签名与公证；首次打开可能需要在系统“隐私与安全性”设置中允许运行。

`SHA256SUMS` 包含全部安装包的校验值。所有安装包都包含适用的第三方许可。
