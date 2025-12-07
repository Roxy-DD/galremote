# GalRemote (Sunshine Galgame Edition)

## 🌐 多语言支持 / Multi-language Support

<div align="center">

[![English](https://img.shields.io/badge/English-README.en.md-blue?style=for-the-badge)](README.en.md)
[![中文简体](https://img.shields.io/badge/中文简体-README.zh--CN.md-red?style=for-the-badge)](README.md)
[![Français](https://img.shields.io/badge/Français-README.fr.md-green?style=for-the-badge)](README.fr.md)
[![Deutsch](https://img.shields.io/badge/Deutsch-README.de.md-yellow?style=for-the-badge)](README.de.md)
[![日本語](https://img.shields.io/badge/日本語-README.ja.md-purple?style=for-the-badge)](README.ja.md)

</div>

---

**GalRemote** 是基于 **Sunshine 基地版** 深度定制的串流服务端，专为 **Galgame 玩家** 打造。

它不仅继承了 Sunshine 基地版在 HDR、虚拟显示器管理等方面的所有优势，更内置了强大的 **Galgame 管理中心** 和全新的现代化 **Control Panel**，为您提供从游戏管理、存档云同步到远程串流的一站式解决方案。

## 🌟 核心亮点

### 🎮 Galgame 管理中心 (Built-in Manager)
专为视觉小说和 Galgame 设计的本地库管理功能，告别杂乱的文件夹。
- **自动刮削**: 集成 VNDB (Visual Novel Database) API，自动获取游戏封面、开发商、发售日期和简介信息。
- **智能游玩统计**: 精确记录每次游玩的起止时间和时长，支持跨设备合并统计数据。
- **一键启动**: 直接从面板启动游戏，自动接管串流会话。

### ☁️ 这个杀手不太冷 (Ultimate Cloud Sync)
也许是目前最强大的 Galgame 存档同步方案。
- **多后端支持**: 原生支持 WebDAV, S3, Aliyun OSS, MinIO 等多种云存储协议。
- **镜像级同步**: 本地存档的变动（包括删除）会实时镜像到云端，确保存档状态完全一致。
- **智能冲突合并 (Intelligent Merge)**: 
    - 自动识别多端游玩记录，通过并集算法智能合并游玩历史和总时长。
    - 智能处理游戏状态（未开始 -> 进行中 -> 已完成）。
    - 自动合并元数据（描述、厂商信息），优先保留云端更完整的信息。
- **剪贴板云同步**: 支持多端剪贴板文本的实时云同步，方便跨设备复制粘贴攻略或翻译。

### 🛡️ 存档守护者 (Save Guardian)
再也不用担心掉档或坏档。
- **自动快照**: 游戏退出时自动创建存档快照。
- **版本回溯**: 支持查看和还原任意时间点的存档快照。
- **快照管理**: 可视化管理存档历史，支持手动创建和删除快照。

### �️ 现代化控制面板 (Control Panel)
基于 Tauri 2.8.4 + Vue 3 重构的全新界面。
- **桌面级 UI 框架**: 采用模块化设计的 Desktop UI 组件库，提供原生应用的流畅体验。
- **虚拟显示器管理 (VDD)**: 可视化管理虚拟显示器，支持动态分辨率和刷新率调整。
- **本地代理服务器**: 内置 Axum 代理，彻底解决 WebUI 的跨域和安全策略问题。
- **个性化定制**: 支持拖拽更换背景图，深色/浅色主题自动同步。

## 🚀 推荐客户端配置

为了获得最佳体验（尤其是激活 HDR 和 120Hz 串流），推荐搭配以下客户端：

### 🖥️ PC / Mac / Linux
[![Moonlight-PC](https://img.shields.io/badge/Moonlight-PC-red?style=for-the-badge&logo=windows)](https://github.com/qiin2333/moonlight-qt)

### 📱 Android
[![威力加强版 Moonlight-Android](https://img.shields.io/badge/威力加强版-Moonlight--Android-green?style=for-the-badge&logo=android)](https://github.com/qiin2333/moonlight-android/releases/tag/shortcut)
[![王冠版 Moonlight-Android](https://img.shields.io/badge/王冠版-Moonlight--Android-blue?style=for-the-badge&logo=android)](https://github.com/WACrown/moonlight-android)

### 📱 iOS
[![虚空终端 Moonlight-iOS](https://img.shields.io/badge/Voidlink-Moonlight--iOS-lightgrey?style=for-the-badge&logo=apple)](https://github.com/The-Fried-Fish/VoidLink-previously-moonlight-zwm)

---

## 🛠️ 技术架构

本项目采用了前后端分离的现代化架构：

### Frontend (User Interface)
- **Framework**: Vue 3 (Composition API)
- **UI Library**: Element Plus + 自研 Desktop UI Framework
- **Style**: Less
- **Build Tool**: Vite

### Backend (Core Logic)
- **Runtime**: Rust (Tauri 2.8.4)
- **Proxy**: Axum (HTTP Proxy)
- **Storage**: Serde JSON (Config), OpenDAL (Cloud Sync)
- **Process**: Win32 API (Game Launching & Monitoring)

## 📦 开发与构建

如果您想参与开发或自行构建：

```bash
# 1. 安装依赖
npm install

# 2. 启动开发环境 (Tauri + Vite)
npm run dev

# 3. 仅构建 Web 前端
npm run build:renderer

# 4. 构建完整 Windows 应用
npm run build:win
```

## 📄 系统要求

| 组件 | 最低要求 | 推荐配置 (4K HDR) |
|------|---------|------------------|
| **OS** | Windows 10 22H2+ | Windows 11 23H2+ |
| **GPU** | NVIDIA GTX 10-series / AMD RX 5000+ | NVIDIA RTX 30-series+ |
| **RAM** | 8 GB | 16 GB |
| **Network**| 5GHz Wi-Fi (ac) | Wi-Fi 6 (ax) / 2.5G LAN |

---

## 🤝 社区与支持

- [加入 QQ 交流群](https://qm.qq.com/cgi-bin/qm/qr?k=WC2PSZ3Q6Hk6j8U_DG9S7522GPtItk0m&jump_from=webapi)
- [查看详细文档](https://docs.qq.com/aio/DSGdQc3htbFJjSFdO?p=YTpMj5JNNdB5hEKJhhqlSB)
- [提交 Issue](https://github.com/Roxy-DD/galremote/issues)

## ❤️ 鸣谢与支持 (Credits & Sponsor)

**GalRemote** 站在巨人的肩膀上。本项目的诞生离不开 **Sunshine 基地版 (Sunshine Foundation)** 团队的卓越工作。

如果您喜欢本项目带来的体验，**请把您的赞助给予 Sunshine 基地版的开发者们**，是他们的无私奉献让这一切成为可能：

- [爱发电 - qiin2333](https://www.ifdian.net/a/qiin2333)
- [爱发电 - Yundi339](https://www.ifdian.net/a/Yundi339)

**注：GalRemote 项目本身不接受任何形式的捐赠。**

**GalRemote** - 让每一次点击都值得铭记。
