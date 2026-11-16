# J8 — parcours élève et missions sur macOS, Linux et Windows

Complément de [J8-UX-NIVEAUX](J8-UX-NIVEAUX.md), [J8-MISSIONS](J8-MISSIONS.md) et
[J8-NATIF-MACOS](J8-NATIF-MACOS.md) pour F-UX-001 à F-UX-006 et F-ADV-004.

| Système | Procédure | Résultat |
| --- | --- | --- |
| macOS 27.2 arm64 | Suites complètes (`cargo xtask ci` équivalent, captures GPU) et recette native de la fenêtre pilotée par accessibilité | pass — [recette native](J8-NATIF-MACOS.md) |
| Debian 12 arm64 | Conteneur `rust:1.95-bookworm`, paquets `pkg-config libwayland-dev libxkbcommon-dev libdbus-1-dev libudev-dev`, `CARGO_BUILD_JOBS=2`, `cargo test -p sns-missions -p sns-settings -p sns-topology -p sns-ui-egui -p sns-cli --locked --offline` | 138 tests, 0 échec — [journal](j8-linux-debian-arm64-tests.log) |
| Windows x64 | Compilation croisée `x86_64-pc-windows-gnu` (MinGW dans `rust:1.95-bookworm`), binaires PE32+ de `advice`, `missions` et `student`, exécutés dans une bouteille CrossOver temporaire `win10_64` | 15 tests, 0 échec — [journal](j8-windows-crossover-tests.log) |

Les tests Linux et Windows sont ceux du harnais : ils pilotent la même interface et
le même moteur que l’application, sans capture GPU (les références PNG sont
comparées sous macOS). Le premier essai Linux a échoué à l’édition de liens faute
de mémoire (liens parallèles dans un conteneur de 7,7 Go); la reprise avec deux
tâches passe. Les tests de contrats qui lisent des fichiers du dépôt à l’exécution
ne sont pas lancés sous Windows (chemins du conteneur de compilation).

Limite : la fenêtre native n’a été manipulée que sous macOS; sous Linux et Windows,
la preuve est celle du harnais et du moteur sur chaque système.
