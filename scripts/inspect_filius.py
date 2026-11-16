#!/usr/bin/env python3
"""Bounded, read-only .fls inventory. Never extracts entries or executes XML/Java."""
import argparse
import collections
import json
from pathlib import Path, PurePosixPath
import stat
import xml.etree.ElementTree as ET
import zipfile
ROOT = Path(__file__).resolve().parents[1]
KNOWN = set(json.loads((ROOT / 'contracts/filius-known-classes.json').read_text())['classes'])
MAX_ENTRY = 8 * 1024 * 1024
MAX_TOTAL = 32 * 1024 * 1024


def inspect(path):
    if path.stat().st_size > MAX_TOTAL:
        raise ValueError('resource.limit_exceeded: compressed archive')
    with zipfile.ZipFile(path) as archive:
        entries = archive.infolist()
        if len(entries) > 1024 or sum(e.file_size for e in entries) > MAX_TOTAL:
            raise ValueError('resource.limit_exceeded: archive')
        names = set()
        for entry in entries:
            name = entry.filename
            if (name in names or name.startswith('/') or '\\' in name or ':' in name
                    or '..' in PurePosixPath(name).parts
                    or stat.S_ISLNK(entry.external_attr >> 16)):
                raise ValueError('project.unsafe_archive_path: ' + name)
            names.add(name)
            if entry.file_size > MAX_ENTRY or entry.file_size > max(1, entry.compress_size) * 1000:
                raise ValueError('resource.limit_exceeded: entry')
        configs = [e for e in entries if e.filename.endswith('/konfiguration.xml')]
        if len(configs) != 1:
            raise ValueError('project.invalid_configuration_count')
        with archive.open(configs[0]) as stream:
            data = stream.read(MAX_ENTRY + 1)
        if len(data) > MAX_ENTRY:
            raise ValueError('resource.limit_exceeded: xml')
        text = data.decode('utf-8')
        if '<!DOCTYPE' in text.upper() or '<!ENTITY' in text.upper():
            raise ValueError('project.unsafe_xml_declaration')
        root = ET.fromstring(text)
        todo, classes, count = [(root, 0)], collections.Counter(), 0
        while todo:
            node, depth = todo.pop()
            count += 1
            if count > 200000 or depth > 256:
                raise ValueError('resource.limit_exceeded: xml structure')
            if node.get('class'):
                classes[node.get('class')] += 1
            todo.extend((child, depth + 1) for child in node)
        return dict(path=path.as_posix(), configuration=configs[0].filename,
                    entries=sorted(names), classes=dict(sorted(classes.items())),
                    unknown_classes=sorted(set(classes) - KNOWN),
                    executable_payloads=sorted(n for n in names if n.endswith(('.java', '.class', '.jar'))),
                    imported=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('paths', nargs='+', type=Path)
    args = parser.parse_args()
    print(json.dumps({'inspection_version': 1, 'examples': [inspect(p) for p in args.paths]},
                     ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
