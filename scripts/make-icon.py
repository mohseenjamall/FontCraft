#!/usr/bin/env python3
"""Writes assets/icon.ico (16, 32, 48, 64 and 256 px, PNG-compressed) from assets/app-art.png.

The 256 px entry is reduced to 256 colours: it keeps the exe about 0.1 MiB smaller than a
full-colour one and looks the same on this flat artwork. Run it again if the artwork changes.
"""
import io
import struct
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SIZES = [16, 32, 48, 64, 256]


def png_bytes(img: Image.Image, size: int) -> bytes:
    img = img.resize((size, size), Image.LANCZOS)
    if size == 256:
        img = img.convert("RGB").quantize(256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    out = io.BytesIO()
    img.save(out, "PNG", optimize=True)
    return out.getvalue()


def main() -> None:
    art = Image.open(ROOT / "assets" / "app-art.png").convert("RGBA")
    images = [(s, png_bytes(art, s)) for s in SIZES]
    header = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    entries, data = b"", b""
    for size, png in images:
        dim = 0 if size >= 256 else size  # 0 means 256 in an ICO directory entry
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(png), offset + len(data))
        data += png
    dst = ROOT / "assets" / "icon.ico"
    dst.write_bytes(header + entries + data)
    print(f"Wrote {dst} ({dst.stat().st_size} bytes) with sizes: {', '.join(map(str, SIZES))}")


if __name__ == "__main__":
    main()
