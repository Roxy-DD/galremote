# 第4章 概要设计

基于第3章中提出的各项功能需求（跨端串流接管、Galgame 原生库管理、云存档并集同步等）与非功能需求（超低串流延迟、轻量化资源占用），本章将详细阐述 GalRemote 系统的整体架构与各个核心功能模块的内部设计。为满足上述严苛要求，本系统采用了**“C++ 底层引擎驱动 + Rust 中台层管理 + Vue3 现代化前端呈现”**的三层分离架构，从而有效解决跨端串流与状态同步的技术难题。

## 4.1 系统架构设计

为保证串流的超低延迟并跨越不同操作系统的底层 API 壁垒，同时考虑到 GUI 面板的高度可扩展性，本系统摒弃了传统的单一单体架构，转而采用了基于 **Tauri** 跨平台框架配合本地代理协议的混合型物理与逻辑架构。

### 4.1.1 物理与逻辑架构模型

系统整体划分为 **三大核心层（Three-Tier Architecture）**，如下图所示：

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontFamily': 'arial', 'primaryColor': '#F8FAFC', 'primaryBorderColor': '#CBD5E1', 'lineColor': '#64748B'}}}%%
graph TB
    classDef layer fill:#ffffff,stroke:#CBD5E1,stroke-width:2px,stroke-dasharray: 4 4;
    classDef vue fill:#ECFDF5,stroke:#10B981,stroke-width:1.5px,color:#064E3B;
    classDef rust fill:#FEF2F2,stroke:#EF4444,stroke-width:1.5px,color:#7F1D1D;
    classDef cpp fill:#EFF6FF,stroke:#3B82F6,stroke-width:1.5px,color:#1E3A8A;
    
    subgraph PresentationLayer [Presentation Layer: Vue 3 Desktop UI]
        UI_Dash("Dash Monitor"):::vue
        UI_Gal("Galgame Organizer"):::vue
        UI_VDD("Display Settings"):::vue
        UI_Sync("Sync Dashboard"):::vue
    end

    subgraph MiddlewareLayer [Middleware Layer: Tauri Rust Backend]
        Axum("Axum CORS Proxy"):::rust
        SysMgr("Daemon Watchdog"):::rust
        VNDB_Scraper("VNDB API Client"):::rust
        Cloud_Sync("OpenDAL Sync Engine"):::rust
        VDD_Mgr("VDD Registry Driver"):::rust
        File_IO("Async FS Pipeline"):::rust
    end

    subgraph CoreEngineLayer [Core Engine Layer: C++ Streaming Engine]
        Video_Encode("DXGI/NVENC Hardware Encoder"):::cpp
        Input_Inject("ViGEm Input Injector"):::cpp
        RTSP_Server("RTSP/Moonlight Protocol Stack"):::cpp
        Audio_Catch("WASAPI Audio Catcher"):::cpp
    end

    UI_Dash -->|HTTP REST| Axum
    UI_Gal -->|Tauri IPC| VNDB_Scraper
    UI_Gal -->|Tauri IPC| File_IO
    UI_VDD -->|Tauri IPC| VDD_Mgr
    UI_Sync -->|Tauri IPC| Cloud_Sync

    Axum -.->|Config Dispatch| RTSP_Server
    SysMgr ==>|Heartbeat Monitor| CoreEngineLayer
    VDD_Mgr -.->|Physical Morph| Video_Encode

    class PresentationLayer,MiddlewareLayer,CoreEngineLayer layer;
```

1. **C++ 串流底层引擎（Streaming Core Engine Layer）**
   - **定位**：系统的大脑神经与骨骼，承担低延迟串流性能需求。
   - **职责**：直接同操作系统 API 交互，负责桌面画面捕获（如 Windows DXGI）、音频捕获（WASAPI）、硬件视频编码，以及高频的外设输入信号注入。

2. **Rust 面板中台层（Tauri Backend / Middleware Layer）**
   - **定位**：连接底层服务与顶层 UI 的中枢，承接“跨平台管理”与“结构化扫描”的核心业务。
   - **职责**：提供虚拟硬件挂载（VDD Model）、Galgame 硬盘目录的并发扫描、VNDB (Visual Novel Database) 原生 API 刮削，以及实现跨协议云存档同步逻辑。

3. **Vue 3 现代化前端层（Presentation Layer）**
   - **职责**：通过自研桌面 UI 框架，在不妥协性能的前提下提供贴近原生应用的交互体验，包括窗口管理、瀑布流展示与存档时间机器等界面。

---

## 4.2 功能模块设计

针对系统整体的宏观设计，我们需要将其拆解为独立协同运转的各个业务模块。

### 4.2.1 串流控制模块

为满足“移动端全功能指控”的需求，串流控制模块的设计重点在于“无感介入”与“资源动态匹配”。
- **引擎进程管控与心跳设计**：由于底层可能因显卡驱动等外部因素崩溃，面板应用（Tauri）采用伴随模式管理引擎。当检测到 `sunshine.exe` 退出码异常时，Rust 后端可在 500ms 内收集日志并重新拉起服务。
- **虚拟操作接管**：客户端握手时报告布局，系统通过解包坐标流，动态调用 `InjectSyntheticPointerInput` 或 `ViGEmBus` 对系统执行指令注入。

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'actorBkg': '#F1F5F9', 'actorBorder': '#64748B', 'actorLineColor': '#94A3B8', 'signalColor': '#334155', 'signalTextColor': '#1E293B', 'fontFamily': 'arial'}}}%%
sequenceDiagram
    participant Client as Client (Mobile/Web)
    participant Cpp as C++ Stream Engine
    participant WinAPI as Win32 API (Ring 0)
    
    Client->>Cpp: Handshake (Report Layout)
    loop 100+ FPS Stream
        Client->>Cpp: Send Touch/Gyro Data
        Cpp->>Cpp: Unpack (x, y, Button)
        alt Pure Touch Mode
            Cpp->>WinAPI: InjectSyntheticPointerInput (Absolute X/Y)
        else Virtual Gamepad Mode
            Cpp->>WinAPI: ViGEmBus Injection (Xbox360/DS4 State)
        end
    end
```

### 4.2.2 游戏库管理与刮削模块

针对“海量未结构化游戏目录管理困难”，该模块使用并发算法将其转换为结构化资产。使用 Rust 异步运行时结合 VNDB 的 GraphQL API进行处理，其时序如下：

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'actorBkg': '#F1F5F9', 'actorBorder': '#64748B', 'actorLineColor': '#94A3B8', 'signalColor': '#334155', 'signalTextColor': '#1E293B', 'fontFamily': 'arial'}}}%%
sequenceDiagram
    participant User
    participant Rust as Tauri Rust (Scraper)
    participant FS as Local File System
    participant VNDB as VNDB (GraphQL API)
    
    User->>Rust: Start Library Scan
    Rust->>FS: Async Tree Traversal (.exe/.xp3)
    FS-->>Rust: Return Raw List
    
    loop For each unknown title
        Rust->>Rust: Regex Normalize Path Name
        Rust->>VNDB: GraphQL Fuzzy Rank Search
        VNDB-->>Rust: Return Candidate List
        Rust->>Rust: Cosine Similarity Match (NLP)
        Rust->>VNDB: Async Download Cover & Screen
    end
    
    Rust->>FS: Persist JSON DB & Flush Images
    Rust-->>User: Trigger UI Refresh
```

### 4.2.3 智能云存档同步模块

为实现“任何地点、任何设备无缝接力”，由于 Galgame 包含全局系统记录文件（SystemData）与槽位快照文件（SlotData），简单的全量覆盖必然导致丢档。系统采用 **基于元数据快照的并集冲突解决算法**：

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontFamily': 'arial', 'primaryColor': '#F1F5F9', 'primaryBorderColor': '#94A3B8', 'labelBoxBkgColor': '#E2E8F0'}}}%%
stateDiagram-v2
    [*] --> Stream_Disconnected
    Stream_Disconnected --> Diff_Scanner
    Diff_Scanner --> Union_Decision_Matrix
    
    state Union_Decision_Matrix {
        state "A. Local New / Cloud Null" as SceneA
        state "B. Cloud New / Local Null" as SceneB
        state "C. Hard Conflict (Both Mutated)" as SceneC
        
        SceneA --> Exec_Push
        SceneB --> Exec_Pull
        SceneC --> Bi_Diff_Strategy
    }
    
    Bi_Diff_Strategy --> Slot_Data: Incremental Sync (Keep Both)
    Bi_Diff_Strategy --> Sys_Data: LWW Strategy (Timestamp Override)
    
    Exec_Push --> Zlib_Archive_Snapshot
    Exec_Pull --> Flush_Handle_Yield
```

---

## 4.3 数据库设计

出于系统对主机的“极低侵入性”（零依赖、免安装）与极低资源开销的非功能性指标考量，本系统在架构树级摒弃了传统的 SQLite 或 MySQL 等重量级关系型数据库解决方案，转而采用了基于 Rust `serde_json` 的强类型 JSON 序列化技术。

为保证在前端状态树与后端 Rust 结构体、物理文件之间的数据一致性，系统规划了以下非关系型（NoSQL）文档的**核心实体关系 (Entity-Relationship) 模型**：

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontFamily': 'arial', 'primaryColor': '#F1F5F9', 'primaryBorderColor': '#94A3B8'}}}%%
erDiagram
    GAME {
        string ID PK "UUID / VNDB_ID"
        string Title "Display Name"
        string ExecutablePath "Launch Routing Path"
        string CoverUrl "Local Cover Cache Path"
        boolean SyncEnabled "Is Cloud Sync Active"
    }
    PLAY_STATS {
        string GameID FK "Linked Game Entity"
        int TotalPlayTime "Cumulative Seconds"
        int LastPlayedTime "Unix Epoch Timestamp"
    }
    CLOUD_BACKEND {
        string ID PK "Provider UUID"
        string Type "WebDAV / S3 / Local"
        string Endpoint "Mount URL"
        string AuthToken "Encrypted Token"
    }
    SNAPSHOT_LOG {
        string HashID PK "SHA-256 Checksum"
        string GameID FK "Linked Game"
        string ArchivePath "Zlib Compressed Bin Path"
        string SyncStrategy "LWW or Keep-Both Tag"
    }

    GAME ||--o{ PLAY_STATS : "High-Freq Mount"
    GAME ||--o{ SNAPSHOT_LOG : "History Safenet"
    CLOUD_BACKEND ||--o{ GAME : "Cross-Device Sync"
```

### 4.3.1 领域驱动设计下的冷热分离策略

在工程落地中，为了避免高频的“心跳追踪机制”引发磁盘 IO 风暴乃至写坏包含复杂属性的静态游戏元数据，本系统将大型结构体进行了**严格的冷热隔离**：

1. **静态配置流 (冷数据)**：`GAME` 的核心展现描述剥离为只读频率极高的配置文件（`galgames.config.json`），仅在刮削器更新或用户修改路径时发生落盘写操作。
2. **动态日志流 (热数据)**：`PLAY_STATS` 游玩统计独立映射为 `play_stats.json`，在系统推流挂钩时由后台线程以定期步长或进程退出时进行防抖写出。

两份数据字典在内存堆中进行“伪联表”（Pointer Reference），既保证了读取的高吞吐，又杜绝了存储层面的“脏写”与原子性被破坏的风险。

---

## 4.4 本章小结

本章围绕需求分析阶段的核心要点，详细描绘了 GalRemote 系统架构的顶层设计原貌。通过“表示层、中间件层、引擎层”这三层复合架构将繁琐的功能进行了有效解耦。接着在功能模块设计中，针对串流控制、自动刮削以及云存档同步确立了合理的时序逻辑与状态机表现。最后，借由无数据库的强类型 JSON 映射设计，进一步保障了系统免安装、防脏写的轻量特性。至此，整个系统的蓝图规划完毕，为下一章核心代码的实际工程编写打下了坚实基础。
