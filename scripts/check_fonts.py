#!/usr/bin/env python3
"""Verify the offline document font inventory, licensing, weights and Latin coverage."""
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
manifest = json.loads((ROOT / 'assets/fonts/provenance.json').read_text())
assert manifest['license'] == 'OFL-1.1' and manifest['modified'] is False
sample = 'École française — Grüße Straße ÄÖÜ ¡Educación! perché città ação português œŒæÆßéèêëáàâãçñíóúòùï0123456789'


def u16(data, offset):
    return struct.unpack_from('>H', data, offset)[0]


def u32(data, offset):
    return struct.unpack_from('>I', data, offset)[0]


def has_glyph(data, cmap, code):
    for index in range(u16(data, cmap + 2)):
        table = cmap + u32(data, cmap + 8 + index * 8)
        if u16(data, table) != 4:
            continue
        count = u16(data, table + 6) // 2
        end = table + 14
        start = end + count * 2 + 2
        delta = start + count * 2
        offsets = delta + count * 2
        for segment in range(count):
            lo, hi = u16(data, start + segment * 2), u16(data, end + segment * 2)
            if not lo <= code <= hi:
                continue
            address = offsets + segment * 2
            distance = u16(data, address)
            glyph = u16(data, address + distance + (code - lo) * 2) if distance else code
            return bool(glyph and (glyph + u16(data, delta + segment * 2)) % 65536)
    return False


fonts = 0
for entry in manifest['files']:
    path = ROOT / entry['path']
    data = path.read_bytes()
    assert len(data) == entry['bytes']
    assert hashlib.sha256(data).hexdigest() == entry['sha256'], path
    assert manifest['revision'] in entry['source'], path
    if path.suffix != '.ttf':
        assert b'SIL OPEN FONT LICENSE Version 1.1' in data
        continue
    assert data[:4] == b'\x00\x01\x00\x00', path
    tables = {}
    for index in range(u16(data, 4)):
        offset = 12 + index * 16
        tables[data[offset:offset + 4]] = u32(data, offset + 8)
    weight = u16(data, tables[b'OS/2'] + 4)
    assert weight == (700 if 'Bold' in path.stem else 400), (path, weight)
    for character in set(sample):
        assert has_glyph(data, tables[b'cmap'], ord(character)), (path, character)
    fonts += 1
assert fonts == 4
print('OK — four pinned OFL fonts, regular/bold weights and six-language Latin coverage; no download required.')
