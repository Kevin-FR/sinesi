# Transport J2 — sous-ensemble du cœur

État : le cœur des sockets et ses recettes techniques sont implémentés. La porte complète J2 (bureaux, logiciels, terminal, fichiers et observation) reste ouverte. Le suivi détaillé est dans la [matrice](../suivi/MATRICE.md); le [rapport intermédiaire](../verification/J2-EN-COURS.md) distingue le code présent des intégrations restantes.

## Sockets, ports et sessions

Les identités de socket sont propres à un équipement. Leur propriétaire est une identité applicative opaque : un arrêt/désinstallation pourra ainsi libérer toutes ses sockets. Chaque écoute réserve un protocole, une adresse locale ou toutes les adresses, éventuellement une interface, et un port. Une réservation de port zéro choisit un port éphémère de 49152 à 65535. TCP et UDP ont des espaces de ports séparés. Les réservations incompatibles sont refusées sans modifier tables, trace ou file.

Une session TCP possède ses deux endpoints et ses compteurs de séquence. Une écoute peut accepter plusieurs clients. L’identité d’un client ne se réduit pas au numéro du port serveur. Les sessions passives héritent du propriétaire de l’écoute. Les vues de sockets exposent propriétaire, protocole, interface, endpoints et état.

## TCP

L’ouverture produit SYN, SYN/ACK puis ACK. Les états exposés comprennent SYN_SENT, SYN_RECEIVED, ESTABLISHED, FIN_WAIT_1, FIN_WAIT_2, CLOSE_WAIT, LAST_ACK, CLOSING et TIME_WAIT. Une cible muette finit par expirer; les doublons de SYN/ACK ou d’ACK final ne créent pas de seconde connexion applicative.

Le transport est un flux d’octets, pas une liste de messages texte. Le contenu binaire et les caractères multioctets sont préservés. Le sous-ensemble segmente à 512 octets et conserve au plus un segment en vol par direction, avec une fenêtre annoncée de 512 octets. Les séquences sont sur 32 bits avec arithmétique modulaire; SYN et FIN consomment chacun une unité. Les ACK valides libèrent les octets en attente et permettent le segment suivant. Les doublons de données sont acquittés sans seconde livraison applicative; un segment hors séquence n’est pas livré.

Le délai initial de retransmission est de 500 000 µs virtuelles, doublé à chaque réessai, au plus quatre réessais après l’émission initiale. Une minuterie périmée n’agit pas sur un segment plus récent ni sur une socket disparue. Un échec final ferme la session avec diagnostic `tcp.timeout`.

Une fermeture normale attend les données en cours, puis utilise FIN/ACK. La réception d’une fermeture provoque une fermeture réciproque après vidage des données en attente : la demi-fermeture durable n’est pas exposée dans ce profil. TIME_WAIT dure 2 s virtuelles; FIN_WAIT_2 est borné à 5 s. Une fermeture forcée produit RST et libère immédiatement les ressources locales. Une fermeture par propriétaire inclut écoute et sessions acceptées; le port peut être réutilisé immédiatement. Les paquets encore en vol restent des événements de cette exécution jusqu’à leur traitement; arrêter l’exécution complète vide la file.

Il n’y a pas de sockets système, de congestion TCP, d’estimation RTT adaptative, de SACK, d’options TCP, de fenêtre dynamique, de buffering hors ordre ni de protection normalisée contre les RST forgés. Les ISN sont déterministes et non cryptographiques. Ce n’est pas une implémentation conforme à l’intégralité de [TCP RFC 9293](https://www.rfc-editor.org/rfc/rfc9293.html).

## UDP

Un envoi conserve ses frontières de datagramme et sa source/destination. L’écoute est associée au port, à l’adresse et, si choisi, à l’interface. Une perte n’entraîne aucun réessai de transport. Un port sans écoute produit un abandon explicite, sans promettre pour l’instant un ICMP port unreachable filaire.

La diffusion limitée et la diffusion du sous-réseau utilisent une trame Ethernet broadcast sur l’interface d’émission. Une émission avant configuration IP peut utiliser 0.0.0.0 comme source, notamment pour les futurs exercices DHCP. La diffusion atteint les interfaces pertinentes du LAN physique; le cœur J2 ne la relaie pas vers un autre LAN. TCP refuse les connexions vers une adresse de diffusion.

La charge UDP est limitée à 65 507 octets; la longueur inclut ses huit octets d’en-tête. Le checksum et l’encodage filaire ne sont pas simulés. Référence des champs : [UDP RFC 768](https://www.rfc-editor.org/rfc/rfc768.html).

## Intégration, limites et contrats

Tous les paquets passent par la pile Ethernet/ARP/IP déjà réalisée. Une retransmission produit une nouvelle trame Ethernet tout en conservant la séquence TCP; la protection des boucles Ethernet ne doit pas la confondre avec un doublon de trame. Une livraison locale ou loopback ne crée aucune émission externe.

Le cœur borne l’ensemble à 1 024 sockets, 1 Mio de données en attente par session et 8 Mio en attente au total. La saturation produit un diagnostic transactionnel. Les limites existantes de file, faits et commandes restent actives. Le temps réel et la vitesse de rendu ne règlent aucun timeout.

Les événements transport utilisent la version 3; les faits LAN dont la forme reste inchangée gardent leur version 2. `ipv4_transport` est le discriminant de la charge structurée contenant TCP ou UDP, tous deux encapsulés avec EtherType 0x0800. Les discriminants TCP/UDP correspondent respectivement aux numéros de protocole IP 6 et 17. Les données sont en base64 et les temps/séquences de trace restent des chaînes décimales. Les séquences TCP sont des entiers non signés sur 32 bits.

Voir [transport-command.schema.json](../../contracts/transport-command.schema.json) et [transport-event.schema.json](../../contracts/transport-event.schema.json). Ces contrats ne contiennent aucune préférence de langue. L’interface affiche les noms de protocole et identités sans changer leur valeur persistante.
