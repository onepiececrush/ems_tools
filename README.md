# EMS 远程管理工具

这是一个使用 [Tauri](https://tauri.app/) 和 Rust 构建的跨平台桌面应用，用于远程管理 Linux 服务器。它提供了一个简洁的图形用户界面来执行常见的运维任务。

## ✨ 核心功能

- **SSH 安全连接**: 使用 IP 地址、用户名和密码安全地连接到远程 Linux 主机。
- **文件传输**: 支持将本地文件上传到远程服务器的指定目录（默认为 `/ems`）。
- **网络配置**:
  - 自动检测并列出远程服务器上的所有网络接口 (如 `eth0`, `eth1`)。
  - 查看和修改指定网络接口的 IP 地址、子网掩码和默认网关。
- **服务管理 (Systemd)**:
  - 导入新的 `systemd` 服务单元文件。
  - 控制服务的生命周期（启动、停止）。
  - 管理服务的自启动状态（启用、禁用）。
  - 查询服务的当前状态。
- **实时操作日志**: 在界面上提供一个日志窗口，显示所有后端操作的实时反馈，包括成功信息和错误详情。

## 🛠️ 技术栈

- **核心框架**: [Tauri](https://tauri.app/) - 使用 Web 技术构建轻量、快速、安全的跨平台桌面应用。
- **后端**: Rust
  - `ssh2-rs`: 用于处理 SSH 和 SFTP 连接。
  - `tokio`: 提供异步运行时。
  - `serde`: 用于数据的序列化和反序列化。
- **前端**: Vanilla JavaScript, HTML, CSS (无前端框架)

## 🚀 如何开始

### 1. 环境准备

在开始之前，请确保您已经安装了 [Rust](https://www.rust-lang.org/tools/install) 和 [Node.js](https://nodejs.org/) (包含 npm)。

同时，您需要根据您的操作系统，安装 Tauri 的系统依赖。请参考官方文档：[Prerequisites for Tauri Development](https://tauri.app/v1/guides/getting-started/prerequisites)。

### 2. 安装依赖

克隆项目后，在项目根目录打开终端，运行以下命令来安装前端和后端的依赖：

```bash
# 安装 Node.js 依赖
npm install

# Cargo 会在第一次运行时自动处理 Rust 依赖
```

### 3. 运行开发环境

安装完依赖后，使用以下命令启动应用进入开发模式。应用会以开发模式启动，并开启热重载。

```bash
npm run tauri dev
```

### 4. 构建生产版本

要构建一个可分发的、经过优化的生产版本，请运行：

```bash
npm run tauri build
```
构建产物将位于 `/src-tauri/target/release/bundle/` 目录下。

## 📁 项目结构

```
.
├── src/                      # 前端文件
│   ├── index.html            # 应用主页面
│   ├── main.js               # 前端逻辑
│   └── styles.css            # 样式文件
└── src-tauri/                # 后端 Rust & Tauri 配置
    ├── build.rs              # Tauri 构建脚本
    ├── Cargo.toml            # Rust 依赖配置
    ├── icons/                # 应用图标
    ├── src/                  # Rust 源码
    │   ├── main.rs           # 应用入口，Tauri 初始化
    │   ├── ssh.rs            # SSH 连接与命令执行
    │   ├── file_transfer.rs  # SFTP 文件传输
    │   ├── network.rs        # 网络配置模块
    │   ├── service.rs        # Systemd 服务管理
    │   └── logger.rs         # 日志处理
    └── tauri.conf.json       # Tauri 核心配置文件
``` 