"""Architecture regressions, including routes that escaped the old root allowlist."""
import unittest
import json
import os
from pathlib import Path
import subprocess
import tempfile
import check_layers
from unittest.mock import patch


def fixture():
    def node(name, edges):
        return dict(id=name, deps=[dict(pkg=target, dep_kinds=[dict(kind=kind, target=condition)])
                                  for target, kind, condition in edges])
    metadata = dict(packages=[dict(id=name, name=name) for name in ['model', 'settings', 'ui', 'egui']],
                    workspace_members=['model', 'settings', 'ui'],
                    resolve=dict(nodes=[node('model', []), node('settings', [('model', None, None)]),
                                        node('ui', [('egui', None, None)]), node('egui', [])]))
    policy = dict(format_version=1, packages=dict(model=dict(kind='headless', dependencies=[]),
                               settings=dict(kind='headless', dependencies=['model']),
                               ui=dict(kind='ui', dependencies=[])),
                  headless_forbidden=['egui'], headless_forbidden_prefixes=['craft-'])
    return metadata, policy


def member_manifest():
    return dict(package={field: dict(workspace=True) for field in
                         ['version', 'edition', 'rust-version', 'license', 'repository']},
                lints=dict(workspace=True),
                dependencies=dict(model=dict(workspace=True, features=['serde'])),
                target={'cfg(windows)': {'build-dependencies': dict(helper=dict(workspace=True))}})


class Layers(unittest.TestCase):
    def validate_manifest(self, manifest):
        metadata = dict(workspace_members=['model'], packages=[
            dict(id='model', manifest_path='Cargo.toml'),
            dict(id='external', manifest_path='/not-a-workspace-member/Cargo.toml')])
        with patch.object(check_layers.tomllib, 'loads', return_value=manifest), \
                patch.object(Path, 'read_text', return_value=''):
            check_layers.validate_manifests(metadata)

    def test_inherited_metadata_lints_dependencies_and_member_features_are_allowed(self):
        self.validate_manifest(member_manifest())

    def test_missing_or_local_package_metadata_is_rejected(self):
        for field in ['version', 'edition', 'rust-version', 'license', 'repository']:
            for value in [None, 'local value', dict(workspace=False)]:
                with self.subTest(field=field, value=value):
                    manifest = member_manifest()
                    manifest['package'][field] = value
                    with self.assertRaisesRegex(ValueError, f'package.{field} must inherit'):
                        self.validate_manifest(manifest)

    def test_workspace_lints_cannot_be_omitted_or_replaced(self):
        for value in [{}, dict(workspace=False), dict(rust=dict(unsafe_code='allow'))]:
            with self.subTest(value=value):
                manifest = member_manifest()
                manifest['lints'] = value
                with self.assertRaisesRegex(ValueError, 'workspace lints must be inherited'):
                    self.validate_manifest(manifest)

    def test_all_dependency_tables_require_workspace_inheritance(self):
        for group in ['dependencies', 'dev-dependencies', 'build-dependencies']:
            for conditional in [False, True]:
                for dependency in ['1', dict(path='../local'), dict(workspace=False)]:
                    with self.subTest(group=group, conditional=conditional, dependency=dependency):
                        manifest = member_manifest()
                        table = manifest['target']['cfg(windows)'] if conditional else manifest
                        table[group] = dict(bypass=dependency)
                        with self.assertRaisesRegex(ValueError, f'{group}.bypass must inherit'):
                            self.validate_manifest(manifest)

    def test_optional_gui_dependency_is_checked_even_when_disabled_by_default(self):
        # Real Cargo resolution: the old metadata command omitted this production edge.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'Cargo.toml').write_text(
                '[workspace]\nmembers = ["model"]\nexclude = ["egui"]\nresolver = "3"\n')
            for name in ['model', 'egui']:
                (root / name / 'src').mkdir(parents=True)
                (root / name / 'src/lib.rs').write_text('')
                manifest = f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n'
                if name == 'model':
                    manifest += '[dependencies]\negui = { path = "../egui", optional = true }\n'
                (root / name / 'Cargo.toml').write_text(manifest)
            metadata = json.loads(subprocess.check_output(
                [os.environ.get('CARGO', 'cargo'), 'metadata', '--format-version', '1', '--offline'],
                cwd=root, stderr=subprocess.PIPE))
            policy = dict(format_version=1, packages=dict(model=dict(kind='headless', dependencies=[])),
                          headless_forbidden=['egui'], headless_forbidden_prefixes=[])
            check_layers.validate(metadata, policy)  # Prove the default-feature blind spot.
            with self.assertRaisesRegex(ValueError, 'model -> egui'):
                check_layers.validate(check_layers.read_metadata(root), policy)

    def test_misspelled_kind_cannot_disable_headless_isolation(self):
        metadata, policy = fixture()
        policy['packages']['settings']['kind'] = 'headles'
        metadata['resolve']['nodes'][1]['deps'].append(
            dict(pkg='egui', dep_kinds=[dict(kind=None, target=None)]))
        with self.assertRaisesRegex(ValueError, 'settings.*kind'):
            check_layers.validate(metadata, policy)

    def test_invalid_policy_cannot_silently_weaken_checks(self):
        for change in [
                lambda p: p.update(format_version=2),
                lambda p: p.update(headless_forbiden=['egui']),
                lambda p: p['packages']['model'].update(dependences=[]),
                lambda p: p['packages']['settings'].update(dependencies='model'),
                lambda p: p['packages']['settings'].update(dependencies=['model', 'model']),
                lambda p: p['packages']['settings'].update(dependencies=['missing']),
                lambda p: p.update(headless_forbidden_prefixes=['']),
                lambda p: p.update(headless_forbidden='egui')]:
            with self.subTest(change=change):
                metadata, policy = fixture()
                change(policy)
                with self.assertRaises(ValueError):
                    check_layers.validate(metadata, policy)

    def test_current_shape_and_test_only_reverse_edge_are_allowed(self):
        metadata, policy = fixture()
        metadata['resolve']['nodes'][0]['deps'].append(dict(pkg='ui', dep_kinds=[dict(kind='dev', target=None)]))
        check_layers.validate(metadata, policy)

    def test_new_member_cannot_escape_checks(self):
        metadata, policy = fixture()
        metadata['packages'].append(dict(id='new', name='new'))
        metadata['workspace_members'].append('new')
        with self.assertRaisesRegex(ValueError, 'unknown=.*new'):
            check_layers.validate(metadata, policy)

    def test_undeclared_reverse_dependency_is_rejected(self):
        metadata, policy = fixture()
        metadata['resolve']['nodes'][0]['deps'].append(dict(pkg='settings', dep_kinds=[dict(kind=None, target=None)]))
        with self.assertRaisesRegex(ValueError, 'internal dependencies differ'):
            check_layers.validate(metadata, policy)

    def test_transitive_gui_in_build_target_or_mixed_dev_edge_is_rejected(self):
        for kinds in [[dict(kind='build', target='cfg(windows)')],
                      [dict(kind=None, target='cfg(unix)')],
                      [dict(kind='dev', target=None), dict(kind=None, target=None)]]:
            with self.subTest(kinds=kinds):
                metadata, policy = fixture()
                metadata['resolve']['nodes'][0]['deps'].append(dict(pkg='egui', dep_kinds=kinds))
                with self.assertRaisesRegex(ValueError, 'model -> egui'):
                    check_layers.validate(metadata, policy)

    def test_explicit_headless_to_adapter_edge_is_still_rejected(self):
        metadata, policy = fixture()
        policy['packages']['model']['dependencies'] = ['ui']
        metadata['resolve']['nodes'][0]['deps'].append(dict(pkg='ui', dep_kinds=[dict(kind=None, target=None)]))
        with self.assertRaisesRegex(ValueError, 'model -> ui'):
            check_layers.validate(metadata, policy)


if __name__ == '__main__':
    unittest.main()
