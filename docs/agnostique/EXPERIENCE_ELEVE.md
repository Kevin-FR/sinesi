# Expérience élève — collège et lycée

Exigences F-UX-001 à F-UX-006. Ce document décrit le comportement attendu,
indépendamment du langage et de la bibliothèque graphique.

## Principes

- Un seul atelier. Le niveau d’affichage **Élève** (défaut) ou **Expert** choisit
  les commandes proposées; il ne modifie ni le projet, ni la simulation, ni leurs
  formats. Une commande masquée au niveau Élève reste la même transaction au
  niveau Expert et en ligne de commande.
- Trois espaces nommés par l’action de l’élève : **Construire** (conception),
  **Tester** (simulation) et **Annoter** (documentation). Leurs raccourcis F2, F3
  et F4 restent inchangés.
- Chaque espace affiche une consigne courte et visible, traduite dans les six
  langues. Une traduction n’est jamais un discriminant métier.
- Le vocabulaire visible au niveau Élève évite les identifiants internes
  (identifiant d’exécution, révision, compteur de faits); le niveau Expert les
  conserve.
- Les cibles cliquables principales mesurent au moins 40 points logiques de haut
  et l’espace actif se distingue aussi par la luminosité, pas seulement par la teinte.

## Niveau Élève

Commandes visibles : espaces de travail, annuler/rétablir, nouveau, ouvrir,
enregistrer, enregistrer sous, exemples, reprise de session, langue, préférences,
aide, choix du niveau, pages Topologie/Échanges/Tables/Bureau.

Commandes réservées au niveau Expert : import Filius et rapport d’import,
exécutions (scénarios, checkpoints, rejeu, diagnostic technique), export du
réseau, ligne détaillée d’exécution.

## Palette et câble guidé (F-UX-003)

La palette présente chaque famille d’équipement par son symbole, dans la vue
Icônes ou Schéma courante, avec un nom court et une couleur de famille. Le nom
accessible et l’infobulle restent la commande complète (« Ajouter un ordinateur »).
Un clic ajoute l’équipement; le glisser-déposer reste disponible.

L’outil Câble est un bouton à bascule (raccourci C). Tant qu’il est actif, une
consigne lisible par les technologies d’assistance s’affiche sur le canevas :
choisir le premier équipement, puis celui à relier à l’équipement nommé, puis une
confirmation « Câble posé entre A et B ». La pose passe par la même transaction
de connexion qu’auparavant.

## Paquets animés et récit du trajet (F-UX-004)

En mode Tester, chaque trame émise dans la trace est dessinée comme une pastille qui
parcourt son câble, de l’émetteur vers l’autre extrémité. Couleurs : demande d’écho
orange, réponse bleue, résolution d’adresse violette, autre trafic turquoise. Les
départs sont espacés pour qu’un trajet en plusieurs sauts se lise dans l’ordre; à
grande vitesse, les départs trop en retard sont omis au lieu de s’accumuler.
L’animation ne crée, ne retarde ni ne modifie aucun fait de simulation.

Sous le canevas, « Le voyage du paquet » raconte les dernières étapes en phrases
courtes : envoi du ping, question ARP et réponse, commutateur, routeur traversé,
réception, réponse avec son délai. Les échecs sont expliqués : absence de passerelle
ou de route, personne ne répond à la demande d’adresse, délai dépassé, TTL épuisé.
Les étapes portent la couleur de leur nature (envoi, résolution, réussite, échec).
Le niveau Expert ajoute le type d’événement et les interfaces.

## Terminal lisible (F-UX-005)

Le terminal simulé conserve ses résultats structurés; seule leur présentation
change selon le niveau. Au niveau Élève :

- `ipconfig` : une phrase par carte (adresse IP, préfixe, mode fixe ou automatique,
  adresse MAC), puis la passerelle et le serveur DNS, ou l’explication de leur absence;
- `arp` : « adresse IP → adresse MAC (carte) », ou l’explication d’une table vide;
- `route` : réseaux reliés directement, réseaux joints par une passerelle, route par
  défaut; la boucle locale est omise;
- `ls`/`dir` : dossiers marqués, fichiers avec leur taille;
- `ping` : délai en millisecondes, conclusion « ça marche ! », bilan des pertes.

Les autres commandes et toute forme inattendue gardent la présentation structurée.
Au niveau Expert, la sortie reste celle d’avant (JSON, microsecondes).

## Fiche pédagogique (F-UX-006)

Dans Construire, au niveau Élève, l’adresse IP, la passerelle et le serveur DNS
sont accompagnés d’une phrase d’explication. Pendant la saisie, la passerelle d’un
ordinateur ou d’un portable à adresse fixe est jugée sur le brouillon, aux deux
niveaux et dans tous les espaces : vide (seul le réseau local est joignable),
adresse invalide, adresse de l’appareil lui-même, hors du réseau de l’appareil
(réseau indiqué), ou correcte. Rien n’est appliqué avant la commande
« Appliquer les paramètres ». Un hôte en DHCP et les équipements réseau ne sont
pas jugés sur une supposition.

## Persistance

Le niveau est une préférence d’affichage, enregistrée avec la représentation
Icônes/Schéma. Son absence vaut Élève. Le changer n’ouvre pas de dialogue et
n’interrompt pas une simulation en cours.

## Recettes

| Recette | Attendu |
| --- | --- |
| R-UX-001 | Au premier lancement, l’atelier est au niveau Élève; les commandes Expert sont absentes; passer en Expert les fait apparaître sans changer le projet ni sa révision. |
| R-UX-002 | Le niveau choisi est relu au démarrage suivant; un profil sans niveau est accepté et vaut Élève. |
| R-UX-003 | Construire, Tester et Annoter correspondent aux trois espaces existants, au clic comme par F2/F3/F4, et chaque espace affiche sa consigne dans les six langues. |
| R-UX-004 | Les captures de référence couvrent les six langues et les vues Icônes/Schéma pour la nouvelle barre. |
| R-UX-005 | Ajouter un ordinateur depuis la palette puis le relier au commutateur en deux clics : les trois messages d’étape apparaissent dans les six langues et les deux vues, et le lien est créé. |
| R-UX-006 | Un ping routé à travers deux routeurs anime des paquets et raconte l’envoi, le passage du routeur, la réception et la réponse avec son délai; sans passerelle, l’échec est expliqué; la trace n’est pas modifiée; six langues et deux vues. |
| R-UX-007 | Au niveau Élève, `ipconfig`, `arp` (vide puis rempli après un ping) et `ping` s’affichent en phrases dans les six langues; au niveau Expert, la même session affiche la sortie structurée; `route` et `ls` sont couverts sur des sorties de forme réelle. |
| R-UX-008 | Dans la fiche de A sans passerelle : aides visibles; avis « vide », puis « hors du réseau 192.168.10.0/24 », « invalide » et « correcte » pendant la saisie; rien n’est appliqué avant Appliquer; au niveau Expert les aides disparaissent et l’avis reste. |
