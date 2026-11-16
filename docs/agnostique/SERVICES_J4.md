# J4 — Configuration automatique, noms et pages web

Ce profil complète le [transport](TRANSPORT_J2.md) et le [routage](ROUTAGE_J3.md). Il couvre les vingt exigences J4 : F-OS-004, F-CLI-013, F-DHCP-001 à 006, F-DNS-001 à 006 et F-WEB-001 à 006. La [matrice de suivi](../suivi/MATRICE.md) relie ces exigences à l’adaptateur courant et aux [preuves](../verification/J4.md).

## Configuration et état

Le projet de version 5, profil `sns-educational-services-v1`, sauvegarde le choix DHCP, les paramètres du serveur, les enregistrements DNS avec leur TTL, les dossiers et les hôtes virtuels HTTP. Une acquisition DHCP ne transforme jamais le document en configuration statique. Le panneau réseau et `ipconfig` utilisent une projection effective et montrent adresse, préfixe, MAC, passerelle et DNS. Les baux, offres, refus, caches, connexions et pages en cours appartiennent à l’exécution; un redémarrage les reconstruit.

Les profils précédents restent lisibles. Ajouter un équipement, modifier une route ou éditer un fichier dans un projet de services conserve sa version. La conception, la simulation et la documentation restent distinctes; les six langues FR/EN/ES/IT/PT/DE et les vues Icônes/Schéma sont conservées.

## DHCP local

Le service est une fonction réseau activable d’un hôte ou d’une passerelle, attachée à une interface statique Ethernet. Le pool reste dans son sous-réseau, hors adresse réseau, diffusion et serveur; il contient au maximum 4 096 adresses. Les réservations par MAC sont exclues des offres aux autres clients. Masque, passerelle et DNS proviennent des valeurs spécifiques ou, à défaut lorsque l’héritage est activé, de la configuration du serveur.

Discover, Offer, Request, ACK/NACK et Decline traversent réellement le LAN sous forme de datagrammes simulés UDP 68/67 et 67/68. Le serveur distingue offres temporaires, baux et adresses refusées. Avant de demander l’adresse proposée, le client envoie une sonde ARP; une réponse ou une sonde concurrente invalide le candidat. Il signale le conflit par Decline puis recommence. Les offres durent 3 secondes, les refus 30 secondes et la sonde 250 ms. Les baux sont configurables de 1 seconde à 7 jours. Tous ces délais sont virtuels.

Trois tentatives DHCP sont suivies de trois candidats déterministes dans 169.254/16, chacun contrôlé par ARP. Le dernier conflit produit `dhcp.failed`; une acquisition locale enlève passerelle et DNS. À expiration d’un bail, les connexions de l’hôte sont libérées et une nouvelle acquisition commence. Le serveur rend les attributions expirées disponibles. Le modèle ne revendique ni renouvellement T1/T2, ni relais DHCP, ni négociation BOOTP complète.

La [RFC 2131](https://www.rfc-editor.org/rfc/rfc2131.html) sert de référence pour l’attribution, le refus d’une adresse en conflit et le cycle des baux. Les messages pédagogiques structurés ne sont pas des octets DHCP interopérables avec un équipement réel.

## DNS

Un serveur installé peut être démarré ou arrêté; fermer sa vue le laisse actif. Les enregistrements A, MX et NS sont éditables, supprimables et sauvegardés avec leur TTL, priorité MX et choix de récursion. Les noms sont normalisés en minuscules, sans point terminal. Les noms pédagogiques sont ASCII; l’internationalisation des noms et IPv6 restent hors de ce profil.

Le résolveur utilise des sockets UDP simulées sur le port 53. Il interroge le DNS configuré du poste, suit la délégation NS de suffixe le plus long et son adresse A de liaison. Un serveur récursif émet sa propre requête vers la cible ou son serveur amont; un serveur sans récursion retourne une délégation que le client suit. Le parcours est limité à huit serveurs et détecte les cycles. Chaque requête expire après trois secondes virtuelles. Les réponses distinguent succès, nom inexistant, type absent, boucle et absence de réponse.

Les réponses MX contiennent le nom cible et ses A supplémentaires quand ils sont présents. La livraison de courrier relève de J6. Chaque hôte dispose d’un cache indépendant, limité à 1 024 réponses; son expiration utilise le plus court TTL des réponses conservées. Une interrogation avec serveur explicite contourne la lecture du cache. La purge est disponible dans le navigateur et avec `dnscache -d`. Les suppressions restent effectives après expiration/purge et après sauvegarde/réouverture.

Le terminal expose `host [-t A|MX|NS] NOM [IP_DNS]` et `nslookup [-type=A|MX|NS] NOM [IP_DNS]`. Le résultat inclut le serveur et les enregistrements ou un état explicite. Ping, traceroute et le client TCP utilisent aussi le DNS configuré pour les noms; les alias locaux de topologie restent disponibles en absence de DNS configuré. Aucun résolveur du système n’intervient.

La [RFC 1034](https://www.rfc-editor.org/rfc/rfc1034.html) motive hiérarchie, récursion et durée de cache. Le contenu UDP est un message pédagogique structuré; compression binaire, DNSSEC, transferts de zone, réponses DNS sur TCP et cache négatif ne sont pas implémentés.

## HTTP et navigateur

Le serveur écoute un port configurable et lit exclusivement les fichiers virtuels de son hôte. `/` correspond à `index.html` sous le dossier configuré. Les noms d’hôtes virtuels sélectionnent des dossiers différents sur la même IP. Les noms de fichiers encodés sont décodés une fois; traversées hors du dossier, chemins invalides et encodages malformés sont refusés.

Le navigateur émet de vrais octets HTTP/1.1 sur le TCP simulé : ligne de requête, Host, Content-Type, Content-Length, séparation CRLF et corps UTF-8. Les en-têtes sont limités à 16 Kio et les contenus à la capacité du transport. L’analyse attend les segments manquants; les longueurs ambiguës, les champs répétés et Transfer-Encoding sont refusés. Le serveur répond notamment 200, 400, 403, 404 et 501. Les formes invalides et méthodes non réalisées ne déclenchent aucune exécution locale.

POST vers la route déclarée génère une page dynamique récapitulant les champs décodés et échappés. Ce traitement déclaratif remplace les extensions exécutables historiques; aucun script serveur ou client n’est lancé. Le navigateur rend textes, titres, liens, images et formulaires POST du sous-ensemble contrôlé. Chaque clic de lien et chaque image produit sa propre requête HTTP observable. PNG, BMP, GIF et JPEG sont servis avec leur type; le décodage graphique est borné.

Les références peuvent être relatives ou HTTP absolues. Elles passent toujours par le réseau simulé, y compris un nom public saisi par un élève. Les autres schémas, les identifiants dans les URL et les références ambiguës sont refusés. Le navigateur n’emploie ni vue web générale ni chargeur réseau du système.

Les ressources sont identifiées par hôte client, application, génération de navigation et URL absolue complète (nom, port, chemin). Une nouvelle navigation annule les anciennes requêtes et vide ses ressources. Les images homonymes de deux origines et les deux postes ne partagent donc pas de cache. Une page peut charger au maximum 32 images distinctes; les requêtes et connexions sont bornées et expirent. Ce profil ne comprend ni HTTPS, CSS, JavaScript, redirections automatiques, cookies ni conformité HTML générale.

La [RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html) sert de référence pour le cadrage des messages; le profil volontairement restreint refuse les combinaisons non réalisées.

## Recettes

L’exemple **DHCP, DNS et pages web** comporte deux clients automatiques, une réservation MAC et un serveur DNS/HTTP. En simulation : ouvrir le bureau du serveur, démarrer DNS puis HTTP, reprendre brièvement ou avancer pas à pas, ouvrir le navigateur de Poste A et charger `http://www.ecole.test/`. Observer DHCP, ARP, DNS, TCP et HTTP dans la trace. Le second nom `autre.ecole.test` sert un autre dossier sur la même adresse.

R-DHCP vérifie deux attributions, options exactes, réservation, épuisement, conflit, Decline, expiration, repli local et échec final borné. R-WEB vérifie délégation DNS, TTL/purge et sauvegarde des suppressions, HTTP à travers des routeurs, hôtes virtuels, images, lien, formulaire, 404/501 et isolation des ressources. Les [preuves J4](../verification/J4.md) précisent ce qui a réellement été exécuté.
