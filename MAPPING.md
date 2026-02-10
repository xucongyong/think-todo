# 🚀 Think-Todo 全功能复刻与可视化进度表

| Web | Gastown (Go) | Think-Todo (Rust) | 功能说明 | 使用方法、案例 |
| :--- | :--- | :--- | :--- | :--- |
| ✅ | `mayor` | `admin` | 中央主控台，负责全局协调和 Web 服务 | `tt serve` (访问 localhost:3030) |
| ✅ | `sling` | `sling` | **核心派发**：指派特定代理并选择 AI 引擎 | 点击任务卡片的 `SLING` 按钮 |
| ✅ | `beads` | `beads` | **全景看板**：汇总任务、状态、成本和日志 | Web 首页 Dashboard |
| ✅ | `bead` | `task` | **任务管理**：基础的增、删、改、查 | `+ NEW TASK` 按钮与 `DELETE` 按钮 |
| ✅ | `peek` | `peek` | **实时透视**：在网页上直接看 Agent 思考过程 | 点击活跃 Agent 卡片查看日志流 |
| ✅ | `nudge` | `nudge` | **同步提醒**：强制向 Agent 发送指令或弹窗 | 驾驶舱内的 `NUDGE` 按钮 |
| ✅ | `done` | `done` | **任务归档**：标记完成并物理清理隔离环境 | 列表或驾驶舱内的 `DONE` 按钮 |
| ✅ | `activity` | `audit` | **审计足迹**：系统最近发生的关键事件流 | 右侧 `System Trail` 实时面板 |
| ✅ | `costs` | `costs` | **经济统计**：统计 AI 会话消耗的 Token 成本 | 顶部 `SYSTEM ECONOMY` 实时显示 |
| ❌ | `mail` | `mail` | 代理间异步消息系统 (Web 端目前仅作展示) | 计划增加独立的 `Inbox` 页面 |
| ❌ | `rig` | `rig` | 注册和管理物理工作区/代码仓库节点 | 目前需命令行 `tt rig add` |
| 🏗️ | `witness` | `monitor` | 节点级别监控，自动扫描日志完成信号 | 后端已部分实现，WebUI 待集成 |
| 🏗️ | `handoff` | `handoff` | 交接棒：将当前会话挂起并交给另一位代理 | 计划增加 `Transfer` 按钮 |
| ❌ | `seance` | - | 会话招魂：回溯之前的会话上下文和决策路径 | 计划中 |
| ❌ | `resume` | - | 恢复被 park 或 handoff 挂起的任务 | 计划中 |
| ❌ | `mq` | - | 合并队列管理，处理多代理并发提交冲突 | 计划中 |

---

### 系统当前状态概要
- **指挥方式**: 命令行 (CLI) + 响应式网页 (WebUI)
- **核心引擎**: 支持 **Gemini**, **OpenCode**, **Claude** 动态切换
- **复刻进度**: 核心指令已完成 13/19，WebUI 已覆盖 80% 的日常操作。
- **技术栈**: Rust (Backend) + Axum (API) + Alpine.js (UI) + SQLite (Data)