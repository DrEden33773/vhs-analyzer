# 首次插件发布当日操作清单

## 目标

在首次正式发布 `vhs-analyzer` 扩展当天，用一份可执行清单降低发布事故、商店元信息缺漏和真实安装回归的风险。

## 完成标准

- 扩展的元信息、文档、图标、版本说明和发布凭据都已准备好。
- 本地与 CI 的关键门禁全部为绿。
- 至少做过一次真实 VSIX 安装验证，而不只是在 Vitest 中通过。
- 发布后能快速确认 Marketplace、Open VSX 和 GitHub Release 三条分发链路都正常。

## P0：发布当天必须完成

- [ ] 确认本次发布是 `stable` 还是 `beta` / `pre-release`
- [ ] 明确版本叙事
  - 首次 public release 基线是否统一为 `0.1.0`
  - 扩展版本
  - Rust workspace / crate 版本
  - 如需解释，说明 private 开发阶段曾使用内部里程碑版本号，已在首次公开发布前完成归一化
- [ ] 检查 `editors/code/package.json`
  - `publisher`
  - `repository`
  - `license`
  - `icon`
  - `engines`
  - `categories`
  - `keywords`
  - 建议补 `homepage`
  - 建议补 `bugs`
- [ ] 检查 `editors/code/README.md`
  - 平台包与 universal 包差异是否写清
  - 运行时依赖 `vhs`、`ttyd`、`ffmpeg` 是否写清
  - 安装说明是否顺畅
- [ ] 检查 `editors/code/CHANGELOG.md`
  - 是否覆盖所有用户可见变化
  - 是否能支撑本次发布说明
- [ ] 检查 `icon.png`
  - 路径与 `package.json` 一致
  - 显示效果符合商店首页需求

## P0：门禁与打包验证

- [ ] 扩展侧验证通过

```bash
pnpm run lint
pnpm run typecheck
pnpm run test
pnpm run build
pnpm exec vsce ls --no-dependencies
pnpm exec vsce package --no-dependencies
```

- [ ] Rust 侧验证通过

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --release -p vhs-analyzer-lsp --locked
```

- [ ] `extension-ci.yml` 通过
- [ ] `release.yml` 的 dry run、beta tag 或手动触发已经验证过至少一次
- [ ] 预期产物数量正确
  - 6 个平台 VSIX
  - 1 个 universal VSIX

## P0：真实安装冒烟

- [ ] 在真实 VS Code 或 Cursor 中安装一个平台 VSIX
- [ ] 打开 `.tape` 文件，确认：
  - 激活成功
  - LSP 握手正常
  - hover / completion / diagnostics / formatting 可用
  - CodeLens 可见
  - Preview 可正常渲染
- [ ] 安装 universal VSIX，确认：
  - 无 bundled LSP 时进入 no-server 模式
  - 语法高亮、CodeLens、Preview 仍可用
  - hover / diagnostics / formatting 不可用
- [ ] 有条件时分别验证：
  - 缺少 `vhs`
  - 缺少 `ttyd`
  - 缺少 `ffmpeg`
  - 用户提示是否足够清晰

## P0：发布凭据与注册表

- [ ] `VSCE_PAT` 已配置且权限正确
- [ ] `OVSX_PAT` 已配置且权限正确
- [ ] Open VSX namespace / publisher 与 `package.json` 中的 `publisher` 一致
- [ ] GitHub Release 上传权限正常
- [ ] 仓库可见性、描述和主页链接已经准备好

## 发布执行步骤

1. [ ] 确认工作树干净、目标 commit 正确
2. [ ] 创建预定版本 tag
3. [ ] 触发 `release.yml`
4. [ ] 观察作业顺序
   - `lint-and-test`
   - `build-rust`
   - `package-vsix`
   - `publish`
5. [ ] 检查 GitHub Release
   - tag 正确
   - 说明正确
   - 7 个 VSIX 资产齐全
6. [ ] 检查 VS Code Marketplace 页面
7. [ ] 检查 Open VSX 页面
8. [ ] 从至少一个注册表完成一次真实安装

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

## 不建议带病发布的信号

- `release.yml` 从未跑过 dry run
- 真实 VSIX 安装还没验证
- `publisher` / Open VSX namespace 关系不清楚
- `README.md` 没把平台包与 universal 包解释清楚
- 发布说明无法回答“用户装完后会得到什么”

## 参考文件

- `editors/code/package.json`
- `editors/code/README.md`
- `editors/code/CHANGELOG.md`
- `editors/code/icon.png`
- `.github/workflows/extension-ci.yml`
- `.github/workflows/release.yml`
- `Cargo.toml`
- `trace/phase3/tracker.md`
- `trace/phase3/status.yaml`

## 参考原则

- 参考 [rust-analyzer](https://github.com/rust-lang/rust-analyzer) 的发布纪律与多工件分发思路。
- 但以本项目实际情况为准：
  你的首发重点是“可安装、可运行、可理解”，不是一次性补齐所有成熟项目治理配置。
