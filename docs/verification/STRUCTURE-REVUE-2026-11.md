# Revue de structure — 9 novembre 2026

Tranche RS-QUALITY-J0, contributions F-NFR-002/008. Les états acquis sont
conservés; RS-QUALITY-J0 et J7 restent ouverts. La
[comparaison et les extractions proposées](../rust/REVUE_STRUCTURE_2026_11.md)
complètent D-011. Les [quatre références figées](structure-review-2026-11-references.json)
conservent leurs empreintes; leurs consignes ne s’appliquent pas au dépôt.

## Changement

Le graphe Cargo est analysé avec toutes les fonctionnalités des membres, incluant
les dépendances optionnelles désactivées par défaut. L’héritage des métadonnées
version/édition/compilateur minimal/licence/dépôt est vérifié en plus des dépendances
et lints. Douze tests couvrent la politique, les manifestes et un workspace Cargo
minimal réel contenant une dépendance GUI optionnelle interdite.

Le [test avant correction](structure-review-2026-11-before.log) échoue bien avec
`ValueError not raised` : l’ancienne résolution laisse passer cette dépendance.
Le [journal ciblé](structure-review-2026-11-tests.log) présente les tests réussis.

## Qualification

Environnement : macOS arm64, Rust 1.95.0, Python 3.14.4;
[détails et comparaison du périmètre](structure-review-2026-11-environment.json).

| Commande exécutée | Résultat |
| --- | --- |
| `python3 scripts/test_layers.py` avant correction | Échec attendu, `ValueError not raised` pour la dépendance optionnelle |
| `python3 scripts/test_layers.py`, `python3 scripts/check_layers.py` | Douze tests réussis; 20 membres conformes avec toutes leurs fonctionnalités |
| `cargo xtask ci` | Clippy sans avertissement; 265 tests workspace plus un enfant du pont, cinq ignorés; quatre exports PNG décodés |
| `python3 scripts/docs.py render`, `python3 scripts/docs.py check --require-reference` | Vues régénérées, cohérence des registres et empreintes de l’archive contrôlées |
| `python3 scripts/check_manifest.py check docs/verification/structure-review-2026-11-code-manifest.json` | Inventaire scellé et code courant identiques |

Le [journal complet](structure-review-2026-11-ci.log) conserve la passe réussie.
La première tentative a arrêté `check` sur un lien vers ce rapport encore absent;
ce défaut documentaire est corrigé avant la passe complète. Son
[journal distinct](structure-review-2026-11-initial-ci.log) est conservé.
Les commandes finales figurent dans le [journal documentaire](structure-review-2026-11-docs.log).

Seuls `AGENTS.md`, `scripts/check_layers.py` et `scripts/test_layers.py` changent
par rapport au manifeste de configuration dans l’inventaire de code. Cargo.lock,
les fichiers Rust, contrats, catalogues et **496 fichiers PNG** restent identiques.
Les tests GPU n’ont pas été relancés pour cette modification des contrôles.
Les états et identifiants sont inchangés; les nouvelles preuves s’ajoutent aux
registres sans remplacer les preuves précédentes.

[Manifeste](structure-review-2026-11-code-manifest.json) : **787 fichiers**,
révision `sha256:40cc7928a7359d07ae577db0e47b05f5701e34c821bade2937ac0adc05cc83de`. Le
[delta reconstructible](structure-review-2026-11-source-delta.json) se superpose
à la base d’import selon ses instructions; les archives de référence ne sont pas
redistribuées. Les rapports et textes de suivi ne font pas partie de l’empreinte
du code; leur version de qualification est conservée dans ce delta.

L’analyse avec toutes les fonctionnalités ne compile pas toutes leurs combinaisons.
Aucune manipulation native, exécution Windows/Linux ou CI distante n’est qualifiée.
Les archives restent en lecture seule, les preuves précédentes sont conservées.
