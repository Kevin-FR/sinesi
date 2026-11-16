# Qualification J7 — RIP et documentation importés

Dix tests portables, cinq tests du corpus, les parcours CLI et le harnais egui
passent sur macOS 27.2 arm64 avec Rust 1.95.0. Les octets des entrées ZIP
reconstruites et les empreintes des 20 fichiers d’origine sont vérifiés.
Le corpus fournit 17 routeurs RIP et 67 annotations stylées dans sept exemples.
[Résumé et empreinte de la trace](j7-import-rip-doc-summary.json)

| Recette | Résultat |
| --- | --- |
| Internet complet EN | Sept routeurs RIP apprennent leurs routes. Le ping de `130.100.200.25` vers `140.100.200.25` arrive et revient par des routes apprises. Le rejeu et la continuation depuis le checkpoint reproduisent la trace canonique à l’octet près. [Trace compressée](j7-import-rip-doc-trace.jsonl.gz) |
| Archive synthétique de routage | Le mode RIP est indépendant du transfert IPv4. La passerelle par défaut et la table statique sont signalées et inactives lorsque RIP est choisi. Le mode statique conserve les deux. |
| Archive synthétique de documentation | La zone conserve ses dimensions et sa couleur, la note passe au-dessus, et le texte accentué garde le gras et l’italique. Une hauteur trop petite est signalée. |
| `nat_with_mapped_port.fls` | Les deux PNG décodés contiennent 38 objets, dont 9 annotations. L’inspection confirme les zones et les textes. Onze textes sont signalés comme tronqués dans chaque vue. Des équipements recouvrent aussi certaines notes. [Icônes](j7-import-rip-doc-icons.png) · [Schéma](j7-import-rip-doc-diagram.png) |

Le moteur emploie les temporisations et la sélection RIP de Sinesi. Les notes
conservent leur contenu intégral dans le projet et les polices historiques sont
remplacées par des familles portables. Les angles de zone sont droits. Le harnais
egui vérifie import, reprise, rejeu et édition des annotations dans les six langues
et les deux vues. [Journal CLI et egui](j7-import-rip-doc-entry.log)

```sh
cargo test -p sns-filius-import --test import --locked --offline
mkdir -p /tmp/sns-j7-rip
SNS_RECORD_J7_IMPORT=/tmp/sns-j7-rip/corpus.json SNS_RECORD_J7_IMPORT_DIR=/tmp/sns-j7-rip cargo test -p sns-filius-import --test import --locked --offline -- --ignored
cargo clippy -p sns-filius-import --all-targets --locked --offline -- -D warnings
cargo test -p sns-cli --test runner --locked --offline
cargo test -p sns-ui-egui --test lan j7_background_import --locked --offline
cargo test -p sns-ui-egui --test lan j7_runtime_restore_export_replay --locked --offline
cargo test -p sns-ui-egui --test annotations --locked --offline
cargo build -p sns-cli --locked --offline
target/debug/sinesi import-filius _archive/filius-master/beispiele/nat_with_mapped_port.fls --output /tmp/sns-j7-rip/annotated.json --report /tmp/sns-j7-rip/annotated-report.json
for view in icons diagram; do
  target/debug/sinesi render-png /tmp/sns-j7-rip/annotated.json --output "/tmp/sns-j7-rip/annotated-$view.png" --view "$view" --size 1400
  python3 scripts/check_png_exports.py "/tmp/sns-j7-rip/annotated-$view.png" --metadata "docs/verification/j7-import-rip-doc-$view.json"
done
cargo xtask check
python3 scripts/docs.py render
python3 scripts/docs.py check --require-reference
python3 scripts/check_manifest.py check docs/verification/j7-import-rip-doc-code-manifest.json
```

Les métadonnées PNG associent la sortie du CLI au SHA-256 du PNG et aux
identifiants des appareils, liaisons et annotations lus dans le projet importé.
La conversion du courrier historique et la livraison multiplateforme restent
ouvertes.

[Journal des tests](j7-import-rip-doc-tests.log) ·
[Contrôles et registres](j7-import-rip-doc-docs.log) ·
[Manifeste](j7-import-rip-doc-code-manifest.json) ·
[Correctif reconstructible](j7-import-rip-doc-source-patch.json)
