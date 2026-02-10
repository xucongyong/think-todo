# Think-Todo (tt)

Think-Todo 是一个基于 Rust 开发的 AI 代理编排系统，旨在复刻并优化 `gastown` 的核心逻辑。它采用“主控-工人”架构，通过 Tmux 隔离会话，利用 SQLite 持久化状态，为大规模代理协作提供基础设施。

## 🏗️ 项目结构

```text
think-todo/
├── src/                # Rust 核心源代码
│   ├── main.rs         # 命令行入口与路由
│   ├── admin.rs        # 中央控制逻辑 (Mayor)
│   ├── worker.rs       # 工人生命周期管理 (Polecat)
│   ├── db.rs           # SQLite 数据库持久层
│   ├── monitor.rs      # 日志监控与状态扫描 (Witness)
│   └── tmux.rs         # Tmux 会话包装器
├── docs/               # 设计文档与概念说明
├── prompts/            # 代理角色提示词定义
├── workers/            # 动态创建的代理工作空间 (Runtime)
├── .logs/              # 任务与系统审计日志
├── think.db            # 系统状态数据库 (SQLite)
└── Cargo.toml          # Rust 依赖配置
```

## 🚀 极简上手教程

### 1. 初始化与管理
首先启动中央控制台（Admin）：
```bash
tt admin start
```
你可以通过以下命令随时进入或退出管理界面：
```bash
tt admin attach  # 进入界面
# 在 Tmux 中按 Ctrl+B, D 退出
```

### 2. 任务分发流程
**第一步：添加一个任务**
```bash
tt task add T1 "实现登录页面的单元测试"
```

**第二步：指派代理执行 (Sling)**
将任务 T1 指派给名为 `tester-01` 的代理：
```bash
tt sling T1 tester-01
```
这会自动创建一个 `workers/tester-01` 目录，并开启一个专用的 Tmux 会话。

**第三步：实时观察 (Peek)**
在不打扰代理的情况下，偷看它的最新日志输出：
```bash
tt peek tester-01
```

### 3. 完成与审计
**标记任务完成：**
```bash
tt done T1
```
这会标记任务为 closed，并自动清理（Nuke）相关的 worker 目录。

**查看系统足迹：**
```bash
tt trail
```

## 📜 核心原则 (GUPP)
1. **任务即授权**: 只要指派了任务，代理就应立即执行，无需二次确认。
2. **环境隔离**: 每个 Worker 必须在独立的物理目录下操作。
3. **审计必达**: 所有关键动作必须记录在 `trail` 中。

---
更多详细文档请参考 `docs/` 目录。
