# 首次插件发布当日操作清单

## 目标

在首次正式发布 `vhs-analyzer` 扩展当天，用一份可执行清单降低发布事故、商店元信息缺漏和真实安装回归的风险。

## 完成标准

- 扩展的元信息、文档、图标、版本说明和发布凭据都已准备好。
- 本地与 CI 的关键门禁全部为绿。
- 至少做过一次真实 VSIX 安装验证，而不只是在 Vitest 中通过。
- 发布后能快速确认 Marketplace、Open VSX 和 GitHub Release 三条分发链路都正常。

## P0：发布当天必须完成

- [x] 确认本次发布是 `stable` 还是 `beta` / `pre-release`
  - 决定：`stable`（v0.1.1）
- [x] 明确版本叙事
  - 私有开发阶段使用了内部里程碑版本号（Rust workspace `0.2.0`、扩展 `0.3.0`），在首次公开发布前归一化为 `0.1.0`
  - `v0.1.0-rc.1` 用于 dry-run 预发布；期间发现并修复了五个 release workflow bug
  - 正式发布 bump 到 `0.1.1` 以避免与已被占用的 `0.1.0` 预发布版本冲突
  - 扩展版本：`0.1.1`
  - Rust workspace / crate 版本：`0.1.1`
- [x] 检查 `editors/code/package.json`
  - `publisher`：`DrEden33773`
  - `repository`：已设置
  - `license`：`MIT`
  - `icon`：`icon.png`（128x128，8x 超采样，7647 bytes）
  - `engines`：`vscode ^1.85.0`
  - `categories`：Programming Languages, Linters, Formatters
  - `keywords`：vhs, tape, terminal, recording, gif, lsp
  - `homepage`：已补充
  - `bugs`：已补充
- [x] 检查 `editors/code/README.md`
  - 平台包与 universal 包差异已写清
  - 运行时依赖 `vhs`、`ttyd`、`ffmpeg` 已写清
  - 安装说明顺畅
- [x] 检查 `editors/code/CHANGELOG.md`
  - 覆盖 `0.1.0`（初始功能集）和 `0.1.1`（workflow 修复 + 图标升级）
  - 可支撑发布说明
- [x] 检查 `icon.png`
  - 路径与 `package.json` 一致
  - VHS 磁带 + `</>` 设计在两个商店页均显示良好
  - 通过 `icon-generator/` 程序化生成，可复现

## P0：门禁与打包验证

- [x] 扩展侧验证通过

```bash
pnpm run lint
pnpm run typecheck
pnpm run test
pnpm run build
pnpm exec vsce ls --no-dependencies
pnpm exec vsce package --no-dependencies
```

- [x] Rust 侧验证通过

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --release -p vhs-analyzer-lsp --locked
```

- [x] `extension-ci.yml` 通过
- [x] `release.yml` 已通过 `v0.1.0-rc.1` dry-run 和 `v0.1.1` 正式发布验证
- [x] 预期产物数量正确
  - 7 个平台 VSIX（win32-x64、darwin-arm64、darwin-x64、linux-x64、linux-arm64、alpine-x64）
  - 1 个 universal VSIX

## P0：真实安装冒烟

- [x] 在真实 VS Code 和 Cursor 中安装了平台 VSIX
- [x] 打开 `.tape` 文件，确认：
  - 激活成功
  - LSP 握手正常
  - hover / completion / diagnostics / formatting 可用
  - CodeLens 可见
  - Preview 可正常渲染
- [x] 安装 universal VSIX，确认：
  - 无 bundled LSP 时进入 no-server 模式
  - 语法高亮、CodeLens、Preview 仍可用
  - hover / diagnostics / formatting 不可用
- [x] 验证了缺少运行时依赖时的行为
  - 用户提示足够清晰

## P0：发布凭据与注册表

- [x] `VSCE_PAT` 已配置且权限正确
- [x] `OVSX_PAT` 已配置且权限正确
- [x] Open VSX namespace / publisher 与 `package.json` 中的 `publisher` 一致（namespace 验证已提交，等待审核）
- [x] GitHub Release 上传权限正常
- [x] 仓库可见性、描述和主页链接已经准备好

## 发布执行步骤

1. [x] 确认工作树干净、目标 commit 正确
2. [x] 创建版本 tag（`v0.1.1`）
3. [x] 触发 `release.yml`（tag 推送）
4. [x] 观察作业顺序
   - `lint-and-test`
   - `build-rust`
   - `package-vsix`
   - `publish`（VS Code Marketplace + Open VSX，独立步骤）
5. [x] 检查 GitHub Release
   - tag `v0.1.1`，stable（非 pre-release）
   - 7 个 VSIX 资产齐全
6. [x] 检查 VS Code Marketplace 页面
7. [x] 检查 Open VSX 页面
8. [x] 从 VS Code Marketplace（VS Code）和 Open VSX（Cursor，已知最长 24h 缓存延迟）完成真实安装

## 发布后 24 小时内建议做的事

- [ ] 置顶或关注首批 issue
- [ ] 发布演示 GIF / 截图 / 简短介绍
- [ ] 记录首批安装问题和平台差异
- [ ] 检查下载、安装、激活反馈
- [ ] 决定是否立刻补充 Issue 模板和支持文档

## 可以明确延后，但要心里有数

- [ ] 用 `@vscode/test-electron` 自动化 `T-INT3-004` / `T-INT3-005`
- [ ] 更细的商店页样式字段
- [ ] 更完整的隐私 / 支持 / 社区文档
- [ ] 更重的发布宣传资产
- [ ] 完成 Open VSX namespace 验证

## Dry-Run 中的经验教训

`v0.1.0-rc.1` dry-run 暴露了 `release.yml` 中五个从未被发现的 bug（因为该 workflow 此前只编写、从未实际执行过端到端流程）：

1. 集成测试在 LSP 二进制构建之前运行（缺少 `VHS_ANALYZER_LSP_BINARY` 环境变量）。
2. `macos-13` runner 已退役；替换为 `macos-14` 上对 `x86_64-apple-darwin` 的交叉编译。
3. 二进制产物下载路径差了一级目录（`../binary` 应为 `../../binary`）。
4. publish 步骤中 VSIX 文件路径是相对的，但 `vsce` 在不同的工作目录下执行；改为绝对路径。
5. `--pre-release` 标记只传给了 `vsce publish` 而未传给 `vsce package`；marketplace 会拒绝标记不一致的包。

此外，rc 预发布占用了两个 marketplace 上的 `0.1.0` 版本号，导致无法以相同版本号发布 stable。正式发布 bump 到了 `0.1.1`。

## 不建议带病发布的信号

- `release.yml` 从未跑过 dry run
- 真实 VSIX 安装还没验证
- `publisher` / Open VSX namespace 关系不清楚
- `README.md` 没把平台包与 universal 包解释清楚
- 发布说明无法回答"用户装完后会得到什么"

## 参考文件

- `editors/code/package.json`
- `editors/code/README.md`
- `editors/code/CHANGELOG.md`
- `editors/code/icon.png`
- `icon-generator/`（程序化图标源码）
- `.github/workflows/extension-ci.yml`
- `.github/workflows/release.yml`
- `Cargo.toml`
- `trace/phase3/tracker.md`
- `trace/phase3/status.yaml`

## 参考原则

- 参考 [rust-analyzer](https://github.com/rust-lang/rust-analyzer) 的发布纪律与多工件分发思路。
- 但以本项目实际情况为准：首发重点是"可安装、可运行、可理解"，不是一次性补齐所有成熟项目治理配置。
