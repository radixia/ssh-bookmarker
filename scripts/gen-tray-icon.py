#!/usr/bin/env python3
"""Render a minimal monochrome '>_' template icon for the macOS menu bar.

Outputs `src-tauri/icons/tray-icon.png` — 44x44 8-bit grayscale+alpha PNG.
The image is a template (alpha-only matters); macOS tints it for light/dark
menu bars. Built with stdlib only (zlib + struct), so no PIL needed.
"""
import os
import struct
import zlib

W = H = 44
pixels = bytearray(2 * W * H)  # (gray, alpha) per pixel; zero-init = transparent


def set_pixel(x: int, y: int) -> None:
    if 0 <= x < W and 0 <= y < H:
        i = (y * W + x) * 2
        pixels[i] = 255       # gray (ignored when used as a template image)
        pixels[i + 1] = 255   # alpha


def line(x0: int, y0: int, x1: int, y1: int, thickness: int) -> None:
    """Bresenham line with a square brush of `thickness` pixels."""
    dx = abs(x1 - x0)
    dy = abs(y1 - y0)
    sx = 1 if x0 < x1 else -1
    sy = 1 if y0 < y1 else -1
    err = dx - dy
    x, y = x0, y0
    half = thickness // 2
    extra = thickness & 1
    while True:
        for ox in range(-half, half + extra):
            for oy in range(-half, half + extra):
                set_pixel(x + ox, y + oy)
        if x == x1 and y == y1:
            break
        e2 = 2 * err
        if e2 > -dy:
            err -= dy
            x += sx
        if e2 < dx:
            err += dx
            y += sy


# Chevron '>' on the left half
line(10, 12, 22, 22, 4)
line(22, 22, 10, 32, 4)

# Underscore '_' on the right
line(26, 32, 38, 32, 4)


def write_png(path: str) -> None:
    sig = bytes([137, 80, 78, 71, 13, 10, 26, 10])

    def chunk(typ: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(typ + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + typ + data + struct.pack(">I", crc)

    # 8-bit grayscale + alpha (color type 4)
    ihdr = struct.pack(">IIBBBBB", W, H, 8, 4, 0, 0, 0)

    raw = bytearray()
    stride = 2 * W
    for y in range(H):
        raw.append(0)  # no row filter
        raw.extend(pixels[y * stride : (y + 1) * stride])
    idat = zlib.compress(bytes(raw), 9)

    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(sig)
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"IDAT", idat))
        f.write(chunk(b"IEND", b""))


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    out = os.path.join(here, "..", "src-tauri", "icons", "tray-icon.png")
    write_png(out)
    print(f"wrote {os.path.normpath(out)}")
