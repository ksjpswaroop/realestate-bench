#!/usr/bin/env python3
"""
RealEstate-Bench icon + brand asset regenerator.

Pure Pillow + macOS `iconutil` for .icns. No external services, no API
calls, safe to run in CI. Generates:

  - icon.png (512)              Tauri base icon
  - icon-1024.png               App Store / marketing
  - icon-on-light.png           README / docs (paper background)
  - icon-{monochrome-forest,monochrome-paper}.png   Alt single-color marks
  - Square{30,44,71,89,107,142,150,284,310}x*.png   Windows Store tiles
  - StoreLogo.png               Windows Store badge
  - 32x32.png, 128x128.png, 128x128@2x.png  Tauri-required
  - wordmark-{400,800,1200}x*.png            Header / docs
  - wordmark-dark-800x200.png                Dark mode header
  - og-1200x630.png                          Open Graph / social
  - icon.icns                                 macOS bundle (via iconutil)
  - icon.ico                                  Windows bundle (via ImageMagick)

Color story (no violet/purple):
  forest     #0d6b5e   primary
  slate      #1e4a6b   secondary
  terracotta #b06a3a   accent
  paper      #fafaf8   light surface
"""
import os
import shutil
import subprocess
import sys
from PIL import Image, ImageDraw, ImageFont

OUT = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "src-tauri", "icons",
)
os.makedirs(OUT, exist_ok=True)

# Brand colors
FOREST     = (13, 107, 94)        # #0d6b5e
SLATE      = (30, 74, 107)        # #1e4a6b
TERRACOTTA = (176, 106, 58)       # #b06a3a
PAPER      = (250, 250, 248)      # #fafaf8
GRAPHITE   = (26, 26, 26)         # #1a1a1a


def make_logo(size: int, bg=None, fg=FOREST) -> Image.Image:
    """Render the brand mark at the given size.

    Anatomy: a house silhouette (roof + walls) in `fg`, with a JSON
    brace { } in PAPER cut into the wall, and a TERRACOTTA ground
    line. Works from 16x16 up to 1024x1024.
    """
    s4 = size * 4
    img = Image.new("RGBA", (s4, s4), (0, 0, 0, 0) if bg is None else bg + (255,))
    d = ImageDraw.Draw(img)
    house = [
        (int(0.10 * s4), int(0.36 * s4)),
        (int(0.50 * s4), int(0.10 * s4)),
        (int(0.90 * s4), int(0.36 * s4)),
        (int(0.80 * s4), int(0.36 * s4)),
        (int(0.80 * s4), int(0.82 * s4)),
        (int(0.20 * s4), int(0.82 * s4)),
        (int(0.20 * s4), int(0.36 * s4)),
    ]
    d.polygon(house, fill=fg)
    d.rectangle((int(0.05 * s4), int(0.82 * s4),
                 int(0.95 * s4), int(0.90 * s4)), fill=TERRACOTTA)
    cx = s4 // 2
    by1, by2 = int(0.46 * s4), int(0.76 * s4)
    my = (by1 + by2) // 2
    sw = max(int(s4 * 0.045), 2)
    lxo, lxi = cx - int(0.16 * s4), cx - int(0.08 * s4)
    rxo, rxi = cx + int(0.16 * s4), cx + int(0.08 * s4)
    brace_color = PAPER if bg != fg else GRAPHITE
    d.polygon([
        (lxo, by1), (lxi, by1), (lxi, my - sw), (cx - sw // 2, my),
        (lxi, my + sw), (lxi, by2), (lxo, by2),
        (lxo, my + sw * 2), (cx - int(0.03 * s4), my), (lxo, my - sw * 2),
    ], fill=brace_color)
    d.polygon([
        (rxo, by1), (rxi, by1), (rxi, my - sw), (cx + sw // 2, my),
        (rxi, my + sw), (rxi, by2), (rxo, by2),
        (rxo, my + sw * 2), (cx + int(0.03 * s4), my), (rxo, my - sw * 2),
    ], fill=brace_color)
    return img.resize((size, size), Image.LANCZOS)


def make_wordmark(width: int, height: int, bg=None, font_path=None,
                  fg1=FOREST, fg2=TERRACOTTA) -> Image.Image:
    """Render the wordmark: icon + 'RealEstate·Bench' + tagline."""
    w4, h4 = width * 2, height * 2
    img = Image.new("RGBA", (w4, h4), (0, 0, 0, 0) if bg is None else bg + (255,))
    d = ImageDraw.Draw(img)
    icon_size = int(h4 * 0.7)
    icon = make_logo(icon_size, bg=None, fg=fg1)
    icon_y = (h4 - icon_size) // 2
    img.paste(icon, (int(0.02 * w4), icon_y), icon)
    if font_path is None:
        for p in [
            "/System/Library/Fonts/Supplemental/Arial Black.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/Library/Fonts/Arial Bold.ttf",
            "/System/Library/Fonts/SFNS.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        ]:
            if os.path.exists(p):
                font_path = p
                break
    main_font = ImageFont.truetype(font_path, int(h4 * 0.30)) if font_path else ImageFont.load_default()
    sub_font  = ImageFont.truetype(font_path, int(h4 * 0.10)) if font_path else ImageFont.load_default()
    text_x = int(0.02 * w4) + icon_size + int(0.12 * h4)
    text_y = int(h4 * 0.18)
    full = "RealEstate\u00b7Bench"
    d.text((text_x, text_y), full, fill=fg1, font=main_font)
    prefix = "RealEstate\u00b7"
    pre_bbox = d.textbbox((0, 0), prefix, font=main_font)
    bench_bbox = d.textbbox((0, 0), "Bench", font=main_font)
    bench_w = bench_bbox[2] - bench_bbox[0]
    bench_x = text_x + (pre_bbox[2] - pre_bbox[0]) - bench_w
    d.text((bench_x, text_y), "Bench", fill=fg2, font=main_font)
    if bg is not None:
        d.text((text_x, int(h4 * 0.62)), "Local document extraction for real estate",
               fill=SLATE, font=sub_font)
    return img.resize((width, height), Image.LANCZOS)


def build_icns() -> None:
    """Use macOS iconutil to build a multi-size .icns."""
    ICONSET = os.path.join(OUT, "_regen.iconset")
    if os.path.exists(ICONSET):
        shutil.rmtree(ICONSET)
    os.makedirs(ICONSET, exist_ok=True)
    master = Image.open(os.path.join(OUT, "icon-1024.png")).convert("RGBA")
    mapping = {
        "icon_16x16.png": 16, "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32, "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128, "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256, "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512, "icon_512x512@2x.png": 1024,
    }
    for fname, sz in mapping.items():
        master.resize((sz, sz), Image.LANCZOS).save(os.path.join(ICONSET, fname), "PNG")
    subprocess.run(["iconutil", "-c", "icns", ICONSET, "-o", os.path.join(OUT, "icon.icns")], check=True)
    shutil.rmtree(ICONSET, ignore_errors=True)


def build_ico() -> None:
    """Use ImageMagick (if available) to build a proper multi-size .ico."""
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    master = Image.open(os.path.join(OUT, "icon-1024.png")).convert("RGBA")
    tmp = "/tmp/re_ico_build"
    os.makedirs(tmp, exist_ok=True)
    pngs = []
    for s in ico_sizes:
        p = os.path.join(tmp, f"ico_{s}.png")
        master.resize((s, s), Image.LANCZOS).save(p, "PNG")
        pngs.append(p)
    for tool in ("magick", "convert"):
        if subprocess.run(["which", tool], capture_output=True, text=True).stdout.strip():
            r = subprocess.run([tool] + (["convert"] if tool == "magick" else []) + pngs +
                               [os.path.join(OUT, "icon.ico")], capture_output=True, text=True)
            if r.returncode == 0:
                shutil.rmtree(tmp, ignore_errors=True)
                return
    # Fallback: Pillow 256-only .ico (single layer)
    master.resize((256, 256), Image.LANCZOS).save(
        os.path.join(OUT, "icon.ico"), "ICO", sizes=[(256, 256)])
    shutil.rmtree(tmp, ignore_errors=True)


def main() -> int:
    print(f"Regenerating brand assets in {OUT}")

    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
        "icon-1024.png": 1024,
    }
    for fname, sz in sizes.items():
        make_logo(sz, bg=None, fg=FOREST).save(os.path.join(OUT, fname), "PNG")
    print(f"  {len(sizes)} icon sizes written")

    make_logo(512, bg=PAPER, fg=FOREST).save(os.path.join(OUT, "icon-on-light.png"), "PNG")
    make_logo(512, bg=None, fg=FOREST).save(os.path.join(OUT, "icon-monochrome-forest.png"), "PNG")
    make_logo(512, bg=None, fg=PAPER).save(os.path.join(OUT, "icon-monochrome-paper.png"), "PNG")
    print("  on-light + 2 monochrome variants")

    for w, h, fname in [
        (800, 200, "wordmark-800x200.png"),
        (1200, 300, "wordmark-1200x300.png"),
        (400, 100, "wordmark-400x100.png"),
        (1200, 630, "og-1200x630.png"),
    ]:
        bg = PAPER if "og" in fname or "1200x300" in fname else None
        make_wordmark(w, h, bg=bg).save(os.path.join(OUT, fname), "PNG")
    make_wordmark(800, 200, bg=GRAPHITE).save(os.path.join(OUT, "wordmark-dark-800x200.png"), "PNG")
    print("  5 wordmark variants")

    if sys.platform == "darwin":
        build_icns()
        print(f"  icon.icns written ({os.path.getsize(os.path.join(OUT, 'icon.icns'))} bytes)")
    build_ico()
    print(f"  icon.ico written ({os.path.getsize(os.path.join(OUT, 'icon.ico'))} bytes)")

    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
