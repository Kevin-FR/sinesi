#!/usr/bin/env python3
"""Independently compare portable trace exports using the standard CSV parser."""
import argparse
import csv
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('jsonl', type=Path)
parser.add_argument('csv', type=Path)
parser.add_argument('--source', type=Path)
args = parser.parse_args()
csv.field_size_limit(64 * 1024 * 1024)
events = [json.loads(line) for line in args.jsonl.read_text().splitlines()]
fields = ['run_id', 'event_id', 'event_schema_version', 'sequence', 'time_us',
          'type', 'device_id', 'caused_by', 'correlation_id', 'payload_json']
with args.csv.open(newline='') as stream:
    reader = csv.DictReader(stream)
    assert reader.fieldnames == fields, reader.fieldnames
    records = list(reader)
assert len(events) == len(records), (len(events), len(records))
identities = [(e['run_id'], e['event_id']) for e in events]
assert len(set(identities)) == len(events), 'Duplicate identities'
for event, record in zip(events, records):
    assert set(record) == set(fields), 'Wrong number of CSV cells'
    for key in fields[:-1]:
        assert str(event[key] if event[key] is not None else '') == record[key], key
    assert event['payload'] == json.loads(record['payload_json']), 'payload'
if args.source:
    source = [json.loads(line) for line in args.source.read_text().splitlines()]
    index = {(e['run_id'], e['event_id']): (i, e) for i, e in enumerate(source)}
    previous = -1
    for identity, event in zip(identities, events):
        position, original = index[identity]
        assert position > previous and event == original, identity
        previous = position
print(f'OK — {len(events)} records: all fields, stable identities, payloads and order match.'
      + (' Exact source subset checked.' if args.source else ''))
