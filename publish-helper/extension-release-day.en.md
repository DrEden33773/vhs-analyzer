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

- [ ] Confirm whether this release is `stable` or `beta` / `pre-release`
- [ ] Confirm the version story
  - whether the first public release baseline is unified at `0.1.0`
  - extension version
  - Rust workspace / crate version
  - if needed, explain that private development used internal milestone versions and they were normalized before the first public release
- [ ] Review `editors/code/package.json`
  - `publisher`
  - `repository`
  - `license`
  - `icon`
  - `engines`
  - `categories`
  - `keywords`
  - recommended: `homepage`
  - recommended: `bugs`
- [ ] Review `editors/code/README.md`
  - clearly explain platform-specific vs universal packages
  - clearly explain runtime dependencies: `vhs`, `ttyd`, `ffmpeg`
  - make sure installation guidance reads cleanly
- [ ] Review `editors/code/CHANGELOG.md`
  - confirm it covers all user-visible changes
  - confirm it can support release notes
- [ ] Review `icon.png`
  - confirm the path matches `package.json`
  - confirm it looks good enough for a store listing

## P0: Gate and Packaging Verification

- [ ] Extension-side verification passes

```bash
pnpm run lint
pnpm run typecheck
pnpm run test
pnpm run build
pnpm exec vsce ls --no-dependencies
pnpm exec vsce package --no-dependencies
```

- [ ] Rust-side verification passes

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --release -p vhs-analyzer-lsp --locked
```

- [ ] `extension-ci.yml` is green
- [ ] `release.yml` has been validated at least once through a dry run, beta
  tag, or manual dispatch
- [ ] Expected asset count is correct
  - 6 platform VSIX files
  - 1 universal VSIX file

## P0: Real Installation Smoke Tests

- [ ] Install one platform VSIX in real VS Code or Cursor
- [ ] Open a `.tape` file and confirm:
  - activation works
  - LSP handshake works
  - hover / completion / diagnostics / formatting work
  - CodeLens is visible
  - Preview renders correctly
- [ ] Install the universal VSIX and confirm:
  - it enters no-server mode without a bundled LSP
  - syntax highlighting, CodeLens, and Preview still work
  - hover / diagnostics / formatting are unavailable
- [ ] If possible, verify behavior when:
  - `vhs` is missing
  - `ttyd` is missing
  - `ffmpeg` is missing
  - user-facing guidance remains clear

## P0: Credentials and Registries

- [ ] `VSCE_PAT` is configured and has the correct permissions
- [ ] `OVSX_PAT` is configured and has the correct permissions
- [ ] Open VSX namespace / publisher matches the `publisher` field in
  `package.json`
- [ ] GitHub Release upload permissions are working
- [ ] Repository visibility, description, and homepage links are ready

## Release Execution Steps

1. [ ] Confirm the working tree is clean and the target commit is correct
2. [ ] Create the intended release tag
3. [ ] Trigger `release.yml`
4. [ ] Watch the job order
   - `lint-and-test`
   - `build-rust`
   - `package-vsix`
   - `publish`
5. [ ] Check the GitHub Release
   - tag is correct
   - release notes are correct
   - all 7 VSIX assets are present
6. [ ] Check the VS Code Marketplace page
7. [ ] Check the Open VSX page
8. [ ] Complete at least one real install from one registry

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

## Warning Signs That Should Block the Release

- `release.yml` has never been dry-run tested
- real VSIX installation has not been validated
- `publisher` / Open VSX namespace alignment is unclear
- `README.md` does not explain platform-specific vs universal packages cleanly
- release notes cannot answer the question:
  “What does the user get after installing this?”

## Reference Files

- `editors/code/package.json`
- `editors/code/README.md`
- `editors/code/CHANGELOG.md`
- `editors/code/icon.png`
- `.github/workflows/extension-ci.yml`
- `.github/workflows/release.yml`
- `Cargo.toml`
- `trace/phase3/tracker.md`
- `trace/phase3/status.yaml`

## Guiding Principle

- Use [rust-analyzer](https://github.com/rust-lang/rust-analyzer) as a model
  for release discipline and multi-artifact distribution.
- But follow this repository’s real needs:
  for a first launch, “installable, runnable, understandable” matters more than
  trying to match every mature-project governance feature on day one.
