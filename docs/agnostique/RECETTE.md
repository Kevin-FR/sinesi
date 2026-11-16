# Recettes intégrées de référence

Ce plan décrit les scénarios de référence. R-LAN est qualifiée dans le [rapport J1](../verification/J1.md); R-TCP et R-FICHIERS le sont dans le [rapport J2](../verification/J2.md), pour les profils pédagogiques déclarés. R-ROUTES et R-RIP sont qualifiées dans le [rapport J3](../verification/J3.md). Les autres recettes restent à réaliser. Les critères élémentaires de toutes les exigences restent dans le catalogue; ces recettes vérifient leurs interactions.

Chaque recette conserve : version du produit, versions de contrat/profil, OS, fichier de scénario, graine, liste ordonnée des commandes, borne virtuelle, assertions et trace canonique. Les mesures de performance ajoutent modèle de machine, mémoire, configuration de build et méthode de calcul des percentiles.

## R-LAN — ARP, ICMP et switch

**Références :** F-EQU-001/002/006, F-L2-001 à 005, F-IP4-001/002/004, F-SIM-002/005/006, F-OBS-001 à 005.

Créer A=192.168.1.2/24, B=192.168.1.3/24 et C=192.168.1.4/24 sur un switch. Émettre un ping de A vers B avec cache vide. Vérifier diffusion ARP, réponse de B, apprentissage des MAC, echo request/reply et absence de livraison applicative à C. Répéter : le cache évite une nouvelle résolution. Purger ARP et SAT et vérifier le retour à l’état initial attendu.

Rejouer en pas à pas puis à deux vitesses. Comparer les traces canoniques. Ajouter une boucle de switches : la file reste bornée et la cause d’abandon est visible. Effectuer dix cycles arrêt/démarrage : aucune trame de la génération précédente ne réapparaît.

## R-TCP — Fiabilité et cycle de vie

**Références :** F-TCP-001 à 005, F-UDP-001/002, F-OS-003/005/006, F-CLI-011/014/016/018.

Démarrer echo sur un port choisi et connecter deux clients. Vérifier handshake, endpoints et états. Envoyer un texte nécessitant plusieurs segments, perdre une transmission de façon déterministe puis vérifier les réessais et une seule livraison du contenu reconstitué. Tester une écoute déjà occupée et une connexion expirée.

Fermer la vue du serveur : les sessions restent actives. Arrêter puis désinstaller le serveur : les ressources sont libérées et le port peut être réutilisé. Répéter avec un datagramme UDP perdu : le transport ne le réémet pas spontanément. Interrompre ping/tcpdump sans arrêter les autres applications.

## R-FICHIERS — Isolation et persistance

**Références :** F-FS-001 à 005, F-CLI-002 à 008, F-CLI-020/021, F-PROJ-001/004.

Créer un dossier, un texte et une image importée sur A. Copier, renommer, déplacer et supprimer via explorateur et terminal. Vérifier les alias, les chemins relatifs/absolus et les erreurs de cible absente. Les mêmes chemins sur B ne donnent pas accès aux fichiers de A.

Enregistrer et rouvrir le projet : comparer topologie, ressources et contenus binaires. `cls`/`reset` n’altèrent ni les fichiers ni le réseau; `exit` ferme uniquement la vue du terminal. Simuler une écriture interrompue et vérifier que la dernière sauvegarde explicite reste lisible.

## R-ROUTES — Choix du prochain saut et TTL

**Références :** F-EQU-003, F-TOP-005, F-IP4-003/005, F-ROUT-001/002/003, F-CLI-010/015/017, F-OBS-009.

Relier trois LAN par deux routeurs. Configurer une route par défaut et une route statique plus précise. Vérifier le prochain saut et l’interface choisis, puis supprimer la route spécifique. Traceroute commence à TTL=1 et montre chaque saut. Une route manquante, un TTL expiré et une absence de réponse sont distingués.

Activer/désactiver le transfert sur un hôte intermédiaire. Vérifier la diffusion locale sans fuite entre LAN. Injecter une réponse ARP simulée et constater la modification du cache et le changement de chemin; aucune trame n’est émise sur le réseau réel.

## R-RIP — Convergence et expiration

**Références :** F-ROUT-004/005, F-SIM-005, F-OBS-009.

Utiliser au moins trois routeurs avec deux chemins de longueurs différentes. Activer RIP pédagogique; avancer jusqu’à convergence et vérifier la route apprise. Couper les annonces par une panne contrôlée; avancer au-delà de l’expiration et constater l’inaccessibilité. Publier les valeurs de timers et la politique de départage testées. Ne pas présenter cette recette comme une certification du protocole complet.

## R-DHCP — Attribution, conflit et repli

**Références :** F-DHCP-001 à 006, F-OS-004, F-CLI-009.

Configurer une petite plage, une réservation MAC/IP et des paramètres DNS/passerelle distincts. Démarrer plusieurs clients : vérifier discover/offer/request/ack, absence de doublons et respect de la réservation. Occuper une adresse avant sa proposition : le client la refuse et le serveur la traite selon la politique prévue. Épuiser la plage et vérifier l’erreur.

Supprimer le serveur et avancer les échecs : un client sélectionne une adresse link-local non conflictuelle. Tester aussi le cas d’échec final borné. Faire expirer une attribution et vérifier sa disponibilité future; ne pas supposer que toutes les variantes de renouvellement DHCP sont couvertes.

## R-WEB — DNS et HTTP sans réseau réel

**Références :** F-DNS-001 à 006, F-WEB-001 à 006, F-FS-004.

Configurer une délégation NS, un A et deux noms de vhost sur une même IP. Créer deux pages et une image. Résoudre puis naviguer : chaque nom affiche sa bonne page; l’image provoque une requête simulée. Soumettre un formulaire POST et consulter un chemin absent. Supprimer un enregistrement et vérifier sa non-réapparition après édition/réouverture.

Pour la modernisation de cache, vérifier réutilisation avant TTL, expiration puis nouvelle requête. Deux hôtes ne partagent ni cache de ressources ni résultat DNS. Couper le réseau réel et intercepter toute tentative de sortie : toutes les ressources pédagogiques restent disponibles; les schémas d’URL non autorisés sont refusés.

## R-NAT — Multiplexage, redirection et ICMP

**Références :** F-EQU-004, F-NAT-001 à 006, F-OBS-003/007.

Créer deux clients LAN utilisant les mêmes ports source et un serveur WAN. Ouvrir des sessions TCP et UDP : les mappings externes sont distincts et les réponses rejoignent leurs clients. Ajouter une redirection externe 8080 vers un serveur LAN 80 et vérifier la requête entrante.

Exécuter ping et traceroute depuis les deux clients; vérifier les identifiants et les erreurs ICMP contenant un paquet encapsulé. Vérifier qu’une requête WAN ne reçoit pas une réponse destinée à une adresse LAN locale du routeur. Purger/expirer les mappings. Comparer les captures avant et après NAT et confirmer que les premières n’ont pas été réécrites. Tester acquisition WAN par DHCP et attribution LAN simultanée.

## R-FILTRE — Politique et administration

**Références :** F-FW-001 à 007, F-ROUT-006, F-WEB-007, F-NAT-003.

Poser deux règles contradictoires et une politique par défaut. Vérifier première correspondance, absence de correspondance et changement après réordonnancement. Tester masques, protocole, port, ICMP, mode SYN seul et réponses UDP. Contrôler le journal et la cause attachée au paquet.

Modifier les règles depuis l’administration HTTP simulée; l’inspecteur doit montrer les mêmes valeurs. Publier la fixture fixant l’ordre filtrage/traduction sur les flux entrants et sortants pour éviter deux comportements contradictoires.

## R-MAIL — Livraison et confidentialité des champs

**Références :** F-MAIL-001 à 008, F-DNS-002/003, F-TCP-003.

Créer deux domaines, chacun avec A, MX et serveur de messagerie. Ajouter des comptes; configurer les clients, envoyer localement puis vers l’autre domaine. Récupérer par POP3, répondre et supprimer. Vérifier destinataires multiples, CC, BCC non exposés dans les en-têtes livrés et erreur de destinataire inconnu.

Comparer les boîtes après réouverture du projet. Tester identifiants invalides, compte supprimé et port occupé. Les contacts du modèle sont testés par l’API; une UI de carnet complète n’est pas déduite du code historique. Aucun SMTP ou POP3 réel ne doit être utilisé.

## R-P2P — Découverte et téléchargement

**Références :** F-P2P-001 à 005, F-FS-001.

Relier trois pairs avec au moins une boucle logique. Partager un fichier sur le pair distant, rechercher et télécharger depuis un autre. Vérifier propagation Ping/Pong et Query/QueryHit, TTL, déduplication et égalité du contenu. Annuler une recherche, vider les résultats, réinitialiser le réseau et recommencer sans messages fantômes.

## R-PONT — SSID et réseau entre instances

**Références :** F-EQU-005/007, F-EXT-005, F-SIM-007.

Associer un portable à un SSID et vérifier la connectivité; changer le SSID du point d’accès et constater la déconnexion concernée. Deux instances réelles relient leurs réseaux virtuels avec un modem serveur et un modem client : un ping traverse le tunnel. Tester endpoint absent, version incompatible, message trop grand et coupure en cours de transfert.

En J6, enregistrer les entrées externes. En J7, les rejouer sans aucune connexion réelle : retrouver la trace attendue. Le protocole de tunnel nouveau n’est pas réputé interopérable avec une instance historique sans recette dédiée.

## R-IMPORT — Corpus historique et erreurs de format

**Références :** F-PROJ-002/004/005, F-NFR-006; [corpus](../audit/exemples.json).

Analyser les 18 exemples de `beispiele/` et les deux fixtures .fls historiques de tests. Pour chacun, publier les comptes d’équipements/liens, applications, ressources, annotations, champs convertis et inconnus. Les scénarios avec Accumulator/Broadcast embarqués doivent rester importables en données avec pièces inactives et rapport d’extension non portée.

Fournir des archives invalides : entrée sortante, chemin absolu, doublon de nom, référence inexistante, taille excessive et XML à entités externes. L’import refuse proprement sans exécuter ni écrire hors destination. Ne pas confondre import structurel réussi et scénario fonctionnellement équivalent : les recettes réseau sont ensuite requises.

## R-REPRISE — Checkpoint, génération et rejeu

**Références :** F-SIM-005/007/008, F-PROJ-006.

Suspendre un scénario avec session TCP active, timer RIP, attribution DHCP, cache DNS et mapping NAT. Sauvegarder un checkpoint; terminer la simulation et conserver la trace suffixe. Reprendre le checkpoint dans une nouvelle instance et comparer. Répéter avec pertes déterministes et entrées du pont enregistrées.

Forcer une fermeture lors d’une autosauvegarde et récupérer les modifications sans écraser le fichier utilisateur. Refuser un checkpoint incompatible avec une erreur explicite. Une commande tardive visant une ancienne génération n’a aucun effet sur la nouvelle.

## R-RAPPORT — Annotations, exports et plateformes

**Références :** F-DOC-001 à 004, F-OBS-008, F-CFG-001 à 008, F-NFR-005/007.

Annoter un réseau routé avec texte stylé et zone colorée; enregistrer/rouvrir. Exporter PNG et rapport PDF : vérifier visuellement cadrage, transparence, caractères accentués, colonnes, coupures de page et sections de configuration/trafic. Exporter les mêmes traces en texte, JSONL et CSV; contrôler leur sélection et leur cohérence.

Valider le parcours minimum sur Windows/Linux/macOS et FR/EN/DE, texte agrandi et clavier seul. Le choix d’icône ne modifie pas les échanges. Une configuration globale invalide est diagnostiquée sans empêcher l’ouverture. SVG reste une recette optionnelle distincte.

## R-EXT — Extension portable et quotas

**Références :** F-EXT-001 à 004/006, F-PROJ-005.

Créer un echo personnalisé depuis un modèle, provoquer une erreur de construction, corriger, installer et utiliser depuis un client générique. Embarquer le projet et rouvrir sur une autre machine : l’extension est reconnue mais inactive avant action explicite. Tester version incompatible et retrait du logiciel.

Une extension en boucle infinie est arrêtée par quota; une allocation excessive est refusée; les accès réseau/fichiers réels ne sont pas disponibles par défaut. Après arrêt, l’hôte et les autres applications restent utilisables.

## Preuve minimale

Une entrée `verification` doit identifier la recette ou le critère élémentaire, le fichier de test/scénario, la commande réellement exécutée, le résultat, la version et les artefacts utiles. Les contrôles visuels et d’accessibilité indiquent le parcours et la plateforme. Une exigence non encore couverte reste ouverte, même si la plupart de son lot est réalisée.
