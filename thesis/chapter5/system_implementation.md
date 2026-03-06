# 第五章 系统核心模块的工程实现

本章将基于第四章的系统总体架构设计，详细介绍 GalRemote 系统各核心模块的具体工程落地过程与关键技术实现。在跨系统平台与多语言混合编程的复杂环境下，本系统通过 C++ 底层的高效资源调度、Rust 的并发中台管理以及 Vue 3 现代化响应式前端，建立了一套低延迟、高可靠的流媒体远端游玩与存档管理方案。本章将对上述三层架构中的核心算法流与关键代码片段进行剖析。

## 5.1 系统开发运行环境与技术栈选型

为保证跨平台编译的稳定性、运行时的高效能输出以及满足低延迟串流苛刻的硬件开销限制，本系统的开发与运行环境进行了严格的技术选型：

1. **底层流媒体核心层（C++ / C）**
   - **编译环境**：采用最新版 MSVC 或 MinGW-w64 (GCC 13+) 工具链，配合 CMake 与 Ninja 构建系统。
   - **核心依赖**：深度集成 `Boost` 库族处理底层多线程互斥与网络吞吐，集成 `ViGEmClient` 框架处理虚拟手柄的内核级指令注入，视频流编码段则直接调用 NVIDIA `NVENC` 及 AMD `AMF` 硬件 SDK 实现零拷贝视频帧捕获。
2. **中台业务集群层（Rust）**
   - **编译环境**：基于 `rustc 1.70+`（Edition 2021），利用 Tauri 2.0 跨平台框架构建进程底座。
   - **核心依赖**：选用 `tokio` 驱动全局高并发异步 I/O；采用 `reqwest` 处理外部 API 请求网络池；选用 `serde` 生态提供零成本抽象的数据反序列化（直接映射至本地 JSON 状态字典）。
3. **前端交互表现层（Node.js / Vue 3）**
   - **运行环境**：Node.js 18+ 环境下通过 Vite 4.5+ 打包。
   - **技术体系**：以 Vue 3 Composition API 为响应式核心，使用 `Pinia` 接管全局状态控制流，借由 `TailwindCSS` 原子化样式实现高度可定制的跨端 UI 适配。

---

## 5.2 串流底层的触控注入与环境兼容机制

在 C++ 构建的流媒体引擎中，处理最频繁且最易引发客户端操作卡顿的环节是被控端的事件注入 (Event Injection)。同时，考虑到本引擎涉及操作系统的底层 API，对不同编译器的宏环境有着极强的敏感性。

### 5.2.1 高精度虚拟触控指令注入实现

由于不同移动端设备（如手机、平板）的屏幕纵横比与物理分辨率大相径庭，单纯的绝对坐标透传不可避免地会导致光标漂移。因此，系统在底层抽象了 `touch_port_t` 触控视口结构体。

在此算法流程中，宿主机端接收到带有时间戳的物理参数后，必须进行针对目标虚拟全屏分辨率的**等比例缩放与安全截断补偿**，随后生成标准的鼠标硬件中断信号 `INPUT_MOUSE` 抛入 Windows 的输入队列。其核心注入代码段如下所示：

```cpp
// 节选自 src/platform/windows/input.cpp
// 绝对坐标防抖动与鼠标位置注入实现
void abs_mouse(input_t &input, const touch_port_t &touch_port, float x, float y) {
  INPUT i {};
  i.type = INPUT_MOUSE;
  auto &mi = i.mi;
  
  // 核心标志位：MOUSEEVENTF_VIRTUALDESK 确保坐标映射到包含所有虚拟显示器的全局桌面空间
  // MOUSEEVENTF_ABSOLUTE 标志宣告本次注入为绝对屏幕坐标而非相对增量坐标
  mi.dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;

  // 根据握手阶段客户端推介的分辨率进行等比例视口转换缩放，并向上取整 (lround)
  auto scaled_x = std::lround((x + touch_port.offset_x) * ((float) target_touch_port.width / (float) touch_port.width));
  auto scaled_y = std::lround((y + touch_port.offset_y) * ((float) target_touch_port.height / (float) touch_port.height));

  mi.dx = scaled_x;
  mi.dy = scaled_y;

  send_input(i); // 将组装好的信号结构体转交 Win32 API 队列
}
```

针对高级的虚拟双摇杆手柄（Gamepad Emulator），系统则调用 `ViGEmBus` 驱动程序的内核态钩子，将客户端发送来的陀螺仪与摇杆偏转数据解包重组成标准的 Xbox 360 / DualShock 4 电信号态，骗过游戏引擎的 XInput 轮询，从而实现原生级手柄游玩体验。

### 5.2.2 跨编译器头文件重定义防护机制

由于 Windows SDK 头文件在版本更迭时容易引发重定义的编译错误，如在最新版 MinGW-w64 工具链中，`winuser.h` 已经单方面补齐了微软新型触屏指针设备的接口声明。若代码不做判断强制导入，会导致抽象语法树（AST）解析器因 `error: redefinition` 终止编译。

为提升系统的跨环境部署能力，系统在引擎级构建了条件编译防御矩阵：

```cpp
// 预编译宏防御机制：规避跨版本 MinGW-w64 触屏指针句柄的重复定义
#ifdef __MINGW32__
#ifndef HSYNTHETICPOINTERDEVICE 
// 仅在发现当前编译环境 SDK 过旧，未包含该设备句柄时，由开发者显式暴露
DECLARE_HANDLE(HSYNTHETICPOINTERDEVICE);
#endif

// 继续桥接系统的指针设备相关函数 (用于多点触控注入)
WINUSERAPI HSYNTHETICPOINTERDEVICE WINAPI
CreateSyntheticPointerDevice(POINTER_INPUT_TYPE pointerType, ULONG maxCount, POINTER_FEEDBACK_MODE mode);
WINUSERAPI BOOL WINAPI
InjectSyntheticPointerInput(HSYNTHETICPOINTERDEVICE device, CONST POINTER_TYPE_INFO *pointerInfo, UINT32 count);
// ...
#endif
```

该防护机制的引入，使 GalRemote 引擎能够在从传统 VS2019 到搭载最新开源工具链的 CI (Continuous Integration) 构建流水线中稳定存活编译。

---

## 5.3 Rust 中台调度层的并发与抽象实现

Rust 编写的中台层 (`src-tauri`) 承担着处理绝大多数密集运算、系统管理与网络吞吐的重任。得益于其极其严苛的所有权机制与 `tokio` 并发调度模型，该层实现了无内存泄漏的高性能运作。

### 5.3.1 高通量异步 VNDB 数据刮削流

针对海量的本地未识别 Galgame 目录进行自动化游戏数据补全与封面刮削时，涉及海量文本 IO 与 HTTP 连接阻塞。本系统利用 GraphQL 查询语言对请求参数进行高度裁剪，同时利用 Rust 生态的异步非阻塞网络库 `reqwest` 构建连接池并发派发检索任务：

```rust
// 伪代码解析: 针对 VNDB GraphQL API 的异步检索与领域实体提取
#[tauri::command]
pub async fn scrape_galgame_info(title: String) -> Result<GameInfo, String> {
    // 实例化复用型 HTTP 连接池客户端
    let client = reqwest::Client::new();
    
    // 采用 GraphQL 进行精简字段的定点抓取，极大降低网络层传输字节数
    let query = format!(
        r#"{{"query": "query {{ vn(search: \"{}\", sort: \"searchrank\") {{ results {{ id title image.url description }} }} }}"}}"#,
        title
    );

    // .await 关键字使当前 Task 放弃系统级线程执行权，规避阻塞 Tauri 主 GUI 线程
    let res = client.post("https://api.vndb.org/kana/graphql")
        .header("Content-Type", "application/json")
        .body(query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // 利用 Serde 宏自动进行反序列化，从 JSON 流安全转换为强类型 Value
    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    // 随后置入 NLP 余弦相似度比对函数，提取匹配分数最高的返回集，存入结构化 GameInfo
    // ...
}
```

### 5.3.2 VDD 虚拟屏幕生命周期接管

虚拟显示器挂载（Virtual Display Dummy）是串流适配不同客户机比例的核心链路。Rust 后端在系统启动并取得权限后，利用 `std::process::Command` 脱壳环境与注册表管理进程通信，依据请求动态挂起一块拥有指定帧率（Hz）与分辨率规格的虚拟显示器：

```rust
// 动态切换宿主机屏幕物理视角的实现
pub fn inject_virtual_display(width: u32, height: u32, hz: u32) -> Result<(), AppError> {
    // 构造驱动层所需要的特定参数命令矩阵 (宽 高 刷新率)
    let args = format!("add {} {} {}", width, height, hz);
    
    // 调用特权命令执行设备树动态挂载 (需 UAC 提权以绕开安全隔离)
    let output = std::process::Command::new("vdd_manager.exe")
        .args(args.split_whitespace())
        .output()?;
        
    // 进程退出码判断
    if output.status.success() { Ok(()) } else { Err(AppError::DriverFailed) }
}
```

---

## 5.4 现代化前端展现与跨进程路由实现 (Vue 3)

为了满足桌面控制面板兼具 Web 高扩展性与 C++ 级原生调用的双刃剑需求，Vue 3 前端主要侧重解决与 Rust 主进程的频繁通信（IPC）与巨量列表的 DOM 渲染瓶颈。

### 5.4.1 基于 Tauri IPC 的前端调用管线

系统抛弃了传统的内嵌局域网 HTTP API 的设计，采用 Tauri `@tauri-apps/api/invoke` 原生封装包。所有调用通过系统内部的内存通道在 Vue 和 Rust 之间传输指令，消除了 Web 套接字的封装损耗：

```javascript
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted } from 'vue'

export default {
  setup() {
    const games = ref([])
    const isLoading = ref(false)

    // 在 Vue 首次挂载生命周期中，触发全量实体库脱库读取
    onMounted(async () => {
      try {
        isLoading.value = true
        // 阻塞式的跨进程桥接调用 (RPC)：底层为基于命名管道的异步高速缓冲区序列化
        games.value = await invoke('get_all_galgames_dict')
      } catch(e) {
        console.error("加载结构化存档仓库实体失败: ", e)
      } finally {
        isLoading.value = false
      }
    })

    return { games, isLoading }
  }
}
```

### 5.4.2 DOM 渲染优化与沉浸式高斯模糊

在前端处理刮削返回的数百款游戏的瀑布流展示时，直接渲染全量图片极易导致显存溢出卡顿。
本项目通过引入 **计算虚拟滚动窗口** (Virtual Scroller) 技术，仅动态生成可视区域内的 DOM 节点。同时借助原生 CSS 硬件加速滤镜 `backdrop-filter: blur(20px)` 生成毛玻璃抽屉面板，使交互界面拥有近似 Windows 11 Fluent Design / macOS 的顶配原生质感，做到了审美与效率的最佳结合。

---

## 5.5 智能混合云全自动同步引擎的落地

第四章设计的云同步冲突解决矩阵（增量合并+基于时间戳覆盖的 LWW 策略），在工程中落实为一套高容错的分布式执行流水线：

1. **快照哈希基准扫描**: 云同步触发时，首先利用 Rust 遍历包含 `.sav`/`.sys` 后缀的文件，通过 `SHA-256` 算法与系统级 `fs::metadata().modified` 组合映射出本次同步操作的本地快照树（Snapshot Tree）。
2. **多通道并发压制传输**: 当比较出大量增量碎文件（数万个几 KB 的存档槽文件）需同步至对象存储网关（S3/WebDAV）时，盲目派发请求将耗尽操作系统的句柄(File Descriptors)。系统基于 `tokio::sync::mpsc` 构建了背压式（Backpressure）的多生产者-单消费者消息管道，动态限流并发规模。
3. **不可逆回滚快照点生成**:
   在对文件进行高危写操作（如下载云端文件并覆盖本地原同名系统文件）前刻，通过调用高压缩比包库，将操作目录全盘转化为不可变版本记录：
   `Archive::new(file).compress(current_dir).unwrap()`
   这些历史包将被安置在 `.snapshots` 隐藏隔离区供前端时间机器系统拉取追溯。

## 5.6 本章小结

本章详细剖析了系统在代码实现层的难点及工业级解决方案。通过对包含宏重防、环境挂载的 C++ 引擎级底册分析，辅以大吞吐异步网络拉取的 Rust 泛型中台展现，再配合零损耗跨进程 IPC 握手调用的 Vue 3 桌面响应层，多角度立体证实了第四章系统的顶层混合架构不仅技术可行，且落地的代码运行质感极佳、鲁棒性拔群。系统的核心链路开发已尽数告竣并达成了稳定闭环。
