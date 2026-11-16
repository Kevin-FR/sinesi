#!/usr/bin/env python3
"""Catch missing literal UI/diagnostic keys and unexpanded translation templates."""
import json
import re
from pathlib import Path
root = Path(__file__).resolve().parents[1]
messages = json.loads((root / 'crates/sns-i18n/locales/messages.json').read_text())
langs = {'fr', 'en', 'es', 'it', 'pt', 'de'}
for key, entries in messages.items():
    assert set(entries) == langs, key
    parameters = [set(re.findall(r'\{(\w+)\}', text)) for text in entries.values()]
    assert all(p == parameters[0] for p in parameters), key
for crate in ['sns-desktop', 'sns-settings', 'sns-report', 'sns-render', 'sns-trace', 'sns-filius-import', 'sns-model', 'sns-apps', 'sns-vfs', 'sns-protocols', 'sns-engine', 'sns-project', 'sns-topology', 'sns-ui-egui']:
    for path in (root / ('apps' if crate == 'sns-desktop' else 'crates') / crate / 'src').rglob('*.rs'):
        text = path.read_text()
        patterns = [r'(?:Diagnostic::new|diag|error)\(\s*"([\w.]+)"']
        if crate in ['sns-desktop', 'sns-ui-egui', 'sns-apps', 'sns-report']:
            patterns += [r'(?:\.text|\.format|message)\(\s*"([\w.]+)"']
        for pattern in patterns:
            for key in re.findall(pattern, text):
                assert key in messages, (path, key)
        for key in re.findall(r'\.text\(\s*"([\w.]+)"', text):
            assert '{' not in messages[key]['fr'], (path, key, 'use format() for parameters')
print(f'OK — {len(messages)} keys, six languages, literal diagnostics and templates checked.')
