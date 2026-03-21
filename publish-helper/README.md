# Publish Helper

This directory contains publication-readiness notes for `vhs-analyzer`.
The content is based on the current repository state and uses
[rust-analyzer](https://github.com/rust-lang/rust-analyzer) as a reference for
release discipline, not as a template to copy blindly.

## Current Conclusion

- The project is close to public-release ready at the code and CI level.
- The main remaining work is repository surface polish, governance basics, and
  release-day operational checks.
- The highest-value remaining items are a strong root repository entrypoint,
  minimal open-source governance files, and one real-editor smoke test before
  the first public extension release.

## Recommended Order

1. Finish the repository-public checklist.
2. Confirm release-day checklist items.
3. Switch the repository to public.
4. Run one beta or dry-run release.
5. Publish the extension.

## Document Map

| File | Language | Purpose |
| --- | --- | --- |
| [repo-public-preflight.zh-CN.md](./repo-public-preflight.zh-CN.md) | Chinese | One-hour checklist before making the repository public |
| [repo-public-preflight.en.md](./repo-public-preflight.en.md) | English | One-hour checklist before making the repository public |
| [extension-release-day.zh-CN.md](./extension-release-day.zh-CN.md) | Chinese | Release-day checklist for the first extension launch |
| [extension-release-day.en.md](./extension-release-day.en.md) | English | Release-day checklist for the first extension launch |

## Scope Notes

- These documents focus on what still matters before the first public release.
- They are intentionally practical, short, and friendly to both humans and
  agents.
- Optional `MAY`-level work from the frozen specs is treated as deferred unless
  it materially reduces first-release risk.

## Main Themes

- Repository clarity:
  A new visitor should understand what the project is, how to install it, and
  where to report issues within one minute.
- Governance minimum:
  `README.md`, `CONTRIBUTING.md`, and `SECURITY.md` matter more than broad
  process ceremony at this stage.
- Release confidence:
  Real VSIX smoke tests and release workflow dry runs are worth more than extra
  local abstraction or polish.
