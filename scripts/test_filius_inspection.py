#!/usr/bin/env python3
import tempfile
from pathlib import Path
import unittest
import zipfile
from inspect_filius import inspect

class Inspection(unittest.TestCase):
    def archive(self, entries):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / 'example.fls'
        with zipfile.ZipFile(path, 'w') as archive:
            for name, data in entries.items():
                archive.writestr(name, data)
        return path

    def test_unknown_classes_and_code_are_reported_without_execution(self):
        path = self.archive({'projekt/konfiguration.xml': '<java class="java.beans.XMLDecoder"><object class="java.lang.Runtime"><void method="exec"><string>bad</string></void></object></java>', 'projekt/evil.class': b'ignored'})
        report = inspect(path)
        self.assertEqual(report['unknown_classes'], ['java.lang.Runtime'])
        self.assertEqual(report['executable_payloads'], ['projekt/evil.class'])
        self.assertFalse(report['imported'])

    def test_paths_entities_and_limits_are_rejected(self):
        for entries in [
            {'../escape': 'x', 'projekt/konfiguration.xml': '<java/>'},
            {'projekt/konfiguration.xml': '<!DOCTYPE a [<!ENTITY x "x">]><a>&x;</a>'},
            {'projekt/konfiguration.xml': '<a>' * 258 + '</a>' * 258},
        ]:
            with self.assertRaises(ValueError):
                inspect(self.archive(entries))

if __name__ == '__main__':
    unittest.main()
