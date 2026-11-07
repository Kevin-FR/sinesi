"""Regression tests for the tracking gates, without modifying the working tree."""
import copy
import unittest
from unittest.mock import patch
import docs


class VerificationGates(unittest.TestCase):
    def run_with_features(self, features):
        original = docs.read
        def read(path):
            return features if path == 'docs/suivi/features.json' else original(path)
        with patch.object(docs, 'read', side_effect=read):
            docs.check()

    def test_verified_requires_a_reproducible_proof(self):
        data = copy.deepcopy(docs.read('docs/suivi/features.json'))
        data['features'][0]['status'] = 'verified'
        data['features'][0]['verification'] = []
        with self.assertRaisesRegex(AssertionError, 'preuve de recette manquante'):
            self.run_with_features(data)

    def test_verified_requires_verified_dependencies(self):
        data = copy.deepcopy(docs.read('docs/suivi/features.json'))
        by_id = {feature['id']: feature for feature in data['features']}
        by_id['F-SIM-005']['status'] = 'in_progress'
        by_id['F-PROJ-004']['status'] = 'verified'
        by_id['F-PROJ-004']['verification'] = [{
            'recipe': 'negative-test', 'command': 'negative-test', 'result': 'pass',
            'environment': 'negative-test', 'revision': 'negative-test', 'artifacts': ['negative-test']}]
        with self.assertRaisesRegex(AssertionError, 'prérequis non vérifié'):
            self.run_with_features(data)


if __name__ == '__main__':
    unittest.main()
