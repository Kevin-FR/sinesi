# Qualification J7 — DHCP et routage statique importés

Sept tests portables et trois tests du corpus passent sur macOS 27.2 arm64 avec
Rust 1.95.0. Le corpus comprend 18 exemples et deux fixtures. Toutes les
entrées ZIP de chaque fichier sont reconstruites et comparées octet par octet.
Le projet sérialisé est relu avec les contrats courants. Les 20 empreintes
d’origine concordent avec la qualification précédente. Le corpus et les
extensions restent des données inactives.
[Résumé des 20 rapports](j7-import-network-corpus-summary.json)

| Recette | Résultat |
| --- | --- |
| `dhcp_test_scenario.fls` | Deux baux, dont un avec la passerelle LAN `192.168.0.10`. Le poste relié au WAN ne reçoit pas d’offre du serveur LAN. [Trace](j7-import-network-dhcp.jsonl) |
| `internetwork_komplett.fls` | Ping de `141.99.1.10` à `141.99.5.10` reçu en retour. Les routes statiques sont observées via `141.99.4.2` à l’aller et `141.99.2.1` au retour. [Trace](j7-import-network-routes.jsonl) |
| Archive synthétique | DHCP client sur WAN, serveur sur LAN, DNS annoncé, réservation valide conservée et réservation hors sous-réseau signalée. Le serveur désactivé reste inactif. |

Le relevé couvre 15 configurations de serveurs, quatre réservations et
quatre lignes statiques dans les 20 fichiers. Les recettes d’échange portent
sur les exemples nommés ci-dessus. Le CLI et le harnais egui valident leurs
parcours d’import. Le harnais confirme aussi la reprise et le rejeu dans les
six langues et les deux vues. L’import global reste partiel, en particulier
pour les réglages NAT et filtrage, les options RIP et le courrier historique.

```sh
cargo test -p sns-filius-import --test import --locked --offline
mkdir -p /tmp/sns-j7-net-import
SNS_RECORD_J7_IMPORT=/tmp/sns-j7-net-import/corpus.json SNS_RECORD_J7_IMPORT_DIR=/tmp/sns-j7-net-import cargo test -p sns-filius-import --test import --locked --offline -- --ignored
cargo clippy -p sns-filius-import --all-targets --locked --offline -- -D warnings
cargo test -p sns-cli --test runner filius_import_cli --locked --offline
cargo test -p sns-ui-egui --test lan j7_background_import --locked --offline
cargo test -p sns-ui-egui --test lan j7_runtime_restore_export_replay --locked --offline
cargo xtask check
python3 scripts/docs.py render
python3 scripts/docs.py check --require-reference
python3 scripts/check_manifest.py check docs/verification/j7-import-network-code-manifest.json
```

[Journal des tests](j7-import-network-tests.log) ·
[Parcours CLI et egui](j7-import-network-entry.log) ·
[Contrôles et registres](j7-import-network-docs.log) ·
[Manifeste](j7-import-network-code-manifest.json) ·
[Correctif reconstructible](j7-import-network-source-patch.json)
