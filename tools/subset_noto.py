#!/usr/bin/env python3
"""Build the GB2312 subset of Noto Sans SC Regular shipped with the shell.

Usage:
    python3 tools/subset_noto.py NotoSansSC-Regular.otf crates/shell/assets/fonts/NotoSansSC-Regular.subset.otf

Source font: notofonts/noto-cjk tag Sans2.004,
Sans/SubsetOTF/SC/NotoSansSC-Regular.otf (OFL 1.1, copyright Adobe, reserved name Source).
"""

import sys

from fontTools.subset import Options, Subsetter
from fontTools.ttLib import TTFont


def codepoints() -> set[int]:
    chars = set(range(0x20, 0x7F))
    chars.update(range(0xA0, 0x0250))
    chars.update(range(0x2000, 0x2070))
    chars.update(range(0x2070, 0x20A0))
    chars.update(range(0x20A0, 0x20D0))
    chars.update(range(0x2100, 0x2150))
    chars.update(range(0x2190, 0x2200))
    chars.update(range(0x2200, 0x2300))
    chars.update(range(0x2300, 0x2400))
    chars.update(range(0x2460, 0x2500))
    chars.update(range(0x2500, 0x25A0))
    chars.update(range(0x25A0, 0x2600))
    chars.update(range(0x2600, 0x2700))
    chars.update(range(0x2700, 0x27C0))
    chars.update(range(0x2E80, 0x2F00))
    chars.update(range(0x3000, 0x3040))
    chars.update(range(0xFF00, 0xFFF0))
    chars.update(range(0xFE10, 0xFE70))
    chars.update(range(0xFE30, 0xFE50))
    for lead in range(0xA1, 0xF8):
        for trail in range(0xA1, 0xFF):
            try:
                text = bytes([lead, trail]).decode("gb2312")
            except UnicodeDecodeError:
                continue
            chars.add(ord(text))
    return chars


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: subset_noto.py SOURCE.otf OUTPUT.otf")
    options = Options()
    options.layout_features = ["*"]
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    options.notdef_outline = True
    options.recommended_glyphs = True
    options.glyph_names = True
    options.hinting = False
    font = TTFont(sys.argv[1])
    subsetter = Subsetter(options=options)
    subsetter.populate(unicodes=codepoints())
    subsetter.subset(font)
    font.save(sys.argv[2])


if __name__ == "__main__":
    main()
