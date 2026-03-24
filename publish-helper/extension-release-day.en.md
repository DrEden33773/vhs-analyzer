# Release-Day Checklist for the First Extension Launch

## Goal

On the day of the first public `vhs-analyzer` extension release, use one
practical checklist to reduce the risk of release failures, missing marketplace
metadata, and real-install regressions.

## Done Criteria

- Extension metadata, docs, icon, version notes, and publishing credentials are
  all ready.
- Local and CI gates are green.
- At least one real VSIX installation has been validated, not only Vitest runs.
- After release, Marketplace, Open VSX, and GitHub Release can all be checked
  quickly and confidently.

## P0: Must Finish on Release Day

- [x] Confirm whether this release is `stable` or `beta` / `pre-release`
  - Decision: `stable` (v0.1.1)
- [x] Confirm the version story
  - Private development used internal milestone versions (`0.2.0` Rust
    workspace, `0.3.0` extension), normalized to `0.1.0` before public release
  - `v0.1.0-rc.1` was used as the dry-run pre-release; five release-workflow
    bugs were found and fixed during the dry-run
  - Stable release bumped to `0.1.1` to avoid marketplace version conflicts
    with the consumed `0.1.0` pre-release
  - Extension version: `0.1.1`
  - Rust workspace / crate version: `0.1.1`
- [x] Review `editors/code/package.json`
  - `publisher`: `DrEden33773`
  - `repository`: set
  - `license`: `MIT`
  - `icon`: `icon.png` (128x128, 8x supersampled, 7647 bytes)
  - `engines`: `vscode ^1.85.0`
  - `categories`: Programming Languages, Linters, Formatters
  - `keywords`: vhs, tape, terminal, recording, gif, lsp
  - `homepage`: set
  - `bugs`: set
- [x] Review `editors/code/README.md`
  - clearly explains platform-specific vs universal packages
  - clearly explains runtime dependencies: `vhs`, `ttyd`, `ffmpeg`
  - installation guidance reads cleanly
- [x] Review `editors/code/CHANGELOG.md`
  - covers `0.1.0` (initial feature set) and `0.1.1` (workflow fixes + icon)
  - supports release notes
- [x] Review `icon.png`
  - path matches `package.json`
  - VHS tape + `</>` design displays well on both marketplace pages
  - generated programmatically via `icon-generator/` for reproducibility

## P0: Gate and Packaging Verification

- [x] Extension-side verification passes

```bash
pnpm run lint
pnpm run typecheck
pnpm run test
pnpm run build
pnpm exec vsce ls --no-dependencies
pnpm exec vsce package --no-dependencies
```

- [x] Rust-side verification passes

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --release -p vhs-analyzer-lsp --locked
```

- [x] `extension-ci.yml` is green
- [x] `release.yml` has been validated through `v0.1.0-rc.1` dry-run and
  `v0.1.1` stable release
- [x] Expected asset count is correct
  - 7 platform VSIX files (win32-x64, darwin-arm64, darwin-x64, linux-x64,
    linux-arm64, alpine-x64)
  - 1 universal VSIX file

## P0: Real Installation Smoke Tests

- [x] Install one platform VSIX in real VS Code and Cursor
- [x] Open a `.tape` file and confirm:
  - activation works
  - LSP handshake works
  - hover / completion / diagnostics / formatting work
  - CodeLens is visible
  - Preview renders correctly
- [x] Install the universal VSIX and confirm:
  - it enters no-server mode without a bundled LSP
  - syntax highlighting, CodeLens, and Preview still work
  - hover / diagnostics / formatting are unavailable
- [x] Verified behavior when runtime dependencies are missing
  - user-facing guidance is clear

## P0: Credentials and Registries

- [x] `VSCE_PAT` is configured and has the correct permissions
- [x] `OVSX_PAT` is configured and has the correct permissions
- [x] Open VSX namespace / publisher matches the `publisher` field in
  `package.json` (namespace verification submitted, pending review)
- [x] GitHub Release upload permissions are working
- [x] Repository visibility, description, and homepage links are ready

## Release Execution Steps

1. [x] Confirm the working tree is clean and the target commit is correct
2. [x] Create the release tag (`v0.1.1`)
3. [x] Trigger `release.yml` (tag push)
4. [x] Watch the job order
   - `lint-and-test`
   - `build-rust`
   - `package-vsix`
   - `publish` (VS Code Marketplace + Open VSX, separate steps)
5. [x] Check the GitHub Release
   - tag `v0.1.1`, stable (not pre-release)
   - 7 VSIX assets present
6. [x] Check the VS Code Marketplace page
7. [x] Check the Open VSX page
8. [x] Complete real installs from VS Code Marketplace (VS Code) and
   Open VSX (Cursor, with up to 24h cache delay noted)

## Recommended Work in the First 24 Hours After Release

- [ ] Watch and triage the first wave of issues
- [ ] Publish a demo GIF, screenshot set, or short announcement
- [ ] Log first-install problems and platform differences
- [ ] Review installation and activation feedback
- [ ] Decide whether to add issue templates and support docs immediately

## Safe to Defer, but Do It Deliberately

- [ ] Automate `T-INT3-004` / `T-INT3-005` with `@vscode/test-electron`
- [ ] Add more store-page styling metadata
- [ ] Expand privacy / support / community docs
- [ ] Build heavier launch-marketing assets
- [ ] Complete Open VSX namespace verification

## Lessons Learned from the Dry-Run

The `v0.1.0-rc.1` dry-run exposed five bugs in `release.yml` that had never
been caught because the workflow was written but never executed end-to-end:

1. Integration tests ran before the LSP binary was built (missing
   `VHS_ANALYZER_LSP_BINARY` env var).
2. `macos-13` runner was retired; replaced with `macos-14` cross-compilation
   for `x86_64-apple-darwin`.
3. Binary artifact download path was off by one directory level
   (`../binary` vs `../../binary`).
4. VSIX file paths in the publish step were relative but `vsce` ran from a
   different working directory; fixed with absolute paths.
5. `--pre-release` flag was only passed to `vsce publish` but not
   `vsce package`; the marketplace rejects mismatched flags.

Additionally, the rc pre-release consumed version `0.1.0` on both
marketplaces, making it impossible to re-publish the same version as stable.
The stable release was bumped to `0.1.1`.

## Warning Signs That Should Block the Release

- `release.yml` has never been dry-run tested
- real VSIX installation has not been validated
- `publisher` / Open VSX namespace alignment is unclear
- `README.md` does not explain platform-specific vs universal packages cleanly
- release notes cannot answer the question:
  "What does the user get after installing this?"

## Reference Files

- `editors/code/package.json`
- `editors/code/README.md`
- `editors/code/CHANGELOG.md`
- `editors/code/icon.png`
- `icon-generator/` (programmatic icon source)
- `.github/workflows/extension-ci.yml`
- `.github/workflows/release.yml`
- `Cargo.toml`
- `trace/phase3/tracker.md`
- `trace/phase3/status.yaml`

## Guiding Principle

- Use [rust-analyzer](https://github.com/rust-lang/rust-analyzer) as a model
  for release discipline and multi-artifact distribution.
- But follow this repository's real needs:
  for a first launch, "installable, runnable, understandable" matters more than
  trying to match every mature-project governance feature on day one.
