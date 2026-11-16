#!/usr/bin/env python3
"""Seal or verify the exact source inventory used by a milestone's evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[1]


def inventory():
    paths = {ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'AGENTS.md']}
    for directory in ['crates', 'apps', 'xtask', '.cargo', 'contracts', 'fixtures', 'scripts', '.github', 'assets', 'packaging']:
        paths.update(p for p in (ROOT / directory).rglob('*')
                     if p.is_file() and (p.suffix in ['.rs', '.toml', '.json', '.jsonl', '.png', '.py', '.yml', '.svg', '.xml', '.fls', '.ttf', '.otf', '.txt', '.swift', '.desktop', '.iss', '.ps1', '.sh']
                                         or (directory == 'assets' and p.suffix == '.md') or p.name == 'AGENTS.md'))
    paths.add(ROOT / 'docs/rust/layers.json')
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(paths) if '.new.' not in p.name and '.diff.' not in p.name}


def revision(files):
    return hashlib.sha256(json.dumps(files, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['write', 'check'])
    parser.add_argument('manifest', type=Path)
    args = parser.parse_args()
    files = inventory()
    if args.action == 'write':
        args.manifest.write_text(json.dumps(dict(format_version=1,
            base_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            revision=revision(files), files=files), indent=2) + '\n')
    else:
        manifest = json.loads(args.manifest.read_text())
        if manifest['files'] != files or manifest['revision'] != revision(files):
            raise SystemExit('Source inventory differs from the milestone manifest.')
    print(f'OK — {len(files)} source files, revision {revision(files)}')


if __name__ == '__main__':
    main()
