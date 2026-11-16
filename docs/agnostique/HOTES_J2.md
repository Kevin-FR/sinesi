# Hôtes, logiciels et observation — profil J2

Ce complément à [LAN_J1](LAN_J1.md) et [TRANSPORT_J2](TRANSPORT_J2.md) fixe le comportement portable des logiciels. La réalisation est reliée aux exigences dans la [matrice de traçabilité](../suivi/MATRICE.md); les preuves sont dans le [rapport J2](../verification/J2.md).

## Installation, service et fenêtre

Chaque ordinateur, portable ou routeur possède ses installations, fichiers et états d’application. Le catalogue expose les 13 logiciels identifiés par F-OS-002. Terminal, explorateur, éditeur de texte, afficheur d’images, client TCP et serveur echo sont utilisables en J2. Les autres logiciels peuvent être installés comme configuration, mais leur ouverture est explicitement indisponible avec leur jalon cible. Installer un logiciel ne constitue pas une preuve du protocole futur.

Une installation est unique par type et par hôte. L’identifiant d’application est stable et unique dans le document. Installation, configuration et fichiers persistent; fenêtres, connexions, journaux et tâches en cours appartiennent à la simulation. Fermer une vue conserve son service. Arrêter ou désinstaller le logiciel ferme toutes les sockets qu’il possède, y compris les sessions acceptées. Arrêter/recommencer la simulation reconstruit les états d’exécution depuis le document.

Le serveur echo écoute sur un port configuré. Il accepte plusieurs clients et renvoie les octets reçus par la connexion correspondante. Le client configure cible et port, affiche endpoints et état, envoie du texte UTF-8 et conserve les octets de réponse. L’écho utilise exclusivement le transport simulé : aucune livraison directe entre logiciels. Une réception de flux peut arriver en plusieurs fragments; la concaténation doit être identique au contenu envoyé, y compris après perte et retransmission.

## Fichiers et outils

Chaque hôte dispose d’une racine `/` isolée. Chemins absolus/relatifs, `.` et `..` sont résolus dans cette racine. Les chemins système, URL, séparateurs inversés et chemins ambigus sont refusés. La racine est protégée. Créer, copier, déplacer ou supprimer une entrée est transactionnel : une erreur laisse l’arbre inchangé. Copier conserve la source; déplacer la retire seulement après validation de la destination. Les dossiers non vides exigent une suppression récursive explicite. Limites J2 : 4 096 entrées et 1 Mio de contenu par hôte.

L’explorateur parcourt les dossiers, crée des fichiers/dossiers, copie, renomme/déplace et supprime. L’ouverture choisit l’éditeur ou l’afficheur déjà installé; une installation absente est signalée. L’import copie uniquement le fichier explicitement choisi dans le système hôte. Les collisions ne sont pas écrasées implicitement et aucune autre partie du disque n’est exposée.

L’éditeur ouvre le texte UTF-8, conserve un brouillon et écrit seulement après Enregistrer/Enregistrer sous. Un échec d’écriture conserve le brouillon. Fermer un brouillon exige de le conserver ou de l’abandonner explicitement; une désinstallation est refusée tant qu’il n’est pas résolu. L’afficheur lit PNG, JPEG, GIF (première image) et BMP, sans modifier les octets stockés. Les dimensions sont limitées à 4 096 × 4 096 et le décodage à 64 Mio.

Le document conserve texte, données binaires et dossiers explicites (`inode/directory`). Une sauvegarde de projet n’expose jamais un document partiellement écrit. L’état HTTP de J4 devra lire les mêmes fichiers enregistrés; le serveur de qualification J0 ne démontre pas cette future intégration.

## Terminal

Le parseur accepte les guillemets simples/doubles et une liste de commandes fermée. Il n’exécute aucun shell, processus, expansion ou redirection du système hôte. La saisie est bornée à 4 096 octets, l’historique à 256 commandes et le journal à 1 024 entrées, avec compteur des entrées omises.

| Famille | Commandes J2 |
| --- | --- |
| Aide | `help`, `help COMMANDE`, erreur sur commande ou arguments inconnus |
| Navigation | `cd CHEMIN`, `pwd`, `ls [CHEMIN]` / `dir [CHEMIN]` |
| Fichiers | `touch`, `mkdir`, `cat` / `type`, `cp` / `copy`, `mv` / `move`, `rm [-r]` / `del [-r]` |
| Réseau | `ipconfig`, `netstat`, `arp`, `arp -a [IP]`, `arp -d [IP]` |
| Sondes | `ping [-c N / -n N] [-b] IP_OU_NOM` |
| Observation | `tcpdump`, interruption par Ctrl+C ou bouton |
| Fenêtre | `exit`, `cls` / `reset`, rappel par flèches haute/basse |

Ping envoie quatre sondes par défaut, espacées d’une seconde virtuelle; le nombre explicite est compris entre 1 et 1 024. La collecte finale attend trois secondes virtuelles. Réponses, erreurs, délais et statistiques apparaissent dans le terminal. La diffusion limitée ou dirigée au LAN exige `-b`; chaque répondant distinct est compté. Une interruption empêche les nouvelles émissions du job; les trames déjà en transit peuvent terminer leur parcours.

Les noms J2 désignent les identifiants ou libellés non ambigus de la topologie, ainsi que `localhost`. Ce résolveur local ne réalise pas DNS, prévu en J4. `netstat` et ARP sont limités à l’hôte courant. `tcpdump` ajoute les nouveaux faits de cet hôte jusqu’à interruption. Interrompre ne coupe aucun autre logiciel; `exit` ferme seulement sa vue. `cls`/`reset` effacent l’affichage et réaffichent l’accueil sans réinitialiser réseau, fichiers ni services.

## Observation et langues

Le journal canonique associe numéro, temps virtuel, équipement, corrélation et cause. Les trames portent interface, direction et en-têtes historiques. La vue agrégée et la décomposition en quatre couches inspectent les mêmes données immuables. Les filtres de texte/corrélation, interface et couche sont des projections; masquer ne supprime pas les faits. La purge remet le journal affiché à vide à partir du numéro courant, tout en conservant la trace canonique utilisée pour le rejeu. Le suivi est désactivable. L’activité des liens signale les trames récentes et disparaît à l’arrêt. L’export texte UTF-8 contient exactement les lignes sélectionnées et leurs colonnes; JSONL conserve les événements structurés.

FR, EN, ES, IT, PT et DE couvrent les parcours essentiels, l’aide, les messages et les diagnostics. La préférence persiste séparément du projet; noms d’équipements, fichiers, contenus et identifiants de protocole ne sont pas traduits. Les contrôles fournis par le système dans les dialogues natifs suivent la langue du système.

## Versions et contrôle des changements

Le projet version 3 / `sns-educational-hosts-v1` conserve les mêmes structures de données que le profil LAN, avec installations/fichiers effectivement manipulables. Les versions 1 et 2 restent lisibles. Les commandes applicatives utilisent l’enveloppe API 1 avec `app.command`; les nouveaux faits applicatifs et la fin de ping broadcast sont en version 4. Les faits LAN inchangés restent en version 2 et ceux de transport en version 3.

Une mutation persistante incrémente la révision; les commandes d’une révision ou génération périmée sont refusées. Une réémission strictement identique du même identifiant est idempotente. Application, réseau, fichier, quotas et effets différés sont validés avant publication : un échec ne publie aucun état partiel. Une projection d’une ancienne génération ne peut pas remplacer le nouveau document.
