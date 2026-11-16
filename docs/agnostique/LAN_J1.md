# Réseau local — profil pédagogique J1

Ce document définit le comportement portable de J1. Les identifiants d’exigences et les preuves restent dans [features.json](../suivi/features.json). Les détails de l’adaptateur sont dans la [matrice de traçabilité](../suivi/MATRICE.md).

## Édition et cycle de vie

La palette disponible contient ordinateur, portable et commutateur huit ports. Les autres types conservés dans les documents ne deviennent pas des logiciels utilisables par leur simple présence. Chaque équipement possède une identité stable et des interfaces distinctes. Un câble relie deux interfaces Ethernet libres d’équipements différents; les références inexistantes, ports occupés, doublons et médias incompatibles sont rejetés sans modification du projet.

La conception permet ajout par bouton ou dépôt, rectangle de sélection, ajout/retrait par Maj+clic, déplacement de groupe, suppression de nœuds/câbles et déconnexion d’un équipement. Les extrémités des câbles sont dérivées des identités des interfaces. Un déplacement ne recrée aucun câble. L’inspecteur conserve l’identité lorsque nom, affichage nom/IP/MAC, adresse, préfixe, MAC, passerelle ou DNS changent. Un préfixe 0–32 représente exclusivement un masque contigu. Les masques pointés sont également validables par le cœur.

Chaque édition est validée avant publication et conserve un instantané pour annuler/rétablir. Une nouvelle édition invalide la branche de rétablissement. L’historique est limité à 100 éditions; il n’est pas enregistré dans le projet. Les positions absentes d’un document sont complétées en grille et le projet est signalé modifié. Zoom, pan et cadrage agissent sur la vue, pas sur les liens.

Conception, simulation et documentation conservent le même document. La documentation ajoute/retire des annotations persistantes. L’entrée en simulation crée une nouvelle génération, initialement en pause, avec tables et file vides. La topologie est alors verrouillée. Quitter la simulation arrête ses événements et vide ses états réseau. Revenir en simulation recommence proprement. Une ouverture/nouveau réinitialise aussi la projection de simulation. Les changements non enregistrés déclenchent un choix enregistrer/abandonner/continuer avant remplacement ou fermeture de fenêtre.

## Ethernet et commutation — F-L2-001 à F-L2-004

Les trames structurées portent identité de trame, MAC source/destination, EtherType (IPv4 0x0800 ou ARP 0x0806) et charge utile. Le broadcast est `ff:ff:ff:ff:ff:ff`. Les hôtes filtrent les unicasts destinés à une autre MAC avant livraison IP. Le commutateur conserve les en-têtes, apprend la MAC source sur le port entrant et rafraîchit sa date d’expiration.

Une destination connue est transmise à son port, sauf si ce port est celui d’entrée. Une destination inconnue ou broadcast est diffusée aux autres ports câblés. Les ports déconnectés ne créent pas d’abandons artificiels de diffusion. La SAT est consultable, purgeable et sa rétention configurable de 1 µs à 24 h; défaut 300 s. Une entrée expire en temps virtuel, sans effacer une entrée rafraîchie depuis la programmation du timer.

La protection pédagogique mémorise les couples commutateur/identité de trame. Un doublon est abandonné avec `switch.loop_detected`; une limite interne de 64 traversées borne également une trame dont la mémoire de déduplication aurait été évincée. Ce mécanisme est une simplification de simulation, pas STP/RSTP et pas un champ Ethernet standard. Le moteur ne représente ni collision, ni négociation de lien, ni débit/duplex, ni VLAN, ni FCS filaire.

## ARP — F-L2-005

Le choix IP précède la résolution. Une MAC absente déclenche une requête ARP broadcast (opération 1); la cible répond en unicast (opération 2). Les adresses matérielles et protocolaires des deux extrémités sont présentes dans la charge. Le destinataire apprend l’émetteur; une entrée existante peut être rafraîchie. Un hôte tiers ne livre pas le paquet à sa couche IP.

Le cache est indexé par interface et IPv4. Sa durée est de 60 s virtuelles; la consultation réutilise une entrée valide. Une résolution en cours regroupe les paquets en attente tout en conservant leurs corrélations distinctes. Une tentative sans réponse expire après 1 s et produit `arp.timeout` / destination inaccessible. Il n’y a pas de réessai automatique dans ce profil. La purge administrative est immédiate, même en pause, et produit un fait distinct; elle ne fait pas avancer les minuteries. Les expirations périmées ne suppriment pas les entrées renouvelées.

## IPv4 et ICMP — F-IP4-001/002/004

Les calculs utilisent des adresses numériques. Le prochain saut choisit l’interface du sous-réseau directement connecté au préfixe le plus long (identité d’interface comme départage), puis une passerelle par défaut joignable sur un sous-réseau connecté. L’adresse finale IP reste celle de la cible; la résolution ARP porte sur le prochain saut. Une route absente produit `routing.no_route`, distinct d’un échec de résolution.

Une destination propre ou toute adresse 127/8 est locale : aucune trame n’est émise et l’echo réussit sans câble. Sinon, le ping produit IPv4 avec ICMP echo request type 8/code 0, puis une réponse type 0/code 0 conservant l’identifiant de sonde et sa séquence. Chaque commande J1 émet une sonde; les répétitions sont des commandes distinctes. Le délai maximum est de 3 s virtuelles. La mesure RTT inclut ARP lors de la première résolution.

Les erreurs type 3 (inaccessible) et type 11 (temps dépassé) sont distinguées. Un TTL initial nul est refusé localement. Un équipement routeur déjà configuré comme passerelle peut émettre une erreur ICMP sur le réseau : TTL ≤ 1 donne type 11/code 0; une destination nécessitant un transfert non réalisé donne type 3/code 0. Ces erreurs ne déclenchent pas de nouvelles erreurs ICMP. Les tables de routage et le transfert entre sous-réseaux restent à J3.

Les paquets sont structurés, sans sérialisation octet par octet, checksum, fragmentation/réassemblage, options IP, données utilisateur d’echo, citation complète du datagramme dans une erreur, multicast, réponse au ping broadcast ou découverte de doublons IPv4. L’identifiant de sonde est opaque plutôt qu’un identifiant ICMP filaire de 16 bits. Aucune conformité complète aux RFC n’est revendiquée. Références des champs : [ARP — RFC 826](https://www.rfc-editor.org/rfc/rfc826), [IPv4 — RFC 791](https://www.rfc-editor.org/rfc/rfc791), [ICMP — RFC 792](https://www.rfc-editor.org/rfc/rfc792).

## Ordonnancement et observation

Une seule file ordonne les actions par temps/priorité/séquence. Réceptions et minuteries ont la priorité 10, les demandes d’envoi la priorité 20; à temps et priorité égaux, la première insertion passe en premier. Une réponse reçue exactement à l’échéance peut donc suivre une expiration déjà programmée. Les bornes d’exécution sont inclusives. Le pas consomme exactement un événement; un événement peut produire plusieurs faits (réception, apprentissage, transfert). En pause, aucune expiration n’est consommée. La vitesse ne règle que la présentation des bornes de temps virtuel. Les pertes manuelles abandonnent les trames à leur réception pendant le maintien du contrôle; les abandons restent inspectables avec leur cause et leurs en-têtes.

Les faits réseau sont append-only, corrélés, versionnés et exportables. Une réception est reliée au fait d’émission correspondant. Le détail affiche trame, interfaces, lien, MAC, ARP, IPv4, TTL et ICMP. Les tables et la trace sont des projections distinctes : purger une table ne supprime pas les échanges.

Les limites sont explicites : 1 024 nœuds, 4 096 liens, 128 interfaces par équipement, 65 536 événements en file, 100 000 faits, 4 096 commandes acceptées par génération, 4 096 paquets en attente de résolution/sondes et 65 536 entrées par famille de table/mémoire de boucle. Une saturation produit un diagnostic; l’événement en échec reste non consommé et ses changements sont annulés. La mémoire reste bornée, mais ces plafonds ne sont pas une garantie de performance. La charge mesurée et les limites de mesure sont dans [J1](../verification/J1.md).

## Persistance et compatibilité

Le projet LAN utilise version 2 / profil `sns-educational-lan-v1`. L’objet optionnel `lan` associe à chaque identité d’équipement l’affichage et la rétention SAT. Les caches, sondes, trames en vol, horloge et historique d’édition ne sont pas sauvegardés. Logiciels, fichiers virtuels, ressources et annotations présents sont conservés sans annoncer leur exécution.

Les documents J0 version 1 restent lisibles; leur première édition les fait passer explicitement au profil LAN version 2. Un lecteur J0 rejette ce profil. Les événements réseau utilisent version 2; les événements de démonstration J0 gardent leur version 1 et leur préfixe. Voir les [contrats formels](../../contracts/README.md).

L’inspection J1 des archives `.fls` est en lecture seule : inventaire du ZIP, classes XML reconnues/inconnues et présence de code embarqué. Aucune extraction, instanciation ou exécution. Reconnaître un nom de classe ne signifie pas savoir importer son comportement. La conversion historique complète reste un lot ultérieur.
