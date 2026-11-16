# Qualification J7 — clavier de la fenêtre installée sous Debian

Tranche F-CFG-007 et F-NFR-007, lots RS-PLATFORM-J7 et RS-QUALITY-J7.
Les prérequis F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés. Le défaut de
sauvegarde au clavier a été observé lors de l’exercice natif Debian. Une
pression et un relâchement rapides de Ctrl pouvaient laisser la frame lire
les modificateurs au repos. Les deux commandes de sauvegarde s’appuient
désormais sur les modificateurs de l’événement et donnent la priorité à
Enregistrer sous. Le test de harnais échoue avant correction et passe après,
sur les six langues et les deux vues. [Avant](j7-native-input-shortcut-before.log) ·
[Après](j7-native-input-shortcut-after.log)

Le premier paquet du nouvel essai avait pris les exécutables macOS présents
sous le répertoire Cargo par défaut. Le fabricant Debian contrôle maintenant
le format ELF 64 bits et l’architecture des deux binaires. Les essais négatifs
refusent les exécutables macOS sur les deux cibles, ainsi que les exécutables
Linux amd64 pour le paquet arm64. [Contrôles de fabrication](j7-native-input-packaging.log)

## Parcours et portée

La recette Linux ouvre le fichier `.sinesi` via GIO dans la session X11.
Elle vérifie le PID et le chemin remis au processus. Une sonde
[AT-SPI](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/) lit le
focus pendant la navigation réelle par Tab et Enter. À 150 %, le scénario
ajoute un portable, enregistre par Ctrl+S et relit le projet. Il passe en
simulation par F3, sélectionne Poste A, lance le ping au clavier et observe
`icmp.echo.reply` dans les échanges. Le retrait vérifie les empreintes des
projets, préférences et données de récupération de l’exercice.

La recette s’exécute sous Debian 12 avec Xvfb, Openbox, Vulkan Mesa et un bus
D-Bus privé. Elle couvre l’interface native X11 et son arbre d’accessibilité.
Debian arm64 et amd64 passent chacun les douze combinaisons et préservent
les 61 fichiers recensés après désinstallation. Le conteneur amd64
s’exécute sous émulation sur l’hôte arm64.
[Journal Debian arm64](j7-native-input-debian-arm64.log) ·
[Journal Debian amd64](j7-native-input-debian-amd64.log)
La combinaison avec un lecteur d’écran en session de bureau réelle et la
qualification Wayland sont distinctes. Le ZIP macOS arm64 est reconstruit,
signé ad hoc, contrôlé avec `codesign` et testé par chemin explicite dans
les douze combinaisons. La session Finder est verrouillée. Le parcours
natif Windows reste à exécuter.

## Commandes

Préparer les dépendances du [workflow de paquets](../../.github/workflows/packages.yml)
dans un conteneur Debian 12 de l’architecture voulue. Avec Rust 1.95.0 :

```sh
cargo fetch --locked
cargo build --release --locked --offline -p sns-desktop -p sns-cli
python3 packaging/build.py --bin-dir /target/release --output /out/packages
sh packaging/smoke-deb.sh /out/packages/sinesi_0.1.0_arm64.deb /out/exercise
```

Les volumes Cargo des deux architectures sont séparés et `CARGO_TARGET_DIR`
est fixé à `/target`. Utiliser le paquet `amd64` sous le conteneur x86_64.
La commande de smoke s’exécute dans un conteneur sans Sinesi installé.

Sous macOS 27.2 arm64 :

```sh
cargo test -p sns-ui-egui --test lan --locked --offline
cargo clippy -p sns-ui-egui --all-targets --locked --offline -- -D warnings
cargo xtask ui
cargo build --release --locked --offline -p sns-desktop -p sns-cli
python3 packaging/build.py --output /tmp/sns-j7-native/packages-macos
```

[Tests Rust et Clippy](j7-native-input-repository.log) ·
[25 tests graphiques](j7-native-input-ui.log) ·
[ZIP macOS](j7-native-input-macos.log) ·
[Empreintes des paquets](j7-native-input-packages.sha256.txt) ·
[Manifeste](j7-native-input-code-manifest.json) ·
[Reconstruction](j7-native-input-source-patch.json)
