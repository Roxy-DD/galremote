# 第五章 系统核心模块实现

本章将详细介绍 GalRemote 系统的核心代码实现细节与关键技术突破。结合第四章的架构设计，本章将分别从 C++ 底层串流服务、Rust (Tauri) 中台层业务逻辑以及 Vue 3 前端界面交互三个维度，阐述系统是如何将理论设计转化为实际工程代码的。

## 5.1 开发环境与核心依赖包

为保证跨平台编译的稳定性与系统运行的高效性，本系统选用了以下开发环境与核心组件：

- **底层引擎 (C++)**: 
  - 编译器: MSVC / MinGW-w64 (GCC 13+)
  - 核心构建流: CMake + Ninja
  - 关键依赖: `Boost` (系统进程与网络), `ViGEmClient` (虚拟手柄注入), `NVENC/AMF` (硬件视频抗锯齿与编码)。
- **中台管理层 (Rust)**:
  - 编译器: Rust `rustc 1.70+` (Edition 2021)
  - 框架平台: `Tauri 1.x / 2.0`
  - 关键依赖: `tokio` (高并发异步 I/O 运行时), `Axum` (REST 代理与流媒体路由), `reqwest` (VNDB 网络请求), `serde_json` (数据字典持久化)。
- **展现层 (前端)**:
  - 运行环境: `Node.js 18+`, `Vite 4.5+`
  - 技术栈: `Vue 3` (Composition API), `Element Plus` (UI 组库), `Pinia` (全局状态管理), `TailwindCSS` (原子化 CSS)。

---

## 5.2 串流底层核心控制实现 (C++ 端)

C++ 引擎主要处理最高频的视音频数据流与外设信号捕获。本节以系统的 **虚拟信号注入 (Input Inject)** 和 **系统进程兼容性修复** 为例进行代码级剖析。

### 5.2.1 触控与手柄虚拟操作注入实现
在手机端连接后，客户端会持续发送带有时间戳的触控点或陀螺仪参数。引擎内部的 `input.cpp` 负责将这些结构化数据还原为操作系统的全局事件。

```cpp
// 节选自 src/platform/windows/input.cpp
// 绝对坐标防抖动与鼠标位置注入实现
void abs_mouse(input_t &input, const touch_port_t &touch_port, float x, float y) {
  INPUT i {};
  i.type = INPUT_MOUSE;
  auto &mi = i.mi;
  
  // MOUSEEVENTF_VIRTUALDESK 映射到包含所有虚拟显示器的整个桌面空间
  mi.dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;

  // 根据客户端推介的分辨率进行等比例视口转换缩放
  auto scaled_x = std::lround((x + touch_port.offset_x) * ((float) target_touch_port.width / (float) touch_port.width));
  auto scaled_y = std::lround((y + touch_port.offset_y) * ((float) target_touch_port.height / (float) touch_port.height));

  mi.dx = scaled_x;
  mi.dy = scaled_y;

  send_input(i); // 丢入系统队列底层
}
```

针对高级的虚拟手柄（如通过安卓屏幕搓手柄映射的摇杆），则通过 `ViGEmClient` (Virtual Gamepad Emulation Framework)在 Ring 0 层面欺骗 Windows，让系统认为插上了一只真实的 Xbox 360 控制器，由于代码涉密较底层，此处不予长篇列出。

### 5.2.2 环境重定义冲突防护机制
实际开发中，因系统 API 版本迭代常常产生宏定义越界或重复（如 MinGW 头文件变更导致的编译 Panic）。在本项目中，针对 `HSYNTHETICPOINTERDEVICE` 系统类型定义的冲突问题，设计了精准的防护宏：

```cpp
// 通过预编译头指令规避跨版本 MinGW-w64 的重定义编译终止异常
#ifdef __MINGW32__
#ifndef HSYNTHETICPOINTERDEVICE // 确保只有未定义时才暴露该柄
DECLARE_HANDLE(HSYNTHETICPOINTERDEVICE);
#endif
WINUSERAPI HSYNTHETICPOINTERDEVICE WINAPI
CreateSyntheticPointerDevice(POINTER_INPUT_TYPE pointerType, ULONG maxCount, POINTER_FEEDBACK_MODE mode);
// ... 注销与注入接口绑定
#endif
```

---

## 5.3 Rust 中台业务逻辑实现 (Tauri 端)

Rust 中台层 (`src-tauri` 目录) 承载着数据存取的核心逻辑，其基于 `tokio` 的并发调度在处理 I/O 密集型任务时展现出极高的性能优势。

### 5.3.1 并发刮削与网络通信 (VNDB API)
此模块需要遍历大量文件夹并向远端 API 去重发起请求。通过 `reqwest` 构建异步连接池，并借助 `serde` 解析庞大的 JSON 返回树：

```rust
// 伪代码: 针对 VNDB GraphQL 的异步请求与提取
#[tauri::command]
pub async fn scrape_galgame_info(title: String) -> Result<GameInfo, String> {
    let client = reqwest::Client::new();
    let query = format!(
        r#"{{"query": "query {{ vn(search: \"{}\", sort: \"searchrank\") {{ results {{ id title image.url description }} }} }}"}}"#,
        title
    );

    // 采用异步等待避免阻塞 Tauri 主线程 (GUI无响应)
    let res = client.post("https://api.vndb.org/kana/graphql")
        .header("Content-Type", "application/json")
        .body(query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    // 解析最匹配的第一条记录并组织构建领域模型实体 (Entity)
    // ...
}
```

### 5.3.2 虚拟屏幕挂载与心跳托管 (VDD Lifecycle)
中台在启动时向托盘注册自身生命周期，并暴露出对 `VDD` 虚拟外设的命令行注入接口。

```rust
// 动态切换屏幕分辨率
pub fn inject_virtual_display(width: u32, height: u32, hz: u32) -> Result<(), AppError> {
    // 构造驱动层所需要的特定参数矩阵
    let args = format!("add {} {} {}", width, height, hz);
    
    // 调用特权命令执行设备树挂载 (需 UAC 提权)
    let output = std::process::Command::new("vdd_manager.exe")
        .args(args.split_whitespace())
        .output()?;
        
    if output.status.success() { Ok(()) } else { Err(AppError::DriverFailed) }
}
```

---

## 5.4 现代化前端展现实现 (Vue 3 端)

前端(`sunshine-control-panel` 面板项目) 通过响应式系统，动态展现各种状态面板与云存档操作。

### 5.4.1 前端跨进程路由调用机制 (Tauri IPC)
基于 `@tauri-apps/api/invoke`，Vue 3 的 `setup` 生命周期中通过封装实现了无缝对接后端的能力：

```javascript
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted } from 'vue'

export default {
  setup() {
    const games = ref([])
    const isLoading = ref(false)

    // 挂载时触发全量库读取
    onMounted(async () => {
      try {
        isLoading.value = true
        // 无缝调用 Rust 层的读取命令，跨过网络层直接拿到底层内存映射
        games.value = await invoke('get_all_galgames_dict')
      } catch(e) {
        console.error("加载存档仓库失败", e)
      } finally {
        isLoading.value = false
      }
    })

    return { games, isLoading }
  }
}
```

### 5.4.2 瀑布流组件与虚化封面渲染
界面采用了 `v-for` 循环与瀑布流布局将从云端或本地懒加载的封面渲染至 DOM。通过 CSS 的 `backdrop-filter: blur(20px)` 动态生成每款游戏的沉浸式高斯模糊背景膜，使界面呈现极其现代化、类似主机 OS 的原生质感，解决了传统 Web 面板渲染长列表卡顿的问题（配合虚拟滚动 `v-virtual-scroll`技术）。

---

## 5.5 智能云同步算法的工程落地

第四章设计的云同步冲突解决（并集+时间戳）算法在 Rust 端实现如下控制流：

1. **差异比对哈希阶段**: 读取所有包含 `.sav`/`.sys` 后缀的文件，使用 `SHA-256` 散列算法结合系统 `fs::metadata().modified` 生成当前快照(Snapshot Tree)。
2. **异步传输队列**:
   - 当遇到网络受限时，基于 `tokio::sync::mpsc` 构建的多生产者-单消费者消息管道能动态压制过多的小文件上传请求，防止因大批量小文件 IO 导致的 TCP 窗口阻塞。
3. **回滚压缩**:
   借助 `zip` 库或 `tar-gz` 工具链，在触发覆盖之前，强行锁死文件句柄并生成不可变快照记录：
   `Archive::new(file).compress(current_dir).unwrap()`。

## 5.6 本章小结
本章详细剖析了系统在代码实现层的难点与解决方案。通过对涉及硬件注入的 C++ 底册、并发网络调度的 Rust 业务层、以及跨进程交互的 Vue 3 前端的核心代码解析，证实了第四章所提架构设计的工程落地可行性。系统的所有子模块在此阶段已整合运行并实现了互联互通。
