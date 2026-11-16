# Qualification J7 — apparence nuage

Qualification locale du **8 novembre 2026**, macOS arm64, Rust 1.95.0. F-EQU-008 et RS-DEVICES-J7 sont vérifiés selon le [profil portable](../agnostique/PRESENTATION_J7.md) et l’[adaptateur](../rust/J7-EN-COURS.md). Le jalon J7 complet reste ouvert.

## Contrôles exécutés

| Commande | Résultat |
| --- | --- |
| `SNS_RECORD_J7_CLOUD="$PWD/docs/verification" cargo test --workspace --locked --offline` | **192 tests du workspace**, plus **1 test enfant** du pont; trois ignorés par défaut; [journal](j7-cloud-workspace.log) |
| `cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_cloud --locked --offline` | 1 parcours rendu, six langues et deux vues, **12 captures comparées sans mise à jour**; [journal](j7-cloud-gpu.log) |
| `cargo test -p sns-filius-import --test import --locked --offline -- --ignored` | Deux recettes réussies : 20 fichiers de référence et ping historique; [journal](j7-cloud-reference.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi sans avertissement; [journal](j7-cloud-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-cloud-fmt.log) |
| `python3 scripts/check_i18n.py`; `check_network_assets.py`; `check_layers.py` | **625 clés × six langues**, 20 SVG et frontières vérifiés; [langues](j7-cloud-i18n.log), [assets](j7-cloud-assets.log), [couches](j7-cloud-layers.log) |
| `python3 scripts/docs.py render`; `python3 scripts/docs.py check --require-reference` | Registres, dépendances et références vérifiés; [journal](j7-cloud-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-cloud-code-manifest.json` | **480 fichiers**, révision `sha256:38ec18ebbcfe4eecf5b81177ba872047ee1908501c37ba2eeccaa867d6c4b8e1` |

L’[environnement](j7-cloud-environment.json), le [manifeste](j7-cloud-code-manifest.json) et le [complément de sources](j7-cloud-source-delta.json) identifient cette qualification. Les sources et preuves historiques restent conservées.

## Couverture des critères

`sns-engine/tests/presentation.rs` alimente réellement la SAT par ARP/ICMP, puis conserve une trame en vol pendant la commande de présentation. Le réseau complet, les applications, la file, le générateur aléatoire et le temps sont comparés exactement avant/après. Les deux pings réussissent; ports, génération et rétention restent identiques. Checkpoint, suppression du symbole et rejeu intégral sont contrôlés. Une cible absente, un hôte non commutateur ou une trace saturée sont refusés sans mutation de l’état complet.

`sns-topology/tests/editor.rs` vérifie annulation/rétablissement et fusion des changements d’apparence sans écraser les autres valeurs locales de topologie. `sns-project/tests/contracts.rs` valide projet 8, commande API 1, fait 10, scénario et checkpoint; il refuse `cloud` dans un ancien profil. Le test synthétique d’import compare ports et connexions avant/après conversion du booléen historique.

Le parcours graphique `j7_cloud_switch_changes_live_with_ports_sat_and_run_preserved_in_six_languages` charge une SAT active, sélectionne le commutateur, active le nuage, change de vue, exporte/recharge un checkpoint et revient au symbole ordinaire. Il vérifie ports, temps, génération, réseau complet et file dans les six langues. L’arbre accessible est contrôlé, notamment le champ de rétention SAT désormais associé à son libellé.

Les captures FR en Icônes et DE en Schéma ont été inspectées : nuage distinct, mêmes trois connexions et réglage accessible dans l’inspecteur. La [trace](j7-cloud-trace.jsonl) et le [journal d’entrées](j7-cloud-recording.json) sont émis par la recette du moteur.

## Limites et suite

Cette propriété visuelle ne fournit aucun accès Internet. Le moteur arrêté conserve sa règle de refus des commandes; la commande graphique reflète l’état confirmé après acquittement. Les préférences, exports documentaires, observations avancées et la récupération restent dans les lots suivants.

La suite graphique historique complète et les dimensions générales de l’inspecteur doivent être qualifiées sur le produit final. Aucun parcours manuel autonome ni résultat Windows/Linux n’est revendiqué.
