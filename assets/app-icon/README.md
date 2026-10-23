# Tinta Texto app icon

<img src="wordcraft.svg" alt="Tinta Texto app icon: an ivory azulejo tile with blue corners and a page of text lines" width="128">

**Design:** the Tinta Office "azulejo" tile. An ivory rounded square with a quarter circle in each
corner, like a glazed wall tile, and the app's glyph in the middle: for Tinta Texto, a page of text lines. The four
Tinta Office apps (Tinta PDF, Tinta Texto, Tinta Calculo, Tinta Presenta) share the tile and differ by
colour and glyph.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Ivory | `#F3EDE1` | The tile |
| Edge | `#D8CBB2` | The tile outline |
| Tinta Texto (app colour) | `#2D4BA0` | The corners and the glyph |

## Geometry

A 512 × 512 viewBox, a rounded square with `rx=112`, full bleed. Windows and Linux use the full
tile. The macOS icons (`.icns`, `-macos-512.png`) sit on Apple's grid: an 824/1024 body with a
transparent margin. At 32 px and below a simplified tile is drawn (smaller corners, bolder glyph)
so the icon stays legible at 16 px.

## Files

| File | Use |
|---|---|
| `wordcraft-1024.png` | 1024 px render of the full tile |
| `wordcraft-macos-512.png` | Runtime Dock icon on macOS (Apple's grid) |
| `wordcraft.icns` | macOS bundle icon (Apple's grid) |
| `wordcraft.ico` | Windows exe icon, 16–256 px (embedded by `apps/wordcraft/build.rs`) |
| `wordcraft.svg` | Master vector, the full-bleed tile; also `hicolor/scalable` |
| `hicolor/<size>/apps/ai.storyteller.wordcraft.png` | Linux icon theme, 16–512 px; the 256 px one is also the runtime window icon on Windows and Linux |
| `LICENSE.txt` | MIT OR Apache-2.0 |

The app ID is `ai.storyteller.wordcraft`. It's used for the hicolor icon names, the Wayland app ID and
`packaging/linux/ai.storyteller.wordcraft.desktop`.

## Provenance and regenerating

Original artwork for Tinta Office, drawn in code by
[`packaging/tinta-icons.py`](../../packaging/tinta-icons.py); no third-party material.
Run `packaging/icons.sh` (python3 with cairosvg and Pillow) to rewrite every file here.
