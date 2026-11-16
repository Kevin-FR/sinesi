#!/usr/bin/env python3
"""Inspect a generated report with an independent PDF reader (pypdf 6.10.0)."""
import argparse
import hashlib
import json
from pathlib import Path
from pypdf import PdfReader


def check(path, metadata):
    raw = path.read_bytes()
    expected = json.loads(metadata.read_text())
    reader = PdfReader(path, strict=True)
    if reader.is_encrypted or len(reader.pages) != expected['pages']:
        raise ValueError('Unexpected encryption or page count')
    if hashlib.sha256(raw).hexdigest() != expected['sha256']:
        raise ValueError('PDF bytes differ from report metadata')
    if reader.trailer['/Root'].get('/OpenAction') or reader.trailer['/Root'].get('/AA'):
        raise ValueError('Report must not contain automatic actions')
    if reader.trailer['/Root'].get('/AcroForm'):
        raise ValueError('Unexpected form in static report')
    text = '\n'.join(page.extract_text() for page in reader.pages)
    for value in expected['required_text']:
        if value not in text:
            raise ValueError(f'Missing report content: {value}')
    for value in expected.get('forbidden_text', []):
        if value in text:
            raise ValueError(f'Unexpected sensitive content: {value}')
    if '{revision}' in text or '\ufffd' in text:
        raise ValueError('Unexpanded template or replacement character')
    rows = sum(text.count(kind + ' · ') for kind in ['ethernet.sent', 'ethernet.received'])
    if rows != expected['packet_rows']:
        raise ValueError(f'Packet rows differ: {rows} / {expected["packet_rows"]}')
    fonts = {}
    for index, page in enumerate(reader.pages, 1):
        if f'{index} / {len(reader.pages)}' not in page.extract_text():
            raise ValueError(f'Missing page number: {index}')
        if abs(float(page.mediabox.width) - 595.28) > .02 or abs(float(page.mediabox.height) - 841.89) > .02:
            raise ValueError('Expected A4 pages')
        for key, value in page['/Resources'].get('/Font', {}).items():
            font = value.get_object()
            fonts[str(font.get('/BaseFont'))] = font
    if not fonts or not all('/ToUnicode' in font for font in fonts.values()):
        raise ValueError('Expected selectable text with Unicode mapping')
    for font in fonts.values():
        descendant = font['/DescendantFonts'][0].get_object()
        descriptor = descendant['/FontDescriptor'].get_object()
        if not any(key in descriptor for key in ['/FontFile', '/FontFile2', '/FontFile3']):
            raise ValueError('Expected embedded fonts')
    print(f'OK — {path.name}: {len(reader.pages)} A4 pages, {rows} packet rows, '
          f'{len(fonts)} embedded Unicode fonts, required sections and SHA-256 checked.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pdf', type=Path)
    parser.add_argument('--metadata', type=Path, required=True)
    args = parser.parse_args()
    check(args.pdf, args.metadata)
