# Audit du référentiel Filius

## Corpus

Référence : `_archive/filius-master`, version **2.14.0** déclarée dans `pom.xml`, changelog daté du **29 août 2026**. Audit établi le **7 novembre 2026** sur la copie locale, sans présumer l’identité d’un commit distant. Les empreintes de [sources.json](sources.json) identifient précisément cette copie.

| Mesure | Résultat |
| --- | ---: |
| Fichiers présents, y compris métadonnées et runtime embarqué | 1 061 |
| Fichiers Java de production | 262 |
| Lignes Java de production, commentaires compris | 54 962 |
| Classes Java de tests | 21 |
| Scénarios livrés dans `beispiele/` | 18 |
| Fixtures .fls de tests supplémentaires | 2 |
| Classes de production citées directement dans les fiches | 81 |
| Logiciels du catalogue FR des hôtes | 13 |

L’[index de sources](SOURCES.md) rattache tous les fichiers de production à des familles fonctionnelles. Le JSON complet conserve empreinte, taille et déclarations publiques/protégées pour parcourir les classes de support. Un rattachement structurel n’est pas une preuve que tous les chemins d’exécution ont été testés.

## Méthode

1. Inventaire de l’arborescence, des types matériels, piles logicielles, fenêtres, ressources, configuration, exemples et tests.
2. Index des déclarations de production et regroupement par capacité, plutôt que traduction littérale de chaque classe en une fonctionnalité.
3. Lecture ciblée des implémentations et de leurs points d’entrée : menus, contrôles, automates, commandes, tables, persistance et services.
4. Croisement avec le catalogue `Desktop_fr_FR.txt`, l’aide FR/EN, le manuel Markdown, `filius.ini`, les options de lancement et le changelog.
5. Analyse des 18 ZIP de scénarios en lecture seule : entrées, configuration, classes et code embarqué. Aucun XML JavaBeans n’a été exécuté.
6. Transformation des capacités en exigences identifiées, critères de recette et jalons, puis liaison à des lots d’adaptation séparés.
7. Contrôle des preuves ligne par ligne, des empreintes, dépendances, références et vues générées.

L’inventaire vise la totalité des **capacités produit identifiables statiquement** dans ce corpus. Il ne constitue pas une revue manuelle ligne par ligne des 54 962 lignes, une certification de conformité des protocoles ou un essai de toutes les interactions UI. Lors de cet audit initial, Filius et ses tests historiques n’ont pas été exécutés; les validations ultérieures du socle Sinesi sont suivies séparément dans `docs/suivi/ETAT.md`. Les comportements nouveaux et renforcés sont donc des exigences cibles, pas des résultats déjà acquis.

## Lecture des preuves

Chaque fiche d’origine `filius` contient un chemin, une ligne, une ancre et le texte exact de cette ligne. L’ancre pointe un point d’entrée ou un modèle pertinent; l’ensemble de sa famille et les autres méthodes sont accessibles dans l’index. Une seule ligne de preuve ne prétend pas démontrer toute la recette future.

Les catégories du référentiel sont :

| Catégorie | Interprétation |
| --- | --- |
| `present` | Capacité exposée par un chemin actif ou modèle utilisé; les détails cibles peuvent renforcer sa robustesse |
| `experimental` | Fonction proposée derrière un réglage expérimental ou encore instable selon le référentiel |
| `internal` | Primitive du modèle/helper sans supposer une UI complète |
| `removed` | Fonction historique retirée du parcours actif malgré des traces résiduelles |
| `absent_du_referentiel` | Ajout de modernisation ou évolution optionnelle, sans attribution à Filius |

Les états `not_started` à `verified` concernent exclusivement le **nouveau produit**. Ils ne sont pas déduits de l’existence d’une classe historique.

## Correspondance du vocabulaire

| Source allemand/nom de package | Sens dans le cahier des charges |
| --- | --- |
| `Rechner`, `Notebook`, `Host` | Ordinateur, portable et classe d’hôte |
| `Vermittlungsrechner` | Routeur multi-interface |
| `Gateway` | Routeur domestique avec LAN/WAN et NAT |
| `Switch`, `SwitchFirmware`, SAT | Commutateur/point d’accès pédagogique et table d’apprentissage MAC |
| `NetzwerkInterface`, `Port`, `Kabel` | Interface réseau, port et liaison |
| `Netzzugangsschicht` | Couche d’accès réseau, dont Ethernet |
| `Vermittlungsschicht` | IPv4, ICMP, ARP et routage dans l’organisation historique |
| `Weiterleitungstabelle` | Table de transfert/forwarding, distincte des routes RIP apprises |
| `Anwendung`, `Betriebssystem` | Application et environnement d’hôte simulé |
| `Lauscher`, `Nachrichtensicht` | Collecte et vue des échanges |
| `Schichtensicht` | Visualisation du parcours entre couches |
| `Baustein`, `Mitarbeiter` | Application générique et traitement de connexion côté serveur |
| `SzenarioVerwaltung`, `.fls` | Gestion des scénarios et format historique |

## Divergences et limites vérifiées dans le code

| Sujet | Observation statique | Décision du dossier |
| --- | --- | --- |
| SVG | L’aide documentation FR annonce PNG/SVG; le changelog 2.6.0 indique son retrait. `GUIContainer.exportAsImage` ne propose que PNG; `exportAsSVG` subsiste comme méthode privée | F-DOC-003 = PNG actif; F-DOC-005 = historique retiré, option J9 |
| DNS et TTL | `ResourceRecord` stocke un TTL. `DNSServer.cachedRrList` représente les enregistrements édités/persistés; la lecture de `Resolver.resolve` ne démontre pas un cache de résolution à expiration | F-DNS-004 conserve données/TTL; F-DNS-006 ajoute explicitement un véritable cache |
| Wi-Fi | Association par SSID dans `JHostKonfiguration`; changement du SSID du switch retire les liens wireless | Préserver cette abstraction; ne pas annoncer radio, portée, interférences ou WPA |
| Boucles de switch | `SwitchPortBeobachter` et `SwitchFirmware` mémorisent les trames déjà passées | Protection pédagogique à borner; aucun protocole normalisé de spanning tree déduit |
| Pare-feu | Premier match, ACCEPT/DROP, mode SYN seul, réglage ICMP et traitement des retours UDP | Préserver des modes explicitement pédagogiques; ne pas les présenter comme un suivi de connexion complet |
| RIP | Annonces UDP, métrique en sauts, infini 16 et expiration; `RIPTable` contient ses propres durées et `Weiterleitungstabelle` sa sélection | Matrice de fidélité obligatoire; aucune conformité RIPv2 complète déduite |
| DHCP | Enum des messages : DISCOVER, REQUEST, ACK, NACK, OFFER, DECLINE. Les commentaires évoquent aussi RELEASE/INFORM, sans les inclure dans cette enum | Ne pas promettre RELEASE/INFORM ou le cycle complet T1/T2; documenter la matrice cible avant extension |
| Link-local | Repli explicite du client après cinq échecs DHCP; génération historique sous `169.254.1.*`, masque /16 | Préserver l’intention et valider les conflits; la plage exacte peut être modernisée par décision |
| TCP | Automate de connexion/fermeture, ACK, segmentation et réessais; plusieurs champs d’en-tête existent dans les modèles | Ne pas confondre présence de champs avec fenêtres/congestion/options/checksums pleinement simulés |
| Navigateur | Client HTTP et affichage pédagogique avec récupération des images, pas navigateur Internet généraliste | Rendu modernisé mais toujours alimenté par la pile simulée |
| Web | Port 80 par défaut et tableau historique de cinq vhosts; API héritée de port côté serveur | Réglages cibles explicites; ne pas prétendre qu’un éditeur de port est déjà offert dans l’écran web historique |
| Mail | To/CC/BCC dans le modèle; `Email.toString` n’émet pas BCC; gestion de contacts dans `EmailAnwendung` | Recetter les champs et distinguer API contacts d’une UI de carnet complète |
| Terminal | `exit`, `reset`, `cls` sont traités par la fenêtre; `test` écrit un fichier de diagnostic; l’aide mentionne moins que tous les points d’entrée | Les recenser séparément; `test` reste hors parcours élève |
| netcat et shell | Un commentaire d’introduction du terminal évoque netcat; cela ne constitue pas un dispatch fonctionnel démontré | Ne pas l’inventer comme commande disponible; client/serveur générique couvre les transferts texte existants |
| Logiciels | Fermer fenêtre/bureau ne signifie pas arrêter l’application; désinstaller libère les ressources | Reprendre la distinction mais rendre démarrer/arrêter visible |
| `.fls` | ZIP contenant configuration XML JavaBeans, données, et parfois sources/classes personnalisées | Nouveau format en données et import à liste autorisée; aucune exécution de XML/classes |
| Exemples personnalisés | `anwendung_komplett.fls` et `multicast_server.fls` référencent Accumulator/Broadcast embarqués | Classer comme extensions à porter; leur nom ne prouve pas un support multicast IP natif |
| Versions et prérequis | README évoque encore JRE 8; `pom.xml` fixe Java 17 | Pour l’analyse technique de l’archive, retenir le build; ne pas recopier une indication README obsolète |

## Commandes terminal recensées

Commandes métier : `arp`, `arpsend`, `cat/type`, `cd`, `copy/cp`, `del/rm`, `dir/ls`, `help`, `host`, `ipconfig`, `mkdir`, `move/mv`, `netstat`, `nslookup`, `ping`, `pwd`, `route`, `tcpdump`, `touch`, `traceroute`.

Commandes de fenêtre : `exit`, `reset`, `cls`. Interactions : historique haut/bas, interruption Ctrl+C. Helper interne : `test`. Le ping propose un mode broadcast, un nombre de répétitions configuré et des statistiques; traceroute commence par TTL=1. `route` est une consultation, pas une preuve de tous les sous-commandes d’un outil système homonyme.

## Configuration historique à préserver par intention

Sources : `src/main/resources/config/filius.ini`, `Information.loadIni`, `FiliusArgs` et `LanguageSelectionDialog`. Les clés historiques ne doivent pas imposer le format du nouveau fichier de préférences.

| Clé/option | Intention | Traitement cible |
| --- | --- | --- |
| `locale` | FR/EN/DE | Préférence localisée portable |
| `rtt`, option `-r` | Adapter les délais à la machine, facteur 1–5 | Remplacé dans le moteur par temps virtuel; conserver une vitesse de présentation indépendante |
| `native-look-n-feel`, `-n` | Apparence adaptée à l’OS | Préférence d’affichage, sans dépendance de protocole |
| `posix-behaviour` | Ancien mode ping dix répétitions | Compatibilité documentée; déprécié dans le référentiel |
| `ping-repeat` | Nombre de requêtes ping, quatre par défaut | Paramètre pédagogique explicite |
| `old-exchange-dialog` | Choisir ancienne/nouvelle vue des échanges | Une seule source de faits, deux projections si utiles |
| `desktop-mode` | Arrangement row/column/stack | Disposition des vues d’hôtes |
| `software-wizard` | Visibilité de l’assistant selon environnement | Disponibilité et diagnostics explicites de l’outillage d’extension |
| `pane-width`, `pane-height` | Taille de la surface de conception | Canevas navigable/zoomable; import des positions |
| `gateway` | Disponibilité de la passerelle NAT | Politique de palette ou disponibilité de fonctionnalité |
| `terminal-font-size` | Police du terminal | Taille adaptable sans casser les colonnes |
| Réglages de langue/icônes/couches | Langue, trois jeux d’icônes, visualisation expérimentale | Préférences séparées du projet |
| `-h`, `-v`, `-w/--wd`, `-l` | Aide, verbosité, dossier de travail, journal | Interface CLI documentée et chemins explicites |
| `-s` | Affichage pour petite résolution | UI adaptable et parcours clavier |
| Argument fichier | Ouvrir un scénario au lancement | Ouverture explicite du projet et association de fichiers |

La priorité exacte CLI/profil/configuration globale devra être définie pour la réécriture; ne pas reprendre les exceptions historiques implicites comme contrat portable.

## Tests historiques et corpus de recette

Les 21 classes de tests historiques couvrent notamment arguments et validation des entrées, tableaux d’échanges/rapports, enregistrements DNS, DHCP client/serveur/messages, NAT/adresses/ports, pare-feu/règles, mail/adresses, fichiers virtuels, messages HTTP et calculs de réseau. Elles fournissent des cas limites utiles; leur présence n’assure pas une couverture des automates TCP, de l’UI ou de toutes les interactions réseau.

Le [manifeste des exemples](exemples.json) expose pour chaque scénario ses entrées et classes. Les exemples modernes complets combinent web/mail/DHCP/NAT/pare-feu; ceux-ci devront être rejoués après conversion, au-delà d’un simple comptage de nœuds et de liens.

## Provenance

`Readme.md` annonce GPLv2 ou GPLv3; les en-têtes du code indiquent version 2 ou, au choix, version 3; `pom.xml` mentionne GPLv3. Les deux textes de licence sont dans l’archive. Ce dossier conserve ces observations et la provenance, sans arrêter une nouvelle licence ni réattribuer les actifs existants. Le choix des éléments réutilisés et des notices de distribution est une décision de fondation, distincte du choix d’architecture.

Les recommandations de bibliothèques sont sourcées dans le volet d’adaptation et séparées de l’audit des comportements historiques.
