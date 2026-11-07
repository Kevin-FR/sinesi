#!/usr/bin/env python3
"""Regenerate and validate planning artifacts using only the Python standard library."""
from pathlib import Path
import argparse
import collections
import hashlib
import json
import re
import sys
from urllib.parse import unquote, urlsplit
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / '_archive/filius-master'


def read(path):
    return json.loads((ROOT / path).read_text())


def write_json(path, value):
    (ROOT / path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def source_domain(path):
    """Family attribution is an index, not a claim of behavior verification."""
    if path.endswith('Terminal.java') or path.endswith('GUIApplicationTerminalWindow.java'):
        return ['CLI']
    rules = [
        ('/software/nat/', ['NAT']), ('/software/dhcp/', ['DHCP']),
        ('/software/dns/', ['DNS']), ('/software/email/', ['MAIL']),
        ('/software/www/', ['WEB']), ('/software/firewall/', ['FW']),
        ('/software/dateiaustausch/', ['P2P']), ('/software/rip/', ['ROUT']),
        ('/software/transportschicht/', ['TCP', 'UDP']),
        ('/software/netzzugangsschicht/', ['L2']),
        ('/software/vermittlungsschicht/', ['IP4', 'L2', 'ROUT']),
        ('/gui/schichtensicht/', ['OBS']), ('/gui/nachrichtensicht/', ['OBS']),
        ('/rahmenprogramm/nachrichten/', ['OBS']), ('/gui/documentation/', ['DOC']),
        ('/gui/quelltextsicht/', ['EXT']), ('/hardware/', ['EQU', 'TOP']),
        ('/gui/netzwerksicht/', ['TOP', 'EQU', 'DOC']),
        ('/gui/anwendungssicht/', ['OS']), ('/software/lokal/', ['FS']),
        ('/software/system/', ['OS', 'EQU']), ('/software/clientserver/', ['OS', 'EXT']),
        ('SzenarioVerwaltung.java', ['PROJ']), ('ZipDatei.java', ['PROJ']),
        ('PersistenceDelegate.java', ['PROJ']), ('FiliusClassLoader.java', ['EXT']),
        ('/rahmenprogramm/', ['CFG']), ('/exception/', ['SIM']),
        ('/gui/', ['SIM', 'CFG']), ('/software/', ['SIM']), ('/Main.java', ['CFG']),
    ]
    for token, domains in rules:
        if token in path:
            return domains
    return []


def audit():
    assert ARCHIVE.is_dir(), 'Archive de référence absente; audit interdit sans le corpus local'
    features = read('docs/suivi/features.json')['features']
    by_path = collections.defaultdict(list)
    for feature in features:
        for evidence in feature['evidence']:
            by_path[evidence['path']].append(feature['id'])
    files = []
    for file in sorted(ARCHIVE.rglob('*')):
        if not file.is_file():
            continue
        relative = file.relative_to(ROOT).as_posix()
        raw = file.read_bytes()
        item = dict(path=relative, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
        if '/src/main/java/' in relative:
            text = raw.decode('utf-8')
            item.update(kind='production_java', domains=source_domain(relative),
                        direct_evidence_for=by_path[relative],
                        public_and_protected_methods=[
                            dict(line=i + 1, declaration=line.strip())
                            for i, line in enumerate(text.splitlines())
                            if re.match(r'\s*(public|protected)\s+.*\(', line)
                            and not re.search(r'\b(if|for|while)\s*\(', line)])
        elif '/src/test/java/' in relative:
            item.update(kind='legacy_test', domains=source_domain(relative.replace('/src/test/', '/src/main/')))
        elif file.suffix == '.fls':
            item['kind'] = 'scenario'
        elif '/java-runtime/' in relative:
            item['kind'] = 'runtime_packaging'
        elif '/resources/' in relative:
            item['kind'] = 'resource'
        elif '/info/' in relative or file.suffix in ['.md', '.txt', '.pdf']:
            item['kind'] = 'reference_document'
        else:
            item['kind'] = 'build_or_metadata'
        files.append(item)
    write_json('docs/audit/sources.json', dict(date='2026-11-07', files=files))
    examples = []
    for file in sorted((ARCHIVE / 'beispiele').glob('*.fls')):
        with zipfile.ZipFile(file) as archive:
            config = next(name for name in archive.namelist() if name.endswith('konfiguration.xml'))
            text = archive.read(config).decode('utf-8')
            classes = collections.Counter(re.findall(r'class="(filius\.[^"]+)"', text))
            examples.append(dict(path=file.relative_to(ROOT).as_posix(), entries=archive.namelist(),
                                 configuration=config, classes=dict(sorted(classes.items())),
                                 executable_payloads=[name for name in archive.namelist()
                                                      if name.endswith(('.java', '.class', '.jar'))]))
    write_json('docs/audit/exemples.json', dict(examples=examples))
    main = [file for file in files if file['kind'] == 'production_java']
    counts = collections.Counter(file['kind'] for file in files)
    lines = ['# Index de l’archive', '',
             'Généré par `python3 scripts/docs.py audit`. Les empreintes identifient le corpus local; '
             'les rattachements par famille sont un index structurel, pas une preuve de validation dynamique.', '',
             '| Catégorie | Fichiers |', '| --- | ---: |']
    lines += [f'| {kind} | {count} |' for kind, count in sorted(counts.items())]
    lines += ['', '## Code de production', '',
              'Les identifiants de la dernière colonne pointent les preuves directement citées par le catalogue. '
              'Un tiret indique une classe de support indexée, sans preuve directe retenue.', '',
              '| Fichier | Familles | Preuves directes |', '| --- | --- | --- |']
    for file in main:
        path = file['path']
        lines.append(f'| [{path.split("/filius/", 1)[-1]}](../../{path}) | '
                     f'{", ".join(file["domains"])} | {", ".join(file["direct_evidence_for"]) or "—"} |')
    lines += ['', '## Scénarios livrés', '', '| Fichier | Entrées ZIP | Code embarqué |', '| --- | ---: | --- |']
    for example in examples:
        lines.append(f'| [{Path(example["path"]).name}](../../{example["path"]}) | '
                     f'{len(example["entries"])} | {", ".join(example["executable_payloads"]) or "aucun"} |')
    lines += ['', 'Les 21 classes de tests historiques sont listées avec empreinte dans '
              '[sources.json](sources.json). Elles constituent des pistes de recettes, pas des tests exécutés du nouveau produit.', '']
    (ROOT / 'docs/audit/SOURCES.md').write_text('\n'.join(lines))


def render(check_only=False):
    def emit(path, content):
        target = ROOT / path
        if check_only:
            assert target.read_text() == content, f'Vue périmée : {path}; lancer render'
        else:
            target.write_text(content)
    data = read('docs/suivi/features.json')
    domains = read('docs/suivi/domaines.json')
    features = data['features']
    tasks = read('docs/rust/taches.json')['tasks']
    tasks_by_feature = collections.defaultdict(list)
    for task in tasks:
        for ident in task['features']:
            tasks_by_feature[ident].append(task['id'])
    lines = ['# Catalogue des exigences', '',
             'Vue générée depuis [features.json](../suivi/features.json); modifier le registre puis lancer '
             '`python3 scripts/docs.py render`. Les critères décrivent le produit cible, même lorsqu’ils corrigent '
             'une limite historique. Les états de réalisation et leurs preuves figurent dans le registre.', '',
             f'{len(features)} exigences : ' + ', '.join(f'{count} {origin}' for origin, count in collections.Counter(
                 f['origin'] for f in features).items()) + '.', '',
             'Une preuve de code établit une présence statique; elle ne prouve ni conformité complète à un standard '
             'ni fonctionnement constaté à l’exécution.', '', '## Domaines', '',
             '| Famille | Domaine | Nombre |', '| --- | --- | ---: |']
    lines += [f'| {key} | [{value["title"]}](#{key.lower()}) | '
              f'{sum(f["domain"] == key for f in features)} |' for key, value in domains.items()]
    for key, value in domains.items():
        lines += ['', f'<a id="{key.lower()}"></a>', '', f'## {key} — {value["title"]}', '']
        for feature in [f for f in features if f['domain'] == key]:
            lines += [f'<a id="{feature["id"].lower()}"></a>', '',
                      f'### {feature["id"]} — {feature["title"]}', '',
                      f'- Origine : `{feature["origin"]}`; état du référentiel : `{feature["baseline"]}`.',
                      f'- Exigence : {feature["requirement"]}',
                      f'- Recette : {" ".join(feature["acceptance"])}',
                      f'- Composant : `{feature["component"]}`; jalon : `{feature["milestone"]}`; état : `{feature["status"]}`.',
                      f'- Prérequis fonctionnels : {", ".join(feature["depends_on"]) or "aucun"}.']
            for evidence in feature['evidence']:
                path = evidence['path']
                lines += [f'- Preuve : [{path}](../../{path}#L{evidence["line"]}), '
                          f'ligne {evidence["line"]}, `{evidence["anchor"]}`.']
            lines += ['']
    emit('docs/agnostique/CATALOGUE.md', '\n'.join(lines))
    matrix = ['# Traçabilité exigences ↔ implémentation', '',
              'Vue générée. Autorité fonctionnelle : [features.json](features.json). '
              'Autorité des tâches Rust : [taches.json](../rust/taches.json). '
              'Les états sont indépendants et ne constituent pas une preuve de recette.', '',
              '| Exigence | Jalon | État fonctionnel | Tâche(s) Rust | État technique |',
              '| --- | --- | --- | --- | --- |']
    task_states = {task['id']: task['status'] for task in tasks}
    for feature in features:
        linked = [f'[{ident}](../rust/PLAN.md#{ident.lower()})' for ident in tasks_by_feature[feature['id']]]
        matrix.append(f'| [{feature["id"]}](../agnostique/CATALOGUE.md#{feature["id"].lower()}) | '
                      f'{feature["milestone"]} | {feature["status"]} | '
                      f'{", ".join(linked)} | '
                      f'{", ".join(task_states[ident] for ident in tasks_by_feature[feature["id"]])} |')
    emit('docs/suivi/MATRICE.md', '\n'.join(matrix) + '\n')
    plan = ['# Plan d’implémentation Rust', '',
            'Vue générée depuis [taches.json](taches.json). Les chemins sont des cibles proposées; '
            'les fichiers effectivement réalisés sont indiqués séparément. Les dépendances expriment les prérequis techniques; '
            'les recettes restent celles du catalogue agnostique.', '']
    for task in tasks:
        linked_features = [f'[{ident}](../agnostique/CATALOGUE.md#{ident.lower()})' for ident in task['features']]
        plan += [f'<a id="{task["id"].lower()}"></a>', '',
                 f'## {task["id"]} — {task["title"]}', '',
                 f'- Jalon : `{task["milestone"]}`; état : `{task["status"]}`; taille indicative : `{task["size"]}`.',
                 f'- Exigences : {", ".join(linked_features)}.',
                 f'- Prérequis : {", ".join(task["depends_on"]) or "aucun"}.',
                 f'- Cibles : {", ".join("`"+p+"`" for p in task["planned_paths"])}.',
                 f'- Fichiers réalisés : {", ".join("`"+p+"`" for p in task["actual_paths"]) or "aucun"}.',
                 f'- Approche : {task["approach"]}',
                 '- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves '
                 'dans les deux registres et vérifier les contrats communs.', '']
    emit('docs/rust/PLAN.md', '\n'.join(plan))


def cycles(items, label):
    by_id = {item['id']: item for item in items}
    active, done = set(), set()
    def visit(ident):
        if ident in active:
            raise ValueError(f'{label}: cycle impliquant {ident}')
        if ident in done:
            return
        if ident not in by_id:
            raise ValueError(f'{label}: référence inconnue {ident}')
        active.add(ident)
        for dep in by_id[ident]['depends_on']:
            visit(dep)
        active.remove(ident)
        done.add(ident)
    for ident in by_id:
        visit(ident)


def check(require_reference=False):
    data = read('docs/suivi/features.json')
    features = data['features']
    tasks = read('docs/rust/taches.json')['tasks']
    domains = read('docs/suivi/domaines.json')
    inventory = read('docs/audit/sources.json')['files']
    indexed = {file['path'] for file in inventory}
    reference_available = ARCHIVE.is_dir()
    assert reference_available or not require_reference, 'Archive de référence requise mais absente'
    ids = {f['id'] for f in features}
    assert len(ids) == len(features), 'Identifiants fonctionnels dupliqués'
    assert len({t['id'] for t in tasks}) == len(tasks), 'Identifiants de tâches dupliqués'
    for label, items in [('features', features), ('tasks', tasks)]:
        cycles(items, label)
        by_id = {x['id']: x for x in items}
        for item in items:
            assert item['status'] in data['status_vocabulary'], item['id']
            assert re.fullmatch(r'J[0-9]', item['milestone']), item['id']
            for dep in item['depends_on']:
                assert by_id[dep]['milestone'] <= item['milestone'], (item['id'], dep, 'jalon ultérieur')
            if item['status'] == 'verified':
                assert item['verification'], (item['id'], 'preuve de recette manquante')
                for proof in item['verification']:
                    assert isinstance(proof, dict) and all(proof.get(key) for key in (
                        'recipe', 'command', 'environment', 'revision', 'artifacts')), (
                            item['id'], 'preuve de recette incomplète')
                    assert proof.get('result') == 'pass', (item['id'], 'recette en échec')
                assert all(by_id[dep]['status'] == 'verified' for dep in item['depends_on']), (
                    item['id'], 'prérequis non vérifié')
    for feature in features:
        assert feature['domain'] in domains
        assert re.fullmatch(r'F-' + re.escape(feature['domain']) + r'-\d{3}', feature['id'])
        assert feature['origin'] in ['filius', 'modernisation', 'extension_optionnelle']
        assert feature['baseline'] in ['present', 'experimental', 'internal', 'removed', 'absent_du_referentiel']
        assert feature['requirement'] and feature['acceptance']
        if feature['origin'] == 'filius':
            assert feature['evidence'], feature['id']
        for evidence in feature['evidence']:
            assert evidence['path'] in indexed, (feature['id'], 'source hors inventaire')
            assert evidence['line'] > 0 and evidence['anchor'] in evidence['snippet'], feature['id']
            if not reference_available:
                continue
            lines = (ROOT / evidence['path']).read_text().splitlines()
            assert evidence['anchor'] in lines[evidence['line'] - 1], (feature['id'], 'preuve déplacée')
            assert evidence['snippet'] == lines[evidence['line'] - 1].strip(), feature['id']
    covered = collections.Counter()
    for task in tasks:
        assert task['features'] and task['planned_paths'] and task['approach']
        assert isinstance(task['actual_paths'], list), task['id']
        if task['status'] in ['implemented', 'verified']:
            assert task['actual_paths'], (task['id'], 'fichiers réalisés manquants')
        for path in task['actual_paths']:
            assert (ROOT / path).exists(), (task['id'], path, 'réalisation absente')
        for ident in task['features']:
            assert ident in ids, ident
            covered[ident] += 1
        if task['status'] == 'verified':
            assert all(f['status'] == 'verified' for f in features if f['id'] in task['features']), (
                task['id'], 'exigence non vérifiée')
    assert set(covered) == ids, ('Exigences sans tâche', ids - set(covered))
    actual = {file.relative_to(ROOT).as_posix() for file in ARCHIVE.rglob('*') if file.is_file()}
    if reference_available:
        assert actual == indexed, 'Inventaire source incomplet ou archive modifiée'
    for file in inventory:
        if reference_available:
            assert hashlib.sha256((ROOT / file['path']).read_bytes()).hexdigest() == file['sha256'], file['path']
        if file['kind'] == 'production_java':
            assert file['domains'] and all(d in domains for d in file['domains']), file['path']
    # Portable documentation must not embed implementation-language choices.
    for file in (ROOT / 'docs/agnostique').glob('*.md'):
        assert not re.search(
            r'\b(Rust|Cargo|Tokio|Tauri|serde|crate|egui|eframe|egui_extras|egui_kittest|'
            r'wgpu|winit|rfd|arboard|tiny-skia|lyon_tessellation|skrifa|harfrust|'
            r'unicode-bidi|icu_segmenter|craft-[a-z][\w-]*)\b', file.read_text(), re.I), file
    render(check_only=True)
    check_links()
    print(f'OK — {len(features)} exigences, {len(tasks)} tâches, {len(inventory)} fichiers source; '
          'preuves, dépendances, jalons, couverture et séparation vérifiés.')
    if not reference_available:
        print('Référence locale absente : inventaire seulement; contenu/empreintes historiques non revérifiés.')


def check_links():
    archived_paths = {file['path'] for file in read('docs/audit/sources.json')['files']}
    documents = list((ROOT / 'docs').rglob('*.md')) + [ROOT / 'README.md', ROOT / 'AGENTS.md']
    for document in documents:
        content = re.sub(r'```.*?```', '', document.read_text(), flags=re.S)
        for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', content):
            parts = urlsplit(target)
            if parts.scheme or parts.netloc:
                continue
            path = (document.parent / unquote(parts.path)).resolve() if parts.path else document
            if not ARCHIVE.is_dir() and path.is_relative_to(ROOT) and path.relative_to(ROOT).as_posix() in archived_paths:
                continue
            assert path.is_file(), (document.relative_to(ROOT).as_posix(), target, 'lien absent')
            if re.fullmatch(r'L\d+', parts.fragment):
                line = int(parts.fragment[1:])
                assert 1 <= line <= len(path.read_text().splitlines()), (document.name, target, 'ligne absente')
                continue
            if parts.fragment and path.suffix == '.md':
                text = path.read_text()
                anchors = set(re.findall(r'<a id="([^"]+)">', text))
                for heading in re.findall(r'^#{1,6}\s+(.+)', text, re.M):
                    heading = re.sub(r'[^\w\s-]', '', heading.lower())
                    anchors.add(re.sub(r'\s', '-', heading))
                assert unquote(parts.fragment) in anchors, (document.name, target, 'ancre absente')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['audit', 'render', 'check'])
    parser.add_argument('--require-reference', action='store_true', help='Exiger le corpus Filius et revérifier ses empreintes')
    args = parser.parse_args()
    try:
        if args.action == 'check':
            check(require_reference=args.require_reference)
        else:
            globals()[args.action]()
    except (AssertionError, ValueError, KeyError) as error:
        print(f'ERREUR — {error}', file=sys.stderr)
        sys.exit(1)
