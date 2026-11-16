# Qualification J7 — clavier et paquet Debian amd64

F-CFG-007 reste implémentée et F-NFR-007 reste en cours. Les lots
RS-PLATFORM-J7 et RS-QUALITY-J7 restent ouverts.

| Environnement | Recette | Résultat |
| --- | --- | --- |
| macOS 27.2 arm64 | Harnais AccessKit à 150 %, FR EN ES IT PT DE × Icônes/Schéma | 12 combinaisons. Tab et Enter pour l’édition, raccourcis de sauvegarde et de simulation, relecture du projet, trace complète de ping identique |
| Debian 12 arm64, Docker | Même test de harnais en release | 12 combinaisons réussies |
| Debian 12 amd64, Docker sous émulation | Même test de harnais en release | 12 combinaisons réussies |
| Debian 12 amd64, Docker sous émulation | Paquet release, GIO et Xvfb | 12 démarrages, sauvegarde, ping, checkpoint, rejeu et PNG. Projet préservé après désinstallation. GIO passe le chemin Unicode au bon processus |
| Windows x64 | Test d’association préparé pour l’installateur | PowerShell 7.4.20 analyse le script et compile l’interop C# sur macOS arm64. La vérification d’alignement porte sur la structure 64 bits |
| macOS natif | Ouverture par Finder | Session verrouillée pendant la tentative |

Le test AccessKit utilise les entrées clavier du harnais et observe le focus
après Tab. La topologie est modifiée, relue depuis un fichier temporaire puis
simulée. Il qualifie le parcours egui. La navigation dans les dialogues natifs,
l’usage des technologies d’assistance de chaque OS et le passage réel par
l’Explorateur Windows restent à qualifier. Le workflow trois OS contient le
test de harnais. Le contrôle du workflow est local.

## Reproduction

Sur macOS arm64 et dans les conteneurs Debian 12, Rust 1.95.0 :

```sh
cargo test --release -p sns-ui-egui --test lan enlarged_keyboard_only_project_roundtrip --locked --offline
cargo clippy -p sns-ui-egui --test lan --locked --offline -- -D warnings
```

Pour le paquet, compiler dans `rust:1.95-bookworm` avec
`--platform linux/amd64` et les dépendances système Debian du workflow.
Ouvrir ensuite un conteneur `debian:bookworm-slim` de même architecture, avec
Python, GIO, Xvfb, xauth et Mesa Vulkan. Monter les sources en lecture seule et
un répertoire de travail externe. Depuis la racine des sources :

```sh
cargo fetch --locked
cargo build --release --locked --offline -p sns-desktop -p sns-cli
python3 packaging/build.py --output /out/packages
sh packaging/smoke-deb.sh /out/packages/sinesi_0.1.0_amd64.deb /out/exercise
```

La commande de smoke s’exécute dans le conteneur de qualification, après les
trois commandes de construction. Pour le contrôle statique Windows, analyser
`packaging/smoke-windows.ps1` avec `Parser.ParseFile`, compiler le littéral
`SinesiShellSmoke` avec `Add-Type`, puis vérifier que
`Marshal.OffsetOf(ShellExecuteInfo, "ProcessHandle")` donne 104 en 64 bits.
L’appel au shell et le contrôle du registre requièrent Windows.

[Clavier macOS](j7-qualification-repository.log) ·
[Clavier Debian arm64](j7-qualification-arm64-keyboard-final.log) ·
[Clavier Debian amd64](j7-qualification-amd64-keyboard.log) ·
[Construction amd64](j7-qualification-amd64-build.log) ·
[Essai du paquet](j7-qualification-amd64-smoke.log) ·
[Empreinte du paquet](j7-qualification-packages.sha256.txt) ·
[Contrôle statique Windows](j7-qualification-windows-static.log) ·
[Manifeste](j7-qualification-code-manifest.json) ·
[Reconstruction](j7-qualification-source-patch.json)
