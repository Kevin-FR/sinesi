# J7 — Scénarios réseau et reprise qualifiés, jalon ouvert

Le 7 novembre 2026, macOS arm64, Rust 1.95.0. Cette tranche qualifie **F-SIM-007**, avec [contrat portable](../agnostique/REPRISE_J7.md) et [adaptateur](../rust/J7-EN-COURS.md). Les 22 exigences et 9 lots J7 restent le périmètre complet. RS-ENGINE-J7 reste en cours pour les pannes par liaison; aucun achèvement global n’est revendiqué.

## Vérification exécutée

| Commande | Résultat |
| --- | --- |
| `SNS_RECORD_J7_CHECKPOINT="$PWD/docs/verification" cargo test --workspace --locked --offline` | **174 tests du workspace**, plus **1 test dans le processus enfant** du pont; trois ignorés par défaut; [journal](j7-reprise-workspace.log) |
| `cargo test -p sns-filius-import --test import --locked --offline -- --ignored` | 2 recettes de référence réussies : 20 imports et ping historique; [journal](j7-reprise-reference.log) |
| `cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_runtime --locked --offline` | 1 parcours rendu, six langues et deux vues, **12 captures nouvelles comparées sans mise à jour**; [journal](j7-reprise-gpu.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi sans avertissement; [journal](j7-reprise-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-reprise-fmt.log) |
| `python3 scripts/check_i18n.py`; `check_layers.py`; `check_network_assets.py` | **588 clés × six langues**, couches et 18 SVG vérifiés; [langues](j7-reprise-i18n.log), [couches](j7-reprise-layers.log), [assets](j7-reprise-assets.log) |
| `python3 scripts/docs.py render`; `python3 scripts/docs.py check --require-reference` | Registres et références vérifiés; [journal](j7-reprise-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-reprise-code-manifest.json` | **424 fichiers source**, révision `sha256:02f6cb9f761bb56561017fd99620884d1d4ddcc53d749a68beced8c7d6e54e1b` |

L’[environnement](j7-reprise-environment.json) et le [manifeste](j7-reprise-code-manifest.json) identifient la qualification. Le [complément de sources](j7-reprise-source-delta.json) permet de reconstruire cette révision à partir du paquet historique de la tranche import, même lorsque le checkout évolue. Les preuves J0–J6 et la tranche import sont conservées.

## Couverture de F-SIM-007 et recettes

- `sns-engine/tests/checkpoint.rs` vérifie les trames en vol et leur budget de transit; commandes tardives inertes; mêmes traces suffixes et état réseau; checkpoint après une reprise précédente.
- La recette riche conserve TCP établi, minuterie RIP, bail DHCP, cache DNS et mapping NAT. Après 95 secondes virtuelles, les deux instances ont la même trace et les mêmes états réseau/applications. Le mapping expire; une nouvelle résolution du nom expiré émet une requête sans utiliser le cache périmé.
- La continuation d’un checkpoint traite le préfixe d’événements fusionné, les nouvelles commandes et l’origine du journal. Le journal résultant reste rejouable; une origine différente est refusée.
- Le scénario réseau conserve son second ping dans la file du checkpoint. Reprise et rejeu produisent deux réponses. Les identités dupliquées, temps désordonnés et contrôles interdits sont refusés; une action réseau invalide reste transactionnelle.
- `sns-bridge/tests/bridge.rs` utilise deux processus OS et l’adaptateur réel. Un checkpoint pris avant les arrivées externes, prolongé par leur journal, reproduit le ping hors ligne. Le journal complet et l’état du moteur correspondent, sans nouvel adaptateur.
- `sns-cli/tests/runner.rs` compare trois traces produites dans des processus distincts : exécution complète, reprise et rejeu; il vérifie aussi continuation, protection des sources et conservation d’une destination en cas de checkpoint incompatible.
- `sns-project/tests/contracts.rs` valide scénario, journal et checkpoint avec leurs schémas autonomes; propriétés inconnues et frontière nulle sont refusées. L’atteignabilité et le hash sont contrôlés par le moteur, au-delà de la structure du schéma.
- `sns-ui-egui` vérifie restauration/export/rejeu dans FR EN ES IT PT DE, maintien de la vue Schéma puis Icônes et conservation de l’état antérieur après fichier invalide. Un test du worker vérifie les identités réservées et les acquittements après restauration.

## Artefacts reproductibles

La [recette CLI](j7-reprise-cli.json) consigne les quatre invocations et la comparaison des traces [complète](j7-scenario-full.jsonl), [reprise](j7-scenario-resumed.jsonl) et [rejouée](j7-scenario-replayed.jsonl). Le [scénario](../../fixtures/valid/network-scenario.json), son [checkpoint](j7-scenario-checkpoint.json) et son [journal](j7-scenario-recording.json) sont fournis.

La recette riche produit un [état actif](j7-active-checkpoint.json), une [trace initiale](j7-reprise-original.jsonl) et une [trace après reprise](j7-reprise-restored.jsonl). Les données sont synthétiques. Un checkpoint utilisateur contient aussi les données pédagogiques, dont les identifiants de ses comptes; il n’est pas un journal technique anonyme.

Les captures FR de la fenêtre en Schéma et DE en Icônes ont été inspectées. Les références historiques J0–J6 ne sont pas remplacées. La comparaison de l’ensemble des anciennes captures n’est pas déclarée réussie : les nouvelles commandes de barre d’outils modifient leur rendu.

## Limites et suite

La restauration vérifie l’état par reconstruction de son historique borné; son coût n’est pas indépendant de la durée d’exécution. Les exports de plusieurs fichiers sont atomiques individuellement. Les scénarios et journaux sont des fichiers distincts du projet, sans restauration automatique de session.

Restent notamment : pertes/délais par liaison et graine, duplication/alignement, nuage du switch, parcours par couches/filtres/exports, annotations/styles et PNG/PDF, session/autosauvegarde, aide/préférences/configuration/diagnostics/options complètes, exemples par famille, installation/associations et validation réelle sur Windows/Linux/macOS. Aucun résultat Windows/Linux, parcours manuel autonome ou livraison finale n’est revendiqué. J7 reste actif.
