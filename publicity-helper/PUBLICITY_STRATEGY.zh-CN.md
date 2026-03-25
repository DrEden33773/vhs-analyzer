# vhs-analyzer 宣发策略

## 项目现状

- vhs-analyzer 是 VHS `.tape` 文件的**首个 LSP 语言服务器**，功能完整：hover、completion、diagnostics、formatting、safety check。
- VS Code / Cursor 扩展已发布到 Marketplace 和 Open VSX（v0.1.1），附带 CodeLens 和实时预览。
- 当前 GitHub 5 stars，尚未进行任何主动宣发。
- 单人开发，80 commits，287 条自动化测试，三阶段全部完成。

## 竞品格局

VHS 上游（charmbracelet/vhs）有 ~19K stars，用户基数大，但编辑器工具极度匮乏：

| 现有工具 | 能力 | 差距 |
| --- | --- | --- |
| `griimick/vscode-vhs` | 仅语法高亮 | 无 LSP、无诊断、无补全、无预览 |
| `charmbracelet/tree-sitter-vhs` | 仅语法高亮（Neovim/Emacs） | 同上 |
| Zed VHS 扩展 | 仅语法高亮 | 同上 |

**不存在任何竞争性的 VHS LSP 实现。** VHS 核心贡献者 `caarlos0` 在 2022 年的 [issue #162](https://github.com/charmbracelet/vhs/issues/162) 中提到 "or a lsp server"，至今没有官方实现。

## 核心卖点

按受众优先级排列：

| 优先级 | 卖点 | 目标受众 |
| --- | --- | --- |
| P0 | 首个 VHS LSP — 从纯文本编辑升级到 IDE 级体验 | 所有 VHS 用户 |
| P0 | 一键预览 — CodeLens + 并排 Preview，告别终端反复 `vhs < file.tape` | VHS 日常用户 |
| P1 | 安全检查 — 对 `Type` 指令中的危险命令给出警告 | DevOps / CI 场景 |
| P1 | AI 三角色工程范式 — Scout/Architect/Builder 分工，spec-first 全流程 | Rust / AI 工程社区 |
| P2 | Rust + rowan 无损语法树实现 | Rust 开发者 |
| P2 | 跨平台 VSIX 打包 + 无服务器降级 | 扩展开发者 |

## 宣发渠道

### 第一波：Charmbracelet 社区（最高 ROI）

核心目标受众聚集地，最先触达。

1. **Charmbracelet Discord / Slack** — 在 showcase 频道发布简短介绍 + Demo GIF，附 Marketplace 链接。
2. **VHS GitHub Discussions** — 开一个 Discussion 帖，引用 issue #162 的历史对话，强调 "complementary to vhs"。
3. **联系 VHS 维护者** — 给 `caarlos0` 或 `maaslalani` 发消息，争取在 VHS 官方 README 的社区工具区获得引用。

### 第二波：Hacker News + Reddit（最大曝光）

1. **Hacker News — Show HN** — 标题建议："Show HN: VHS Analyzer – A Rust LSP and VS Code extension for terminal recording scripts"。选周二或周三美东上午发布。准备好回答技术选型问题（rowan、tower-lsp-server）。
2. **Reddit** — 按子版侧重不同角度：
   - `r/rust` — Rust 技术实现
   - `r/vscode` — 扩展使用体验
   - `r/commandline` — 终端录制工作流改进
   - `r/programming` — LSP 生态填补空白

### 第三波：中文技术社区

1. **V2EX** — "分享创造"节点
2. **掘金** — 技术文章（工程方法论角度）
3. **知乎** — 回答终端录制或 Rust 工具链相关问题时引用

### 第四波：Twitter/X + 长尾

1. **Twitter/X** — 附 GIF 推文，@charmbracelet，标签 `#rust #vscode #devtools #terminal`。
2. **技术博文** — 发布在 dev.to / Medium / 个人博客。两个独立主题：
    - 产品线："为什么要给 VHS 做一个 LSP"
    - 方法论线："用 AI 三角色开发完整 LSP + 扩展的经验"

## 宣发前必须准备的内容

### P0：发布前必须完成

| 序号 | 内容 | 当前状态 | 说明 |
| --- | --- | --- | --- |
| 1 | Demo GIF / 演示动画 | **缺失** | 最关键缺口。录制展示 hover / completion / diagnostics / formatting / CodeLens / Preview 全流程的 GIF。用 VHS 自身录制会形成 meta 闭环，本身就是宣传素材。 |
| 2 | README 嵌入演示 GIF | **缺失** | 根 README 和扩展 README 均为纯文本。没有视觉展示的项目几乎不可能获得关注。 |
| 3 | 功能截图集 | **缺失** | 至少 5-6 张：hover、completion、diagnostics、formatting 前后对比、CodeLens、Preview 面板。 |
| 4 | GitHub topics | 未设置 | 添加：`vhs`、`tape`、`lsp`、`language-server`、`vscode-extension`、`rust`、`terminal`、`gif`、`devtools`。 |

### P1：强烈建议准备

| 序号 | 内容 | 当前状态 | 说明 |
| --- | --- | --- | --- |
| 5 | `examples/` 目录 | **不存在** | 创建 3-5 个典型 `.tape` 示例文件，覆盖简单、复杂、含安全警告等场景。 |
| 6 | Issue / PR 模板 | **不存在** | Bug Report / Feature Request / PR 模板，降低社区参与门槛。 |
| 7 | GitHub Release Notes 优化 | 内容可丰富 | 为 v0.1.1 添加功能亮点摘要 + 截图。 |
| 8 | 30-60 秒演示视频 | **不存在** | 比 GIF 更完整，上传 YouTube / Bilibili，在 README 中嵌入链接。 |

### P2：可延后但值得做

| 序号 | 内容 | 说明 |
| --- | --- | --- |
| 9 | 技术博文草稿 | 两条叙事线各一篇 |
| 10 | Social media 封面图 | 适合 Twitter/X 的 16:9 图片 |
| 11 | Marketplace 页面截图审查 | 确认扩展在商店页面的展示效果 |
| 12 | Getting Started 引导文档 | 从安装到第一个 tape 文件的快速上手 |

## 时间线建议

```text
Day 0-2   准备阶段
          ├─ 录制 Demo GIF
          ├─ 截取功能截图集
          ├─ 更新 README（嵌入 GIF + 截图）
          ├─ 创建 examples/ 目录
          ├─ 设置 GitHub topics、Issue/PR 模板
          └─ 优化 GitHub Release Notes

Day 3     第一波 — Charmbracelet 社区
          ├─ Discord / Slack 发帖
          └─ VHS GitHub Discussion

Day 4-5   第二波 — 英文技术社区
          ├─ Show HN（选周二或周三上午美东）
          ├─ Reddit r/rust + r/vscode + r/commandline
          └─ Twitter/X @charmbracelet

Day 5-7   第三波 — 中文社区
          ├─ V2EX "分享创造"
          ├─ 掘金技术文章
          └─ 其他中文平台

Day 7+    第四波 — 长尾
          ├─ 技术博文发布
          ├─ 持续响应 issue 和 discussion
          └─ 根据反馈迭代
```

## 双叙事线策略

项目有一个多数开源工具不具备的独特卖点：完整记录了 AI 三角色协作的开发过程（`prompt/`、`trace/`、`docs/agentic-workflow.md`）。

建议分两条线宣发，面向不同受众，在不同渠道分别推送：

- **产品线**（面向 VHS 用户）："你的 `.tape` 文件终于有了真正的 IDE 支持"
- **方法论线**（面向 Rust / AI 工程师）："我如何用 AI 三角色开发了一个完整的 LSP + 扩展"

## 核心行动项

**最大宣发障碍是缺少视觉展示材料。** README 中没有任何 GIF 或截图，在开源世界里几乎等同于隐身。一个 30 秒的 Demo GIF 嵌入 README 顶部，可以带来数量级的页面停留时间和 star 转化率提升。

**第一优先级：录制 Demo GIF + 更新 README。**
