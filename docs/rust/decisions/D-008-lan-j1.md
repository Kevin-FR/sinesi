# D-008 — Édition transactionnelle et pile LAN structurée

Date : 7 novembre 2026. Décision réalisée pour RS-TOPOLOGY/ENGINE/DEVICES/LINK/IPV4/PROJECTS/QUALITY-J1.

Le noyau reste synchrone, sans runtime réseau ni interface graphique. `sns-topology` possède les commandes réversibles et leur validation. `sns-protocols` possède l’état Ethernet/ARP/IP/ICMP; `sns-engine` conserve la seule file, le temps et le générateur pseudo-aléatoire. Un effet réseau indique le lien à traverser ou un délai virtuel; l’adaptateur du moteur calcule les dates avec débordements vérifiés. L’ancienne route BFS ne sert qu’aux messages de prototype J0.

Chaque événement est préparé sur un clone de l’état réseau. Les quotas, séquences et dates sont contrôlés avant toute mutation de la file, du générateur, des tables ou de la trace. Ce coût est mesuré, et une future optimisation devra conserver les tests transactionnels. Les timers d’expiration sont gardés par leur date/token, sans boucle périodique infinie. Les commandes de purge sont administratives, immédiates et tracées sans consommer un événement de protocole.

Le projet LAN passe explicitement en version 2 avec profil `sns-educational-lan-v1`; les métadonnées de présentation/rétention sont dans `Project::lan`, sans détourner une configuration d’application. Le lecteur conserve J0 version 1; l’éditeur convertit à la première édition. Les schémas J0 restent historiques et les contrats LAN sont ajoutés à côté. Une exécution ne s’enregistre pas dans le document.

L’interface J1 utilise le Painter egui upstream. Un worker unique traite la simulation; les dialogues `rfd 0.17.2` restent une action native explicite. Les briques Craft et les forks ne sont pas nécessaires pour ce lot. Un deuxième shell `--prototype-j0` garde le navigateur de qualification accessible sans présenter ses messages comme du trafic TCP.

La fidélité est celle d’un simulateur pédagogique : structures typées, champs actifs documentés, traces inspectables et tests de causalité, sans encodeur filaire complet ni promesse de conformité générale aux RFC. Le routage effectif reste J3; une passerelle peut seulement émettre les erreurs ICMP requises pour diagnostiquer TTL et route absente.

Preuves et mesures : [rapport J1](../../verification/J1.md). Les contrats et critères réutilisables restent dans [LAN_J1](../../agnostique/LAN_J1.md).
