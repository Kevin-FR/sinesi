#!/usr/bin/env python3
"""Assert that headless production dependencies remain independent of the UI and sockets."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(
    ['cargo', 'metadata', '--format-version', '1', '--locked', '--offline'], cwd=ROOT))
packages = {package['id']: package for package in metadata['packages']}
nodes = {node['id']: node for node in metadata['resolve']['nodes']}
forbidden = {'egui', 'eframe', 'wgpu', 'winit', 'rfd', 'arboard', 'tokio', 'reqwest', 'ureq'}
roots = {'sns-model', 'sns-engine', 'sns-project', 'sns-html', 'sns-cli'}
for package in packages.values():
    if package['name'] not in roots:
        continue
    pending, visited = [package['id']], set()
    while pending:
        ident = pending.pop()
        if ident in visited:
            continue
        visited.add(ident)
        dependency = packages[ident]
        assert dependency['name'] not in forbidden, (package['name'], dependency['name'])
        assert not dependency['name'].startswith('craft-'), (package['name'], dependency['name'])
        for edge in nodes[ident]['deps']:
            # Test-only dependencies may deliberately exercise the other adapter.
            if any(kind['kind'] != 'dev' for kind in edge['dep_kinds']):
                pending.append(edge['pkg'])
print('OK — production headless graph has no GUI, Craft or networking runtime dependencies.')
