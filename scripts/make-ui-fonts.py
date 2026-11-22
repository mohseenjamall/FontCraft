"""Build the three UI fonts (OFL, family names kept) embedded by the app.
Run from the repo root: python scripts/make-ui-fonts.py"""
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
from fontTools import subset

SITE = "site/node_modules"
OUT = "assets/fonts"
# Basic Latin, Latin-1, general punctuation, arrows, multiplication sign, ellipsis.
UNICODES = list(range(0x20, 0x7F)) + list(range(0xA0, 0x100)) + list(range(0x2000, 0x2070)) + list(range(0x2190, 0x21A0)) + [0xD7, 0x2026]


def save_subset(font, path):
    opts = subset.Options()
    opts.layout_features = ["*"]
    opts.flavor = None
    opts.notdef_outline = True
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=UNICODES)
    sub.subset(font)
    font.flavor = None
    font.save(path)


def rename(f, family, style):
    for n in f["name"].names:
        if n.nameID in (16, 17, 21, 22, 25):
            continue
        if n.nameID == 1: n.string = family if style == "Regular" else f"{family} {style}"
        elif n.nameID == 2: n.string = "Regular"
        elif n.nameID == 4: n.string = f"{family} {style}"
        elif n.nameID == 6: n.string = f"{family}-{style}".replace(" ", "")
    f["name"].names = [n for n in f["name"].names if n.nameID not in (16, 17, 25)]
    for pid, eid, lid in ((3, 1, 0x409), (1, 0, 0)):
        f["name"].setName(family, 16, pid, eid, lid)
        f["name"].setName(style, 17, pid, eid, lid)
    f["OS/2"].usWeightClass = 400 if style == "Regular" else 500


def archivo(weight, name):
    f = TTFont(f"{SITE}/@fontsource-variable/archivo/files/archivo-latin-wght-normal.woff2")
    f = instancer.instantiateVariableFont(f, {"wght": weight, "wdth": 100} if "wdth" in [a.axisTag for a in f["fvar"].axes] else {"wght": weight})
    rename(f, "Archivo", "Regular" if weight == 400 else "Medium")
    save_subset(f, f"{OUT}/{name}")


archivo(400, "Archivo-Regular.ttf")
archivo(500, "Archivo-Medium.ttf")
save_subset(TTFont(f"{SITE}/@fontsource/ibm-plex-mono/files/ibm-plex-mono-latin-400-normal.woff2"), f"{OUT}/PlexMono-Regular.ttf")
