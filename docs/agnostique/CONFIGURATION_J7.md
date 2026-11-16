# Configuration de démarrage — J7

**F-CFG-004** définit la résolution; **F-CFG-005** conserve séparément le suivi des
options de lancement. Le projet et les préférences d’affichage restent distincts.

## Priorité et durée de vie

Pour chaque champ : **valeur intégrée < administrateur < utilisateur < ligne de
commande**. Une couche absente n’écrase pas la précédente. Les réglages administrateur
sont des valeurs par défaut, sans verrou imposé aux élèves. Les options de lancement
sont temporaires. L’utilisateur peut ensuite changer les réglages dans l’interface.

La résolution conserve l’état effectif, un instantané avant les options temporaires,
l’origine de chaque champ et des diagnostics. Modifier la langue, une fonte ou une
largeur enregistre seulement les champs effectivement modifiés. Les autres options
temporaires ne se retrouvent pas dans le profil. Rétablir explicitement les valeurs
d’affichage par défaut remplace aussi les valeurs utilisateur masquées par une option.
Un clic ordinaire dans l’aide n’enregistre aucune géométrie; un déplacement ou
redimensionnement de fenêtre est une modification explicite de disposition.

Le profil utilisateur 2 est un instantané complet. Une première sauvegarde peut donc
figer les valeurs administrateur héritées. Le profil 1 ne contient que la langue et
hérite des valeurs d’affichage de priorité inférieure. Aucun fichier n’est réécrit
pendant sa lecture ou par le contrôle de démarrage.

## Contrats et validation

- [configuration](../../contracts/configuration.schema.json), version 1 : valeurs
  administrateur facultatives `language` et `display`; `configuration_version` requis.
  Les champs d’affichage sont ceux du [profil portable](AFFICHAGE_J7.md).
- [preferences](../../contracts/preferences.schema.json), versions 1 et 2 : profil
  utilisateur existant. La validation stricte d’un profil reste disponible; la
  résolution de démarrage peut conserver ses champs valides et diagnostiquer les autres.
- [startup-report](../../contracts/startup-report.schema.json), versions 1/2 : état
  effectif, état persistant, origines, diagnostics et résumé du projet chargé. Ce
  rapport ne contient ni paquets, ni fichiers virtuels, ni comptes ou corps de courrier.

Une valeur mal typée, hors limites ou inconnue conserve la valeur valide de la couche
inférieure. Les champs sont indépendants; la table des géométries est un champ atomique.
Les formes JSON intégrales telles que `125.0` ou `1.25e2` désignent l’entier 125.
Les clés et valeurs d’énumération sont sensibles à la casse; en cas de répétition
d’une clé JSON ou d’une option, la dernière occurrence est considérée.

Un fichier malformé, illisible, trop volumineux ou d’une version inconnue est ignoré
avec diagnostic. Une absence administrative est normale par défaut; un fichier
explicitement demandé absent est signalé. Un profil utilisateur absent constitue
un premier démarrage normal. Un profil invalide est conservé en lecture seule,
même si ses valeurs valides restent utiles; l’utilisateur doit le corriger ou choisir
un autre profil avant de sauvegarder des préférences. L’application peut continuer
avec les valeurs sûres. Après correction du fichier, relancer l’application pour
recharger sa configuration.

| Limite | Valeur |
| --- | --- |
| Taille d’un fichier | 65536 octets |
| Diagnostics conservés | 32, avec résumé de débordement |
| Valeurs d’affichage | Bornes du profil d’affichage 2 |
| Argumentaire | 128 arguments maximum |

Les diagnostics portent un code, une couche et un champ connu. Ils ne recopient pas
les valeurs rejetées, les clés inconnues ou des détails de contenu issus d’une erreur
de décodage. Le message est traduit dans la langue effective. Les chemins des fichiers
peuvent être consultés localement dans un volet replié; ils sont omis du rapport JSON.
`user_writable` indique l’autorisation donnée par le format et les rôles, sans prédire
les permissions réelles du système de fichiers.

Administrateur, profil, projet et récupération ont des rôles distincts. Une sauvegarde
ou un export ne doit pas remplacer une entrée de configuration protégée. Les alias
existants et les parents encore absents, y compris `..` et les liens symboliques,
sont résolus avant les comparaisons de destination.

## Lancement actuellement raccordé

L’aide, la version et le contrôle de configuration fonctionnent sans fenêtre. Le
contrôle charge et valide aussi le projet explicitement demandé par le chemin courant
de l’application. Une erreur de configuration permet ce contrôle et le lancement;
une syntaxe d’arguments incohérente ou un projet invalide produit un échec explicite.
Le contrôle n’enregistre pas de profil et n’initialise pas le cache de récupération.

Les options réalisées couvrent choix des fichiers administrateur/utilisateur et du
cache, répertoire de travail, langue, fonte, taille, représentation, panneaux et
dimensions. Les chemins relatifs sont résolus depuis le répertoire de travail choisi;
les chemins absolus sont conservés. Les arguments natifs restent des chemins de
fichiers, sans passage par une commande système ou un encodage texte avec perte.

Les options de lancement couvrent h, v, w, l, r, n et s. Le projet peut être
passé en argument. La version utilise une option longue.
[Facteur d’attente](ATTENTES_J7.md) · [Diagnostic technique](DIAGNOSTIC_J7.md)

`-n` ou `--native-theme` suit les thèmes clair et sombre du système, avec repli
sombre lorsque l’information manque. L’apparence par défaut est sombre.
`-s` ou `--small-screen` propose 840 × 600, une échelle d’interface de 90 %,
une échelle des applications de 85 % et les panneaux en onglets. La fenêtre
compacte peut être réduite à 640 × 480. Les options explicites valides
affinent ce réglage. Une valeur explicite rejetée laisse le préréglage
valide en vigueur. Ces options s’appliquent pour le lancement en cours.

## Recettes

1. Superposer trois fichiers/paramètres contradictoires, vérifier chaque origine,
   puis supprimer une couche et constater le repli attendu.
2. Introduire une valeur invalide parmi des valeurs valides : diagnostic traduit,
   autres champs conservés, lancement possible et original préservé.
3. Démarrer avec un affichage temporaire, modifier seulement la langue ou la fonte,
   puis rouvrir sans option : les autres réglages temporaires n’ont pas été enregistrés.
4. Consulter l’aide sans déplacer sa fenêtre : profil inchangé. Déplacer la fenêtre :
   seule cette modification explicite de disposition devient persistante.
5. Vérifier les rôles de chemins, le projet natif réellement chargé, les six langues
   et les deux représentations; garder distinctes preuve de harnais et exécution native.
