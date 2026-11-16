#!/usr/bin/env python3
"""Validate every workspace member against the explicit Rust dependency policy."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def validate_policy(policy):
    # A typo must not turn a protected crate into an unchecked adapter.
    expected = {'format_version', 'packages', 'headless_forbidden',
                'headless_forbidden_prefixes'}
    if not isinstance(policy, dict) or set(policy) != expected:
        raise ValueError('Layer policy: invalid fields')
    if type(policy['format_version']) is not int or policy['format_version'] != 1:
        raise ValueError('Layer policy: unsupported format_version')
    rules = policy['packages']
    if not isinstance(rules, dict) or not rules:
        raise ValueError('Layer policy: packages must be a nonempty object')

    def names(value, label):
        if (not isinstance(value, list) or
                any(not isinstance(name, str) or not name.strip() for name in value) or
                len(set(value)) != len(value)):
            raise ValueError(f'Layer policy: {label} must be a list of unique nonempty names')
        return value

    for name, rule in rules.items():
        if not isinstance(name, str) or not name.strip():
            raise ValueError('Layer policy: invalid package name')
        if not isinstance(rule, dict) or set(rule) != {'kind', 'dependencies'}:
            raise ValueError(f'Layer policy: {name}: invalid fields')
        if rule['kind'] not in ('headless', 'ui', 'desktop', 'bridge', 'tool'):
            raise ValueError(f'Layer policy: {name}: invalid kind')
        dependencies = names(rule['dependencies'], f'{name}.dependencies')
        if name in dependencies or any(dependency not in rules for dependency in dependencies):
            raise ValueError(f'Layer policy: {name}: unknown or self dependency')
    names(policy['headless_forbidden'], 'headless_forbidden')
    names(policy['headless_forbidden_prefixes'], 'headless_forbidden_prefixes')


def production_edges(node):
    # Includes normal, build and conditional edges; test-only edges are not product code.
    return [edge['pkg'] for edge in node['deps']
            if any(kind['kind'] != 'dev' for kind in edge['dep_kinds'])]


def validate(metadata, policy):
    validate_policy(policy)
    packages = {p['id']: p for p in metadata['packages']}
    members = {ident: packages[ident] for ident in metadata['workspace_members']}
    nodes = {n['id']: n for n in metadata['resolve']['nodes']}
    rules = policy['packages']
    names = {p['name'] for p in members.values()}
    if names != set(rules):
        raise ValueError(f'Workspace policy mismatch: unknown={sorted(names-set(rules))}, '
                         f'absent={sorted(set(rules)-names)}')
    for ident, package in members.items():
        rule = rules[package['name']]
        direct = {packages[edge]['name'] for edge in production_edges(nodes[ident]) if edge in members}
        if direct != set(rule['dependencies']):
            raise ValueError(f"{package['name']}: internal dependencies differ: "
                             f"actual={sorted(direct)}, allowed={rule['dependencies']}")
        if rule['kind'] != 'headless':
            continue
        pending = [(ident, [package['name']])]
        visited = set()
        while pending:
            dependency_id, chain = pending.pop()
            if dependency_id in visited:
                continue
            visited.add(dependency_id)
            dependency = packages[dependency_id]
            name = dependency['name']
            forbidden_member = (dependency_id in members and rules[name]['kind'] != 'headless')
            if (forbidden_member or name in policy['headless_forbidden'] or
                    any(name.startswith(prefix) for prefix in policy['headless_forbidden_prefixes'])):
                raise ValueError('Forbidden headless dependency: ' + ' -> '.join(chain))
            pending.extend((edge, chain + [packages[edge]['name']])
                           for edge in production_edges(nodes[dependency_id]))


def validate_manifests(metadata):
    for package in metadata['packages']:
        if package['id'] not in metadata['workspace_members']:
            continue
        path = Path(package['manifest_path'])
        manifest = tomllib.loads(path.read_text())
        for field in ['version', 'edition', 'rust-version', 'license', 'repository']:
            if manifest.get('package', {}).get(field) != {'workspace': True}:
                raise ValueError(f'{path}: package.{field} must inherit workspace metadata')
        if manifest.get('lints', {}).get('workspace') is not True:
            raise ValueError(f'{path}: workspace lints must be inherited')
        tables = [manifest, *manifest.get('target', {}).values()]
        for table in tables:
            for group in ['dependencies', 'dev-dependencies', 'build-dependencies']:
                for name, dependency in table.get(group, {}).items():
                    if not isinstance(dependency, dict) or dependency.get('workspace') is not True:
                        raise ValueError(f'{path}: {group}.{name} must inherit workspace dependencies')


def read_metadata(root):
    # Disabled optional features are still product code. Without --all-features,
    # an optional GUI dependency can disappear from the graph and evade isolation.
    return json.loads(subprocess.check_output(
        [os.environ.get('CARGO', 'cargo'), 'metadata', '--format-version', '1',
         '--all-features', '--locked', '--offline'], cwd=root))


def main():
    metadata = read_metadata(ROOT)
    policy = json.loads((ROOT / 'docs/rust/layers.json').read_text())
    validate(metadata, policy)
    validate_manifests(metadata)
    print(f"OK — {len(metadata['workspace_members'])} workspace packages: explicit production edges, "
          'headless isolation and inherited manifests checked (all features, target/build edges).')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
