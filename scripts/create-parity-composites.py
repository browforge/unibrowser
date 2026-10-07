#!/usr/bin/env python3
"""Create labelled mockup-versus-app comparison images."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
DESIGN = ROOT / "docs" / "design" / "mockups"
CAPTURES = ROOT / "docs" / "screenshots" / "shell-density"
OUTPUT = ROOT / "docs" / "screenshots" / "parity"
PANEL_SIZE = (1280, 800)
HEADER_HEIGHT = 64


@dataclass(frozen=True)
class Pair:
    slug: str
    title: str
    mockup: Path
    capture: Path
    crop: tuple[int, int, int, int] | None = None
    schematic: bool = False


PAIRS = (
    Pair(
        "new-tab",
        "New tab",
        DESIGN / "quiet-utility" / "new-tab.png",
        CAPTURES / "new-tab-light-1280.png",
    ),
    Pair(
        "file-browser",
        "File browser",
        DESIGN / "context-matrix.png",
        CAPTURES / "file-browser-light-1280.png",
        (420, 109, 780, 348),
        True,
    ),
    Pair(
        "markdown-rich-text",
        "Markdown / rich text",
        DESIGN / "quiet-utility" / "document-command-compact.png",
        CAPTURES / "markdown-light-1280.png",
    ),
    Pair(
        "pdf",
        "PDF",
        DESIGN / "quiet-utility" / "pdf-annotation.png",
        CAPTURES / "pdf-light-1280.png",
    ),
    Pair(
        "image",
        "Image",
        DESIGN / "context-matrix.png",
        CAPTURES / "image-light-1280.png",
        (39, 370, 400, 609),
        True,
    ),
    Pair(
        "workshop",
        "Workshop",
        DESIGN / "quiet-utility" / "unibrow-create.png",
        CAPTURES / "workshop-light-1280.png",
    ),
    Pair(
        "settings",
        "Settings",
        DESIGN / "settings-mods-1280.png",
        CAPTURES / "settings-light-1280.png",
    ),
    Pair(
        "mod-portal",
        "Mod portal",
        DESIGN / "context-matrix.png",
        CAPTURES / "mod-portal-light-1280.png",
        (800, 631, 1161, 869),
        True,
    ),
)


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    name = "DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf"
    path = Path("/usr/share/fonts/truetype/dejavu") / name
    if not path.exists():
        return ImageFont.load_default()
    return ImageFont.truetype(str(path), size)


def fit_panel(path: Path, crop: tuple[int, int, int, int] | None, schematic: bool) -> Image.Image:
    image = Image.open(path).convert("RGB")
    if crop:
        image = image.crop(crop)
    if not schematic:
        return image.resize(PANEL_SIZE, Image.Resampling.LANCZOS)

    panel = Image.new("RGB", PANEL_SIZE, "#f1f0ec")
    scale = min(1040 / image.width, 650 / image.height)
    image = image.resize(
        (round(image.width * scale), round(image.height * scale)),
        Image.Resampling.LANCZOS,
    )
    x = (PANEL_SIZE[0] - image.width) // 2
    y = (PANEL_SIZE[1] - image.height) // 2
    panel.paste(image, (x, y))
    draw = ImageDraw.Draw(panel)
    draw.rectangle((x - 1, y - 1, x + image.width, y + image.height), outline="#c8c6bf")
    return panel


def create_pair(pair: Pair) -> Path:
    left = fit_panel(pair.mockup, pair.crop, pair.schematic)
    right = fit_panel(pair.capture, None, False)
    canvas = Image.new("RGB", (PANEL_SIZE[0] * 2, PANEL_SIZE[1] + HEADER_HEIGHT), "#172532")
    canvas.paste(left, (0, HEADER_HEIGHT))
    canvas.paste(right, (PANEL_SIZE[0], HEADER_HEIGHT))
    draw = ImageDraw.Draw(canvas)
    title_font = font(22, True)
    label_font = font(18, True)
    draw.text((20, 8), pair.title, fill="#e9edef", font=title_font)
    left_label = "APPROVED MOCKUP"
    if pair.schematic:
        left_label = "APPROVED LAYOUT SCHEMATIC"
    draw.text((20, 36), left_label, fill="#aebac0", font=label_font)
    draw.text((PANEL_SIZE[0] + 20, 36), "CURRENT APP", fill="#aebac0", font=label_font)
    draw.line(
        (PANEL_SIZE[0], HEADER_HEIGHT, PANEL_SIZE[0], canvas.height),
        fill="#79b1c9",
        width=2,
    )
    OUTPUT.mkdir(parents=True, exist_ok=True)
    destination = OUTPUT / f"{pair.slug}.png"
    canvas.save(destination, optimize=True)
    return destination


def main() -> int:
    for pair in PAIRS:
        print(create_pair(pair).relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
