# Projets, scénarios, reprise et livraison — J7

Ce document accompagne les 22 exigences et neuf lots de J7. Le [registre](../suivi/features.json) et l’[état courant](../suivi/ETAT.md) font autorité : aucune réalisation partielle ne vaut fermeture du jalon. Les six langues FR EN ES IT PT DE et les vues Icônes/Schéma restent requises.

## Périmètre intégral et portes de sortie

| Famille | Comportement attendu | Preuve requise |
| --- | --- | --- |
| Import | Topologie et données historiques converties sans exécution; pièces et écarts visibles | R-IMPORT sur 18 exemples et deux fixtures, entrées malformées, recettes réseau ciblées |
| Scénarios et reprise | Commandes/entrées enregistrées, état complet du moteur, pannes déterministes, checkpoints et rejeu | R-REPRISE avec TCP, RIP, DHCP, DNS, NAT et entrées de modem; suffixe canonique identique |
| Récupération | Préférences séparées, dernière session, autosauvegarde atomique indépendante | Arrêt forcé, réouverture et fichier explicite préservé |
| Édition et observation | Duplication/alignement, nuage de commutateur, annotations/styles, recherche et parcours entre couches | Identités distinctes, conflits signalés, données/vues historiques cohérentes |
| Exports | PNG transparent, PDF paginé, texte/JSONL/CSV selon sélection | R-RAPPORT, cadrage, caractères et sélection exacte vérifiés visuellement et structurellement |
| Environnement et livraison | Aide, exemples, configuration, priorité des options, diagnostic sans contenu privé, installation et association | Parcours clavier et sauvegarde sur Windows/Linux/macOS, paramètres et désinstallation contrôlés |

Le reste de J7 demeure à réaliser tant que ses preuves ne sont pas présentes. La disponibilité d’une bibliothèque d’import n’est pas une preuve de récupération, de rejeu ou de livraison multiplateforme.

## Contrat d’import en données — tranche initiale

L’import ne reconstruit aucun objet exécutable. Les noms de classes, propriétés, références et mutations de collections du XML historique sont des données. Seules les correspondances explicitement reconnues alimentent le modèle courant; aucune réflexion, invocation de méthode, instanciation de classe historique ou ouverture de connexion réelle n’a lieu.

L’archive est traitée en mémoire, sans extraction. Limites : 32 Mio compressés et décompressés cumulés, 1024 entrées, 8 Mio par entrée, rapport de compression ≤ 1000, annuaire ZIP ≤ 2 Mio. Seuls ZIP stocké et Deflate non chiffrés sont admis; Les archives multidisques et les annuaires déclarant plus de 1024 entrées sont refusés. Les chemins absolus, segments `.`/`..`, séparateurs ambigus, liens symboliques et doublons de noms insensibles à la casse sont rejetés. Les tailles réelles et contrôles d’intégrité sont vérifiés après décompression.

Une seule configuration `*/konfiguration.xml` UTF-8 est attendue. DTD et déclarations d’entités sont refusées avant lecture structurée. L’arbre est limité à 200 000 nœuds et 128 niveaux. Les identifiants doivent être uniques et toute référence déclarée doit exister; les chaînes d’alias utilisées sont bornées et les cycles refusés. Les chaînes et contenus sont validés par le modèle courant avant publication.

### Correspondances initiales

- Équipements : ordinateurs, portables, commutateurs, routeurs, passerelles et modems; noms, positions, interfaces, adresses, masques, passerelle/DNS et sélection de SSID.
- Liaisons : références de ports connues, câble et délai pédagogique de 1 ms. Les câbles radio historiques sont signalés; l’association courante se calcule par SSID.
- Logiciels : catalogue reconnu, fichiers virtuels texte/images, configuration DNS A/MX/NS et racine web, identités/comptes et endpoints de courrier. Les services et tunnels ne démarrent pas à l’import.
- Annotations : texte, zones, position, dimensions et styles portables.

Les valeurs implicites historiques reconnues sont interprétées comme des constantes de données : adresse initiale 192.168.0.10, masque /24, état booléen d’un contrôle activé, domaine mail filius.de. Les MAC absentes ou en collision sont remplacées avec diagnostic. Une interface sans connexion peut être ajoutée explicitement pour satisfaire les contraintes du modèle courant d’un routeur. Les valeurs invalides et les réglages non repris doivent rester visibles dans le rapport. Les sources d’une application personnalisée ne sont jamais compilées.

### DHCP et routes manuelles

Un serveur DHCP actif dont la plage est valide est configuré avec son interface
primaire et un bail de 24 heures virtuelles. Sur une passerelle Filius, le serveur
utilise le LAN et annonce son adresse comme passerelle. Le mode client DHCP
s’applique au WAN. Les réservations valides sont reprises. Une plage incompatible
ou une réservation refusée est signalée avec sa position dans le fichier. Une
configuration de serveur désactivée reste inactive.

Les routes manuelles d’une table statique sont lues comme des lignes à quatre
champs. L’adresse locale de sortie identifie l’interface. Les routes non
représentables et les destinations répétées sont signalées. Les traces de
recette démontrent des baux et un aller-retour ICMP sur des exemples importés.

Les indicateurs historiques de transfert IPv4 sont lus explicitement. Un routeur
avec RIP actif utilise les routes connectées et celles apprises par le moteur.
Les routes manuelles et la passerelle par défaut enregistrées mais inactives
dans ce mode restent signalées dans les données originales. Les temporisations
et la sélection des routes suivent le profil RIP de Sinesi.

### NAT et filtrage

Les passerelles historiques configurent les rôles WAN et LAN ainsi que la
traduction simulée. La durée de rétention est lue en millisecondes et convertie
en temps virtuel. La plage de ports dynamiques suit le profil Sinesi. Les
redirections TCP et UDP représentables conservent leur port externe, l’adresse
interne et le port interne. Les redirections incompatibles sont signalées.

Le pare-feu conserve son activation distincte du NAT, sa politique par défaut,
les options ICMP, TCP et UDP et l’ordre des règles. La première règle applicable
décide. Une politique contenant une règle non représentable est signalée et
reste dans les données originales. Les règles par protocole ICMP et le réseau
local relatif à plusieurs interfaces d’une passerelle font partie de ces cas.
Les modules Java utilisés pour accéder aux réglages restent inactifs.

### Zones et notes

La documentation historique utilise les zones rectangulaires et les notes du
projet courant. Les zones sont placées derrière les notes. Les couleurs sont
reprises selon l’apparence hors édition. Les familles de polices reconnues sont
représentées par leurs équivalents proportionnel ou monospace, avec taille,
gras et italique. Les très petites dimensions sont ajustées et signalées.
La présentation conserve les positions. Un export peut donc tronquer du texte
ou laisser un équipement le recouvrir dans une topologie dense. Les textes
complets restent disponibles dans le projet. Les angles arrondis historiques
sont représentés par des angles droits.

### Courrier historique

Les comptes proviennent des tables XML ou des fichiers virtuels de comptes.
Le fichier client valide détermine l’identité active lorsqu’il existe. Les
identités supplémentaires restent dans les originaux et sont signalées. Côté
serveur, le fichier valide reflète l’état rechargé par Filius. Le contenu XML
reste utilisable lorsque ce fichier ne fournit aucun compte valide.

Les dossiers reçus et envoyés, les boîtes serveur et les contacts valides
deviennent des données persistantes. Les BCC restent accessibles uniquement
dans la copie envoyée. Les corps, dates et identifiants ne sont pas recopiés
dans le rapport. Les dates locales restent dans l’original et les messages
importés gardent l’ordre du dossier avec une origine de temps virtuel à zéro.
Les brouillons, les états transitoires et les plugins restent dans les
originaux. Les copies historiques indépendantes gardent leurs identités.
Le rapport conserve les écarts de conversion. Les recettes ciblées qualifient
les échanges qu’elles exercent.

### Conservation et rapport

Chaque entrée originale est conservée octet par octet, par morceaux d’au plus 1 Mio dans les ressources du projet. Le rapport donne nom, taille décimale, empreinte SHA-256 et références permettant de reconstruire son contenu. Les sources et binaires détectés sont liés à des pièces d’extension **inactives**, avec une version de contrat qui ne les rend pas exécutables. Les licences des données importées ne sont pas réattribuées.

Le [rapport version 1](../../contracts/import-report.schema.json) contient provenance, comptes d’éléments, propriétés converties et cible, propriétés non converties, méthodes inconnues, structures inconnues, écarts et pièces conservées. Il ne reproduit pas par défaut les valeurs de mots de passe ou corps de courrier dans ses diagnostics. Il porte explicitement `execution_performed: false` et `functional_equivalence_verified: false`.

Le rapport intégré est limité à 1 Mio et le projet sérialisé à 8 Mio. Un dépassement refuse la conversion plutôt que supprimer silencieusement des données. Le document antérieur est conservé en cas d’échec; le fichier historique ne devient jamais implicitement la destination de sauvegarde. Un projet importé est marqué comme non enregistré. Le rapport peut être consulté et exporté séparément; le traitement lourd reste hors de la boucle d’affichage.

Les paramètres d’interface et leur traduction n’affectent pas la conversion. Les originaux conservés ne sont pas assimilés à des fichiers de configuration actifs. Le corpus de référence reste distinct des exemples neufs distribués avec le produit.

## Reprise et rejeu réalisés en tranche dédiée

Le [contrat de reprise J7](REPRISE_J7.md) décrit la tranche réalisée et ses limites. Le checkpoint conserve l’état nécessaire à une reprise exacte : horloge, file et ordre des événements, compteurs, générateur aléatoire, sockets, minuteries, caches, routes, attributions, mappings, applications, fichiers et position du journal d’entrées. Restaurer uniquement le document de configuration ne satisfait pas cette exigence. Le rejeu d’entrées de modem ne doit ouvrir aucune socket réelle. Les générations doivent écarter les commandes retardées d’une ancienne exécution.

Les exports et la livraison seront documentés avec leurs contrats et preuves avant qualification. La [recette](RECETTE.md) conserve les critères complets, notamment la validation sur les trois systèmes ciblés.

## Installation et association

L’extension de projet associée au bureau est `.sinesi`. Le fichier utilise le
même contrat JSON versionné que les projets existants. Les anciens fichiers
`.json` restent ouvrables depuis l’application et en fournissant un chemin
explicite. L’installation associe uniquement `.sinesi`. Le système permet à
l’utilisateur de choisir l’application par défaut.

L’ouverture depuis le bureau transmet un chemin au même chargeur validé que le
lancement explicite. Les espaces, les caractères Unicode et les noms commençant
par un tiret sont acceptés. L’installation et le retrait portent sur
l’application et ses inscriptions système. Les projets et données de récupération
restent dans leurs emplacements utilisateur. Chaque plateforme doit être
qualifiée avec les parcours natifs de la recette.
