# Qualification J7 — recherche, exports et parcours causal

Qualification locale du **8 novembre 2026**, macOS arm64, Rust 1.95.0. F-OBS-006/008 et RS-OBSERVATION-J7 sont vérifiés selon le [profil portable](../agnostique/OBSERVATION_J7.md) et l’[adaptateur](../rust/J7-EN-COURS.md). J7 reste ouvert.

## Commandes exécutées

| Commande | Résultat |
| --- | --- |
| `SNS_RECORD_J7_OBSERVATION="$PWD/docs/verification" cargo test --workspace --locked --offline` | **197 tests du workspace** et **1 test enfant** du pont; trois ignorés par défaut; [journal](j7-observation-workspace.log) |
| `cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_trace_queries --locked --offline` | 1 parcours dans six langues, **12 captures comparées sans mise à jour**; [journal](j7-observation-gpu.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi sans avertissement; [journal](j7-observation-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-observation-fmt.log) |
| `python3 scripts/check_i18n.py`; `check_network_assets.py`; `check_layers.py` | **651 clés × six langues**, 20 SVG et frontières vérifiés; [langues](j7-observation-i18n.log), [assets](j7-observation-assets.log), [couches](j7-observation-layers.log) |
| `python3 scripts/check_trace_exports.py docs/verification/j7-observation-selected.jsonl docs/verification/j7-observation-selected.csv --source docs/verification/j7-observation-web.jsonl` | **2 faits**, tous les champs égaux et sous-ensemble source exact, dans le même ordre; [journal](j7-observation-exports.log) |
| `python3 scripts/check_trace_exports.py docs/verification/j7-observation-escaping.jsonl docs/verification/j7-observation-escaping.csv` | 1 fait avec accents, guillemets, retour de ligne et caractères japonais, sans perte |
| `python3 scripts/docs.py render`; `python3 scripts/docs.py check --require-reference` | Registres, références et séparation vérifiés; [journal](j7-observation-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-observation-code-manifest.json` | **498 fichiers**, révision `sha256:5b4404f0b49216e26c6f6b9139362436650aba12bb458fb49e31e17d7dfcf41f` |

L’[environnement](j7-observation-environment.json), le [manifeste](j7-observation-code-manifest.json) et le [complément de sources](j7-observation-source-delta.json) identifient cette passe. Le nouveau crate est local; aucune version de dépendance externe ne change. Les anciennes sources et captures qualifiées restent conservées.

## Couverture

Les quatre tests de `sns-trace/tests/observation.rs` utilisent le moteur réel, DHCP puis DNS/HTTP, avant toute projection. Ils vérifient les filtres combinés équipement/temps/protocole/adresse/corrélation, les exclusions exactes, l’égalité intégrale du JSONL, les quatre couches et plusieurs équipements du parcours, l’ordre causal et la conservation de l’état complet du moteur. Une branche sœur ayant la même corrélation reste exclue; une cause valide entre générations est suivie. Une cause absente marque le parcours partiel, et les cycles/causes futures sont refusés. Les exports vides et la saturation de 64 Mio sont également testés.

Le [parcours enregistré](j7-observation-path.json) référence les faits de la [trace source](j7-observation-web.jsonl). La [sélection JSONL](j7-observation-selected.jsonl) et son [CSV](j7-observation-selected.csv) sont relus avec le module CSV indépendant de la bibliothèque standard Python. Ce contrôle compare également versions, temps, causes, corrélations et contenus structurés; il ne se limite pas à compter les lignes. Les [données avec échappement](j7-observation-escaping.csv) sont comparées à leur [JSONL](j7-observation-escaping.jsonl).

Le test graphique `j7_trace_queries_exports_and_local_global_animation_preserve_six_languages_and_views` restaure un checkpoint, saisit les cinq champs et choisit un équipement. Il compare les identités de la sélection avec le JSONL et vérifie que l’écriture CSV conserve la sélection initiale malgré une réinitialisation des filtres. Un temps invalide ne remplace pas le fichier existant. Il ouvre ensuite le parcours, avance localement, lance/suspend l’animation globale, montre le réseau et change vers Schéma. Trace, projet, temps simulé et génération restent identiques dans les six langues. Les contrôles d’accessibilité restent actifs.

Les captures FR locale et DE globale ont été inspectées. Les quatre couches restent lisibles, les numéros/temps ne se replient plus dans les colonnes, la chronologie explicite les ramifications possibles et la cause courante est visible. En Schéma, un contour orange marque le poste de l’étape courante. La comparaison rendue a ensuite été exécutée sans mise à jour des images.

## Limites et suite

Le parcours est une projection des causes disponibles, figée à son ouverture. Les étapes manquantes ne sont pas inventées. Les données affichées dans une enveloppe ne signifient pas que l’équipement a exécuté toutes ses couches. La chronologie globale conserve l’ordre des faits; elle ne transforme pas une diffusion en liaison physique entre ses récepteurs.

Cette passe n’annonce ni la suite graphique historique complète verte, ni une qualification Windows/Linux ou en fenêtre autonome. Annotations et exports PNG/PDF, session/récupération, configuration/aide/diagnostics, exemples et livraison trois OS restent requis. Le diagnostic terminal historique F-CLI-019 reste également à documenter.
