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


基于 **Sunshine 基地版** 开发的 Galgame 专属定制版本，提供完整的文档支持 [Read the Docs](https://docs.qq.com/aio/DSGdQc3htbFJjSFdO?p=YTpMj5JNNdB5hEKJhhqlSB)。

**GalRemote** 是在原始 Sunshine 及 Sunshine 基地版基础上进行的深度定制版本，专为 **Galgame 玩家** 打造。不仅继承了基地版的所有优秀特性（HDR、虚拟显示器等），还集成了全新的 Galgame 管理功能和更加现代化的控制面板。

### 🌟 核心特性
- **HDR友好支持** - 经过优化的HDR处理管线，提供真正的HDR游戏流媒体体验
- **虚拟显示器** - 内置虚拟显示器管理，无需额外软件即可创建和管理虚拟显示器
- **远程麦克风** - 支持接收客户端麦克风，提供高音质的语音直通功能
- **高级控制面板** - 直观的Web控制界面，提供实时监控和配置管理
- **低延迟传输** - 结合最新硬件能力优化的编码处理
- **智能配对** - 智能管理配对设备的对应配置文件

### 🖥️ 虚拟显示器集成 (需win10 22H2 及更新的系统）
- 动态虚拟显示器创建和销毁
- 自定义分辨率和刷新率支持
- 多显示器配置管理
- 无需重启的实时配置更改

## 🎮 Sunshine Control Panel (Tauri GUI)

基于 Tauri 2.8.4 的全新 Sunshine 控制面板，提供现代化的用户界面和丰富的功能。

### ✨ 核心特性

- **现代 UI 设计** - 基于 Element Plus 的现代化界面，支持深色/浅色主题同步
- **Galgame 管理器** - 转为 Galgame 玩家打造的游戏管理功能
  - **自动刮削** - 集成 VNDB，自动获取游戏封面、简介、厂商信息
  - **游玩统计** - 记录游玩时长、最后运行时间，支持多端合并
  - **存档云同步** - 支持 WebDAV, S3, Aliyun OSS, MinIO，支持镜像同步和智能合并
  - **自动备份** - 游戏退出时自动打包存档
- **便捷配置** - 拖放背景图片，直观的 Sunshine 配置管理
- **本地代理** - 内置本地代理服务器，解决跨域问题
- **VDD 管理** - 可视化的虚拟显示器驱动管理

### 🛠️ 技术栈

- **前端**: Vue 3 + Element Plus + Less
- **后端**: Rust + Tauri 2.8.4
- **HTTP**: Axum (代理服务器)
- **构建**: Vite

### 📦 开发与构建

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建完整应用
npm run build
```


## 推荐的Moonlight客户端

建议使用以下经过优化的Moonlight客户端获得最佳的串流体验（激活套装属性）：

### 🖥️ Windows(X86_64, Arm64), MacOS, Linux 客户端
[![Moonlight-PC](https://img.shields.io/badge/Moonlight-PC-red?style=for-the-badge&logo=windows)](https://github.com/qiin2333/moonlight-qt)

### 📱 Android客户端
[![威力加强版 Moonlight-Android](https://img.shields.io/badge/威力加强版-Moonlight--Android-green?style=for-the-badge&logo=android)](https://github.com/qiin2333/moonlight-android/releases/tag/shortcut)
[![王冠版 Moonlight-Android](https://img.shields.io/badge/王冠版-Moonlight--Android-blue?style=for-the-badge&logo=android)](https://github.com/WACrown/moonlight-android)

### 📱 iOS客户端
[![虚空终端 Moonlight-iOS](https://img.shields.io/badge/Voidlink-Moonlight--iOS-lightgrey?style=for-the-badge&logo=apple)](https://github.com/The-Fried-Fish/VoidLink-previously-moonlight-zwm)


### 🛠️ 其他资源 
[awesome-sunshine](https://github.com/LizardByte/awesome-sunshine)

## 系统要求


> [!WARNING] 
> 这些表格正在持续更新中。请不要仅基于此信息购买硬件。


<table>
    <caption id="minimum_requirements">最低配置要求</caption>
    <tr>
        <th>组件</th>
        <th>要求</th>
    </tr>
    <tr>
        <td rowspan="3">GPU</td>
        <td>AMD: VCE 1.0或更高版本，参见: <a href="https://github.com/obsproject/obs-amd-encoder/wiki/Hardware-Support">obs-amd硬件支持</a></td>
    </tr>
    <tr>
        <td>Intel: VAAPI兼容，参见: <a href="https://www.intel.com/content/www/us/en/developer/articles/technical/linuxmedia-vaapi.html">VAAPI硬件支持</a></td>
    </tr>
    <tr>
        <td>Nvidia: 支持NVENC的显卡，参见: <a href="https://developer.nvidia.com/video-encode-and-decode-gpu-support-matrix-new">nvenc支持矩阵</a></td>
    </tr>
    <tr>
        <td rowspan="2">CPU</td>
        <td>AMD: Ryzen 3或更高</td>
    </tr>
    <tr>
        <td>Intel: Core i3或更高</td>
    </tr>
    <tr>
        <td>RAM</td>
        <td>4GB或更多</td>
    </tr>
    <tr>
        <td rowspan="5">操作系统</td>
        <td>Windows: 10 22H2+ (Windows Server不支持虚拟游戏手柄)</td>
    </tr>
    <tr>
        <td>macOS: 12+</td>
    </tr>
    <tr>
        <td>Linux/Debian: 12+ (bookworm)</td>
    </tr>
    <tr>
        <td>Linux/Fedora: 39+</td>
    </tr>
    <tr>
        <td>Linux/Ubuntu: 22.04+ (jammy)</td>
    </tr>
    <tr>
        <td rowspan="2">网络</td>
        <td>主机: 5GHz, 802.11ac</td>
    </tr>
    <tr>
        <td>客户端: 5GHz, 802.11ac</td>
    </tr>
</table>

<table>
    <caption id="4k_suggestions">4K推荐配置</caption>
    <tr>
        <th>组件</th>
        <th>要求</th>
    </tr>
    <tr>
        <td rowspan="3">GPU</td>
        <td>AMD: Video Coding Engine 3.1或更高</td>
    </tr>
    <tr>
        <td>Intel: HD Graphics 510或更高</td>
    </tr>
    <tr>
        <td>Nvidia: GeForce GTX 1080或更高的具有多编码器的型号</td>
    </tr>
    <tr>
        <td rowspan="2">CPU</td>
        <td>AMD: Ryzen 5或更高</td>
    </tr>
    <tr>
        <td>Intel: Core i5或更高</td>
    </tr>
    <tr>
        <td rowspan="2">网络</td>
        <td>主机: CAT5e以太网或更好</td>
    </tr>
    <tr>
        <td>客户端: CAT5e以太网或更好</td>
    </tr>
</table>

## 技术支持

遇到问题时的解决路径：
1. 查看 [使用文档](https://docs.qq.com/aio/DSGdQc3htbFJjSFdO?p=YTpMj5JNNdB5hEKJhhqlSB) [LizardByte文档](https://docs.lizardbyte.dev/projects/sunshine/latest/)
2. 在设置中打开详细的日志等级找到相关信息
3. [加入QQ交流群获取帮助](https://qm.qq.com/cgi-bin/qm/qr?k=5qnkzSaLIrIaU4FvumftZH_6Hg7fUuLD&jump_from=webapi)
4. [使用两个字母！](https://uuyc.163.com/)

**问题反馈标签：**
- `hdr-support` - HDR相关问题
- `virtual-display` - 虚拟显示器问题  
- `config-help` - 配置相关问题

## 📚 开发文档

- **[构建说明](docs/building.md)** - 项目编译和构建说明
- **[配置指南](docs/configuration.md)** - 运行时配置选项说明
- **[WebUI开发](docs/WEBUI_DEVELOPMENT.md)** - Vue 3 + Vite Web界面开发完整指南

## 加入社区

我们欢迎大家参与讨论和贡献代码！
[![加入QQ群](https://pub.idqqimg.com/wpa/images/group.png '加入QQ群')](https://qm.qq.com/cgi-bin/qm/qr?k=WC2PSZ3Q6Hk6j8U_DG9S7522GPtItk0m&jump_from=webapi&authKey=zVDLFrS83s/0Xg3hMbkMeAqI7xoHXaM3sxZIF/u9JW7qO/D8xd0npytVBC2lOS+z)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=qiin2333/Sunshine-Foundation&type=Date)](https://www.star-history.com/#qiin2333/Sunshine-Foundation&Date)

---

**Sunshine基地版 - 让游戏串流更优雅**
