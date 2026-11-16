# Qualification partielle J7 — bundle et recette LaunchServices macOS

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7. Les
prérequis F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés.

| Contrôle | Résultat local |
| --- | --- |
| Fabrication et format | ZIP signé ad hoc. Les deux binaires et le lanceur portent l’architecture arm64. Un PE Windows mis à la place du binaire de bureau est refusé avant création du ZIP. |
| Association déclarée | Identifiant `fr.sinesi.Sinesi`, lanceur `sinesi-launcher`, type `fr.sinesi.project` et extension `.sinesi`. Un bundle signé déclarant aussi `.json` est refusé. |
| Démarrage et retrait | FR EN ES IT PT DE × Icônes/Schéma par chemin explicite. Import, sauvegarde, ping, checkpoint, rejeu et PNG. L’empreinte du projet reste identique après retrait du bundle. |
| Pilote LaunchServices | Compile avec une cible macOS 13. Le test d’arguments échoue proprement avant appel à LaunchServices. Une sonde séparée vérifie l’identification du processus enfant et son chemin Unicode. |

## Reproduction locale

macOS 27.2 arm64, outils Xcode/Swift et binaires Rust 1.95.0 :

```sh
cargo build --release --locked --offline -p sns-desktop -p sns-cli
python3 packaging/build.py --output /tmp/sinesi-macos-packages
sh packaging/smoke-macos.sh /tmp/sinesi-macos-packages/Sinesi-0.1.0-macos-arm64.zip /tmp/sinesi-macos-exercise
```

En session graphique, ajouter `--association` et choisir un nouveau dossier
d’exercice. Le pilote ouvre le document avec le bundle extrait via NSWorkspace.
Il choisit une nouvelle instance, isole les préférences et vérifie le processus
enfant et sa fenêtre. Il ferme l’enfant et attend la fin du lanceur avant
le retrait. Le workflow expose la même option, désactivée par défaut.
[API NSWorkspace](https://developer.apple.com/documentation/appkit/nsworkspace/open(_:withapplicationat:configuration:completionhandler:))

La recette graphique demeure à exécuter. L’ouverture explicite avec
LaunchServices et la sélection du gestionnaire par défaut dans Finder
couvrent des étapes distinctes. La qualification native au clavier demeure
ouverte sur macOS et Windows.

[Fabrication et contrôle du bundle](j7-macos-launchservices-smoke.log) ·
[Cas de rejet](j7-macos-launchservices-negative.log) ·
[Empreinte du ZIP](j7-macos-launchservices-package.sha256.txt) ·
[Manifeste](j7-macos-launchservices-code-manifest.json) ·
[Reconstruction](j7-macos-launchservices-source-patch.json)
