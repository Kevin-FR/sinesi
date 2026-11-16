# Architecture Rust — état réalisé

Le comportement portable est défini dans [l’architecture agnostique](../agnostique/ARCHITECTURE.md).
Cette page décrit le code courant; [taches.json](taches.json) relie les exigences
aux chemins et preuves. Le [guide de développement](DEVELOPPEMENT.md) fixe les
commandes communes. [D-011](decisions/D-011-structure-et-workflow.md) motive le
refactor du 8 novembre 2026.

## Workspace et frontières

Le workspace utilise Rust 1.95.0, édition 2024, resolver 3 et un lockfile commun.
Les dépendances sont déclarées à la racine et héritées par tous les membres.
Le binaire bureau active les fonctionnalités natives d’eframe. Le crate de
présentation active ses fontes egui et son chargement SVG.
Les versions upstream egui/eframe 0.37.0 sont conservées; aucune dépendance Craft
ou patch de fenêtrage n’est activé. La [politique de dépendances](layers.json)
énumère les **20 membres**, leurs arêtes internes et leur catégorie.

| Crate réalisée | Responsabilité |
| --- | --- |
| `sns-model` | Identités, données, versions et validation de configuration |
| `sns-topology` | Édition transactionnelle, historique, placement de repli et avis de configuration pour l’élève |
| `sns-protocols` | Automates réseau, routage, transport, NAT et filtrage |
| `sns-vfs` | Fichiers virtuels bornés |
| `sns-apps` | Terminal et logiciels pédagogiques sur la pile simulée |
| `sns-engine` | Propriétaire de simulation, événements virtuels, commandes, checkpoints et rejeu |
| `sns-trace` | Requêtes, parcours causaux et exports de faits |
| `sns-missions` | Missions évaluables : format, contrôles sur projet et trace, étoiles, correction ([D-016](decisions/D-016-missions.md)) |
| `sns-project` | Formats portables, écriture atomique et récupération de session |
| `sns-filius-import` | Import ZIP/XML borné comme données, avec rapport de pertes |
| `sns-i18n` | Catalogue FR EN ES IT PT DE |
| `sns-settings` | Langue, affichage, résolution de démarrage et origines |
| `sns-html` | Document HTML pédagogique isolé, sans accès hôte implicite |
| `sns-fonts` | Quatre fontes embarquées, shaping et contours pour l’export |
| `sns-render` | Scène statique et PNG CPU, sans widget ou moteur actif |
| `sns-report` | Rapport paginé, texte Unicode et PDF depuis un instantané |
| `sns-bridge` | Seul adaptateur de sockets réelles entre instances, activation explicite |
| `sns-ui-egui` | Présentation native et workers/adaptateurs de fenêtre |
| `sns-cli` | Exécution sans GUI, import, reprise/rejeu et export |
| `sns-desktop` | Binaire eframe et arguments de lancement |
| `xtask` | Contrôles du dépôt, sans dépendance au produit ni bibliothèque tierce |

Les bibliothèques sans GUI ne dépendent pas de la présentation ou du pont, même
transitivement. La politique vérifie aussi les dépendances de build et conditionnelles.
Elle active toutes les fonctionnalités des membres lors de l’analyse, y compris
leurs dépendances optionnelles, et contrôle l’héritage des manifestes.
Les tests peuvent composer deux adaptateurs; cette exception n’autorise pas une
arête de production. Le code métier ne réalise pas d’I/O hôte implicite. Les appels
à la bibliothèque standard restent également vérifiés par revue et recettes.

## Moteur et commandes

Un seul propriétaire possède horloge, file et automates. L’ordre total des
événements est `(temps, priorité, séquence)`; les timers de protocole restent virtuels.
Les commandes réseau, transactions d’édition et requêtes ont des entrées explicites.
L’interface et le CLI réutilisent ces chemins. Les formats externes restent fixés
par les contrats neutres et fixtures; la représentation mémoire ne les remplace pas.

Le worker de présentation échange des commandes et projections bornées. Il porte
les générations, révisions et acquittements nécessaires à rejeter un retour périmé.
Une frame ne fait pas avancer un protocole et un rendu ne possède pas la simulation.
Les entrées externes du pont sont enregistrées; le rejeu n’ouvre pas de socket réelle.

## Présentation

```text
crates/sns-ui-egui/src/
  lib.rs                     # API de présentation courante
  workbench/
    mod.rs                   # état et coordination
    worker.rs                # propriétaire du moteur pour cette présentation
    chrome.rs, commands.rs   # composition, navigation et raccourcis
    toolbar.rs               # espaces Construire/Tester/Annoter, niveau Élève/Expert, consigne (D-015)
    missions_ui.rs           # carte des missions et mission en cours (D-016)
    packets.rs, story_ui.rs  # paquets animés depuis la trace et « Le voyage du paquet »
    help_ui.rs               # aide contextuelle, exemples et informations
    preferences_ui.rs        # profil d’affichage et écritures asynchrones
    configuration_ui.rs      # diagnostics et rôles des fichiers de démarrage
    canvas.rs, inspector.rs  # dessin/interaction et configuration
    document.rs              # ouverture asynchrone, installation et sauvegarde du projet
    annotations.rs           # zones, notes et gestes
    trace_ui.rs, trace_table.rs
    export_ui.rs, session_ui.rs, import_ui.rs, runtime_ui.rs
    routing_ui.rs, security_ui.rs, link_conditions_ui.rs
    topology_ui.rs, presentation_ui.rs
    desktop/                 # logiciels d’hôte, services, courrier et P2P
  prototype/                 # expérience J0 et son contrôleur isolés
  network_art.rs, traffic.rs # ressources et filtres de présentation
apps/sns-desktop/             # assemblage eframe
crates/sns-cli/               # adaptateur sans fenêtre
xtask/                       # qualité commune local/CI
```

`Workbench` remplace `LanUi` sans alias. Le prototype conserve son entrée explicite
et ses tests, séparément du produit. Le [plan de chemins](refactor-paths.json)
permet de retrouver les preuves antérieures. Les dialogues natifs restent des
adaptateurs; plusieurs chemins synchrones et états de brouillon doivent encore
être extraits. Ce refactor ne prétend pas avoir terminé cette séparation.

## Rendu et données

`Scene` reçoit un projet et des libellés explicites, sans contexte egui. Le placement
de repli et les SVG sont partagés; les annotations utilisent les fontes embarquées.
Le PNG utilise resvg 0.45.1 et sa version tiny-skia 0.11.4, sans seconde pile 0.12.
HarfRust 0.12.0 et Skrifa 0.44.0 fournissent glyphes et contours. Voir
[D-010](decisions/D-010-export-scene-j7.md) et [le rapport PNG](../verification/J7-PNG.md).

Projet durable, exécution, session de récupération et préférences restent distincts.
Les anciens profils de données continuent d’être lus conformément aux contrats;
un renommage de type d’interface ne constitue pas une migration de fichier.

## Cibles encore ouvertes

Le [rapport PDF](decisions/D-012-rapport-pdf-j7.md) est réalisé avec Krilla directement; l’hôte d’extensions et les composants Craft restent ouverts. Il n’existe pas de crate `sns-devices` distincte : les données
et comportements correspondants vivent dans le modèle et les protocoles.
Les anciennes arborescences de cadrage sont des propositions, pas une carte du code.
Consulter [DEPENDANCES](DEPENDANCES.md), [J7 en cours](J7-EN-COURS.md) et l’état de reprise
pour les choix restant à qualifier, notamment livraison, lisibilité et parcours natifs trois OS.

## Aide et informations J7

Le [catalogue hors ligne et son interface](AIDE_J7.md) complètent le lot plateforme. Les données descriptives vivent dans sns-settings, la navigation dans son panneau. Les commandes de chargement réutilisent le cycle existant du document; consulter l’aide ne produit aucune commande moteur.

Les [préférences d’affichage](AFFICHAGE_J7.md) sont distinctes du projet et du worker réseau; le profil utilisateur porte fonte, taille, panneaux et géométrie bornée. Les opérations lentes sont exécutées par un worker unique et coalescées.

La [résolution de démarrage](CONFIGURATION_J7.md) sépare valeurs effectives et profil à enregistrer; le lanceur utilise le même constructeur que le contrôle sans fenêtre. Les options historiques restantes ne sont pas déclarées réalisées.
