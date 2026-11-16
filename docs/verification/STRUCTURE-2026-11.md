# Structure et méthode — qualification du 8 novembre 2026

Tranche **RS-QUALITY-J0**, complément aux exigences **F-NFR-002/003/008**.
Leurs états sont conservés; RS-QUALITY-J0 et F-NFR-001/006 restent en cours.
La [décision D-011](../rust/decisions/D-011-structure-et-workflow.md) contient la
comparaison des quatre fichiers SolidCraft/WordCraft à des révisions figées.
Le [guide courant](../rust/DEVELOPPEMENT.md) donne les commandes de reprise.

## Résultat

- Le workbench de 2 380 lignes est remplacé par une coordination de **456 lignes**
  et des modules de responsabilité. Le produit expose `Workbench`; tous ses
  appelants sont adaptés, sans alias de transition `LanUi`. Le prototype J0 et son
  contrôleur restent isolés et testés. Les chemins standards remplacent `#[path]`.
- Les **19 membres** héritent des versions/features et lints du workspace. Le
  lockfile ajoute seulement `xtask`, sans nouveau paquet tiers ou changement de
  version, source ou empreinte tiers par rapport à la tranche PNG.
- Le contrôle des couches couvre désormais tout membre, notamment `sns-settings`
  auparavant absent de la liste de racines. Il refuse membre inconnu, arête interne
  non déclarée, route indirecte vers GUI/pont/outil et dépendances externes interdites.
  Les arêtes de build et conditionnelles sont incluses; une arête uniquement de test
  reste distincte d’une arête de production.
- `cargo xtask ci` compose les contrôles du dépôt et arrête au premier échec. La
  CI préparée appelle la même commande après préparation du cache verrouillé.
  Les instructions racine/locales, cartes et guide orientent le travail; l’état de
  reprise résume le présent et conserve sa chronologie antérieure séparément.

Les contrats de données, fixtures, FR EN ES IT PT DE et Icônes/Schéma sont conservés.
[refactor-paths.json](../rust/refactor-paths.json) relie les anciens fichiers aux
nouveaux. Les `actual_paths` sont adaptés; tous les `planned_paths` et statuts de
lots sont identiques à la tranche PNG. Les preuves historiques restent inchangées.

## Vérifications réellement exécutées

Environnement : macOS arm64, Rust 1.95.0, egui 0.37.0/wgpu. Voir
[l’environnement détaillé](structure-2026-11-environment.json).

| Commande | Résultat et preuve |
| --- | --- |
| `cargo xtask ci` | **226 tests du workspace et un enfant du pont**, cinq ignorés, Clippy sans avertissement, formatage et contrôles; quatre PNG décodés indépendamment. [Journal complet](structure-2026-11-ci.log) |
| `cargo xtask ui` | **3 tests GPU**, **12 captures comparées sans mise à jour**, six langues et deux vues. [Journal](structure-2026-11-gpu.log) |
| `python3 scripts/test_layers.py` (inclus dans `ci`) | **5 cas négatifs/positifs** : membre inconnu, inversion de couche, GUI transitive, build/condition/mélange dev-production, exception test seule |
| `cargo xtask unknown` | **Sortie 1 attendue**, aucune tâche exécutée. [Journal](structure-2026-11-invalid-task.log) |
| `cargo test -p sns-ui-egui --test lan keyboard_connects_two_hosts_and_inspects_real_echo --locked --offline -- --exact` | **20 répétitions réussies** après correction d’attente. [Journal](structure-2026-11-keyboard-stress.log), [résultats](structure-2026-11-keyboard-stress.json) |
| `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check --require-reference` | Vues régénérées, liens/registres et empreintes de l’archive contrôlés. [Journal](structure-2026-11-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/structure-2026-11-code-manifest.json` | Inventaire final identique au manifeste |

Les contrôles de `ci` incluent également les deux tests d’outillage documentaire,
les deux tests d’inspection, 709 clés × six langues, 20 SVG, quatre fontes et la
politique de couches complète. Deux processus de récupération sont réellement
arrêtés dans la suite workspace; aucun nouveau stress de récupération n’est revendiqué.
La capture DE/Schéma a été inspectée après comparaison; les **321 images UI de la
révision PNG restent identiques octet pour octet**. Il n’y a pas de nouvelle référence
visuelle à approuver pour ce déplacement de fichiers.

## Échec initial et correction

La première passe complète a échoué dans le parcours clavier : après « Nouveau »,
le test demandait immédiatement un ajout alors que la nouvelle génération du worker
pouvait rester non acquittée. La garde métier refuse correctement cette édition;
le test indexait ensuite un poste inexistant. Le chemin de remplacement appelle
`worker.restart`, et `apply` contrôle `pending_commands` : la dépendance est explicite.
Le test utilise maintenant sa barrière `settle` après « Nouveau », sans délai arbitraire
et sans supprimer la garde. Le [journal initial](structure-2026-11-initial-failure.log)
est conservé. Les vingt répétitions et la passe complète finale réussissent.

## Révision et reconstruction

[Manifeste](structure-2026-11-code-manifest.json) : **582 fichiers**, révision
`sha256:d9bb85ef98e05e3afb0a6819357b470ee9efb48923ebb76f3aeec6e05c139cf5`.
[Delta de sources](structure-2026-11-source-delta.json) : superposer à l’archive
initiale d’import, appliquer les suppressions indiquées, puis vérifier le manifeste.
L’archive Filius, en lecture seule, n’est pas redistribuée.
L’inventaire inclut désormais `xtask`, `.cargo`, les instructions applicables et
la politique de couches, en plus du code et des assets déjà couverts.

Les hashes historiques peuvent se comparer sans reconstruire leur code : lire
les entrées `/snapshots/` de `j7-png-code-manifest.json` et recalculer SHA-256 sur
les mêmes chemins. Les métadonnées de delta documentent les fichiers déplacés ou
supprimés; aucun fichier relais n’est nécessaire dans le code courant.

## Limites

Ce refactor améliore les responsabilités et leurs contrôles; il ne retire pas encore
toutes les I/O natives synchrones du workbench. La politique de graphe ne peut pas
prouver l’absence de toute utilisation de sockets via la bibliothèque standard.
La suite GPU historique complète reste à réconcilier avec les changements J7; la
commande `ui` couvre actuellement PNG. Aucun dialogue natif manipulé manuellement,
parcours autonome trois OS, exécution Windows/Linux ou CI distante n’est annoncé.
J7 reste ouvert, notamment PDF, configuration, exemples et livraison. Aucune publication,
aucun commit/push et aucun recalcul d’audit du référentiel n’ont été effectués.
