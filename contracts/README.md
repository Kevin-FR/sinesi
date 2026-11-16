# Contrats Sinesi — socle J0, version 1

Les fichiers JSON Schema Draft 2020-12 de ce dossier sont l’autorité portable des données effectivement présentes dans J0. Ils sont autonomes et ne téléchargent aucune référence externe. Leur identifiant `https://sinesi.dev/contracts/…` est un espace de noms, pas un service requis.

| Schéma | Fixture | Portée actuelle |
| --- | --- | --- |
| [project.schema.json](project.schema.json) | [project.json](../fixtures/valid/project.json) | Équipements/interfaces, liaisons, disposition et données persistantes |
| [scenario.schema.json](scenario.schema.json) | [scenario.json](../fixtures/valid/scenario.json) | Projet, graine, messages initiaux et routes du serveur de prototype |
| [command.schema.json](command.schema.json) | [command.json](../fixtures/valid/command.json) | Enveloppe et commandes pause/reprise/pas/arrêt/envoi de prototype |
| [event.schema.json](event.schema.json) | [trace.jsonl](../fixtures/valid/trace.jsonl) | Faits immuables du moteur J0 |
| [error.schema.json](error.schema.json) | [error.json](../fixtures/valid/error.json) | Diagnostic stable, cible et paramètres |

La validation structurelle est complétée par des invariants métier : unicité des identités et MAC, références existantes, ports physiques non partagés, chemins virtuels, contenus base64 valides et limites. Un schéma seul ne valide pas les références entre deux listes. Les fixtures négatives couvrent notamment versions inconnues, identité dupliquée et référence inexistante.

Le format initial est un JSON UTF-8 avec suffixe `.sinesi.json`; le scénario de recette inclut le projet avec ses actions. Une sauvegarde de projet ne conserve pas une exécution en cours. Les archives, checkpoints et importeurs historiques restent des livrables distincts de la roadmap. La migration v0 est une fixture de qualification définie par D-004, pas un import Filius.

J0 expose des **messages de prototype**, dont des requêtes/réponses applicatives de démonstration, sur une livraison événementielle déterministe. Les événements `prototype.message.*` ne sont pas des captures Ethernet, TCP ou HTTP standards. Les futures familles de commandes et de protocoles devront faire évoluer les contrats explicitement.

Les compteurs/temps sont des chaînes décimales canoniques de 0 à 2^64−1. La trace canonique JSONL trie récursivement les clés, utilise UTF-8, une fin de ligne LF par événement et normalise l’identifiant d’exécution en `run`. Les faits exposés à l’UI gardent l’identifiant de génération réel. Le hash SHA-256 s’applique aux octets JSONL ainsi normalisés.

Les mêmes fixtures sont consommées par le CLI et le prototype UI, et leur sérialisation réelle est contrôlée par les schémas. Pour exécuter les vérifications :

```sh
cargo test -p sns-project --test contracts --locked
cargo test -p sns-cli --test runner --locked
cargo test -p sns-ui-egui --test workbench --locked
```

## Extension LAN J1

Les schémas historiques ci-dessus restent inchangés. J1 ajoute trois schémas autonomes :

| Schéma | Portée |
| --- | --- |
| [project-lan.schema.json](project-lan.schema.json) | Projet version 2 / `sns-educational-lan-v1`, paramètres de présentation et rétention SAT; fixture [lan.json](../fixtures/valid/lan.json) |
| [command-lan.schema.json](command-lan.schema.json) | Enveloppe API 1 enrichie : `icmp.ping`, `arp.clear`, `switch.sat.clear`, `simulation.manual_loss` |
| [event-lan.schema.json](event-lan.schema.json) | Événements version 2, trames Ethernet et charges ARP/IPv4/ICMP structurées |

Un pas est un événement de file, pas une ligne de trace. Les purges sont administratives et immédiates en pause. Le hash canonique conserve la règle J0. Les versions/profils inconnus restent refusés. Les invariants sémantiques, durées et simplifications du sous-ensemble sont définis dans [LAN_J1](../docs/agnostique/LAN_J1.md).

La liste [filius-known-classes.json](filius-known-classes.json) reconnaît uniquement des étiquettes de données dans les inventaires `.fls`; elle n’autorise ni exécution ni import de classes.

## Transport J2

[transport-command.schema.json](transport-command.schema.json) définit la commande `socket.command` (Bind/Connect/Send/SendTo/Close/CloseOwner) avec ses endpoints et propriétaires. [transport-event.schema.json](transport-event.schema.json) définit les faits version 3 et leurs trames TCP/UDP. Les événements LAN inchangés restent en version 2. Les sérialisations réelles sont validées par les tests de contrats.

Les recettes complètes R-TCP et R-FICHIERS sont maintenant qualifiées dans le [rapport J2](../docs/verification/J2.md). Voir le [sous-ensemble transport](../docs/agnostique/TRANSPORT_J2.md) et l’[état de reprise](../docs/suivi/ETAT.md).

## Logiciels et fichiers J2

| Schéma | Portée |
| --- | --- |
| [project-hosts.schema.json](project-hosts.schema.json) | Version 3 / `sns-educational-hosts-v1`, exemple [hosts.json](../fixtures/valid/hosts.json) |
| [application-command.schema.json](application-command.schema.json) | API 1 `app.command`, catalogue, vues/services, fichiers et jobs du terminal |
| [application-event.schema.json](application-event.schema.json) | Faits version 4 : installation, commande applicative, réception, fichiers et fin de ping broadcast |

Les faits LAN et transport gardent leurs versions 2 et 3; un même journal peut donc contenir les trois versions. La révision est incrémentée uniquement par les changements persistants. Voir [HOTES_J2](../docs/agnostique/HOTES_J2.md) pour les invariants, dossiers et six langues. Les tests de contrats valident les sérialisations produites, refusent champs inconnus et versions/profils incompatibles; la validation sémantique complète les schémas structurels.

## Routage et représentations J3

| Schéma | Portée |
| --- | --- |
| [project-routing.schema.json](project-routing.schema.json) | Projet 4 / `sns-educational-routing-v1`; routes, forwarding, RIP, représentations et nombre d’interfaces du routeur |
| [routing-event.schema.json](routing-event.schema.json) | Faits version 5; décisions, paquets historiques, annonces et changements RIP |
| [routing-command.schema.json](routing-command.schema.json) | Contrôle temporaire `routing.rip.mute`, enveloppe API 1 |

Les faits de versions 2/3/4 sont conservés pour les unités inchangées. La validation sémantique vérifie en plus les réseaux canoniques, passerelles locales, interfaces, doublons et limites. Les fixtures [routed](../fixtures/valid/routed.json), [rip](../fixtures/valid/rip.json), [chaîne RIP](../fixtures/valid/rip-chain.json) et [salle de classe](../fixtures/valid/classroom.json) sont autonomes. Voir le [profil J3](../docs/agnostique/ROUTAGE_J3.md).


J4 ajoute `project-services.schema.json` (projet 5), `services-command.schema.json` (API 1) et `services-event.schema.json` (événements 6). Les fichiers J0 à J3 conservent leur périmètre historique. Les tests vérifient la sérialisation effective des configurations, commandes et traces DHCP/DNS/HTTP.


J5 ajoute `project-private.schema.json` (projet 6), `security-command.schema.json` (API 1) et `private-event.schema.json` (événements 7, NAT, filtrage et ICMP cité). Les versions historiques restent inchangées.

## Applications et connexions J6

- `project-advanced.schema.json` : projet 7, comptes/messages/contacts, P2P, SSID et réglages de modem.
- `advanced-command.schema.json` : courrier, P2P et entrées explicites du pont dans l’enveloppe API 1.
- `advanced-event.schema.json` : faits 8 et paquet externe enregistré.

Le [profil portable](../docs/agnostique/APPLICATIONS_J6.md) fixe les limites et la sémantique; les validations de références, tailles cumulées et séquences complètent les schémas. Les nouveaux schémas sont autonomes et les anciens restent inchangés.

## Import Filius — tranche J7 en cours

`import-report.schema.json` définit le rapport de conversion en données, ses champs non convertis et ses pièces originales conservées. Le projet converti suit le profil 7, ou 8 si un commutateur porte l’apparence nuage; ce rapport n’annonce aucune équivalence fonctionnelle. Voir [PROJETS_J7](../docs/agnostique/PROJETS_J7.md) et [le travail en cours](../docs/rust/J7-EN-COURS.md).

## Scénarios réseau et reprise J7

Les schémas autonomes [network-scenario](network-scenario.schema.json), [recording](recording.schema.json) et [checkpoint](checkpoint.schema.json) définissent le scénario réseau 2 et les fichiers d’exécution 1/2 selon la [politique d’attente](../docs/agnostique/ATTENTES_J7.md). Le [contrat portable](../docs/agnostique/REPRISE_J7.md) fixe l’ordre, les quotas, les générations et l’encodage des associations/entiers de l’état complet. Le schéma valide les enveloppes et la structure de l’état; l’empreinte et sa reconstruction déterministe valident l’ensemble de ses champs privés. Une validation structurelle seule ne suffit pas à accepter un checkpoint.

## Conditions des liaisons J7

[link-conditions-command](link-conditions-command.schema.json) ajoute `simulation.link.configure` à l’API 1. [link-conditions-event](link-conditions-event.schema.json) fixe les faits 9 : paramètres, essai de transmission et perte de trame selon les conditions. Les schémas scénario/journal/checkpoint admettent cette commande; les anciens schémas de faits restent inchangés. Le [profil portable](../docs/agnostique/PANNES_J7.md) précise la somme des durées, les flux indépendants et leur conservation à la reprise.

## Présentation J7

[project-presentation](project-presentation.schema.json) définit le projet 8 / `sns-educational-presentation-v1`. [presentation-command](presentation-command.schema.json) porte `presentation.symbol.set` dans l’API 1; [presentation-event](presentation-event.schema.json) définit le fait 10 `presentation.symbol.changed`. Les schémas courants de scénario/journal/checkpoint acceptent ce profil et cette commande. La validation sémantique impose que `cloud` cible un commutateur; les projets 1 à 7 ne l’acceptent pas. Voir le [profil portable](../docs/agnostique/PRESENTATION_J7.md).

## Session et récupération J7

[session.schema.json](session.schema.json) définit l’enveloppe de récupération 1 et embarque ses définitions de projet (profils 1 à 8). La [fixture](../fixtures/valid/session.json) ne nécessite aucune référence externe. `edit_state` est optionnel pour conserver les premières sessions; le profil inactif version 1 est décrit dans [SESSION_J7](../docs/agnostique/SESSION_J7.md). Le schéma contrôle la structure externe, puis le décodeur d’édition et la validation sémantique vérifient propriétaires, doublons, quotas et références. Une validation structurelle seule ne suffit pas à adopter un document. Le projet utilisateur, les préférences et les checkpoints restent des formats distincts.

## Documentation J7

[project-documentation](project-documentation.schema.json) définit le projet 9 / `sns-educational-documentation-v1` : annotations avec géométrie et styles portables. Les schémas courants scénario, journal, checkpoint et session acceptent ce profil; les schémas de projet 1 à 8 restent historiques et refusent `style`. Le [profil](../docs/agnostique/ANNOTATIONS_J7.md) décrit empilement, clipping, quotas et brouillons; la [fixture](../fixtures/valid/annotations.json) contient un LAN avec zone et note multiligne.

## Préférences d’affichage J7

[preferences](preferences.schema.json) définit le profil utilisateur 1 (langue) et 2
(langue et affichage), distinct du projet et de l’exécution. Les [fixtures 1](../fixtures/valid/preferences-v1.json)
et [2](../fixtures/valid/preferences-v2.json) vérifient migration en mémoire et bornes.
Voir le [profil portable](../docs/agnostique/AFFICHAGE_J7.md).

Le profil 2 accepte deux champs facultatifs de l’expérience élève (J8) :
`display.level` (`student` par défaut, ou `expert`) et `missions`, meilleures étoiles
(1 à 3) par mission réussie, omis tant qu’il est vide. Voir
[l’expérience élève](../docs/agnostique/EXPERIENCE_ELEVE.md).

## Missions évaluables J8

[mission](mission.schema.json) version 1 décrit un exercice portable : textes dans
les six langues (français obligatoire), projet de départ complet, objectifs typés,
explications et indices. [mission-report](mission-report.schema.json) version 1 est
la correction exportable produite par l’atelier et par `mission-check`. Les missions
intégrées sont les fixtures du contrat. Voir [le profil portable](../docs/agnostique/MISSIONS.md).

## Résolution de configuration J7

[configuration](configuration.schema.json) version 1 décrit les valeurs administrateur
partielles; [startup-report](startup-report.schema.json) versions 1/2 décrit la résolution,
les origines, diagnostics et le résumé du projet. La [fixture](../fixtures/valid/configuration.json)
complète celles des profils 1/2. Le [profil portable](../docs/agnostique/CONFIGURATION_J7.md)
définit priorité, repli et non-persistance des options temporaires.

## Diagnostic technique J7

Le [schéma du diagnostic](diagnostic.schema.json) décrit l’instantané JSON version 1.
Ses entrées correspondent aux lignes du journal technique de lancement.
Le [contrat portable](../docs/agnostique/DIAGNOSTIC_J7.md) précise les limites et
les protections de destination.
