# État de reprise — Sinesi

## Situation courante — 16 novembre 2026

**J1 à J6 sont qualifiés localement. J7 reste ouvert. La refonte élève (J8, RS-UX-J8)
est terminée : F-UX-001 à F-UX-006 et F-ADV-004 sont vérifiées sur macOS (harnais,
captures et fenêtre native), Debian 12 arm64 et Windows x64 sous CrossOver (harnais).** FR EN ES IT PT DE et les vues Icônes/Schéma sont conservés.

| Registre de référence | État |
| --- | --- |
| [184 exigences](features.json) | **169 verified**, 1 implemented, 3 in_progress, 11 not_started |
| [51 lots Rust](../rust/taches.json) | **45 verified**, 3 in_progress, 1 not_started, 2 deferred |
| J7 | **20 exigences vérifiées sur 22**, 1 implémentée à qualifier, 1 en cours |
| Lots en cours | RS-QUALITY-J0, RS-PLATFORM-J7, RS-QUALITY-J7 |

Les registres et leurs preuves sont l’autorité. Les résumés historiques ne ferment
pas une exigence; [leur chronologie antérieure](ETAT-HISTORIQUE-2026-11-08.md) est conservée.

## Tranche J8 — recette native macOS et clôture

RS-UX-J8 vérifié. Le parcours élève a été exécuté dans la fenêtre native macOS
pilotée par l’accessibilité : carte des missions, mission « Le câble oublié »,
explication, câble guidé, ping réussi raconté, trois étoiles, passage Expert et
profil relu (`level = expert`, étoiles enregistrées). Les missions d’enseignant
s’ouvrent par le chargement de document asynchrone (section « De la classe »).
Les suites élève et missions passent aussi sous Debian 12 arm64 (138 tests) et
Windows x64 sous CrossOver (15 tests). [Recette native](../verification/J8-NATIF-MACOS.md) ·
[Trois systèmes](../verification/J8-TROIS-OS.md)

## Tranche J8 — missions évaluables et finitions

F-ADV-004 (implemented) dans RS-UX-J8, décision [D-016](../rust/decisions/D-016-missions.md).
Le crate sans interface `sns-missions` (format version 1, contrôles sur projet et
trace, étoiles, correction) est déclaré dans la politique de couches; `sinesi
mission-check` exporte la correction. Trois missions intégrées en six langues, dont
la panne DNS réparée de la recette F-ADV-004, vérifiée sur simulation réelle.
L’atelier ajoute le bouton Missions, la carte par chapitre et la carte de mission;
les étoiles sont un champ facultatif `missions` du profil 2. Contrats
`mission`/`mission-report`. F-UX-006 (fiche pédagogique, avis sur la passerelle)
est implémentée dans la tranche précédente de la même série.

Finitions : placement libre des nouveaux équipements, fenêtre de messagerie qui ne
déborde plus, capture de mission stabilisée. Commandes exécutées : `cargo xtask
check`, `cargo xtask layers`, Clippy (avec et sans captures), `cargo test
--workspace`, suite graphique complète et `cargo xtask ui` (33 tests), sur macOS
arm64. Les missions d’enseignant s’ouvrent depuis la carte par le chargement de document
asynchrone existant (section « De la classe »). Limite : harnais seulement.
Prochaine action : qualification native et trois OS du parcours élève.
[Missions](../verification/J8-MISSIONS.md) · [Parcours élève](../verification/J8-UX-NIVEAUX.md)

## Tranche J8 — paquets animés et récit du trajet

F-UX-004 (implemented) dans RS-UX-J8. En mode Tester, chaque `ethernet.sent` de
la trace devient une pastille colorée qui parcourt son câble (demande orange,
réponse bleue, ARP violet); les départs sont espacés et l’arriéré est borné.
« Le voyage du paquet », sous le canevas, raconte les étapes en langage courant
depuis la trace (`src/story.rs`, lecture seule) et explique les échecs : absence de
passerelle (`routing.no_route`), ARP sans réponse, délai, TTL. Détail technique au
niveau Expert. Nouveaux fichiers : `story.rs`, `workbench/packets.rs`,
`workbench/story_ui.rs`; modifiés : `canvas.rs`, `trace_table.rs`, `mod.rs`,
`lib.rs`, catalogue, `tests/student.rs`. 36 références `j8-story-*`/`j8-packet-*`;
9 familles de références en mode Tester changent avec le panneau. Commandes :
`cargo xtask check`, Clippy (avec et sans captures), `cargo test --workspace`,
suite graphique complète et `cargo xtask ui` (30 tests), en succès sur macOS arm64.
Limite : harnais seulement; l’animation à vitesse réelle n’a pas été observée dans
l’application native.
[Commandes et preuves](../verification/J8-UX-NIVEAUX.md)

## Tranche J8 — palette illustrée et câble guidé

F-UX-003 (implemented) dans RS-UX-J8. La palette montre six tuiles colorées avec
le symbole de la vue courante; leur nom accessible reste « Ajouter un … ». L’outil
Câble est un bouton à bascule et une bulle accessible sur le canevas guide les
deux clics puis confirme le câble posé. Fichiers : `inspector.rs`, `canvas.rs`,
`commands.rs`, `mod.rs`, catalogue, `tests/student.rs` et 12 références
`j8-cable-*`; 23 références changent avec la palette. Commandes exécutées :
`cargo xtask check`, Clippy workspace, `cargo test --workspace`, suite graphique
complète et `cargo xtask ui` (29 tests), toutes en succès sur macOS arm64.
Limite : harnais seulement.
[Commandes et preuves](../verification/J8-UX-NIVEAUX.md)

## Tranche J8 — interface élève : niveaux et trois espaces

F-UX-001 et F-UX-002 (implemented) dans RS-UX-J8, nouveau domaine UX.
[D-015](../rust/decisions/D-015-interface-eleve.md) retient une seule interface :
niveau Élève par défaut, niveau Expert pour import Filius, exécutions et export,
préférence `display.level` facultative du profil v2 (aussi en configuration
administrateur et en option temporaire). Construire/Tester/Annoter remplacent
Conception/Simulation/Documentation avec F2/F3/F4 et une consigne traduite.
Le nouveau module `workbench/toolbar.rs` possède la barre; `chrome.rs` la compose.

Les 519 PNG J0–J7 sont archivés avec manifeste; 315 références sont régénérées,
36 `j8-student-*` ajoutées. Commandes exécutées : `cargo xtask check`, Clippy
workspace, `cargo test --workspace`, suite graphique complète et `cargo xtask ui`
(28 tests), toutes en succès sur macOS arm64. Limite : harnais seulement, pas de
recette native ni trois OS. Écart antérieur absorbé, corrigé depuis (tranche missions) : fenêtre de
messagerie plus étroite, zone de texte rognée (`j6-mail-client-*`).
[Commandes et preuves](../verification/J8-UX-NIVEAUX.md)

## Tranche J7 — recette clavier de la fenêtre macOS

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
Le pilote de qualification macOS ajoute un mode clavier à 150 %.
Il prévoit l’ouverture par LaunchServices, le contrôle AX du focus, la
sauvegarde par Commande+S et la lecture de la réponse ICMP. Les touches
sont destinées au seul processus d’exercice au premier plan.

Le pilote compile pour macOS 13 et le binaire du ZIP relit les douze
profils préparés. La recette de paquet préserve les huit fichiers
d’exercice au retrait. Le Mac est verrouillé. L’ouverture via Finder
et l’exécution clavier native restent à valider sur macOS, ainsi que
l’Explorateur et le clavier sur Windows.
[Commandes et preuves](../verification/J7-MACOS-CLAVIER.md)

## Tranche J7 — recette clavier de la fenêtre Windows

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
La recette Windows prépare les douze profils à 150 %, observe le focus
UI Automation avant d’activer un contrôle et refuse d’envoyer des touches
si le premier plan change. Elle vérifie le projet sauvegardé, la réponse
ICMP et les empreintes de tous les fichiers d’exercice au retrait.

Les douze profils sont relus par le binaire Windows sous CrossOver.
L’analyse PowerShell et le test synthétique de séquence et de perte du
premier plan passent. L’exécution UI Automation et le passage par
l’Explorateur restent à faire dans une session Windows. Finder et le
clavier macOS restent également à qualifier.
[Commandes et preuves](../verification/J7-WINDOWS-CLAVIER.md)

## Tranche J7 — recette LaunchServices macOS

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
Le fabricant macOS contrôle les exécutables Mach-O. La recette valide
la signature du bundle et sa déclaration exclusive de `.sinesi`, puis
les douze démarrages par chemin explicite, l’import et le rejeu. Le
retrait préserve le projet. Un binaire Windows et un bundle déclarant
aussi `.json` sont refusés.

Le pilote LaunchServices compile pour macOS 13 et son option graphique
est disponible dans le workflow. La session locale reste verrouillée.
L’événement d’ouverture, le passage par Finder et le clavier natif
macOS restent à valider, ainsi que l’Explorateur et le clavier sous
Windows. [Commandes et preuves](../verification/J7-MACOS-LAUNCHSERVICES.md)

## Tranche J7 — installateur Windows x64 de compatibilité

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
Les deux exécutables x64 sont compilés avec Rust/MinGW et empaquetés par
Inno Setup 6.7.4. Sous CrossOver, l’installation écrit le ProgID et la
commande de document attendus. Les binaires installés passent les douze
combinaisons de démarrage, l’import, le ping, le rejeu et les PNG. Le retrait
efface les inscriptions testées et conserve l’empreinte du projet.
Le harnais clavier Windows x64 passe aussi sous CrossOver, à 150 %
dans les douze combinaisons.

Le fabricant contrôle les en-têtes PE et le workflow provisionne ISCC.
La recette MSVC, l’Explorateur et le clavier Windows natifs, ainsi que
Finder et le clavier macOS natifs, restent à qualifier.
[Commandes et preuves](../verification/J7-WINDOWS-COMPATIBILITE.md)

## Tranche J7 — fenêtre installée et raccourci de sauvegarde

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
Le raccourci utilise les modificateurs de l’événement clavier. Un Ctrl
relâché avant le rendu ne perd plus la sauvegarde. Le fabricant Debian
refuse les binaires d’un autre format ou d’une autre architecture.

Debian 12 arm64 et amd64 sous émulation passent chacun les douze parcours
GIO, AT-SPI et clavier à 150 %. Le retrait préserve les 61 fichiers de
projet, de préférences et de récupération recensés pour chaque architecture.
Le ZIP macOS reconstruit passe par chemin explicite dans les douze
combinaisons. Finder, Windows et leurs parcours clavier natifs restent
à qualifier. [Commandes et preuves](../verification/J7-BUREAU-NATIF-DEBIAN.md)

## Tranche structurelle — manifestes et preuves graphiques

RS-QUALITY-J0 et F-NFR-008. La comparaison de SolidCraft et WordCraft
confirme le découpage actuel. Les manifests bureau et présentation déclarent
leurs fonctionnalités graphiques, avec la même résolution Cargo sur 521
paquets et le même lockfile. `xtask ui` neutralise le mode de régénération
hérité. Les 25 tests graphiques passent et les 519 PNG de référence recensés
sous le crate UI conservent leurs empreintes. Les statuts restent inchangés.
La prochaine extraction produit vise les sélecteurs d’enregistrement et d’export.
[Comparaison et commandes](../verification/STRUCTURE-COMPLEMENTS-2026-11.md)

## Tranche J7 — qualification bureau partielle

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7.
F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés. Le harnais à 150 % couvre les
douze combinaisons langue/vue sur macOS arm64 et Debian 12 arm64/amd64.
Le paquet Debian amd64 passe la recette GIO sous Xvfb, sous émulation.
Le script Windows vérifie désormais la fenêtre du processus lancé via le
ProgID et le retrait de l’inscription. Son contrôle local porte sur PowerShell
et C#. L’exécution native Windows, l’ouverture Finder en session
déverrouillée et l’accessibilité native restent les prochaines validations.
[Commandes et preuves](../verification/J7-LIVRAISON-CLAVIER-AMD64.md)

## Tranche J7 — qualification du corpus et du courrier

F-PROJ-005 et RS-PROJECTS-J7 sont vérifiés. Les 20 rapports couvrent les
champs convertis, les pertes et les extensions conservées inactives. Le
corpus fournit 14 configurations clientes, 12 comptes serveur, 19 messages
dans les dossiers clients et deux en boîte serveur. Les contacts sont
qualifiés sur une archive synthétique. La relève POP3 simulée récupère les
deux messages serveur avec reprise et rejeu identiques.

Les cinq fichiers modifiés sont dans `sns-filius-import`. Les preuves publient
les rapports et un condensat de la recette POP3, sans corps ni secrets.
Les identités supplémentaires, dates locales et brouillons restent dans les
originaux. Les six langues et les vues Icônes/Schéma sont conservées.
La prochaine porte de sortie est la qualification interactive et Windows.
[Commandes et preuves](../verification/J7-IMPORT-COURRIER.md)

## Tranche J7 — paquets et associations

F-CFG-007 est implémentée et F-NFR-007 reste en cours. La construction native et
la matrice sont dans `packaging/` et `.github/workflows/packages.yml`. Les projets
`.sinesi` utilisent le chargeur existant, avec ouverture des anciens JSON.

Debian 12 arm64 installe le paquet, ouvre le projet via GIO sous Xvfb et le
préserve au retrait. Le ZIP macOS arm64 passe l’inspection de l’association,
le lancement par chemin explicite et le retrait en espace temporaire. Les deux
paquets passent 12 combinaisons langue/vue et les recettes CLI de sauvegarde,
ping, checkpoint, rejeu et export. L’installateur Windows et le Finder en
session déverrouillée restent à qualifier, ainsi que les parcours interactifs
de livraison et d’accessibilité.
[Paquets, commandes et preuves](../verification/J7-LIVRAISON-PAQUETS.md)

## Tranche J7 — import RIP et annotations

Cette tranche couvre 17 routeurs RIP
et 67 annotations avec géométrie et style. Les cinq fichiers de code et de tests
modifiés sont dans `sns-filius-import`. Sur l’exemple Internet, sept routeurs
convergent, un ping traverse les routes apprises à l’aller et au retour, et le
rejeu intégral comme la reprise depuis le checkpoint reproduisent la trace.

Les contrôles comparent les octets des entrées des 20 archives. Les deux PNG
importés ont été décodés et inspectés. Ils montrent des textes tronqués et des
recouvrements sur la topologie dense, avec les textes complets dans le projet.
Les langues FR EN ES IT PT DE et les vues Icônes/Schéma restent couvertes.
La livraison multiplateforme reste à qualifier.
[Commandes et preuves](../verification/J7-IMPORT-RIP-DOCUMENTATION.md)

## Tranche J7 — import NAT et filtrage

Le NAT des passerelles, les
redirections et les politiques de filtrage représentables sont convertis.
Les six fichiers modifiés sont dans `sns-filius-import`. Le corpus conserve
trois redirections et deux politiques de six règles. Les 20 originaux sont
préservés. Le moteur reçoit HTTP 200 derrière une redirection, exerce les
décisions d’autorisation et de refus, puis restaure et rejoue les traces.
Les six langues et les vues Icônes/Schéma restent couvertes par le harnais.

Les travaux RIP et présentation sont suivis dans la tranche suivante.
[Commandes et preuves](../verification/J7-IMPORT-NAT-FILTRAGE.md)

## Tranche J7 — import DHCP et routage statique

L’import convertit les plages DHCP
actives, les réservations valides et les routes statiques manuelles. Le mode client
d’une passerelle reste côté WAN. Son serveur distribue les baux côté LAN.
Les cinq fichiers modifiés se trouvent dans `sns-filius-import`. Les traductions,
les vues et le schéma du rapport restent inchangés.

Sept tests portables, trois tests du corpus et Clippy passent. Les 20 fichiers
conservent les octets de leurs entrées. Le moteur délivre deux baux sur le scénario
DHCP et fait passer un ping historique par les routes statiques dans les deux sens.
[Commandes et preuves](../verification/J7-IMPORT-DHCP-ROUTES.md)

## Tranche structurelle — ouverture de documents

F-PROJ-001 et F-PROJ-005, dans la suite de RS-PROJECTS-J7 et RS-QUALITY-J0.
F-TOP-001 et RS-TOPOLOGY-J1 sont qualifiés. Les sélecteurs d’ouverture et
la lecture du projet passent hors de la frame. Le chemin sélectionné rejoint les
traitements existants d’import, de scénario, de checkpoint ou de rejeu.
L’installation d’un projet est commune à Nouveau, Ouvrir, Exemples et Import.
Une sélection annulée, invalide ou périmée préserve le document courant.

Le changement touche `workbench/document.rs`, ses appelants, le catalogue et
AGENTS. F-PROJ-001 garde son statut vérifié et F-PROJ-005 reste partielle.
Les sélecteurs d’enregistrement et certains exports restent synchrones.
La qualification s’appuie sur les chemins injectés par les tests. Le dialogue
natif reste à vérifier en session macOS déverrouillée.
La prochaine tranche fonctionnelle de J7 vise la qualification du corpus d’import.

`cargo xtask ci` et `cargo xtask ui` réussissent localement avec 287 tests
du workspace, un enfant du pont et 25 tests graphiques. Les 520 PNG conservent
leurs empreintes. Les registres et le manifeste ont été contrôlés.
[Commandes et preuves](../verification/STRUCTURE-OUVERTURE.md)

## Dernière tranche — lancement et exemples

**F-CFG-005 et F-CFG-008 sont vérifiées localement.** `-n` suit le thème
système. `-s` propose 840 × 600, les onglets et une échelle de 90 %, avec
un minimum de fenêtre de 640 × 480. Le lanceur applique les options
explicites valides après le préréglage compact. Les contrôles de lancement
et la politique de persistance sont décrits dans le
[profil portable](../agnostique/CONFIGURATION_J7.md).

Le catalogue partagé par le menu et l’aide comprend 14 projets.
La liaison directe, l’atelier ARP et les deux projets modem s’ajoutent
aux exemples précédents. Les treize recettes correspondent aux familles
de F-CFG-008, avec RIP détaillé à côté des routes statiques.
[Catalogue](../agnostique/EXEMPLES_J7.md) · [Recettes](../rust/EXEMPLES_J7.md)

La suite locale a réussi avec 284 tests du workspace et un enfant du
pont, cinq ignorés et Clippy sans avertissement. La suite graphique
compte 25 tests et 132 références. Douze captures compactes couvrent
FR EN ES IT PT DE et les vues Icônes/Schéma. Le manifeste courant compte
821 fichiers.
[Preuves de qualification](../verification/J7-LANCEMENT-EXEMPLES.md)

## Journal technique

**F-CFG-006 est vérifiée.** La tranche suivante a qualifié n/s. Les options
`-v`, `-l` et `--log-file` sont raccordées au lanceur. Le fichier neuf est
annoncé à son chemin absolu et les rôles du projet, des profils et de la
récupération sont protégés. Dans Exécutions, l’export situe une erreur avec des
codes d’étape prédéfinis. Le fichier de lancement conserve les 256 premières
entrées et l’export manuel les 256 dernières. Le journal n’a accès ni aux
secrets ni au contenu des exercices.

Les fichiers principaux sont le lanceur, `sns-settings/src/diagnostics.rs`,
`sns-project/src/destination.rs` et les opérations d’export de l’atelier.
Clippy et les 278 tests du workspace plus un enfant du pont ont réussi, avec
cinq tests ignorés. La suite graphique compte 23 tests et 120 références.
Les douze nouvelles captures couvrent les six langues et les deux vues.
Le manifeste contient 805 fichiers. Les limites de qualification, commandes et
artefacts figurent dans le [rapport](../verification/J7-DIAGNOSTIC.md).
Le [contrat portable](../agnostique/DIAGNOSTIC_J7.md) et
l’[adaptateur Rust](../rust/DIAGNOSTIC_J7.md) décrivent la réalisation.

## Facteur d’attente RTT

**Le facteur RTT est vérifié.** Le
[contrat portable](../agnostique/ATTENTES_J7.md), l’[adaptateur](../rust/ATTENTES_J7.md),
[D-014](../rust/decisions/D-014-attentes-j7.md) et les
[preuves](../verification/J7-ATTENTES.md) décrivent `-r/--rtt-factor` 1–5. Les
attentes ARP, ICMP, TCP, DHCP et applicatives sont étendues en temps virtuel. Les
câbles, durées de vie, cadence RIP et intervalle du ping du terminal sont conservés.
Sur le réseau lent de recette, le facteur 1 expire avant la réponse ARP; le facteur 5
obtient un ping. Rejeu et continuation reproduisent exactement la trace.

Le moteur, le workbench, les lanceurs et les schémas sont raccordés. L’exécution
standard garde les formats 1; les versions 2 stockent le facteur étendu dans le
journal et le checkpoint. La reprise privilégie la politique enregistrée; un nouveau
réseau reprend celle du lancement. Les préférences ne reçoivent pas cette option.

Qualification locale : 38 tests ciblés, **274 tests workspace plus un enfant du pont**,
cinq ignorés, Clippy sans avertissement; **22 tests GPU et 108 références** comparées.
Six aides et douze rapports natifs sans fenêtre, **851 clés × six langues**;
**496 PNG antérieurs inchangés**. Registres et inventaire contrôlés avec le référentiel.
Le manifeste RTT contient **790 fichiers**, révision `sha256:bbd20f87e9e141f1b10d02655d53e95a0c94199ac7418a8018e905aa7eb2bf05`.
Le workspace reste à 20 membres et aucun paquet tiers ne change.

La tranche suivante a raccordé v/l et qualifié F-CFG-006.

## Dernière revue structurelle — 9 novembre 2026

**RS-QUALITY-J0 reste en cours; F-NFR-002/008 conservent leur qualification.** La
[comparaison actualisée](../rust/REVUE_STRUCTURE_2026_11.md) relit les quatre fichiers
SolidCraft/WordCraft à des révisions figées et ordonne les extractions suivantes.
Le contrôle des couches active désormais toutes les fonctionnalités Cargo : une
GUI optionnelle désactivée par défaut ne peut plus lui échapper. Les métadonnées
des membres héritent aussi de la racine. AGENTS, le guide, la décision D-011 et les
deux scripts de politique sont actualisés; aucun fichier Rust ni manifeste Cargo
ne change dans cette tranche.

[Preuves](../verification/STRUCTURE-REVUE-2026-11.md) : douze tests de politique,
`cargo xtask ci` réussi localement, Clippy sans avertissement, 265 tests workspace
plus un enfant du pont, cinq ignorés, quatre PNG contrôlés. Les 496 PNG du manifeste
précédent, les catalogues FR EN ES IT PT DE et les contrats restent identiques.
Aucun test GPU supplémentaire n’est revendiqué pour cette modification d’outillage.
Le manifeste de cette revue contient **787 fichiers**, révision
`sha256:40cc7928a7359d07ae577db0e47b05f5701e34c821bade2937ac0adc05cc83de`.
Les identifiants, `planned_paths` et statuts des deux registres sont préservés.
L’extraction des sélecteurs d’ouverture est suivie en tête de cette page.

## Dernière tranche : configuration de démarrage

**F-CFG-004 est vérifiée.** Le
[profil portable](../agnostique/CONFIGURATION_J7.md), l’[adaptateur](../rust/CONFIGURATION_J7.md),
[D-013](../rust/decisions/D-013-configuration-j7.md) et la
[qualification](../verification/J7-CONFIGURATION.md) fixent les priorités intégrées /
administrateur / utilisateur / CLI, les valeurs sûres et les diagnostics localisés.
L’état avant CLI est distinct de l’affichage effectif; les modifications UI ne
copient pas les options temporaires sans rapport dans le profil.

Le lanceur propose aide, version longue, contrôle sans fenêtre, chemins, répertoire
de travail et options d’affichage. Il partage la résolution et le constructeur du
projet avec l’interface. Les fichiers sont préservés sur invalidité et protégés selon
leur rôle, y compris après traversée `..` et liens. Les clics ordinaires dans l’aide
ne mémorisent plus de géométrie temporaire; un déplacement réel reste enregistré.

Fichiers principaux : `sns-settings/src/configuration.rs`, `workbench/configuration_ui.rs`,
`apps/sns-desktop/src/launch.rs`, point d’entrée natif, contrats/fixtures et recettes.
Les chemins précis sont dans le registre Rust. Les préférences d’affichage restent
qualifiées dans leur [rapport](../verification/J7-AFFICHAGE.md).

Qualification : **265 tests workspace et un enfant**, cinq ignorés, Clippy sans
avertissement et contrôles réussis; **21 tests de la suite graphique**, **108 captures
comparées sans mise à jour**, dont **12 nouvelles**, **483 anciennes images inchangées**;
**849 clés × six langues**. Le vrai binaire est testé sans fenêtre, avec les six
langues, le projet effectivement chargé, les valeurs invalides et les chemins.
Six aides et douze rapports réels sont aussi enregistrés, entrées inchangées.

Manifeste : **787 fichiers**,
`sha256:e4b36f5f1bf57c7a268f27ce0bd12dcb529ad9e170e071e84d69c9c7fcff48d2`.
Le workspace reste à **20 membres**; aucun paquet tiers ajouté, retiré ou changé.

## Socle et périmètre déjà qualifié

La [décision D-011](../rust/decisions/D-011-structure-et-workflow.md), le
[guide de développement](../rust/DEVELOPPEMENT.md) et la [carte actuelle](../rust/ARCHITECTURE.md)
fixent les responsabilités et les vérifications communes. Le [complément de politique](../verification/STRUCTURE-POLITIQUE-2026-11.md)
refuse une catégorie mal orthographiée avant l’analyse du graphe.

- [Import](../verification/J7-IMPORT-EN-COURS.md) : données ZIP/XML bornées, original préservé, rapport de pertes; **F-PROJ-005 reste implemented**, pas verified.
- [Scénarios/reprise/rejeu](../verification/J7-REPRISE-EN-COURS.md) et [pannes de liaison](../verification/J7-PANNES.md) : F-SIM-007/008.
- [Duplication/alignement](../verification/J7-TOPOLOGIE.md) et [nuage](../verification/J7-PRESENTATION.md) : F-TOP-009, F-EQU-008.
- [Observation](../verification/J7-OBSERVATION.md) et [diagnostic interne](../verification/J7-TERMINAL.md) : F-OBS-006/008, F-CLI-019.
- [Session/récupération](../verification/J7-SESSION.md) : F-PROJ-003/006.
- [Notes/zones](../verification/J7-ANNOTATIONS.md), [PNG](../verification/J7-PNG.md) et [PDF](../verification/J7-RAPPORT.md) : F-DOC-001/002/003/004.
- [Aide contextuelle et informations](../verification/J7-AIDE.md) : F-CFG-002; F1 et Échap préservent une saisie active.

## Prochaine action et limites

Finaliser la qualification sémantique de l’import Filius dans
**RS-PROJECTS-J7**. La distribution, l’association du format et les
parcours réels Windows, Linux et macOS restent requis dans
**RS-PLATFORM-J7** et **RS-QUALITY-J7**. F-NFR-001/006 et RS-QUALITY-J0 restent en cours.
La suite GPU historique complète et l’accessibilité globale doivent être réconciliées
avec J7. La commande graphique ciblée couvre PNG, PDF, aide, préférences et configuration.
Les fenêtres d’une même catégorie partagent leur géométrie initiale; les dimensions
principales viennent du panneau et ne suivent pas automatiquement les gestes natifs.
Le zoom réseau reste distinct de la taille des widgets.

Aucun sélecteur natif nouvellement manipulé, parcours manuel autonome,
résultat Windows/Linux ou run distant n’est revendiqué. Les sockets hôte restent
réservées au pont explicitement activé. Les archives Filius et network-kit restent
en lecture seule; ni audit recalculé, ni commit, push ou publication effectués.
