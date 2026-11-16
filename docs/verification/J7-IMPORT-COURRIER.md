# Qualification J7 — corpus Filius et courrier

F-PROJ-005 et RS-PROJECTS-J7 sont qualifiés localement. Les rapports complets
publient la couverture des 18 exemples et deux fixtures. Les tests comparent
toutes les entrées originales octet par octet, valident les projets et
contrôlent que les pièces Accumulator et Broadcast restent inactives.
[Rapports des 20 fichiers](j7-import-mail-corpus.json.gz)

| Recette | Résultat |
| --- | --- |
| Courrier du corpus | 14 configurations clientes, 12 comptes serveur, 4 messages reçus, 15 envoyés et 2 messages en boîte serveur |
| `man-in-the-middle.fls` | Le client relève deux messages par POP3. Son dossier reçu passe de 1 à 3 messages et son dossier Envoyés reste à 3. Le rejeu intégral et la reprise depuis le checkpoint reproduisent la trace. [Compteurs et empreinte](j7-import-mail-pop3.json) |
| Archive synthétique | Identité issue d’une table XML, boîtes XML, contact accentué, copie et BCC. Les copies de réception masquent les BCC. Le contact invalide et la date locale sont signalés. Le rapport ne contient pas les valeurs testées. |
| Sécurité du corpus | Chemins sortants, chemins absolus, doublons, mauvais CRC, alias manquants, entités XML et entrée surdimensionnée refusés |
| Contrôles réseau antérieurs sur le code courant | LAN, DHCP, routes statiques, RIP, NAT et filtrage vérifiés sur les exemples correspondants |
| Parcours utilisateur | CLI, import et reprise egui, récupération de session dans les six langues et les deux vues. [Journal](j7-import-mail-entry.log) |

Les dossiers et boîtes serveur suivent les limites du format portable.
Les brouillons, dates murales et identités supplémentaires restent dans
les originaux. Le corpus ne contient aucun contact, et sa conversion
est vérifiée par l’archive synthétique. Les rapports signalent les pertes
et conservent `functional_equivalence_verified: false`. La preuve POP3
publie uniquement des compteurs et le SHA-256 de la trace brute.

## Commandes

```sh
cargo test -p sns-filius-import --test import --locked --offline
mkdir -p /tmp/sns-j7-mail
SNS_RECORD_J7_IMPORT=/tmp/sns-j7-mail/corpus.json SNS_RECORD_J7_IMPORT_DIR=/tmp/sns-j7-mail cargo test -p sns-filius-import --test import --locked --offline -- --ignored
cargo clippy -p sns-filius-import --all-targets --locked --offline -- -D warnings
cargo test -p sns-engine --test mail --locked --offline
cargo test -p sns-cli --test runner --locked --offline
cargo test -p sns-ui-egui --test lan j7_background_import --locked --offline
cargo test -p sns-ui-egui --test lan j7_runtime_restore_export_replay --locked --offline
cargo test -p sns-ui-egui --test session --locked --offline
cargo xtask check
python3 scripts/docs.py render
python3 scripts/docs.py check --require-reference
python3 scripts/check_manifest.py check docs/verification/j7-import-mail-code-manifest.json
```

Les paquets Debian et macOS arm64 ont été reconstruits dans
`artifacts/packages/j7-mail/` et les recettes de la
[livraison](../rust/LIVRAISON_J7.md) rejouées. Le statut de qualification de
livraison reste partiel. [Empreintes des paquets](j7-import-mail-packages.sha256.txt) ·
[Debian](j7-import-mail-linux-package.log) · [macOS](j7-import-mail-macos-package.log)

[Journal des tests](j7-import-mail-tests.log) ·
[Contrôles et registres](j7-import-mail-docs.log) ·
[Manifeste](j7-import-mail-code-manifest.json) ·
[Correctif reconstructible](j7-import-mail-source-patch.json)
