# Panel System - 高层级规格

## 背景与目标

当前 detail panel 和 region UI（region manager + density chart）各自独立设计，UI 模式不统一。兔总要求将它们统一为 **可折叠 panel 系统**：panel 平时折叠在 log table 底部，按快捷键展开，panel 间用 Ctrl+方向键切换，共享基本操作但各有独立功能。

**业务价值：** 统一的 panel 模型让用户学一次就能操作所有 panel，后续新增 panel（如 stats、bookmarks）也能直接复用框架。

## 问题陈述

1. Detail panel 和 region UI 的交互模式不一致（detail 是固定区域，region manager 是浮窗）
2. 没有统一的 panel 切换机制
3. 无法同时查看多个 panel 的折叠状态

## 用户故事

- 作为日志分析用户，我希望按快捷键展开/折叠底部 panel，以便在不离开 log table 的情况下查看详情/region 信息
- 作为日志分析用户，我希望用统一的快捷键在不同 panel 间切换，以便快速对比不同维度的信息
- 作为日志分析用户，我希望 panel 折叠时不占屏幕空间，以便最大化 log table 的可视范围

## 需求拆解

### P0 - 必须有

- [ ] **Panel 框架** — 统一的 panel trait/接口，定义 render、keybinding、折叠/展开行为（依赖：无）
- [ ] **Detail Panel 迁移** — 将现有 detail panel 迁移为 panel 系统的一个实例（依赖：Panel 框架）
- [ ] **Region Panel** — 将 region manager + region density chart 合并为一个 region panel（依赖：Panel 框架）
- [ ] **Panel 切换** — `Ctrl+←`/`Ctrl+→` 在 panel 间切换（依赖：Panel 框架）
- [ ] **Focus 切换** — `Ctrl+↑`/`Ctrl+↓` 在 log table 和当前 panel 间切换焦点（依赖：Panel 框架）
- [ ] **Panel 最大化** — `z` toggle 最大化/还原，最大化时占满 log table + panel 区域（依赖：Panel 框架）

### P1 - 应该有

（暂无）

### P2 - 最好有

- [ ] **多 Panel 同时展开** — 水平分割底部区域显示多个 panel（依赖：Panel 框架）

## 功能需求

### Panel 系统架构

#### 布局

```
┌───────────────────────────────────────────────────────┐
│                                                       │
│                    Log Table                           │
│                                                       │
├─── [Detail] ── [Region] ─────────────────────────────┤  ← Panel Tab Bar（折叠时仅显示此行）
│                                                       │
│              Active Panel Content                     │
│                                                       │
└───────────────────────────────────────────────────────┘
│ ▁▂▃▅▇█▇▅▃▂▁ │ 1,234/5,678             ← Line 1      │
│ [VIEW] /: Search │ f: Filter            ← Line 2      │
└───────────────────────────────────────────────────────┘
```

- Panel 区域位于 log table 和 status bar 之间
- **折叠状态：** 仅显示 panel tab bar（一行高），tab 名称列表如 `[Detail] [Region]`，当前选中 tab 高亮
- **展开状态：** tab bar + panel 内容区域，每个 panel 有各自的默认高度（见各 panel 定义）
- 同一时间只有一个 panel 展开显示内容

#### Panel Tab Bar

```
折叠：  ▸ Detail │ Region
展开：  ▾ Detail │ Region
```

- `▸` 表示折叠，`▾` 表示展开
- 当前选中 panel 名称高亮（如反色或加粗）
- 非活跃 panel 名称正常显示
- Tab bar 始终显示（无论折叠/展开）

#### Panel 焦点模型

三层焦点：
1. **Log Table** — 默认焦点，标准 log table 操作
2. **Panel Tab Bar** — 切换/选择 panel
3. **Panel Content** — panel 内部操作

焦点切换：
| 快捷键 | 行为 |
|--------|------|
| `Ctrl+↓` | Log Table → Panel Content（展开当前 panel，焦点进入 panel） |
| `Ctrl+↑` | Panel Content → Log Table（焦点回到 log table，panel 保持展开） |
| `Ctrl+←` / `Ctrl+→` | 切换到上一个/下一个 panel（如果 panel 已展开，切换内容；如果折叠，仅切换 tab 高亮） |
| `Esc` | Panel Content → Log Table（焦点回到 log table） |
| 原快捷键 | 直接打开对应 panel 并获得焦点（如 `Enter` 打开 Detail，`r` 打开 Region） |

#### 统一 Panel 操作（所有 panel 共享）

| 快捷键 | 行为 |
|--------|------|
| `j`/`k` | Panel 内上下导航 |
| `Esc` | 焦点回到 log table |
| `Ctrl+↑` | 焦点回到 log table |
| `Ctrl+↓` | （已在 panel 中时无操作） |
| `Ctrl+←`/`Ctrl+→` | 切换 panel |

**最大化：**

| 快捷键 | 行为 |
|--------|------|
| `z` | Toggle 最大化/还原。最大化时 panel 占满 log table + panel 区域，仅保留 tab bar + status bar。再按 `z` 恢复默认高度。 |

#### Panel 注册

```rust
trait Panel: UiComponent {
    fn name(&self) -> &str;           // Tab 显示名称
    fn shortcut(&self) -> char;        // 快速打开快捷键
    fn default_height(&self) -> PanelHeight;  // 默认高度策略
    fn is_available(&self) -> bool;    // 是否有内容可显示
    fn on_log_cursor_changed(&mut self, index: usize);  // log table 光标变化时通知
}

enum PanelHeight {
    FitContent,              // 按内容自适应（如 Detail panel）
    Percentage(u16),         // 终端高度百分比（如 Region panel 40%）
}
```

Panel 通过 trait 注册，新增 panel 只需实现此 trait。

### Detail Panel（迁移）

**Tab 名称：** `Detail`
**打开快捷键：** `Enter`（从 log table）
**默认高度：** `FitContent` — 按内容自适应（与现有 detail panel 高度行为一致）
**内容：** 与现有 detail panel 完全一致 — 左侧 message/expansion tree，右侧 fields

Panel 内独立操作：
| 快捷键 | 行为 |
|--------|------|
| `Tab` | 左右区域切换焦点 |
| `h`/`l` | 树节点展开/折叠（左侧 expansion tree） |
| `H`/`L` | 全部折叠/全部展开 |
| `f` | 从当前字段创建 filter |

**跟随光标：** log table 光标变化时自动更新内容。

### Region Panel（新）

**Tab 名称：** `Region`
**打开快捷键：** `r`（从 log table）
**默认高度：** `Percentage(40)` — 终端高度的 40%
**内容：** 左右分栏布局

#### 布局

```
Region                                                                          
┌─ Region List (~70%) ──────────────────────────────────┬─ Timeline (~30%) ─────┐
│                                                       │  port_startup         │
│  Port Startup Ethernet0   10:30:45→10:30:47  2.1s     │  ──██──████──░░──     │
│  Port Startup Ethernet4   10:30:45→10:30:48  3.0s     │                       │
│▸ Port Startup Ethernet20  10:30:45→?         >30s ⏱   │  http_request         │
│  SAI Create ROUTE_ENTRY   10:30:46→10:30:46  12ms     │  █─█──██──████─       │
│  HTTP GET /api/status     10:31:02→10:31:02  45ms     │                       │
│  HTTP POST /api/login     10:31:05→10:31:06  1.2s     │  sai_create           │
│                                                       │  ─██─█──██──          │
│  Total: 6 regions (3 types) │ 5 completed │ 1 timeout │                       │
└───────────────────────────────────────────────────────┴───────────────────────┘
```

**左侧 — Region List（~70% 宽度）：**
- 每行一个 region：名称、开始时间→结束时间、持续时长、描述
- 按开始时间排序，相同开始时间按结束时间排序（从前到后）
- Timeout region 标记 `⏱`，时长显示 `>timeout`
- `j`/`k` 导航，选中行高亮
- 焦点默认在左侧

**右侧 — Timeline（~30% 宽度，最小 40 字符）：**
- 每行一个 region type，显示 type 名称 + 该 type 所有 region 的迷你 timeline bar
- `████` 表示 region 持续时间，`░░` 表示 timeout region
- 时间轴自动缩放，覆盖所有 region 的时间范围
- 当左侧光标移动时，右侧对应 type 行高亮，且该 region 在 timeline 中用不同颜色标记
- 当终端宽度不足（右侧 < 40 字符）时，右侧隐藏，左侧占满

**左右联动：**
- 左侧选中一个 region → 右侧对应 type 行高亮，选中 region 的 bar 用亮色标记
- 右侧仅作为概览，不可独立操作（焦点始终在左侧）

#### Panel 内操作

| 快捷键 | 行为 |
|--------|------|
| `j`/`k` | 上下导航 region list |
| `Enter` | 跳转到选中 region 的 start record（焦点回到 log table） |
| `f` | Filter log table 到选中 region 的记录范围 |
| `t` | 按 region type 过滤列表（toggle，再按显示全部） |
| `s` | 切换排序（start time / duration） |

**跟随光标：** 当 log table 光标移动到某个 region 内的记录时，region list 中对应 region 自动高亮。

### Region Markers（保留）

log table 左侧的 gutter markers（▶/│/◀）保留不变，这是 log table 的一部分，不属于 panel 系统。

## 非功能需求

- **性能：** Panel 折叠时不执行 render 计算；展开时仅渲染可见区域
- **终端兼容：** 最小终端宽度 60 列；高度不足时 panel 自动折叠
- **响应速度：** Panel 展开/折叠/切换必须是即时的（无动画，无延迟）

## 验收标准

- [ ] `Enter` 打开 Detail panel，再按 `Ctrl+→` 切换到 Region panel
- [ ] `r` 直接打开 Region panel
- [ ] `Ctrl+↑`/`Ctrl+↓` 在 log table 和 panel 之间切换焦点
- [ ] 折叠时 panel tab bar 仅占一行
- [ ] Panel 展开时 log table 高度自动缩小
- [ ] Region panel 左右分栏：左侧 region list，右侧 timeline（~30% 宽度，最小 40 字符）
- [ ] Region list 按开始时间→结束时间排序
- [ ] 左侧选中 region 时右侧对应 type 高亮，选中 bar 用亮色标记
- [ ] 终端宽度不足时右侧 timeline 自动隐藏
- [ ] Detail panel 跟随 log table 光标更新
- [ ] Region panel list 视图跟随光标高亮对应 region
- [ ] `Esc` 从 panel 回到 log table
- [ ] `z` 最大化 panel 后 log table 隐藏，再按 `z` 恢复

## 范围之外

- 用户自定义 panel（plugin panel）— 后续考虑
- Panel 拖拽排序 — 固定顺序足够
- 左右侧 panel — 仅底部
- Panel 浮动/脱离模式 — 仅嵌入式

## 开放问题

（已全部澄清）
