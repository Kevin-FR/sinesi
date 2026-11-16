#!/usr/bin/env python3
"""Check embedded SVG provenance and the selected teaching asset inventory."""
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET
ROOT = Path(__file__).resolve().parents[1]
manifest = json.loads((ROOT / 'assets/network/provenance.json').read_text())
assert len(manifest['files']) == 20
for name, entry in manifest['files'].items():
    path = ROOT / name
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == entry['sha256'], name
    svg = ET.fromstring(data)
    assert svg.tag == '{http://www.w3.org/2000/svg}svg' and svg.attrib['viewBox'] == '0 0 64 64', name
    source = ROOT / entry['source']
    if source.exists():
        assert source.read_bytes() == data, name
    for element in svg.iter():
        assert element.tag.split('}')[-1] not in {'script', 'foreignObject', 'image', 'use'}, name
        assert not any(key.split('}')[-1] == 'href' for key in element.attrib), name
print(f'OK — {len(manifest["files"])} embedded SVG assets, provenance and offline rendering checked.')
