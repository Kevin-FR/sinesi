# Qualification partielle J7 — fabrication Windows x64

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7. Les
prérequis F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés.

| Vérification | Résultat |
| --- | --- |
| Compilation croisée | Rust 1.95.0, cible `x86_64-pc-windows-gnu`, MinGW dans Debian 12 arm64. Les deux exécutables sont des PE32+ x86-64. |
| Fabrication | Inno Setup 6.7.4 compile `packaging/windows/Sinesi.iss` sous CrossOver 26.1.0. Le paquet contient les deux exécutables, la licence du projet et celle des fontes. |
| Installation | Les octets des exécutables installés correspondent aux entrées. Le ProgID et l’entrée Ouvrir avec sont présents. La commande du registre contient `-- "%1"`. |
| Démarrage et données | FR EN ES IT PT DE × Icônes/Schéma, import, sauvegarde portable, ping, checkpoint, rejeu identique et PNG dans les deux vues. |
| Désinstallation | Exécutable, ProgID et entrée Ouvrir avec retirés. L’empreinte du projet Unicode est préservée. |
| Harnais clavier Windows x64 | Binaire de test compilé avec LLVM/MinGW pour `x86_64-pc-windows-gnullvm`. Le test à 150 % passe sous CrossOver sur les douze combinaisons. |

L’essai utilise une bouteille temporaire CrossOver sous macOS arm64.
Les douze démarrages utilisent `--check-config`, puis la recette
portable exerce le CLI installé. Il s’agit d’une preuve de fabrication,
de registre et de compatibilité. L’exécution native avec la chaîne MSVC,
l’ouverture via l’Explorateur et le parcours clavier natif restent ouverts.
Le Mac est verrouillé et le passage par Finder reste à exécuter.

Le workflow Windows provisionne le compilateur avec une version et une
empreinte fixes. Inno Setup ne figure pas dans l’inventaire consulté du
runner. Le fabricant refuse les fichiers qui ne sont pas des PE32+ x64.
[Image Windows 2025](https://github.com/actions/runner-images/blob/42d27e4a2c1232e5640a88ed42a974c8918ecd28/images/windows/Windows2025-Readme.md) ·
[Inno Setup 6.7.4](https://github.com/jrsoftware/issrc/releases/tag/is-6_7_4)

## Reproduction

Dans `rust:1.95-bookworm` arm64, installer
`gcc-mingw-w64-x86-64-posix` et `binutils-mingw-w64-x86-64`, monter la
racine du dépôt dans `/src` et fixer `CARGO_TARGET_DIR=/target`.

```sh
rustup target add x86_64-pc-windows-gnu
cargo fetch --locked --target x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu --release --locked --offline -p sns-desktop -p sns-cli
```

Copier `sns-desktop.exe` et `sinesi.exe` depuis
`/target/x86_64-pc-windows-gnu/release` vers `/tmp/sns-j7-win64`.
Vérifier l’empreinte SHA-256 de l’installateur Inno avec la valeur inscrite
dans le workflow. Créer la bouteille dédiée avec `cxbottle --bottle
SinesiJ7QA --create --template win10_64` et
`CX_BOTTLE_PATH=/tmp/sns-j7-win64/bottles`. Installer le compilateur dans
`C:\Inno6` avec `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`. Lancer
`C:\Inno6\ISCC.exe` dans cette bouteille, avec les définitions
`AppVersion=0.1.0`, `BinDir` pointant vers les exécutables et `OutputDir`
vers le répertoire `cx-installer`. Les chemins de cette commande figurent
dans le journal du compilateur. Depuis la racine du dépôt, exécuter
`sh docs/verification/j7-windows-compatibility-smoke.sh`. Les deux shims
du script convertissent les chemins Unix en chemins Wine et exécutent
les binaires installés.

[Journal de compilation croisée](j7-windows-compatibility-build.log) ·
[Journal Inno](j7-windows-compatibility-inno.log) ·
[En-têtes PE](j7-windows-compatibility-pe.log) ·
[Journal de compatibilité](j7-windows-compatibility-smoke.log) ·
[Empreinte du paquet d’essai](j7-windows-compatibility-package.sha256.txt) ·
[Manifeste](j7-windows-compatibility-code-manifest.json) ·
[Reconstruction](j7-windows-compatibility-source-patch.json)

Pour le harnais seul, installer la cible `x86_64-pc-windows-gnullvm` et
utiliser le linker `x86_64-w64-mingw32-clang` de LLVM/MinGW UCRT
20261104 via `CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER`. Construire
avec `cargo test --target x86_64-pc-windows-gnullvm --release --locked
--offline -p sns-ui-egui --test lan --no-run`. Copier le binaire de
test et le `libunwind.dll` du compilateur dans la bouteille puis
exécuter le test `enlarged_keyboard_only_project_roundtrip_and_ping_match_in_all_views`.
Cette DLL n’entre pas dans l’installateur GNU.
[Journal du harnais](j7-windows-compatibility-keyboard.log)
