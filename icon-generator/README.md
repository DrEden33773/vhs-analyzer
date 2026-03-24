# Icon Generator

Programmatic icon generator for the `vhs-analyzer` VS Code extension.

## Design

The icon depicts a VHS tape cassette with a `</>` code symbol overlay,
communicating "language analysis tooling for VHS tape files" at a glance.

- Dark purple gradient background with a rounded-rectangle tape body.
- Label area with cyan and magenta accent stripes.
- Reel window with dual spoked reels and a centered `</>` glyph in cyan glow.
- Colour palette intentionally differs from the official Charm VHS branding to
  avoid confusion with the upstream project.

## How It Works

The script draws at 8x resolution (1024 x 1024) using Pillow, then downscales
to the Marketplace-required 128 x 128 with LANCZOS resampling. This
supersampling approach produces clean anti-aliased edges without requiring a
vector renderer.

## Usage

```bash
uv sync
uv run python generate.py
```

The script writes `output/icon.png` and copies it to
`editors/code/icon.png` automatically.

## Tooling

- **Runtime**: Python 3.14, managed by uv
- **Imaging**: Pillow
- **Linting**: pyright (standard) + ruff (ALL minus suppressed rules)
