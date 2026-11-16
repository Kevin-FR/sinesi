# J6 — Courrier, P2P et connexions avancées

Ce profil couvre les seize exigences de J6 et conserve les six langues ainsi que les vues Icônes/Schéma. Le [suivi](../suivi/MATRICE.md) reste l’autorité de l’avancement : la présence de cette spécification ne vaut pas qualification.

| Famille | Comportement implémenté | Simplifications et limites |
| --- | --- | --- |
| Courrier | SMTP/POP3 sur TCP simulé, comptes, boîtes, relais MX puis A, envoi/réception/réponse, CC/CCI et contacts persistants | Messages textuels UTF-8 pédagogiques; pas de TLS, pièces jointes ou service de courrier réel |
| P2P | Voisins, Ping/Pong/Query/QueryHit avec identifiants/TTL, partage VFS, transfert vérifié, annulation | Gnutella pédagogique borné; aucun accès au disque hôte ni découverte sur Internet |
| Wi-Fi | Choix Ethernet/Wi-Fi, association par SSID, dissociation et invalidation des communications concernées | Association logique; pas de propagation radio, chiffrement ni modèle physique 802.11 |
| Modem | Adaptateur explicitement activé, écoute/connexion réelle, tunnel borné de trames, déconnexion et journal d’entrée | Contrat versionné propre au simulateur; aucune désérialisation d’objets historiques; pas d’ouverture implicite au chargement d’un projet |

## Profil courrier

Les comptes, réglages, messages et contacts sont des données du projet. Les sockets, transactions, authentifications et opérations en cours appartiennent à l’exécution. Les mots de passe sont des identifiants pédagogiques conservés dans le projet, sans prétention de sécurité réelle.

Les destinataires SMTP sont portés par l’enveloppe. Les messages reçus n’exposent pas la liste CCI; la copie Envoyés du rédacteur la conserve. Un destinataire inconnu produit un résultat explicite. Les relais utilisent le DNS simulé, avec requêtes MX puis A et prévention des boucles. Les délais et quotas sont virtuels et bornés.

POP3 distingue autorisation, transaction et mise à jour. Les suppressions côté serveur ne sont validées qu’à QUIT; une connexion interrompue libère la boîte sans valider ses suppressions. La récupération évite les doublons, sauvegarde les messages reçus et les retire du serveur après confirmation. Les suppressions locales et contacts persistent après réouverture.

Références de comportement : [SMTP, RFC 5321](https://www.rfc-editor.org/rfc/rfc5321.html) et [POP3, RFC 1939](https://www.rfc-editor.org/rfc/rfc1939.html). Les commandes SMTP prises en charge sont EHLO/HELO, AUTH PLAIN, MAIL FROM, RCPT TO, DATA, RSET, NOOP et QUIT; POP3 couvre USER/PASS, STAT, LIST, UIDL, RETR, DELE, RSET, NOOP et QUIT. Les erreurs de relais sont synchrones; aucun message de retour asynchrone n’est généré.

## Sous-ensemble P2P

Chaque pair écoute sur un port configurable (6346 par défaut). Le pair d’entrée est facultatif : le premier pair crée le réseau, puis les autres le rejoignent par adresse et port. La limite porte sur les voisins directs, de 1 à 32. La découverte conserve au plus 128 adresses; une recherche conserve au plus 128 résultats. Le répertoire partagé peut contenir des sous-répertoires; chaque pair publie au plus 16 correspondances par requête pour borner le nombre de diffusions.

Le protocole pédagogique transporte des messages JSON délimités par une fin de ligne sur TCP simulé : `hello` version 1, `ping`, `pong`, `query`, `query_hit`, `get`, `file` et `error`. Il ne revendique aucune interopérabilité Gnutella historique. Les messages propagés ont un identifiant opaque et un TTL de 1 à 16, décrémenté à chaque relais; une mémoire de 4096 identifiants évite les boucles. Atteindre une limite produit une erreur explicite; une remise à zéro renouvelle les connexions et les mémoires.

Une réponse identifie sa recherche, la source, le chemin, le type de contenu, le nombre d’octets et une empreinte SHA-256. Le téléchargement se fait par une connexion distincte à la source; seuls les fichiers sous le répertoire partagé sont accessibles. La taille maximale d’un fichier est 256 Kio; les messages sont bornés à 400 000 octets. Le destinataire vérifie taille et empreinte avant écriture dans son espace virtuel, sans écraser un chemin existant. Un fichier modifié entre recherche et transfert est refusé. Aucun disque hôte, traversée de NAT ou annuaire Internet n’est utilisé.

Annuler ou vider les résultats retire l’identifiant de la recherche active. Les réponses en retard peuvent encore être relayées pour les autres pairs, mais ne sont plus collectées localement. Une nouvelle recherche utilise un nouvel identifiant. Les opérations et négociations inachevées expirent après 30 secondes virtuelles; les voisinages établis restent actifs jusqu’à leur fermeture.

## Association Wi-Fi logique

Une interface d’hôte choisit Ethernet ou Wi-Fi. En Wi-Fi, elle conserve le SSID sélectionné. Un commutateur devient point d’accès avec une interface radio et un SSID unique de 1 à 32 octets. Les autres interfaces peuvent rester câblées. Les associations sont calculées par égalité du SSID; elles ne sont pas enregistrées comme des câbles.

Changer le SSID du point d’accès retire uniquement les associations qui ne correspondent plus. Les hôtes conservent leur sélection et peuvent choisir le nouveau nom pour se réassocier. Deux hôtes sur la même radio communiquent par le point d’accès; le réseau câblé reste indépendant. Une liaison radio est représentée par un tracé bleu discontinu dans les deux vues. Changer une interface câblée en radio retire son câble lors de la validation de la configuration.

La couche radio transporte le même modèle de trames et d’adressage que le LAN, avec un délai fixe de 1 ms. Il n’y a ni distance, puissance, collision radio, chiffrement ni authentification Wi-Fi.

## Tunnel de modem et entrées externes

Un modem a une interface Ethernet et un rôle Écoute ou Connexion, avec un endpoint numérique `IP:port` (IPv4 ou IPv6 entre crochets). Le projet conserve les réglages; charger un document n’ouvre aucune connexion. L’utilisateur active et déconnecte le modem en simulation. Revenir à la conception, remplacer le projet ou arrêter la simulation ferme les adaptateurs. L’interface affiche connexion, attente, connecté, déconnecté et le motif d’échec.

Le tunnel utilise un flux TCP réel réservé à cette fonction. Les deux côtés commencent par `SNS-BRIDGE/1` suivi d’une fin de ligne. Chaque paquet contient une longueur non signée sur quatre octets en ordre réseau, puis un document UTF-8 JSON : version 1, séquence décimale à partir de 1, budget de transit entre 1 et 64, et trame typée. Taille maximale : 128 Kio; files de 32 messages dans chaque sens. Les trames sont validées avant injection : adresses matérielles, type Ethernet, structure ARP/IPv4/ICMP/TCP/UDP, profondeur des citations et encodage du contenu. La séquence et l’identité de session écartent les paquets obsolètes. Le budget partagé avec les relais de trames borne aussi une boucle entre ponts.

L’écoute et les lectures/écritures partielles restent hors de la boucle graphique; une connexion sortante expire après 500 ms, la négociation après deux secondes réelles. Une incompatibilité, un dépassement, une file saturée ou une coupure ferme le tunnel avec un état explicite. Il n’y a pas de reconnexion automatique, d’interopérabilité avec le modem Filius historique, d’authentification ou de chiffrement du tunnel.

Le fait `bridge.input` conserve la session, la séquence, le budget et la trame complète, ainsi que le temps virtuel et l’ordre de l’enveloppe d’événement. Les activations et déconnexions sont également tracées. J6 fournit ces données d’entrée à la future recette de rejeu J7; il ne déclare pas encore le rejeu réalisé.

## Contrats et qualification

Le projet version 7 / `sns-educational-applications-v1` ajoute réglages et données de courrier, réglages P2P, sélection SSID et réglages de modem. Les connexions, recherches et tunnels actifs ne sont pas restaurés au chargement. Les commandes gardent l’enveloppe API 1; les nouveaux faits ont la version 8. Les [contrats autonomes](../../contracts/README.md#applications-et-connexions-j6) complètent les validations de références et quotas.

Les [recettes R-MAIL, R-P2P et R-PONT](RECETTE.md) et le [rapport de qualification](../verification/J6.md) relient les comportements aux preuves. Les versions antérieures et leurs captures restent des références historiques.
