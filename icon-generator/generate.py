"""
Generate the vhs-analyzer VS Code extension icon.

Renders a VHS-tape-inspired icon at 8x resolution and downscales to 128x128
with LANCZOS resampling for clean anti-aliased edges.
"""

from __future__ import annotations

import math
import shutil
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

SCALE = 8
CANVAS = 128 * SCALE  # 1024 px working resolution
OUTPUT_SIZE = 128
ICON_FILENAME = "icon.png"

BG_TOP = (30, 25, 55)
BG_BOTTOM = (15, 12, 35)

TAPE_FILL = (45, 40, 75)
TAPE_BORDER = (100, 90, 160)

REEL_RING = (80, 70, 130)
REEL_CENTER = (60, 55, 100)
REEL_HUB = (110, 100, 170)

WINDOW_FILL = (20, 18, 40)
WINDOW_BORDER = (70, 65, 120)

LABEL_BG = (65, 58, 110)
STRIPE_CYAN = (0, 220, 220)
STRIPE_MAGENTA = (200, 80, 220)

CODE_COLOR = (0, 240, 220)
CODE_GLOW = (0, 180, 180, 80)


def _gradient_bg(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    for y in range(size):
        t = y / max(size - 1, 1)
        r = int(BG_TOP[0] + (BG_BOTTOM[0] - BG_TOP[0]) * t)
        g = int(BG_TOP[1] + (BG_BOTTOM[1] - BG_TOP[1]) * t)
        b = int(BG_TOP[2] + (BG_BOTTOM[2] - BG_TOP[2]) * t)
        draw.line([(0, y), (size - 1, y)], fill=(r, g, b, 255))
    return img


def _rounded_rect(
    draw: ImageDraw.ImageDraw,
    bbox: tuple[float, float, float, float],
    radius: float,
    *,
    fill: tuple[int, ...] | None = None,
    outline: tuple[int, ...] | None = None,
    width: int = 1,
) -> None:
    x0, y0, x1, y1 = bbox
    draw.rounded_rectangle(
        [x0, y0, x1, y1],
        radius=radius,
        fill=fill,
        outline=outline,
        width=width,
    )


def _draw_reel(
    draw: ImageDraw.ImageDraw,
    cx: float,
    cy: float,
    outer_r: float,
    spoke_count: int = 6,
) -> None:
    draw.ellipse(
        [cx - outer_r, cy - outer_r, cx + outer_r, cy + outer_r],
        fill=REEL_CENTER,
        outline=REEL_RING,
        width=max(int(2 * SCALE), 1),
    )

    hub_r = outer_r * 0.35
    draw.ellipse(
        [cx - hub_r, cy - hub_r, cx + hub_r, cy + hub_r],
        fill=REEL_HUB,
        outline=REEL_RING,
        width=max(int(1.5 * SCALE), 1),
    )

    for i in range(spoke_count):
        angle = 2 * math.pi * i / spoke_count
        inner = hub_r * 0.8
        outer = outer_r * 0.85
        x_inner = cx + math.cos(angle) * inner
        y_inner = cy + math.sin(angle) * inner
        x_outer = cx + math.cos(angle) * outer
        y_outer = cy + math.sin(angle) * outer
        draw.line(
            [(x_inner, y_inner), (x_outer, y_outer)],
            fill=REEL_RING,
            width=max(int(1.5 * SCALE), 1),
        )


def _draw_code_symbol(
    draw: ImageDraw.ImageDraw,
    cx: float,
    cy: float,
    size: float,
) -> None:
    font: ImageFont.FreeTypeFont | ImageFont.ImageFont
    font_size = int(size * 0.9)
    try:
        for name in ("DejaVuSansMono-Bold", "DejaVuSansMono", "monospace"):
            try:
                font = ImageFont.truetype(name, font_size)
                break
            except OSError:
                continue
        else:
            font = ImageFont.load_default(font_size)
    except TypeError:
        font = ImageFont.load_default()

    text = "</>"
    bbox = font.getbbox(text)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    tx = cx - tw / 2
    ty = cy - th / 2 - bbox[1]

    for dx in (-2, 0, 2):
        for dy in (-2, 0, 2):
            if dx == 0 and dy == 0:
                continue
            draw.text((tx + dx, ty + dy), text, fill=CODE_GLOW, font=font)

    draw.text((tx, ty), text, fill=CODE_COLOR, font=font)


def generate_icon() -> Image.Image:
    """Compose the full icon at working resolution and downscale."""
    img = _gradient_bg(CANVAS)
    draw = ImageDraw.Draw(img)

    margin = int(16 * SCALE)
    tape_radius = int(16 * SCALE)
    tape_x0 = margin
    tape_y0 = int(margin * 1.1)
    tape_x1 = CANVAS - margin
    tape_y1 = CANVAS - int(margin * 0.9)

    _rounded_rect(
        draw,
        (tape_x0, tape_y0, tape_x1, tape_y1),
        tape_radius,
        fill=TAPE_FILL,
        outline=TAPE_BORDER,
        width=int(2.5 * SCALE),
    )

    label_h = int(16 * SCALE)
    label_margin = int(12 * SCALE)
    label_y0 = tape_y0 + int(8 * SCALE)
    label_y1 = label_y0 + label_h
    _rounded_rect(
        draw,
        (tape_x0 + label_margin, label_y0, tape_x1 - label_margin, label_y1),
        int(6 * SCALE),
        fill=LABEL_BG,
    )

    stripe_h = int(2.5 * SCALE)
    stripe_y = label_y0 + label_h // 3
    draw.rectangle(
        [
            tape_x0 + label_margin + int(4 * SCALE),
            stripe_y,
            tape_x1 - label_margin - int(4 * SCALE),
            stripe_y + stripe_h,
        ],
        fill=STRIPE_CYAN,
    )
    stripe_y2 = stripe_y + int(5 * SCALE)
    draw.rectangle(
        [
            tape_x0 + label_margin + int(4 * SCALE),
            stripe_y2,
            tape_x1 - label_margin - int(4 * SCALE),
            stripe_y2 + stripe_h,
        ],
        fill=STRIPE_MAGENTA,
    )

    window_y0 = label_y1 + int(8 * SCALE)
    window_y1 = tape_y1 - int(14 * SCALE)
    window_margin = int(20 * SCALE)
    window_radius = int(10 * SCALE)
    _rounded_rect(
        draw,
        (tape_x0 + window_margin, window_y0, tape_x1 - window_margin, window_y1),
        window_radius,
        fill=WINDOW_FILL,
        outline=WINDOW_BORDER,
        width=int(2 * SCALE),
    )

    window_cx = CANVAS / 2
    window_cy = (window_y0 + window_y1) / 2
    reel_r = (window_y1 - window_y0) * 0.32
    reel_spacing = (tape_x1 - tape_x0 - 2 * window_margin) * 0.25

    _draw_reel(draw, window_cx - reel_spacing, window_cy, reel_r)
    _draw_reel(draw, window_cx + reel_spacing, window_cy, reel_r)

    code_cy = window_cy + reel_r * 0.1
    _draw_code_symbol(draw, window_cx, code_cy, reel_r * 1.3)

    return img.resize((OUTPUT_SIZE, OUTPUT_SIZE), Image.Resampling.LANCZOS)


def main() -> None:
    icon = generate_icon()

    output_dir = Path(__file__).resolve().parent / "output"
    output_dir.mkdir(exist_ok=True)
    local_path = output_dir / ICON_FILENAME
    icon.save(local_path, format="PNG", optimize=True)
    print(f"Saved: {local_path} ({local_path.stat().st_size} bytes)")

    extension_icon = (
        Path(__file__).resolve().parent.parent / "editors" / "code" / ICON_FILENAME
    )
    shutil.copy2(local_path, extension_icon)
    print(f"Copied to: {extension_icon}")


if __name__ == "__main__":
    main()
