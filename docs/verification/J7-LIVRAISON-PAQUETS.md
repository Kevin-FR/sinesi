# Qualification partielle de la livraison J7

Deux paquets locaux sont construits à partir des binaires Rust 1.95.0 en
profil release. Les empreintes permettent d’identifier les fichiers essayés.
[Sommes SHA-256](j7-delivery-packages.sha256.txt)

| Cible | Preuve obtenue | Qualification restante |
| --- | --- | --- |
| Debian 12 arm64 | Installation du `.deb`, type MIME reconnu, lancement GIO avec le chemin associé sous Xvfb et Vulkan Mesa. Le retrait supprime le binaire et les inscriptions du paquet. Le SHA-256 du projet reste identique. [Journal](j7-delivery-linux-smoke.log) | Parcours interactif depuis le gestionnaire de fichiers, sauvegarde et accessibilité |
| macOS 27.2 arm64 | ZIP extrait, bundle et signature ad hoc vérifiés, UTI et extension inspectés. Le lanceur ouvre le chemin explicite. La suppression de l’application extraite préserve le projet. [Journal](j7-delivery-macos-smoke.log) | Événement Finder et parcours interactif sur session déverrouillée. Signature éditeur et notarisation de diffusion |
| Windows x64 | Recette Inno Setup, inscription du ProgID et vérification du retrait préparées dans la matrice | Compilation, installation, association et parcours bureau à exécuter |

Chaque paquet essayé passe les douze combinaisons FR EN ES IT PT DE avec
Icônes/Schéma, la lecture d’un fichier `.sinesi` Unicode, la sauvegarde
du projet importé, le ping, le checkpoint, le rejeu et l’export PNG. Ces
parcours utilisent le lanceur et le CLI emballés. Les assertions de rendu
portent sur les exports CPU et les tests du bureau restent distincts.

Les premiers essais ont détecté le chargement de `libxkbcommon-x11.so`
à l’ouverture du bureau sous Debian et un conflit entre `Sinesi` et
`sinesi` sur un volume Mac insensible à la casse. Le paquet final déclare
la bibliothèque de clavier et distingue les noms du lanceur et du CLI.
Les journaux finaux portent sur les paquets reconstruits.

## Reproduire

Les commandes natives et les binaires requis sont dans le
[guide de livraison](../rust/LIVRAISON_J7.md). La compilation Linux a utilisé
`rust:1.95-bookworm` arm64 et la qualification un conteneur séparé
`debian:bookworm-slim` arm64. Les digests et le manifeste du code sont
[enregistrés](j7-delivery-environment.json).

```sh
cargo build --release --locked --offline -p sns-desktop -p sns-cli
python3 packaging/build.py --output artifacts/packages
# Dans un conteneur Debian jetable avec python3, GIO, xvfb, xauth et Mesa Vulkan
sh packaging/smoke-deb.sh artifacts/packages/sinesi_0.1.0_arm64.deb artifacts/package-exercise
# Après extraction du ZIP dans un emplacement temporaire
codesign --verify --deep --strict artifacts/staged/Sinesi.app
python3 packaging/smoke.py --desktop artifacts/staged/Sinesi.app/Contents/MacOS/sinesi-launcher --cli artifacts/staged/Sinesi.app/Contents/MacOS/sinesi --exercise-dir artifacts/package-exercise
cargo test -p sns-desktop --locked --offline
cargo test -p sns-ui-egui --lib document_open --locked --offline
cargo xtask check
python3 scripts/docs.py render
python3 scripts/docs.py check --require-reference
```

Le workflow à la demande décrit les commandes automatisées de la matrice.
La preuve locale du Finder est limitée par le verrouillage de la session Mac.
Le workflow distant et l’installateur Windows restent à exécuter.

[Tests du dépôt](j7-delivery-repository.log) ·
[Contrôles et registres](j7-delivery-docs.log) ·
[Manifeste](j7-delivery-code-manifest.json) ·
[Correctif reconstructible](j7-delivery-source-patch.json)
