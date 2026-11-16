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

## Fenêtres natives

- **macOS** : parcours complet, voir [J8-NATIF-MACOS](J8-NATIF-MACOS.md).
- **Linux (Debian 12 arm64, X11)** : `sns-desktop` compilé dans `rust:1.95-bookworm`,
  lancé sous Xvfb 1440×960 et Openbox avec Vulkan logiciel Mesa (lavapipe,
  `WGPU_BACKEND=vulkan`) et un profil temporaire, piloté par `xdotool`. Parcours
  identique à macOS : carte des missions, « Le câble oublié », câble guidé
  (« Câble posé entre Poste C et Commutateur ! »), Tester, ping 192.168.1.4,
  « Mission réussie ! » avec trois étoiles, récit jusqu’à la réponse en 8 ms, puis
  Expert (commandes avancées, ligne d’exécution, détail technique du récit). Profil
  relu : `level = expert`, `missions = {"forgotten-cable": 3}`.
  [Mission réussie](j8-native-linux-mission.png) · [Niveau Expert](j8-native-linux-expert.png)
  Ce passage a révélé une flèche « → » absente de la police monospace dans le
  détail Expert du récit; elle est remplacée par « -> ».
- **Windows x64** : `sns-desktop.exe` (PE32+, MinGW) démarre sous CrossOver et
  ouvre la fenêtre « Sinesi — Simple network simulator », mais aucune voie
  graphique de wgpu ne rend dans cette émulation : DirectX 12 refuse un shader
  d’egui (« Shader model 5.1+ resource array » non implémenté par vkd3d), OpenGL ne
  trouve pas d’adaptateur, Vulkan/MoltenVK laisse la surface blanche.
  [Journal DirectX 12](j8-windows-crossover-gui-dx12.log) · [Journal OpenGL](j8-windows-crossover-gui-gl.log)
  La manipulation de la fenêtre Windows demande donc une machine Windows réelle
  (procédure identique : profil temporaire, carte des missions, mission, ping);
  la logique de cette cible est couverte par les 15 tests exécutés sous CrossOver.
