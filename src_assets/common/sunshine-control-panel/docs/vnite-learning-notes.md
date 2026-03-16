# vnite 学习与增量迁移记录（面向 GalRemote 面板）

> 参考仓库：`https://github.com/ximu3/vnite.git`
>
> 本地学习副本：`./.cache-vnite-ref`
>
> 目标：在**保持当前 Tauri + Vue + Rust 架构**的前提下，增量吸收 vnite 的页面与功能设计；同时保留并强化现有的「游戏存档快照 + 云同步」能力。

---

## 1. 已完成的学习范围

### 1.1 产品能力总览（来自 README）

vnite 强调的核心能力：

- 游戏库管理（现代化 UI）
- 多数据源元数据（Steam/IGDB/Bangumi/VNDB/YMGal/DLsite/Erogamescape）
- 云同步（记录/存档/设置）
- 自动扫描入库与规则化转换
- 丰富筛选、排序、分组与报告

对应参考：

- `.cache-vnite-ref/README.zh-CN.md`

### 1.2 关键页面与交互设计（已拆解）

#### A. Library 页面布局（左右分栏 + 可拖拽宽度）

参考：

- `.cache-vnite-ref/src/renderer/src/pages/Library/main.tsx`

可迁移设计点：

- 左栏宽度可拖拽（library bar）
- 左右分区职责明确（过滤导航 vs 详情内容）
- 双击拖拽条复位宽度

#### B. 排序菜单（排序字段、顺序、分组）

参考：

- `.cache-vnite-ref/src/renderer/src/components/Librarybar/SortMenu.tsx`

可迁移设计点：

- 排序字段可选（名称、发布日期、最近运行、添加时间、游玩时长等）
- 升降序切换
- 与分组策略联动（如按状态分组时可调整组顺序）

#### C. 紧凑头部 / 封面与信息块

参考：

- `.cache-vnite-ref/src/renderer/src/components/Game/HeaderCompact.tsx`

可迁移设计点：

- 长方形封面（而非方形头像式）
- 头部信息与操作按钮分区
- 紧凑信息密度高、按钮区域独立

### 1.3 元数据抓取架构（Provider 聚合）

#### Provider 注册中心

参考：

- `.cache-vnite-ref/src/main/features/scraper/providers/index.ts`

可迁移设计点：

- 抓取源统一注册、统一入口
- 易扩展（新增 provider 不影响主流程）

#### Bangumi Provider（字段映射与结构化）

参考：

- `.cache-vnite-ref/src/main/features/scraper/providers/bangumi/common.ts`

可迁移设计点：

- 角色字段映射（企划/脚本/原画/音乐等）
- `infobox` 字段容错解析
- 关联站点补充（官方 / Bangumi 页面链接）

#### Steam Provider（多地区兜底 + appdetails）

参考：

- `.cache-vnite-ref/src/main/features/scraper/providers/steam/common.ts`

可迁移设计点：

- `storesearch` 多地区候选（cc）兜底
- `appdetails` 拉取详细信息
- 标签、平台、开发发行信息的补全

---

## 2. 与当前 GalRemote 项目的对照

### 2.1 当前已有能力（本项目）

核心文件：

- 前端：`src/renderer/components/GalgameManager.vue`
- 后端命令：`src-tauri/src/galgame/commands.rs`
- 抓取模块：`src-tauri/src/galgame/scraper.rs`
- 存档/快照：`src-tauri/src/galgame/archive.rs`
- 云同步：`src-tauri/src/galgame/cloud.rs`

已具备：

- 游戏列表 / 编辑 / 删除
- 存档扫描
- 快照创建恢复
- 云同步（WebDAV/S3/OSS/GitHub）
- VNDB 搜刮（原始版本）

### 2.2 本轮已完成的增量实现

#### UI/交互

- 左侧按钮对齐与响应式修复
- 排序控件接入（最近游玩/时长/名称/添加时间）
- 卡片/紧凑视图切换可交互
- 右侧详情封面改为长方形比例

#### 元数据

- 后端从单一 VNDB 扩展为多源聚合：
  - VNDB
  - Bangumi（`/v0/search/subjects`）
  - Steam Store Search（`/api/storesearch`）
- 前端结果中展示来源字段 `source`

主要变更文件：

- `src/renderer/components/GalgameManager.vue`
- `src-tauri/src/galgame/scraper.rs`
- `src-tauri/src/galgame/commands.rs`

---

## 3. 迁移原则（必须遵守）

1. **不重写架构**：保持 Tauri + Vue + Rust，不引入 Electron 主进程模型。
2. **能力优先迁移**：迁移交互模式与领域模型，不机械迁移组件技术栈（React/TSX）。
3. **存档能力优先级最高**：任何迁移不能破坏现有快照与云同步链路。
4. **增量迭代**：每次只迁一层（UI / 数据源 / 设置面板），每步可回退。

---

## 4. 推荐迁移路线图（按优先级）

### Phase 1（已开始）— 体验层对齐

- [x] 排序可配置
- [x] 卡片/紧凑视图切换
- [x] 长方形封面与信息布局
- [ ] 左栏可拖拽宽度（可参考 vnite Library）

### Phase 2 — 抓取器 Provider 化

- [ ] 抽象 `MetadataProvider` trait（Rust）
- [ ] 将 VNDB/Bangumi/Steam 变为独立 provider 模块
- [ ] 支持在前端设置抓取优先级（类似 vnite provider 列表）

### Phase 3 — 元数据深度字段

- [ ] 开发商/发行商/标签/平台/关联站点统一模型
- [ ] Bangumi infobox 角色字段映射
- [ ] Steam appdetails 补全详情

### Phase 4 — 游戏库信息架构升级

- [ ] 分组（按状态/开发商/标签）
- [ ] 筛选器增强（多条件）
- [ ] 列表配置持久化（排序、视图、分组）

---

## 5. 风险与边界

1. **许可证差异**：vnite 为 GPL-3.0，直接复制代码要谨慎；建议“设计借鉴 + 重新实现”。
2. **运行时差异**：vnite 使用 Electron `net.fetch`、React 生态；本项目为 Tauri/Rust，接口层要重建。
3. **云同步模型不同**：vnite 偏数据库同步，本项目偏存档文件同步，迁移时要避免目标错位。

---

## 6. 给当前项目的落地建议（短期）

1. 在 `src-tauri/src/galgame/` 新建 `providers/` 目录，开始拆分抓取器。
2. 在 `GalgameManager.vue` 增加“抓取来源偏好”配置 UI（VNDB / Bangumi / Steam）。
3. 为元数据搜索添加统一结果标准化字段：
   - `name`, `original_name`, `description`, `developer`, `release_date`, `tags`, `source`, `cover_url`, `related_links`
4. 在不动快照/云同步主流程前提下迭代 UI（保持数据层稳定）。

---

## 7. 当前结论

可以做，而且建议做**增量重现**：

- UI 交互借鉴 vnite（排序/分组/紧凑布局/信息密度）
- 抓取层采用 provider 设计
- 保留并强化本项目独有优势（快照恢复、云同步冲突处理、Galgame 场景适配）

这条路线比“整仓硬迁移”风险更低、收益更快。
