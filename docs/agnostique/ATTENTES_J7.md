# Attentes de réponse simulées — J7

Tranche de F-CFG-005; F-SIM-007/008 restent des invariants de reprise et de
déterminisme. Les autres options de lancement et la livraison de J7 restent ouvertes.

## Politique

Le facteur RTT est un entier de **1 à 5**, avec **1** par défaut. Il multiplie les
délais d’attente de réponse en temps virtuel. Il ne modifie ni la durée physique
d’une liaison, ni la vitesse d’affichage, ni le nombre de sauts d’un paquet.
Il s’agit d’une tolérance d’attente, pas d’une mesure réelle du RTT ni d’un délai
ajouté aux paquets. Un scénario utilisant des câbles lents peut ainsi attendre une
réponse sans produire une expiration prématurée.

| Attentes multipliées | Durées conservées |
| --- | --- |
| Résolution ARP, réponse ICMP | Rétention ARP et table MAC |
| Retransmission et attente de fermeture TCP | TIME_WAIT TCP |
| Réessais DHCP, vérification ARP d’un bail, réservation d’une offre en attente | Durée d’un bail acquis, quarantaine d’une adresse refusée |
| Réponse DNS, session HTTP cliente/serveur, recherche et sessions P2P, sessions de courrier | TTL des caches DNS, routes, sessions NAT et règles de filtrage |
| Attente d’un saut de traceroute et bilan final d’un ping | Intervalle d’envoi du ping du terminal, cadence RIP et expirations de routes |

La multiplication et l’addition à l’horloge refusent un débordement. Aucun sommeil
réel ni socket hôte n’est ajouté. Les commandes refusées conservent les garanties
transactionnelles existantes.

## Portée et fichiers

Le facteur est une option temporaire du lancement, distincte des préférences
d’affichage et du projet. Il s’applique avant l’initialisation des protocoles et des
applications. Une nouvelle simulation utilise ce facteur. Une reprise ou un rejeu
utilise celui de l’enregistrement, puis le lancement retrouve sa politique lors
de la création d’une nouvelle simulation. Une reprise ne remplace jamais le facteur
enregistré par une option courante.

| Facteur | Journal | Checkpoint et état |
| --- | --- | --- |
| 1 | Version 1, profil `sns-engine-semantics-v1`; facteur implicite 1 | Version 1, `structural-json-v1`, `sns-runtime-state-v1`; sortie historique conservée |
| 2–5 | Version 2, profil `sns-engine-semantics-v2`, champ `wait_factor` obligatoire | Version 2, `structural-json-v2`, `sns-runtime-state-v2`; facteur inclus dans le réseau et l’empreinte d’état |

L’encodage structurel des associations et grands entiers reste identique. Les
versions et profils doivent correspondre. Un journal version 2 sans facteur
étendu, un facteur hors limites ou un checkpoint dont le facteur ne reconstruit
pas l’état est refusé. Une continuation doit prolonger l’historique et la même
politique initiale. Les anciens projets et scénarios ne changent pas de version.
Les [schémas](../../contracts/README.md) fixent les enveloppes; la reconstruction
déterministe vérifie l’état complet.

Le rapport de démarrage conserve sa version 1 pour la politique standard. La
version 2 ajoute `runtime.wait_factor` pour une politique étendue réellement
appliquée. Cette donnée ne rejoint pas le profil sauvegardé.

## Recette reproductible

Le [scénario de réseau lent](../../fixtures/valid/response-wait-scenario.json) utilise
deux câbles à 300 000 microsecondes. Avec le facteur 1, l’attente ARP expire avant
la réponse. Avec le facteur 5, le ping aboutit; la durée des câbles reste identique.
Une capture en cours d’échange, son rejeu et une continuation retrouvent la même
trace canonique. Le facteur ne change ni les durées des baux acquis ni la cadence
des pings ou de RIP. La [qualification](../verification/J7-ATTENTES.md) relie ces
résultats aux commandes réellement exécutées.
