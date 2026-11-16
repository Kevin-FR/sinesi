# Qualification J7 — NAT et filtrage importés

Huit tests portables et quatre tests du corpus réussissent sur macOS 27.2 arm64,
Rust 1.95.0. Les entrées des 18 exemples et deux fixtures sont comparées octet
par octet après reconstruction. Les 20 empreintes d’origine concordent avec
la tranche DHCP et routes. Le corpus fournit trois redirections et deux
politiques ordonnées de six règles.
[Résumé du corpus et empreintes des traces](j7-import-security-summary.json)

| Recette | Résultat observé |
| --- | --- |
| `nat_with_mapped_port.fls` | Notebook 1.10 reçoit HTTP 200 en joignant `141.99.5.10:80`. La destination est traduite vers `192.168.0.10:80` et la requête traverse deux passerelles. [Trace compressée](j7-import-security-nat.jsonl.gz) |
| Internet complet EN | `42.0.0.1` est autorisé par la règle 6 et reçoit HTTP 200. Notebook 3 conserve sa réservation et la règle 1 autorise son SYN. Une carte remplacée dans Notebook 1 obtient `10.50.50.11` et la règle 2 refuse son SYN sur le même chemin. [Trace compressée](j7-import-security-firewall.jsonl.gz) |
| `nat.fls` et archive synthétique | La rétention de 30 secondes est conservée. Pour un port répété, la dernière redirection représentable est utilisée. Les redirections et règles rejetées restent signalées. |
| Restauration et rejeu | Pour les deux échanges historiques, la restauration du checkpoint et le rejeu reproduisent la trace canonique à l’octet près. |

Le scénario de filtrage emploie une copie de projet avec une seule carte réseau
remplacée pour exercer le pool dynamique. Le moteur applique la politique NAT
de Sinesi pour l’allocation des ports et les restrictions du WAN. Les extensions
Java restent des données inactives. Les parcours du CLI et du harnais egui
passent, y compris import, reprise et rejeu dans les six langues et les deux vues.
[Journal des parcours](j7-import-security-entry.log)

```sh
cargo test -p sns-filius-import --test import --locked --offline
mkdir -p /tmp/sns-j7-security
SNS_RECORD_J7_IMPORT=/tmp/sns-j7-security/corpus.json SNS_RECORD_J7_IMPORT_DIR=/tmp/sns-j7-security cargo test -p sns-filius-import --test import --locked --offline -- --ignored
cargo clippy -p sns-filius-import --all-targets --locked --offline -- -D warnings
cargo test -p sns-cli --test runner --locked --offline
cargo test -p sns-ui-egui --test lan j7_background_import --locked --offline
cargo test -p sns-ui-egui --test lan j7_runtime_restore_export_replay --locked --offline
cargo xtask check
python3 scripts/docs.py render
python3 scripts/docs.py check --require-reference
python3 scripts/check_manifest.py check docs/verification/j7-import-security-code-manifest.json
```

Les options RIP, certaines données de courrier et de présentation, ainsi que
la qualification de livraison sur les trois OS restent ouvertes.

[Journal des tests](j7-import-security-tests.log) ·
[Contrôles et registres](j7-import-security-docs.log) ·
[Manifeste](j7-import-security-code-manifest.json) ·
[Correctif reconstructible](j7-import-security-source-patch.json)
