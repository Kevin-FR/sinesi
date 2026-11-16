# Qualification partielle J7 — recette clavier macOS

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
Les prérequis F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés.

Le mode `--keyboard` ouvre le projet de chaque combinaison via
LaunchServices, avec des préférences et une récupération propres à
l’exercice. À 150 %, il cherche le focus AX après Tab et ne valide le
contrôle avec Entrée que si le focus appartient au processus testé.
Il ajoute un portable, enregistre avec Commande+S, entre en simulation
et recherche la réponse ICMP dans les échanges. Les événements clavier
sont envoyés au PID de ce bureau et le pilote contrôle le premier plan
avant chaque touche. La recette relève les empreintes de tous les fichiers
d’exercice avant et après retrait du bundle.
[API Accessibilité](https://developer.apple.com/documentation/applicationservices/axuielement_h)

| Contrôle exécuté | Résultat |
| --- | --- |
| Compilation Swift sur macOS 27.2 arm64 | Pilotes de lancement et de clavier compilés sans avertissement, cible minimale macOS 13. Un appel sans arguments s’arrête avant ouverture. |
| Préparation et relecture des profils | Le binaire Rust du ZIP relit les douze profils FR EN ES IT PT DE × Icônes/Schéma à 150 %. Le projet Unicode à quatre équipements et trois câbles garde ses octets. |
| Recette du ZIP par chemin explicite | L’import, le ping, le checkpoint, le rejeu et les PNG passent. Les huit fichiers d’exercice gardent leurs empreintes après retrait. |
| Workflow | `macos_keyboard` est désactivé par défaut. Il sélectionne le mode clavier, qui inclut l’ouverture via LaunchServices. |

La session macOS locale est verrouillée. Le contrôle AX et l’envoi de
touches n’ont pas été exécutés. L’ouverture via Finder et la qualification
native Windows restent également ouvertes.

## Exécution native prévue

Dans une session graphique macOS de test disposant de l’autorisation
Accessibilité pour le pilote, utiliser un nouveau dossier d’exercice :

```sh
sh packaging/smoke-macos.sh \
  artifacts/packages/j7-macos-launchservices/Sinesi-0.1.0-macos-arm64.zip \
  /tmp/sinesi-macos-native-qualification --keyboard
```

Le contrôle d’autorisation précède le lancement de la fenêtre. Si le
système demande d’autoriser le binaire de test lui-même, le pilote
compilé reste dans le dossier d’exercice. Une fois autorisé, relancer
ce même pilote avec le bundle extrait, le fichier `.sinesi` dans
`exercise`, le chemin du dossier `exercise` et l’option `--keyboard`.
Le parcours par Finder reste une vérification distincte.

[Journal local et profils](j7-macos-keyboard-local.log) ·
[Manifeste](j7-macos-keyboard-code-manifest.json) ·
[Reconstruction](j7-macos-keyboard-source-patch.json)
