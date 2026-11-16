# Réseaux routés — profil pédagogique J3

Ce profil complète [LAN_J1](LAN_J1.md), [TRANSPORT_J2](TRANSPORT_J2.md) et [HOTES_J2](HOTES_J2.md). Les exigences F-EQU-003, F-TOP-005/011, F-IP4-003/005, F-L2-006, F-ROUT-001 à 005, F-CLI-010/015/017 et F-OBS-009 sont reliées à leur réalisation dans la [matrice](../suivi/MATRICE.md). Les preuves figurent dans le [rapport J3](../verification/J3.md).

## Configuration et choix de route

Un routeur possède entre 2 et 32 interfaces configurables séparément. Redimensionner conserve les identités des interfaces restantes; retirer une interface câblée ou utilisée par une route est refusé. Échanger deux ports permute uniquement leurs associations de câbles : adresses, MAC et autres liens restent inchangés. Les opérations sont réversibles dans l’historique.

La table distingue loopback, réseaux directement connectés, routes statiques, passerelle par défaut et routes apprises par RIP. Une route statique contient réseau destination canonique, longueur du masque (préfixe 0–32), passerelle facultative et interface de sortie. Une passerelle doit être un voisin unicast du réseau de cette interface; l’absence de passerelle force une résolution directe de la destination. La table et la commande `route` affichent les entrées effectives, leur origine et leur préfixe; `/24` représente le masque `255.255.255.0`.

La livraison locale précède le routage. Pour une destination distante, le préfixe le plus long l’emporte. À préfixe égal : connecté, statique, défaut configuré, puis RIP; les routes loopback sont locales. RIP est ensuite départagé par métrique, adresse du voisin et identifiant d’interface, dans cet ordre stable. Une route de métrique 16 est inutilisable. La suppression d’une route spécifique rend à nouveau applicable la route moins spécifique.

Filius choisit aussi le masque le plus long dans sa table statique, mais privilégie le nombre de sauts avant le masque dans sa sélection dynamique (`Weiterleitungstabelle.java`, méthodes `determineRouteFromStaticRoutingTable` et `determineRouteFromDynamicRoutingTable`). Le présent profil unifie explicitement la priorité du préfixe sur toutes les origines; une égalité ne dépend pas de l’ordre de stockage.

## Transfert IPv4 et erreurs

Le transfert est actif par défaut sur un routeur et désactivé sur les autres hôtes; une option explicite le remplace. Un hôte peut transférer avec ses interfaces et routes configurées. À chaque transfert, source/destination IP restent inchangées, TTL diminue de un, et une nouvelle trame Ethernet utilise le MAC de sortie et celui du prochain saut. ARP est résolu sur cette interface; le cache d’une autre interface ne peut pas être réutilisé à sa place. TCP et UDP empruntent aussi ce chemin.

Un paquet non local avec TTL ≤ 1 est abandonné et une erreur ICMP Time Exceeded type 11/code 0 est renvoyée lorsque la source est joignable. Une route absente produit Destination Unreachable type 3/code 0; un transfert désactivé utilise code 13; une résolution ARP expirée utilise code 1. La trace distingue absence de route, blocage, expiration ARP, absence de réponse et expiration TTL. Une absence de route de retour est un fait distinct. Les erreurs ICMP ne provoquent pas de nouvelle erreur ICMP en chaîne.

Les diffusions limitées ou dirigées du LAN restent locales. `ping` exige `-b` pour les utiliser; plusieurs hôtes peuvent répondre. Le transfert dirigé de broadcast entre réseaux est refusé. RIP possède sa réception de contrôle locale; le transfert multicast général n’est pas réalisé.

La corrélation ICMP est logique, par identifiant de sonde; ce profil ne sérialise pas une citation binaire complète du datagramme original. Les autres limites de représentation des paquets des profils J1/J2 restent applicables. Les règles de TTL s’appuient sur les [exigences IPv4 des routeurs, RFC 1812](https://www.rfc-editor.org/rfc/rfc1812.html), sans revendiquer leur couverture intégrale.

## RIP pédagogique

Activer RIP sur un routeur lance des annonces au temps virtuel zéro, puis toutes les 30 secondes. Les réseaux directement connectés ont la métrique locale zéro; recevoir une annonce ajoute un saut, plafonné à 16. Les candidatures de voisins différents sont conservées séparément pour pouvoir inspecter et choisir le meilleur chemin.

Les annonces passent réellement sur les liens du LAN : IPv4 destination `224.0.0.9`, TTL 1, UDP source/destination 520, charge structurée contenant version et entrées. Les annonces sont découpées en groupes de 25 entrées. L’interface source, le voisin local, TTL, longueur UDP, version, réseau/préfixe et métrique sont contrôlés avant apprentissage. RIP réserve le port UDP 520 sur les routeurs activés. Un hôte ordinaire ignore ce trafic de contrôle.

L’annonce sur une interface emploie poison reverse : une route apprise sur cette interface est annoncée avec métrique 16. Un changement de route déclenche une annonce après 1 milliseconde virtuelle; les déclenchements simultanés sont regroupés. Une route non rafraîchie expire après 75 secondes et devient immédiatement inutilisable dans la sélection. Un contrôle toutes les secondes publie l’expiration et déclenche son annonce; les entrées inaccessibles sont purgées après 30 secondes. La table est bornée à 4 096 candidatures globales.

Le bouton de suspension coupe les annonces du routeur sans couper ses interfaces ni sa réception. Il permet d’observer l’expiration, puis la reconvergence après reprise. Arrêter ou redémarrer une simulation efface toutes les routes apprises et minuteries; les réglages RIP persistent dans le projet. Les annonces périodiques maintiennent normalement des événements en attente : une exécution RIP doit être bornée ou arrêtée explicitement.

Les valeurs 30/75 secondes et la métrique locale zéro reprennent le modèle Filius (`RIPTable.INTERVAL`, `TIMEOUT`, `addLocalRoutes`). Sa première annonce attend une seconde, son intervalle est aléatoire et son beacon omet les routes de l’interface d’apprentissage par split horizon. Ici, démarrage immédiat, intervalle fixe et poison reverse rendent le comportement reproductible et plus explicite.

Le [protocole RIP version 2, RFC 2453](https://www.rfc-editor.org/rfc/rfc2453.html) décrit notamment une expiration nominale de 180 secondes : ce n’est pas le délai de ce profil. La charge d’annonce est un document structuré, pas l’encodage binaire RIPv2; l’enveloppe Ethernet utilise la diffusion du LAN. Authentification, tags, redistribution des routes statiques/défaut, jitter RIP et conformité/interconnexion avec des routeurs réels ne sont pas réalisés.

## Terminal et explication

- `route` affiche les routes effectives de l’hôte courant.
- `traceroute [-m MAX] IP_OU_NOM` envoie une sonde ICMP par saut, en commençant à TTL=1, puis en augmentant jusqu’à la destination ou la limite (30 par défaut, maximum 64). Chaque collecte attend trois secondes virtuelles. Une absence de réponse affiche `*`; une erreur terminale garde sa cause. Les noms restent ceux de la topologie, sans résolution DNS système. Ctrl+C empêche les nouvelles sondes du job.
- `arpsend IP_SOURCE IP_CIBLE` émet une réponse ARP entièrement simulée vers un voisin du même réseau. L’adresse source choisie permet les exercices de conflit/usurpation; le MAC annoncé est celui de l’interface émettrice. Une cible peut donc apprendre une association différente et envoyer la trame suivante à cet équipement. Aucun paquet n’est émis sur le réseau réel.

Les décisions de transfert portent route retenue, interface entrante/sortante, prochain saut, TTL avant/après, raison et paquet historique. Les refus et changements RIP ont des causes stables et un équipement responsable. L’inspecteur explique la cause dans la langue choisie; les champs de protocole et données des projets gardent leurs identifiants. Les contrôles, l’aide et les diagnostics existent en FR/EN/ES/IT/PT/DE.

## Représentations pédagogiques et exemples

F-TOP-011 ajoute deux vues de la même topologie : icônes illustratives et schéma à symboles. Les sept représentations livrées sont poste, portable, serveur, imprimante, tablette, routeur et commutateur. Elles emploient 14 SVG fournis par l’utilisateur et embarqués avec leurs empreintes/provenance, sans charger l’archive à l’exécution. Les autres actifs du kit restent disponibles pour les étapes suivantes.

La représentation choisie pour un équipement persiste dans le document. Le choix global Icônes/Schéma est une préférence de vue transitoire; il ne modifie ni connexions ni paquets. Serveur, imprimante et tablette partagent les capacités d’hôte IPv4 disponibles; un symbole n’ajoute pas d’impression, de Wi-Fi ou de protocole applicatif. L’exemple Salle de classe est explicitement Ethernet. Les deux autres exemples fournissent trois LAN routés statiquement et trois routeurs RIP avec chemins alternatifs.

## Versions

Les nouvelles configurations utilisent projet version 4 / `sns-educational-routing-v1`. Les versions 1 à 3 restent lisibles; elles ne peuvent pas contenir des champs de routage/symbole non déclarés. Les nouvelles décisions de routage/RIP sont des événements version 5; les faits LAN, transport et logiciels gardent respectivement 2, 3 et 4. Le contrôle temporaire `routing.rip.mute` utilise l’enveloppe de commande API 1 et ne modifie pas le document.
