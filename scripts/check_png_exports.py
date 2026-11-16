#!/usr/bin/env python3
"""Independently decode an RGBA PNG and check its frame, alpha margin and export metadata."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zlib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('image', type=Path)
parser.add_argument('--metadata', type=Path, required=True)
args = parser.parse_args()
data = args.image.read_bytes()
assert data[:8] == b'\x89PNG\r\n\x1a\n' and len(data) <= 64 * 1024 * 1024
position = 8
compressed = bytearray()
header = None
ended = False
while position < len(data):
    length = struct.unpack_from('>I', data, position)[0]
    kind = data[position + 4:position + 8]
    payload = data[position + 8:position + 8 + length]
    assert len(payload) == length
    crc = struct.unpack_from('>I', data, position + 8 + length)[0]
    assert zlib.crc32(kind + payload) == crc
    if kind == b'IHDR':
        assert header is None
        header = struct.unpack('>IIBBBBB', payload)
    elif kind == b'IDAT':
        compressed.extend(payload)
    elif kind == b'IEND':
        ended = True
        assert position + 12 == len(data)
    position += 12 + length
assert ended and header is not None
width, height, depth, colors, compression, filtering, interlace = header
assert 1 <= width <= 4096 and 1 <= height <= 4096
assert (depth, colors, compression, filtering, interlace) == (8, 6, 0, 0, 0)
expected_size = height * (width * 4 + 1)
decoder = zlib.decompressobj()
raw = decoder.decompress(compressed, expected_size + 1)
assert decoder.eof and not decoder.unconsumed_tail and len(raw) == expected_size
stride = width * 4
previous = bytearray(stride)
visible = 0


def paeth(left, above, corner):
    predictor = left + above - corner
    distances = [abs(predictor - value) for value in (left, above, corner)]
    return (left, above, corner)[distances.index(min(distances))]


for y in range(height):
    start = y * (stride + 1)
    method = raw[start]
    assert method in range(5)
    row = bytearray(raw[start + 1:start + 1 + stride])
    for offset in range(stride):
        left = row[offset - 4] if offset >= 4 else 0
        above = previous[offset]
        corner = previous[offset - 4] if offset >= 4 else 0
        predictor = (0, left, above, (left + above) // 2, paeth(left, above, corner))[method]
        row[offset] = (row[offset] + predictor) % 256
    for x in range(width):
        alpha = row[x * 4 + 3]
        visible += alpha > 0
        if x < 16 or y < 16 or x >= width - 16 or y >= height - 16:
            assert alpha == 0, (x, y, alpha)
    previous = row
assert visible > 0
metadata = json.loads(args.metadata.read_text())
assert (metadata['width'], metadata['height']) == (width, height)
assert metadata['sha256'] == hashlib.sha256(data).hexdigest()
identities = metadata['object_ids']
assert len(identities) == len(set(identities)) == sum(metadata[k] for k in ['devices', 'links', 'annotations'])
print(f'OK — {width} × {height} RGBA, CRC/deflate/hash, 16-pixel transparent margin, {len(identities)} objects; {visible} visible pixels.')
