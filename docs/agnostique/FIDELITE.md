# Matrice de fidélité — socle J0

## Extension J6

Le [profil des applications et connexions](APPLICATIONS_J6.md) précise courrier textuel, overlay P2P borné, SSID logique et tunnel de modem explicite. Il distingue les échanges simulés des sockets réelles du pont et réserve le rejeu externe à J7.

## Extension J5 réalisée

Le [profil privé](RESEAUX_PRIVES_J5.md) précise NAT TCP/UDP/ICMP, rôles LAN/WAN, règles entrantes et administration HTTP. Les simplifications sont explicites : pas de suivi complet des connexions, de sommes de contrôle binaires ni de hairpin. Les matrices des jalons précédents restent historiques.


## Extension J4 réalisée

Le [profil de services](SERVICES_J4.md) précise les acquisitions DHCP, sondes ARP, TTL/cache et délégations DNS, octets HTTP sur transport simulé, documents contrôlés et isolation des images. Les messages DHCP/DNS sont des modèles pédagogiques structurés; les extensions exécutables historiques sont remplacées par des formulaires déclaratifs. La matrice J0 ci-dessous reste historique.


## Extension J3 réalisée

Le [profil routé](ROUTAGE_J3.md) réalise transfert/TTL, routes, traceroute, RIP, injection ARP et explications. La sélection et les timers diffèrent explicitement du référentiel. Les [preuves J3](../verification/J3.md) couvrent R-ROUTES et R-RIP; elles ne certifient pas un routeur réel. Les symboles graphiques supplémentaires restent distincts des capacités de protocole.

## Extension J2 réalisée

Le sous-ensemble [TCP/UDP](TRANSPORT_J2.md), les [hôtes et logiciels J2](HOTES_J2.md), les fichiers, le terminal et l’observation sont réalisés avec leurs limites explicites. Le [rapport J2](../verification/J2.md) qualifie R-TCP et R-FICHIERS. La présence des 13 logiciels au catalogue ne démontre aucun protocole de J3 à J6.

## Extension J1 réalisée

Ethernet, commutation, ARP, IPv4 local et ICMP sont maintenant réalisés dans le sous-ensemble explicite [LAN_J1](LAN_J1.md). La [recette J1](../verification/J1.md) en donne les preuves et les limites. La matrice ci-dessous conserve la photographie historique du socle J0.

Cette matrice décrit les capacités effectivement démontrées et leurs limites. Elle distingue les échanges de qualification des futurs protocoles réseau. Aucun nom de champ, d’application ou de scénario ne prouve une conformité à un standard.

| Capacité | État réel dans J0 | Limite / validation ultérieure |
| --- | --- | --- |
| Temps et événements | Entiers virtuels, ordre temps/priorité/séquence, jitter reproductible, quotas, pause/pas et bornes | Checkpoints complets à J7; aucune horloge de protocole réelle |
| Identités | Projet, équipements/interfaces, commandes, exécutions et corrélations stables dans leur contexte | Édition complète du graphe à J1 |
| Livraison de qualification | Messages envoyés et livrés sur le graphe, chemin en largeur trié, cause de route absente | Aucun apprentissage MAC ni choix de route IPv4 dans ce parcours |
| Ethernet / switch / ARP | Non implémentés | Trames, apprentissage et résolution requis à J1, F-L2-* |
| IPv4 / ICMP | Configurations conservées et validées comme données, pas de transfert de paquets | Adressage effectif, loopback, echo et TTL à J1/J3, F-IP4-* |
| TCP / UDP | Non implémentés | Ports, états, segmentation, acquittements et datagrammes à J2, F-TCP-*, F-UDP-* |
| RIP | Non implémenté | Annonces, métriques et expiration à J3, F-ROUT-004/005 |
| DHCP / DNS | Non implémentés | Services et leurs matrices de sous-ensemble à J4, F-DHCP-*, F-DNS-* |
| Requêtes applicatives web | GET/POST structurés, routes locales, réponse/code/type/corps et écho de formulaire via les événements de qualification | Pas de sérialisation HTTP filaire, de TCP ni de DNS; services web complets à J4 |
| Document HTML | Sous-ensemble texte/titres/liens/image/formulaire POST; ressources virtuelles; refus des scripts et sorties externes | Pas de conformité HTML5/CSS ni de moteur généraliste; pages des exercices à compléter à J4/J5 |
| Images | PNG de qualification décodé sous quota depuis une réponse événementielle | Formats supplémentaires et couverture par hôte à J4 |
| NAT / pare-feu | Non implémentés | Tables, transformations, ordre de règles et administration à J5 |
| Mail / P2P / modem | Non implémentés | Contrats et recettes à J6 |
| Persistance | Projet versionné et scénario de recette en données; original de migration conservé et remplacement atomique | Archives, import .fls, récupération et extensions aux jalons prévus |
| Observation | Faits de qualification immuables, corrélés; export canonique et table de 10 000 lignes | Les quatre couches pédagogiques et leurs en-têtes apparaîtront avec les vrais protocoles |

La recette J0 compare les octets des traces et teste le chargement de page/image/formulaire via le service de qualification. Elle ne remplace pas R-LAN, R-TCP, R-DHCP ou R-WEB. Une nouvelle famille de protocole doit publier ses états, champs actifs, délais, erreurs, simplifications et cas non pris en charge avant d’être déclarée vérifiée.

## Diagnostic terminal J7

Le [profil du diagnostic interne](DIAGNOSTIC_TERMINAL_J7.md) précise F-CLI-019 : le helper historique `test` écrit `test.txt`. Il est absent de l’aide mais accessible par le sélecteur dynamique historique. Sinesi le réserve à une recette interne sur une copie du système de fichiers virtuel et refuse `test` / `help test` dans le terminal élève. Cette restriction explicite est une différence assumée du profil pédagogique; elle ne correspond pas à la perte d’une commande réseau.
