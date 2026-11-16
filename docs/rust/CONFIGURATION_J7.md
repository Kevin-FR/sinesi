# Adaptateur de configuration et lanceur

F-CFG-004 et F-CFG-005 / RS-PLATFORM-J7. Le
[profil portable](../agnostique/CONFIGURATION_J7.md) et [D-013](decisions/D-013-configuration-j7.md)
fixent les comportements et leurs limites.

`sns-settings::configuration::resolve` est l’unique chemin de résolution de démarrage.
`Sources` fournit les deux chemins et les valeurs CLI; `ResolvedConfiguration` conserve
`effective`, `persistent`, `origins`, `diagnostics` et `user_writable`. La lecture est
bornée, les erreurs sont structurées et les valeurs invalides n’apparaissent pas dans
les diagnostics. Les nombres intégraux sont normalisés avant la désérialisation des
préférences. `sns-settings::load` reste l’API stricte de données, distincte du repli de
démarrage; validation et bornes d’affichage sont communes.

`Workbench::with_configuration` valide la frontière typée et partage l’initialisation
avec les constructeurs explicites de stockage. `configuration_ui.rs` expose les
problèmes localisés et leurs chemins dans une section repliée; le panneau Préférences
permet de rouvrir ces diagnostics. La sauvegarde garde l’état avant CLI puis copie
les champs explicitement modifiés. Un reset explicite remplace le profil d’affichage
complet, même lorsqu’une option masque ses valeurs. La géométrie n’est enregistrée
qu’après un geste sur le titre ou le bord ayant effectivement déplacé/redimensionné
la fenêtre. Le suivi de pointeur est transitoire et lié à l’identifiant de fenêtre.

La comparaison de destinations résout les parents existants et les suffixes absents,
avec traversée `..` et résolution des liens rencontrés après cette traversée. Les
entrées administrateur sont protégées des sauvegardes/exportations; les collisions
profil/projet et profil/administrateur interdisent l’écriture du profil. Les permissions
et modifications concurrentes du système de fichiers restent des causes d’échec I/O
traitées par l’écriture atomique et son retour d’erreur.

## Emplacements

| Plateforme | Administrateur | Utilisateur |
| --- | --- | --- |
| Windows | `%PROGRAMDATA%/Sinesi/config.json` | `%APPDATA%/Sinesi/settings.json` |
| macOS | `/Library/Application Support/Sinesi/config.json` | `~/Library/Application Support/Sinesi/settings.json` |
| Linux | `/etc/sinesi/config.json` | `$XDG_CONFIG_HOME/sinesi/settings.json`, sinon `~/.config/sinesi/settings.json` |

`SINESI_ADMIN_CONFIG`, `SINESI_SETTINGS_PATH` et `SINESI_SESSION_DIR` sélectionnent des
chemins. Les options explicites `--config`, `--settings`, `--session-dir` les remplacent.
Un `SINESI_ADMIN_CONFIG` absent sur disque est diagnostiqué car il est explicitement
demandé; une configuration système par défaut absente est normale.

## Commandes réalisées

```sh
cargo run --locked --offline -p sns-desktop -- --help
cargo run --locked --offline -p sns-desktop -- --version
cargo run --locked --offline -p sns-desktop -- --check-config --config fixtures/valid/configuration.json --settings fixtures/valid/preferences-v1.json --language de --ui-scale 150 --view diagram fixtures/valid/lan.json
```

Le module `apps/sns-desktop/src/launch.rs` porte parsing, métadonnées de l’aide et
préparation. `--help` / `-h`, `--version`, `--check-config`, `--config PATH`,
`--settings PATH`, `--session-dir PATH`, `--workdir PATH` / `-w PATH`,
`--language fr|en|es|it|pt|de`, `--ui-scale 80..200`, `--font default|noto_sans`,
`--view icons|diagram`, `--layout inspector_right|inspector_left|tabs`,
`--window-width 840..3840` et `--window-height 600..2160` sont raccordés.
Les valeurs utilisent un argument séparé. `--` termine les options et permet un
nom de projet commençant par un tiret. La dernière occurrence gagne; une dernière
valeur invalide retombe sur la couche de fichier, pas sur une occurrence CLI antérieure.

Le répertoire de travail résout tous les chemins relatifs, y compris ceux sélectionnés
par l’environnement, sans changer le cwd global du processus. S’il est invalide, le
cwd initial est conservé avec diagnostic. Les chemins restent `OsString`/`PathBuf`;
seules les valeurs scalaires sont décodées comme texte. Le contrôle JSON omet les
chemins natifs, qui peuvent ne pas être sérialisables en UTF-8. Le test du parseur
conserve leurs octets; un test de fichier non UTF-8 est réservé au système Linux qui
l’admet. Sur le macOS local, la création de ce nom est refusée par le système de fichiers;
le rapport avec un chemin absent non UTF-8 reste testé sans sérialisation avec perte.

`--check-config` passe par le même constructeur que le démarrage, avec cache désactivé,
valide le projet demandé et produit `startup_report_version: 1`. Il n’ouvre pas de
fenêtre, ne sauvegarde pas et ne commande pas d’exécution du moteur. Une configuration rejetée
est signalée sur stderr et dans le rapport, avec code de sortie 0 si le projet est
valide; syntaxe ou projet invalide donnent 1. L’aide et les diagnostics sont traduits.
Le produit courant est construit avant la création de la fenêtre native : aucune
lecture de configuration ou de projet ne s’ajoute à sa boucle de rendu.

Le prototype J0 reste explicitement isolé par son option interne existante, absente
de l’aide produit. Il ne reçoit pas les nouvelles fonctions du workbench.

## Apparence et fenêtre compacte

Le lanceur choisit `egui::ThemePreference::System` pour `-n` et
`egui::ThemePreference::Dark` par défaut. Le harnais injecte le thème hôte
dans une entrée de rendu, puis vérifie la bascule de style. Le workbench
applique la police choisie aux deux thèmes.

Le résolveur applique `-s` après la capture des préférences persistantes et
avant les champs explicites de la CLI. Le préréglage compact utilise le même
validateur de profils. Le constructeur de viewport donne à ce mode un minimum
de 640 × 480, avec les dimensions initiales prises dans le profil effectif.
