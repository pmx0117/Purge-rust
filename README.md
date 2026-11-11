# Purge

面向 Windows、Linux 和 macOS 的本地磁盘清理工具，使用 Rust + GPUI 原生渲染。支持中文界面、浅色与深色主题，扫描和文件比较均在本机完成。

## 功能

| 功能 | 说明 |
| --- | --- |
| 应用缓存 | 检查浏览器、编辑器和常用应用的已知缓存，以及超过 24 小时的用户专属临时目录中的条目 |
| 开发缓存 | 检查包管理器缓存，以及手动添加的项目目录中的闲置构建产物 |
| 大文件 | 检查下载、桌面、文档、视频、音乐和图片等用户文件夹，支持搜索、类型筛选和排序 |
| 重复文件 | 按大小、SHA-256 和逐字节比较确认重复内容，已识别的重复组至少保留一份 |
| 应用管理 | 卡片 / 列表布局切换，显示本机应用图标与安装信息，通过系统应用管理入口管理或卸载 |
| 系统托盘 | 最小化后保留后台任务，通过托盘恢复或退出；托盘不可用时使用普通最小化 |
| 清理与历史 | 清理前审核，文件移入系统回收站，保存本地设置与清理记录 |

扫描范围由已知缓存目录、各系统的用户文件夹和手动添加的项目目录组成，不执行整盘遍历。同一应用的缓存合并展示，实际清理仍逐路径复核。

临时文件与崩溃报告固定保留最近 24 小时内修改的内容。设置中的“开发项目闲置时间”仅用于手动添加的项目目录，不影响临时文件或应用下载缓存；修改后需要重新扫描。

## 构建运行

需要 Rust 1.92 或更新版本。在目标操作系统安装本地构建工具后运行：

```sh
cargo run --locked -p purge
cargo build --locked --release -p purge
```

| 平台 | 环境与依赖 |
| --- | --- |
| Windows 10 / 11 x64 | Rust MSVC 工具链、Visual Studio C++ Build Tools、Windows SDK、支持 Direct3D 11 的显卡驱动 |
| Linux x64 / arm64 | C/C++ 工具链、pkg-config、Vulkan 驱动、X11 或 Wayland、Fontconfig、FreeType、XKB、ALSA 开发库 |
| macOS | Xcode Command Line Tools，使用系统 Metal 后端；安装应用时使用 .app 包及 ICNS 图标 |

Linux（Debian/Ubuntu）依赖示例：

```sh
sudo apt install build-essential pkg-config clang libssl-dev libasound2-dev \
  libfontconfig1-dev libfreetype6-dev libx11-dev libx11-xcb-dev libxcb1-dev \
  libxcb-xkb-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libvulkan-dev fonts-noto-cjk
```

Windows Release 构建需要 HLSL 编译器。如果没有配置 GPUI_FXC_PATH，可在 PowerShell 中先执行：

```powershell
$compiler = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\fxc.exe" |
    Sort-Object FullName -Descending | Select-Object -First 1
$env:GPUI_FXC_PATH = $compiler.FullName
```

程序资源在构建时嵌入，无需另外安装字体或 WebView。构建脚本从同一份标志生成 Windows 多分辨率 ICO、macOS ICNS 和 PNG，输出位于 target 的 build 子目录，运行时不依赖源码路径。

Linux 任务栏使用 dev.purge.Purge 应用标识。安装时将 assets/dev.purge.Purge.desktop 放入用户 applications 目录，并将构建生成的 PNG 安装为 hicolor 主题中的 dev.purge.Purge.png。macOS 的 .app 包应包含主程序、Info.plist 和生成的 purge.icns；直接 cargo run 适合开发验证。

Linux 优先使用原生 Wayland；没有 WAYLAND_DISPLAY 时使用 X11。窗口请求客户端装饰，按系统实际协商结果显示一套标题栏和窗口按钮。X11/WSLg 的窗口管理器可能保留系统边框，原生 Wayland 可避免该系统白框。需要单独检查 X11 后端时可运行 `env -u WAYLAND_DISPLAY cargo run --locked -p purge`。

## 托盘行为

设置中的“最小化到托盘”默认开启。最小化按钮、Ctrl+M（macOS 为 Cmd+M）和设置页的“收起窗口”使用同一行为；托盘菜单可显示主窗口或退出。隐藏后扫描和清理任务继续运行，恢复时保留结果、筛选和选择。

关闭窗口会退出应用；清理执行期间会阻止退出。Linux 托盘使用 StatusNotifier 协议，需要桌面提供相应宿主；没有宿主时保留普通最小化，宿主消失时恢复窗口，避免应用无法找回。

## 检查与测试

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --no-default-features
```

无图形环境检查 Clippy 时添加 `--no-default-features`。无界面测试覆盖领域规则、各平台路径与扫描目录、持久化、后台任务及托盘生命周期；桌面图形、窗口控制和托盘菜单需在对应系统验证。

当前已在 Windows 验证原生图标及托盘隐藏、恢复、退出；在 Linux 原生 Wayland 和 X11/WSLg 验证桌面启动，Wayland 还验证了最大化、还原、改尺寸和最小化请求。macOS 核心与原生窗口接口已做目标编译检查，完整图形界面的 macOS 实机验收尚未完成。

测试默认使用临时样本和注入回收器，不清理本机用户缓存。

## 清理规则

- 清理前需要审核所选项目。安全标签来自规则分类，不代表已检测所有进程占用；应先关闭相关应用。
- 执行前重新检查白名单、排除目录、链接和扫描指纹，变化或无法完整验证的路径会跳过。
- 项目构建产物必须具有对应项目清单，并核对源文件和 Git 活动；`.purge-keep` 和 `.git/index.lock` 可保护项目。
- 清理使用系统回收站，不以永久删除作为失败回退，也不自动清空回收站。移入回收站本身不会释放磁盘可用空间。
- 大文件与重复文件不会自动勾选；已安装应用不作为文件目录回收。

本地设置和历史按系统存储：

| 平台 | 数据目录 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\Purge` |
| Linux | `$XDG_DATA_HOME/Purge`，未设置时为 `~/.local/share/Purge` |
| macOS | `~/Library/Application Support/Purge` |

程序不保存扫描快照，不上传扫描数据，无遥测。

当前不提供定时清理、开机启动、自动更新或卸载残留清理。Windows 应用安装大小来自注册表估算，Linux 应用可能没有可靠的安装大小；文件统计为逻辑大小，可能与实际磁盘占用不同。

## 项目结构

```text
crates/
  purge-core/       领域类型、分组与选择规则
  purge-engine/     扫描、清理校验、后台任务、平台和存储
  purge-desktop/    应用模型、控制器和 GPUI 桌面界面
assets/             字体、矢量图标、Windows 资源和 Linux 桌面入口
licenses/           第三方许可证
```

依赖方向为 `purge → purge-engine → purge-core`，桌面侧也直接使用 core 的领域类型。core 不依赖文件系统和界面；engine 不依赖 GPUI；桌面控制器协调应用模型、后台任务与持久化。

GPUI 通过官方 Zed 仓库的固定提交 `5eb2ff065badabe91d492cdcb51447d2f4104401` 引用，使用官方 `QuitMode::Explicit` 控制托盘驻留，支持 xdg-shell 版本 1。项目不复制或修改 GPUI 源码；首次构建需要下载 Git 依赖。

源码仓库保留 Cargo.lock 和构建所需 assets。`target/`、`dist/`、截图、日志、质量报告和个人配置由 `.gitignore` 排除，不随源码提交。

## 第三方许可

本项目自有代码暂未单独指定开源许可证。第三方组件与复用资源保留原许可文本，适用范围见 [licenses/README.md](licenses/README.md)。
