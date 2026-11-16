# J5 — Réseaux privés, traduction et filtrage

Ce profil complète [le routage](ROUTAGE_J3.md) et [les services](SERVICES_J4.md). Il couvre F-EQU-004, F-ROUT-006, F-WEB-007, F-NAT-001 à 006, F-FW-001 à 007 et F-OBS-007 : dix-sept exigences. La [matrice](../suivi/MATRICE.md) relie les comportements à l’adaptateur courant et aux [preuves J5](../verification/J5.md).

## Routeur domestique et persistance

Un équipement distinct possède exactement deux interfaces Ethernet, désignées LAN et WAN par leur identité. Les rôles restent attachés aux interfaces pendant le câblage, l’édition et la sauvegarde; ils ne dépendent pas de l’ordre d’une liste. Le LAN a une adresse statique et le WAN peut être statique ou automatique. Les sous-réseaux statiques doivent être distincts. Une attribution DHCP WAN qui chevauche le LAN est également refusée explicitement. Le serveur DHCP éventuel est attaché au LAN; sa passerelle distribuée peut être différente de celle acquise sur le WAN.

Le routeur créé par la palette possède un LAN 192.168.0.1/24, un WAN DHCP, un pool LAN 192.168.0.100–200, une traduction active et une administration HTTP interne. Le filtrage reste désactivé tant qu’il n’est pas configuré. Les adresses proposées sont des valeurs initiales éditables, sans accès à un réseau réel. L’exemple **Réseau privé, NAT et pare-feu** emploie deux postes, un commutateur, un routeur domestique, un routeur WAN et un serveur externe.

Le projet version 6, profil `sns-educational-private-v1`, conserve les rôles, règles, politique, options, redirections et service d’administration. Les mappings et journaux restent dans l’exécution; ils ne réapparaissent pas après redémarrage. Les éditions suivantes conservent le profil. Les six langues FR/EN/ES/IT/PT/DE et les représentations Icônes/Schéma restent disponibles.

## Ordre des traitements

| Étape | Comportement |
| --- | --- |
| Entrée Ethernet | Vérifier la destination MAC; ARP reste indépendant du pare-feu IPv4 |
| Filtrage entrant | Décider sur les adresses et ports reçus, avant toute traduction; journaliser la règle ou la politique |
| Services et frontière | Traiter DHCP local; sur WAN, refuser source ou destination appartenant au LAN privé, y compris l’adresse LAN du routeur |
| Traduction entrante | Pour une réponse ou une redirection publiée, restaurer la destination privée et le port/identifiant |
| Livraison ou transfert | Livrer les services du routeur, sinon choisir la route et vérifier portée/TTL; TTL expiré produit une erreur avant traduction sortante |
| Traduction sortante | Lors d’un transfert LAN→WAN, remplacer source et port/identifiant, puis effectuer ARP et l’émission |

Ainsi une règle WAN qui autorise une redirection vise son **port externe**, tandis que la règle sur le serveur LAN vise son port interne. Un mapping existant ne contourne jamais le pare-feu. Les flux à destination des adresses propres du routeur restent locaux. L’accès direct du WAN au LAN demeure refusé même si un routeur WAN connaît une route vers ce LAN.

## NAT pédagogique

Les flux TCP et UDP sortants reçoivent un endpoint WAN distinct. La clé comprend protocole, endpoint interne, destinataire et port distant. Les réponses doivent correspondre à cet endpoint et au même correspondant. TCP et UDP ont des espaces de ports distincts. Le port source est conservé lorsqu’il est disponible dans le pool; sinon le premier port libre est choisi dans l’ordre numérique. Les ports des redirections et des sockets du routeur sont réservés.

Une nouvelle entrée TCP exige un SYN initial; UDP crée une entrée pour un datagramme sortant. Les retransmissions réutilisent le mapping. Les redirections associent protocole et port WAN à une adresse/port LAN, avec un port externe pouvant différer du port interne. Le retour utilise le même port public. Deux postes ayant le même port source ne partagent pas leurs réponses.

Les requêtes ICMP reçoivent un identifiant public distinct; réponses et identifiants privés sont restaurés. Les erreurs de destination inaccessible et de TTL transportent une copie du paquet original. Le NAT traduit aussi cette copie et sa destination extérieure, y compris lorsque l’erreur provient d’un routeur intermédiaire. Un échec ARP sur le WAN restitue l’erreur au poste privé à partir de son paquet d’origine. Ping et traceroute restent donc corrélés pour plusieurs postes.

Une transformation produit un nouveau fait contenant les représentations avant/après. Les trames historiques ne sont jamais réécrites. Les tables exposent protocole, endpoints privés/publics, correspondant, nature dynamique/redirigée et échéance. Elles se rafraîchissent en simulation; leur purge enlève les sessions actives, sans effacer les redirections configurées.

Le temps d’inactivité est virtuel, configurable de 1 seconde à 7 jours, avec 60 secondes proposées. Une communication rafraîchit son échéance; l’expiration libère l’entrée et son port. La table est limitée à 1 024 sessions et les redirections à 128 par routeur. Une saturation ou un épuisement de ports produit un refus explicite. Les expirations et changements d’adresse WAN purgent les traductions concernées. Une expiration DHCP sur une interface conserve les services et connexions liés à l’autre interface, notamment l’administration LAN.

Les [RFC 3022](https://www.rfc-editor.org/rfc/rfc3022.html) et [RFC 5508](https://www.rfc-editor.org/rfc/rfc5508.html) motivent la traduction des endpoints et du contenu des erreurs ICMP. Ce profil utilise les paquets typés du simulateur; il ne revendique ni sommes de contrôle binaires, ni prise en charge des fragments, des protocoles auxiliaires ou des applications nécessitant une réécriture de contenu. Hairpin NAT, NAT IPv6 et conformité complète aux comportements normalisés sont hors de ce jalon.

## Pare-feu

Le même filtre fonctionne sur les entrées Ethernet d’un hôte, d’un routeur ou du routeur domestique. Il peut être activé/désactivé; arrêter ou désinstaller son logiciel désactive le filtrage. Fermer la fenêtre garde la configuration active. L’inspecteur et le logiciel présentent le même modèle.

Les règles sont ordonnées : première correspondance, puis politique ACCEPT ou DROP en l’absence de correspondance. Ajout, modification, suppression, montée et descente sont possibles. Source et destination acceptent une adresse/préfixe canonique, `*` pour tout ou `LAN` pour le réseau local. Sur un routeur domestique, LAN désigne exclusivement son interface LAN; sur les autres hôtes, il désigne leurs réseaux directement connectés. Le protocole est TCP, UDP ou tous les deux; le port destination est facultatif.

Le mode TCP proposé filtre uniquement les SYN initiaux, sans ACK. Les autres segments passent sans prétendre qu’une connexion a été suivie. Le mode tous segments applique les règles à chaque segment reçu; les règles de retour doivent alors être adaptées. UDP peut être filtré ou laissé passer. Avec filtrage UDP, une règle à port explicite peut aussi correspondre dans le sens inverse, en comparant le port source et les adresses inversées. Cette compatibilité pédagogique reprend le comportement de Filius : elle ne constitue pas un suivi de connexion et n’exige pas un datagramme antérieur.

ICMP dispose d’une commande globale autoriser/bloquer, indépendante des règles TCP/UDP et de la politique par défaut. ARP et les communications locales qui ne traversent pas Ethernet ne sont pas filtrés. Il n’y a ni REJECT, ni inspection applicative, ni pare-feu avec suivi complet des connexions.

Chaque décision active est corrélée à une trame, une interface et un instant virtuel. Elle indique adresses/ports, protocole, acceptation/refus, numéro de règle à partir de 1 ou politique/mode responsable. Le journal retient les 1 024 dernières décisions par équipement. Les traces de référence restent immuables et distinctes de cette projection bornée.

## Administration HTTP simulée

Un serveur HTTP dont les pages d’administration sont activées démarre avec l’exécution. Il utilise le même transport que les pages des exercices. Pour le routeur domestique, son accès est limité au LAN; une requête venant du WAN reçoit 403. Les autres équipements acceptent leur réseau directement connecté et leur boucle locale simulée. Ces pages n’ouvrent aucune interface d’administration réelle.

Les pages `/admin`, `/admin/routes`, `/admin/nat`, `/admin/firewall` et `/admin/log` exposent les routes, traductions, règles et décisions du moteur. Les formulaires changent activation, politique, modes, règles et redirections ou purgent la table. La route présentée est la même que celle de l’inspecteur. Chaque modification passe par la même validation que les commandes natives et laisse la configuration intacte si elle est invalide. Avant d’activer une politique DROP, prévoir une règle TCP autorisant le port d’administration; le panneau natif permet de corriger un blocage de la page.

Un formulaire porte une empreinte de la configuration affichée. Si cette configuration a changé entre lecture et envoi, la réponse est 409 et demande un rechargement. Cette empreinte détecte les formulaires obsolètes; ce n’est pas un mécanisme d’authentification. Les pages sont des documents internes déclaratifs, sans exécution des extensions historiques. Elles proposent leurs six langues dans la page, séparément de la préférence de l’interface.

Les listes longues sont paginées pour rester dans les limites du navigateur; les règles ont 16 lignes par page. Ces pages constituent une administration pédagogique, sans HTTPS, authentification, contrôle d’accès multi-utilisateur ou garantie de sécurité d’un appareil réel.

## Recettes et référence historique

R-NAT vérifie collisions TCP/UDP, redirections TCP/UDP, ICMP et paquets encapsulés, frontières, purge, expiration, saturation de ports, DHCP indépendant et immutabilité. R-FILTRE vérifie ordre, politique, masques, ports, modes TCP/UDP/ICMP, décisions, changement via HTTP et convergence avec l’inspecteur. Les [preuves](../verification/J5.md) précisent commandes et environnement.

Les chemins appelés de la référence ont été examinés : `NatGatewayLANThread`, `NatGatewayWANThread`, `Firewall.acceptIPPacket`, `checkAcceptTCP`, `checkAcceptUDP`, ainsi que les pages internes. Le filtre entrant avant traduction et le mode UDP inversé sont conservés explicitement. Les traitements concurrents et horloges réelles sont remplacés par l’ordonnancement virtuel. Les limites des jalons antérieurs restent valables.
