# Qualification du complément de structure et de méthode

Tranche RS-QUALITY-J0 et F-NFR-008, après la
[comparaison des quatre références](../rust/REVUE_STRUCTURE_2026_11.md).
Les quatre contenus de `main` correspondent aux
[révisions et empreintes archivées](structure-review-2026-11-references.json).

- `cargo metadata --format-version 1 --all-features --locked --offline` exécuté
  avant et après le déplacement des fonctionnalités egui/eframe. Les 521
  paquets, arêtes, fonctionnalités résolues, membres et membre par défaut
  coïncident. `Cargo.lock` conserve son empreinte.
- Le binaire `xtask` d’avant changement transmettait
  `UPDATE_SNAPSHOTS=force` à un exécutable Cargo témoin. Après correction,
  le même témoin reçoit un environnement sans la variable. Un test de
  `xtask` vérifie la construction du sous-processus.
- `UPDATE_SNAPSHOTS=force cargo xtask ui` exécute les 25 tests graphiques
  courants. Les chemins et empreintes des 519 PNG de référence recensés
  sous `crates/sns-ui-egui` coïncident avant et après. Les images
  de diagnostic `.new.png` et `.diff.png` sont exclues de ce décompte.
- `cargo xtask ci` et les contrôles du suivi sont consignés dans les journaux
  communs à la qualification J7.

Les commandes ont été exécutées sous macOS 27.2 arm64, Rust 1.95.0 et
Python 3.14.4. Cette comparaison porte sur les fonctionnalités Cargo
résolues et la suite graphique courante. Les tests natifs J7 gardent
leur qualification propre.

[Reproduction et comparaison](j7-qualification-structure.log) ·
[Suite graphique](j7-qualification-ui.log) ·
[CI locale](j7-qualification-ci.log) ·
[Manifeste](j7-qualification-code-manifest.json)
