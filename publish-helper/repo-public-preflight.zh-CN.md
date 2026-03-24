# 公开仓库前 1 小时清单

状态：**已完成** — 仓库已公开。

## 目标

在将 `vhs-analyzer` 仓库设为 public 之前，用最少但关键的动作完成对外入口、治理基础和状态一致性收口。

## 完成标准

- 新访客打开仓库首页后，能在 1 分钟内知道项目是什么、如何安装、代码在哪里、如何反馈问题。
- 顶层文档不再出现"Phase 3 未完成"或类似误导状态。
- 仓库至少具备最小开源治理面：许可证、贡献入口、安全上报入口。
- 公开后不会因为内部目录或历史命名让外部用户明显困惑。

## P0：必须在公开前完成

- [x] 新增根目录 `README.md`
  - 说明项目定位：Rust LSP + VS Code/Cursor 扩展
  - 说明与 `vhs` 官方 CLI 的关系
  - 链接 `editors/code/README.md`、`spec/README.md`、`STATUS.yaml`
  - 给出最短安装和开发入口
- [x] 新增 `CONTRIBUTING.md`
  - 写清 `cargo` / `pnpm` 基本命令
  - 写清扩展开发建议使用 `vhs-analyzer.code-workspace`
  - 简要说明 `spec/` 驱动和变更协议
- [x] 新增 `SECURITY.md`
  - 写明漏洞上报方式
  - 明确公开 issue 是否适合报告安全问题
- [x] 同步顶层状态文档
  - `ROADMAP.md`
  - `EXECUTION_TRACKER.md`
  - `spec/README.md`
- [x] 将扩展目录引用统一规范为 `editors/code`
  - 对公开文档和冻结规范做路径一致性修订
  - 这属于文档修正，不改变行为契约
- [x] 决定是否完整公开以下目录
  - `prompt/` 已保留，并在根 `README.md` 中说明用途
  - `trace/` 已保留，并在根 `README.md` 中说明用途
  - `errors/` 已完成清理，移除了低价值的旧排障产物

## P0：公开前快速检查

- [x] 再扫一遍仓库，确认没有密钥、PAT、账户信息或临时日志泄漏
- [x] 确认根目录 `LICENSE` 明确可见
- [x] 确认 `STATUS.yaml` 与 `trace/phase3/status.yaml` 一致
- [x] 确认对外文档不再把 Phase 2/3 写成未完成
- [x] 确认所有公开入口与文档都已指向 `editors/code/`

## P1：建议在公开后尽快补上

- [ ] `CODE_OF_CONDUCT.md`
- [ ] Issue 模板
- [ ] PR 模板
- [ ] `SUPPORT.md` 或根 README 中的支持渠道说明
- [ ] 示例 `.tape` 文件或 `examples/` 目录
- [ ] 演示 GIF / 截图 / 短视频
- [ ] Dependabot 与 GitHub 安全功能配置

## P2：可以明确延后

- [ ] 文档站点或长篇手册
- [ ] 完整社区治理文件组
- [ ] 更细的仓库维护者分工
- [ ] 更重的自动化宣传或运营资产

## 参考文件

- `AGENTS.md`
- `STATUS.yaml`
- `EXECUTION_TRACKER.md`
- `ROADMAP.md`
- `spec/README.md`
- `trace/phase3/status.yaml`
- `trace/phase3/tracker.md`
- `editors/code/README.md`
- `vhs-analyzer.code-workspace`

## 参考原则

- 以当前仓库实际状态为准。
- 参考 [rust-analyzer](https://github.com/rust-lang/rust-analyzer) 的治理完整度与发布纪律，但不要机械照抄它的规模。
- 先完成让外部用户"看懂并信任"的工作，再追求更重的流程化建设。
