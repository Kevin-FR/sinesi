# J7 — attentes de réponse

Tranche F-CFG-005 / RS-PLATFORM-J7; non-régression F-SIM-007/008 / RS-ENGINE-J7.
Le [contrat portable](../agnostique/ATTENTES_J7.md),
l’[adaptateur](../rust/ATTENTES_J7.md) et [D-014](../rust/decisions/D-014-attentes-j7.md)
fixent la portée. F-CFG-005 et J7 restent ouverts pour les autres exigences.

## Résultat pédagogique

Le [scénario](../../fixtures/valid/response-wait-scenario.json) conserve deux
câbles à 300 ms. Avec le facteur 1, la [trace](j7-timing-cli-default.jsonl) n’a pas
de réponse ICMP : l’attente ARP expire. Avec le facteur 5, la
[réponse arrive](j7-timing-cli-full.jsonl), avec un RTT observé de 2,4 s.
Le [journal version 2](j7-timing-cli-recording.json) et le
[checkpoint en vol](j7-timing-cli-checkpoint.json) conservent le facteur. Les
traces du [rejeu](j7-timing-cli-replay.jsonl) et de la
[continuation](j7-timing-cli-resume.jsonl) sont identiques à celle de l’exécution :
`sha256:b4212d6d8c27687e902d587b378d16d326e014b18982fb88d371f3d07d9c9d95`.

Le moteur teste aussi les attentes TCP, DNS, DHCP et les durées conservées : bail
DHCP acquis, rétention ARP, TIME_WAIT, cadence RIP et intervalle du ping du terminal.
Un facteur incompatible ou une provenance différente empêche la reprise; l’ancien
checkpoint version 1 reste restaurable. La restauration dans le workbench applique
le facteur du fichier; une nouvelle simulation reprend celui du lancement.

## Commandes et preuves

Environnement : macOS arm64, Rust 1.95.0, Python 3.14.4, egui 0.37.0 et wgpu;
[environnement détaillé](j7-timing-environment.json).

| Commande | Résultat |
| --- | --- |
| `cargo test -p sns-engine --test timing -p sns-project --test contracts -p sns-cli --test runner -p sns-desktop --test startup -p sns-ui-egui --test configuration --locked --offline` | 38 tests réussis, dont neuf nouveaux cas de politique, contrats, CLI et workbench. [Journal](j7-timing-targeted.log) |
| `SNS_RECORD_J7_TIMING="$PWD/docs/verification" cargo xtask ci` | Clippy sans avertissement, 274 tests workspace et un enfant du pont réussis, cinq ignorés, quatre PNG décodés. [Journal](j7-timing-ci.log) |
| `cargo xtask ui` | 22 tests et 108 références comparées sans mise à jour. Références configuration Schéma FR et Icônes DE inspectées. [Journal](j7-timing-gpu.log) |
| `python3 scripts/docs.py render`, `python3 scripts/docs.py check --require-reference` | Vues régénérées; registres, liens et empreintes du référentiel vérifiés. [Journal](j7-timing-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-timing-code-manifest.json` | Inventaire courant conforme au manifeste scellé. [Journal](j7-timing-docs.log) |

Le vrai lanceur natif a produit six aides, par exemple [FR](j7-timing-help-fr.txt),
et douze rapports sans fenêtre, par exemple [Schéma FR](j7-timing-startup-fr-diagram.json)
et [Icônes DE](j7-timing-startup-de-icons.json). Ces rapports indiquent le facteur
réel du worker et ne modifient ni profil ni cache. Les tests couvrent les valeurs
invalides dans les six langues. Les 851 clés, six langues et 496 PNG antérieurs
sont conservés; aucune référence graphique ajoutée ou remplacée.

Le [manifeste](j7-timing-code-manifest.json) contient 790 fichiers, révision
`sha256:bbd20f87e9e141f1b10d02655d53e95a0c94199ac7418a8018e905aa7eb2bf05`. Le [delta reconstructible](j7-timing-source-delta.json) se superpose
à la base d’import. Les archives restent en lecture seule et ne sont pas
redistribuées. Le workspace garde 20 membres et le même ensemble de paquets tiers;
seule l’arête interne du lanceur vers le modèle est ajoutée.

La comparaison graphique et les essais sans fenêtre ne qualifient ni un sélecteur
natif ni un parcours Windows/Linux. Aucun run distant ni publication. Les options
v/l/n/s, diagnostics, exemples, livraison et couverture d’import restent au suivi.
