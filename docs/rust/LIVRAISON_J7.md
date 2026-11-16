# Livraison desktop J7

F-CFG-007 et F-NFR-007 sont suivies dans RS-PLATFORM-J7 et RS-QUALITY-J7.
Les formats et critères utilisateur figurent dans le [contrat portable](../agnostique/PROJETS_J7.md#installation-et-association).

| Plateforme | Distribution | Association |
| --- | --- | --- |
| Debian 12 et distributions compatibles | `.deb` avec bureau et CLI, construit pour l’architecture native | MIME dédié, entrée bureau qui passe `-- %f` au lanceur |
| macOS 13 et ultérieur | `Sinesi.app` dans un ZIP, avec bureau et CLI | UTI exporté et délégation AppKit des ouvertures au binaire de bureau |
| Windows x64 | Installateur Inno Setup par utilisateur, avec bureau et CLI | ProgID dans HKCU et commande d’ouverture avec chemin entre guillemets |

Le lanceur AppKit reste actif pendant la vie des fenêtres qu’il a lancées.
Chaque document reçu est ouvert dans son propre processus Rust. Un appel en
ligne de commande du lanceur transmet les arguments et le code retour du
programme. Le CLI emballé possède son propre exécutable. Linux et Windows
utilisent directement le binaire Rust.

La déclaration Mac suit la [documentation AppKit](https://developer.apple.com/documentation/appkit/nsapplicationdelegate/application(_:openfiles:)).
L’entrée Linux suit la [spécification Desktop Entry](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html).
L’inscription Windows suit [Default Programs](https://learn.microsoft.com/en-us/windows/win32/shell/default-programs)
et [ChangesAssociations](https://jrsoftware.org/ishelp/index.php?topic=setup_changesassociations).

## Fabriquer

Préparer le cache puis compiler sur l’OS et l’architecture cibles. La fabrication
n’utilise pas de service externe. Debian requiert `dpkg-deb`, macOS les outils
Swift/AppKit et Windows Inno Setup 6 avec son compilateur ISCC. Le workflow
`Sinesi packages` prépare une matrice lancée à la demande et installe
Inno Setup 6.7.4 avec vérification SHA-256 sur le runner Windows.

```sh
cargo fetch --locked
cargo build --release --locked --offline -p sns-desktop -p sns-cli
python3 packaging/build.py --output artifacts/packages
```

Sous Windows, utiliser `python` si le lanceur Python n’est pas nommé
`python3`. Sous Linux, construire dans Debian 12 ou une image compatible
pour conserver la cible glibc du paquet. Si Cargo utilise un autre répertoire
de sortie, le transmettre avec `--bin-dir <répertoire>/release`. Le fabricant
Debian vérifie le format ELF et l’architecture des deux exécutables.
Le fabricant macOS vérifie l’architecture Mach-O des deux exécutables.
Le fabricant Windows vérifie leur format PE32+ et leur architecture x64.
Sur macOS, le ZIP local contient
une signature ad hoc. Une distribution téléchargée exige une qualification de
signature et de notarisation distincte. L’installateur Windows préparé ne porte
pas de signature éditeur.

## Qualifier

`packaging/smoke.py` importe et enregistre un projet dans un répertoire isolé,
le relit avec le lanceur emballé dans les douze combinaisons langue/vue, puis
exerce un scénario, le checkpoint, le rejeu et les deux exports PNG. Le test
Debian s’exécute dans un conteneur jetable avec Xvfb et Vulkan Mesa.
Il utilise une session D-Bus, Openbox, `python3-pyatspi` et `xdotool` pour
exercer la fenêtre X11 installée. GIO ouvre un chemin Unicode commençant par
un tiret. Le test lit le focus AT-SPI après Tab, enregistre à 150 % avec Ctrl+S
et observe `icmp.echo.reply` dans les échanges. Après fermeture, il compare
les empreintes des projets, préférences et fichiers de récupération au retrait.
Le contrôle macOS `packaging/smoke-macos.sh` valide le bundle, la déclaration
exclusive de l’extension `.sinesi` et la recette par chemin explicite. En session
graphique, son option `--association` sélectionne le bundle temporaire via
LaunchServices, attend la fenêtre du processus qui reçoit le document et contrôle
la fin du lanceur. Le workflow expose ce mode via `macos_association`.
[API NSWorkspace](https://developer.apple.com/documentation/appkit/nsworkspace/open(_:withapplicationat:configuration:completionhandler:))

L’option `--keyboard` ajoute les douze parcours à 150 % dans la fenêtre
ouverte via LaunchServices. Elle nécessite une session graphique avec
l’autorisation Accessibilité pour le pilote. Elle contrôle le focus AX
et cible les touches sur le PID du bureau au premier plan. Le workflow
expose ce mode via `macos_keyboard`, désactivé par défaut.

Le test Windows installe et désinstalle dans un espace dédié du runner.
Il appelle le shell avec le ProgID de Sinesi, vérifie le fichier passé à la
fenêtre et contrôle le retrait de l’inscription. Il compare les empreintes
des fichiers d’exercice après retrait. L’application par défaut reste le choix
de l’utilisateur. En session de test interactive, `-Keyboard` ajoute le
parcours UI Automation à 150 % dans les douze combinaisons. Le workflow
expose ce mode via `windows_keyboard`.
[Référence ShellExecuteEx](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-shellexecuteinfow)

La matrice prévoit aussi `enlarged_keyboard_only_project_roundtrip_and_ping_match_in_all_views`.
Le harnais AccessKit lit le focus réel après chaque Tab. Il n’injecte aucune
demande de focus ni clic de souris. Il sauvegarde au clavier dans un fichier
`.sinesi` déjà ouvert et compare le ping sur les douze combinaisons langue/vue.

| Parcours | Résultat actuel |
| --- | --- |
| Debian 12 arm64 | Installation, GIO, parcours clavier et AT-SPI X11 à 150 % dans les douze combinaisons, retrait avec données préservées |
| Debian 12 amd64 | Même parcours dans les douze combinaisons, sous émulation |
| macOS 27.2 arm64 | Signature, architectures, déclaration `.sinesi`, chemin explicite et retrait vérifiés. Pilote LaunchServices et profils clavier contrôlés localement. Finder et clavier natif à qualifier |
| Windows x64 | Installateur de compatibilité GNU exercé sous CrossOver. Les douze profils de la recette UI Automation sont acceptés. Recette MSVC, Explorateur et clavier natifs à qualifier |
| Bureau interactif | Parcours clavier et focus AT-SPI qualifiés sur Debian X11. macOS et Windows restent à qualifier nativement |

[Résultats, commandes et paquets locaux](../verification/J7-LIVRAISON-PAQUETS.md)
[Qualification clavier et Debian amd64](../verification/J7-LIVRAISON-CLAVIER-AMD64.md)
[Parcours de la fenêtre installée](../verification/J7-BUREAU-NATIF-DEBIAN.md)
[Fabrication et compatibilité Windows x64](../verification/J7-WINDOWS-COMPATIBILITE.md)
[Contrôle de bundle et pilote LaunchServices](../verification/J7-MACOS-LAUNCHSERVICES.md)
[Recette clavier Windows](../verification/J7-WINDOWS-CLAVIER.md)
[Recette clavier macOS](../verification/J7-MACOS-CLAVIER.md)
