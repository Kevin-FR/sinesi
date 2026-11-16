# Cahier des charges — simulateur réseau pédagogique

## Objectif et public

Créer une application desktop moderne pour concevoir, exécuter et expliquer des réseaux informatiques virtuels. L’enseignant prépare et partage un scénario; l’élève configure les équipements, utilise des applications simulées et observe les échanges pour comprendre une réussite ou une panne.

Le périmètre fonctionnel de référence est Filius 2.14.0 dans l’archive locale. La liste normative des exigences est le [catalogue](CATALOGUE.md), alimenté par le [registre fonctionnel](../suivi/features.json). Ce document fixe leur intention, leurs limites et leurs règles de recette.

## Livrables produit

- Une application desktop utilisable hors ligne sur Windows, Linux et macOS.
- Un moteur de simulation exécutable avec ou sans interface graphique.
- Un éditeur de topologie, des inspecteurs de configuration, des applications d’hôtes et une vue des échanges.
- Un format de scénario portable, des exemples pédagogiques et un import des scénarios historiques avec rapport de conversion.
- Des exports de schéma, de traces et de rapport; un mécanisme d’extensions isolées.
- Des recettes reproductibles et une traçabilité reliant chaque exigence à sa réalisation et à sa preuve de validation.

## Périmètre de parité

| Ensemble | Comportements à préserver |
| --- | --- |
| Conception | Palette, placement, sélection multiple, câblage, gestion des ports, suppression, configuration et libellés |
| Équipements | Ordinateur, portable, switch/point d’accès, routeur multi-interface, routeur domestique LAN/WAN, modem entre instances |
| Accès au réseau | Ethernet, commutation avec apprentissage MAC, SAT vieillissante, ARP et cache, association Wi-Fi par SSID |
| Réseau | IPv4, sous-réseaux, loopback, diffusion, transfert, TTL, ICMP, routage statique et RIP pédagogique |
| Transport | Sockets simulées TCP/UDP, ports, handshake TCP, séquences, ACK, réessais et fermeture |
| Services | DHCP et link-local, DNS A/MX/NS, HTTP avec hôtes virtuels, SMTP/POP3, échange Gnutella simplifié |
| Sécurité pédagogique | NAT et redirections de ports, ICMP à travers NAT, pare-feu ordonné, politique de repli, journaux et administration web simulée |
| Hôtes | Bureau, catalogue de 13 logiciels, installation/désinstallation, cycle de vie, fichiers virtuels, terminal, éditeur et images |
| Observation | Échanges par interface, vue agrégée/par couches, inspection du contenu, purge et suivi, animation des liens, tables MAC/NAT, parcours entre couches expérimental |
| Documentation | Textes, zones colorées, styles, export PNG, rapport PDF de configuration et de trafic |
| Projets et environnement | Scénarios, préférences, langues FR/EN/ES/IT/PT/DE, aide, jeux d’icônes, options de lancement, journaux, exemples, logiciels personnalisés |

Le catalogue distingue `present`, `experimental`, `internal` et `removed`. Le helper terminal `test`, le carnet de contacts exposé par le modèle et les anciens chemins de code ne sont pas assimilés à des interfaces utilisateur complètes. L’export SVG est historique et retiré de l’interface de référence; sa réintroduction reste optionnelle.

## Modernisations nécessaires

La refonte doit apporter une horloge virtuelle déterministe, pause et pas à pas, édition réversible, navigation par zoom, données et contrats versionnés, import maîtrisé, autosauvegarde distincte du fichier utilisateur, exécution automatisable, explication des décisions et accès au clavier.

Le rejeu, les checkpoints, les pannes paramétrées, la duplication/alignement et les exports structurés arrivent après les premières tranches utilisables. Ces ajouts figurent explicitement comme modernisations et ne sont pas présentés comme des fonctions déjà offertes par le référentiel.

## Parcours prioritaires

### Élève : comprendre une connexion locale

Créer deux hôtes et un switch, configurer le même sous-réseau, lancer la simulation, ouvrir le terminal et exécuter ping. Inspecter la résolution ARP, l’échange ICMP et l’apprentissage du switch. Provoquer une panne, observer la cause et la corriger.

### Enseignant : préparer un exercice web

Assembler deux LAN, un routeur, un serveur DNS et un serveur web. Créer une page HTML et une image, définir un nom de domaine, enregistrer le scénario et ajouter les annotations. L’élève ouvre le scénario, résout la configuration erronée et produit un rapport montrant les échanges.

### Utilisateur avancé : réseau privé et public

Créer un routeur domestique, distribuer des adresses LAN, obtenir un WAN, tester le NAT sortant puis une redirection entrante. Ajouter un filtrage et expliquer les paquets refusés depuis leur trace et leur règle.

### Auteur d’extension

Créer une application depuis un modèle, consulter les diagnostics, l’installer dans un hôte et partager le scénario. Son code ne doit recevoir que les capacités du simulateur. Une extension importée ne démarre pas automatiquement.

## Règles métier transversales

1. L’identité d’un équipement est stable et indépendante de son nom ou de son adresse réseau.
2. La configuration persistante, l’état d’une exécution et la disposition des panneaux sont des données distinctes.
3. Une fermeture de fenêtre ne désinstalle pas le logiciel et n’arrête pas silencieusement le service.
4. Un changement de topologie requiert le mode conception et un arrêt propre de l’exécution en cours. Une édition d’annotation ne modifie pas les protocoles.
5. Aucune simulation ordinaire ne doit ouvrir une connexion sur le réseau réel. Le modem est un adaptateur explicitement activé et identifiable.
6. Le rendu d’une page HTML simulée charge ses ressources via les applications et protocoles simulés.
7. Les tables et événements expliquent le comportement observé; ils ne sont pas reconstruits à partir d’un état ultérieur mutable.
8. Les erreurs de configuration sont lisibles, localisées et réparables. Les paramètres invalides ne doivent pas corrompre le projet.
9. Les durées de protocoles dépendent du temps virtuel. La vitesse d’animation ne change pas les résultats de la simulation.
10. Un projet historique partiellement convertible fournit un rapport explicite; ses éléments non convertis ne sont pas ignorés silencieusement.

## Niveau de fidélité

La cible est la fidélité pédagogique des échanges et de leurs effets, avec un sous-ensemble de protocole documenté. Une valeur visible dans un en-tête ne suffit pas à prouver sa sémantique complète. Chaque protocole doit avoir une matrice « pris en charge / simplifié / non pris en charge » et des traces de référence.

Il n’est pas demandé pour la parité : une pile Internet conforme en totalité, un système d’exploitation invité, un moteur radio 802.11, une commutation avec prévention normalisée des boucles, un navigateur web généraliste, du chiffrement réseau complet ou la compatibilité binaire des extensions historiques. Les évolutions correspondantes sont isolées dans le domaine `ADV`.

Un bug connu n’est pas un comportement à reproduire : contamination entre sessions, cache partagé entre hôtes, ressources non libérées et trafic mal routé doivent être corrigés. Un comportement simplifié ayant un rôle pédagogique doit rester explicite ou être remplacé par une décision documentée avec recette de migration.

## Exigences mesurables de qualité

Les objectifs suivants sont des budgets initiaux à mesurer au jalon de fondation, pas des performances acquises :

| Dimension | Objectif initial | Mesure |
| --- | --- | --- |
| Réactivité | Réaction UI au 95e percentile ≤ 100 ms hors opérations lourdes annoncées | Parcours LAN avec 100 nœuds et 150 liens; latence entrée → rendu, sur machine de référence documentée |
| Capacité pédagogique | Charger et simuler un scénario de 100 nœuds, 150 liens et 10 000 événements conservés | Consigner mémoire, temps d’ouverture, temps de simulation et taille de trace; borner les files |
| Reproductibilité | Trace canonique identique à entrées, graine, versions et politique de protocole identiques | Deux exécutions indépendantes avec comparaison du hash canonique |
| Isolation | Aucun trafic réel et aucun accès implicite aux fichiers de l’utilisateur | Recettes hors ligne et instrumentation des adaptateurs |
| Robustesse | Une erreur d’import/écriture conserve le document précédent | Archives invalides et interruption d’écriture |
| Accessibilité | Parcours de base entièrement réalisable au clavier | Recette manuelle avec focus visible, annonce des états et libellés explicites |
| Portabilité | Même résultat fonctionnel sur les trois OS ciblés | Jeu commun de fixtures, tests du parcours et des exports sur chaque plateforme |

La taille maximale d’un projet, les quotas d’extension et le budget mémoire seront chiffrés après le prototype et inscrits dans une décision avant activation des fonctions concernées. En attendant, des limites finies conservatrices sont obligatoires; une saturation doit produire un événement explicite.

## Politique de validation et d’acceptation

Une exigence est `verified` si son critère du catalogue a une preuve reproductible, si ses dépendances sont satisfaites et si la réalisation correspondante a été contrôlée. La présence d’un fichier ou la réussite d’une compilation ne valide pas un protocole.

La livraison minimale utilisable se situe à J2. La première version pédagogique routée se situe à J4. La parité fonctionnelle cible est atteinte à J8, après les tranches NAT/pare-feu, applications avancées, documentation, import et extensions. Les fonctions retirées et les évolutions optionnelles de J9 ne bloquent pas cette parité.
