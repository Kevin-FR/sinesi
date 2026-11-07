# Matrice de fidélité — socle J0

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
