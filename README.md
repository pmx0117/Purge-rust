# Purge for Windows

面向 Windows 的本地磁盘清理工具，使用 Rust + GPUI 原生渲染。支持中文界面、浅色与深色主题，扫描和文件比较均在本机完成。

## 功能

| 功能 | 说明 |
| --- | --- |
| 应用缓存 | 检查浏览器、编辑器和常用应用的已知缓存，以及超过 24 小时的用户临时文件 |
| 开发缓存 | 检查包管理器缓存，以及手动添加的项目目录中的闲置构建产物 |
| 大文件 | 检查下载、桌面、文档、视频、音乐和图片等用户文件夹，支持搜索、类型筛选和排序 |
| 重复文件 | 按大小、SHA-256 和逐字节比较确认重复内容，已识别的重复组至少保留一份 |
| 应用管理 | 显示已安装应用信息，通过 Windows 设置完成卸载 |
| 清理与历史 | 清理前审核，文件移入系统回收站，保存本地设置与清理记录 |

扫描范围由已知缓存目录、Windows 用户文件夹和手动添加的项目目录组成，不执行整盘遍历。同一应用的缓存合并展示，实际清理仍逐路径复核。

## 构建运行

需要 Windows 10 / 11 x64、支持 Direct3D 11 的显卡驱动，以及以下开发工具：

- Rust stable，使用 MSVC 工具链。
- Visual Studio C++ Build Tools。
- Windows SDK。

在 Windows PowerShell 中运行：

```powershell
cargo run --locked -p purge
```

程序资源在构建时嵌入，无需额外安装字体或 WebView。WSL 可用于编辑和无界面测试，桌面程序应使用 Windows Rust 工具链构建运行。

Release 构建需要 Windows SDK 的 HLSL 编译器。未设置 `GPUI_FXC_PATH` 时，可先查找已安装的编译器：

```powershell
$compiler = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\fxc.exe" |
    Sort-Object FullName -Descending | Select-Object -First 1
$env:GPUI_FXC_PATH = $compiler.FullName
cargo build --locked --release -p purge
```

生成的程序位于 `target/release/purge.exe`。

## 检查与测试

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --no-default-features
```

Linux/WSL 检查 Clippy 时添加 `--no-default-features`。无界面测试覆盖领域规则、扫描引擎、持久化、后台任务及应用模型；GPUI 桌面渲染需要在 Windows 下验证。

测试默认使用临时样本和注入回收器，不清理本机用户缓存。

## 清理规则

- 清理前需要审核所选项目。安全标签来自规则分类，不代表已检测所有进程占用；应先关闭相关应用。
- 执行前重新检查白名单、排除目录、链接和扫描指纹，变化或无法完整验证的路径会跳过。
- 项目构建产物必须具有对应项目清单，并核对源文件和 Git 活动；`.purge-keep` 和 `.git/index.lock` 可保护项目。
- 清理使用 Windows 回收站，不以永久删除作为失败回退，也不自动清空回收站。移入回收站本身不会释放磁盘可用空间。
- 大文件与重复文件不会自动勾选；已安装应用不作为文件目录回收。

本地设置和历史位于 `%LOCALAPPDATA%\Purge`。程序不保存扫描快照，不上传扫描数据，无遥测。

当前不提供定时清理、托盘、开机启动、自动更新或卸载残留清理。应用安装大小来自注册表估算；文件统计为逻辑大小，可能与实际磁盘占用不同。

## 项目结构

```text
crates/
  purge-core/       领域类型、分组与选择规则
  purge-engine/     扫描、清理校验、后台任务、平台和存储
  purge-desktop/    应用模型、控制器和 GPUI 桌面界面
assets/             编译所需的字体、图标和 Windows 资源
licenses/           第三方许可证
```

依赖方向为 `purge → purge-engine → purge-core`，桌面侧也直接使用 core 的领域类型。core 不依赖文件系统和界面；engine 不依赖 GPUI；桌面控制器协调应用模型、后台任务与持久化。

源码仓库保留 Cargo.lock 和构建所需 assets。`target/`、`dist/`、截图、日志、质量报告和个人配置由 `.gitignore` 排除，不随源码提交。

## 第三方许可

本项目自有代码暂未单独指定开源许可证。第三方组件与复用资源保留原许可文本，适用范围见 [licenses/README.md](licenses/README.md)。
