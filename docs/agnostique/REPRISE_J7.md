# Scénarios réseau, reprise et rejeu — J7

Cette tranche réalise le comportement de F-SIM-007. Les [pannes par liaison](PANNES_J7.md) de F-SIM-008 sont qualifiées dans une tranche distincte. Les autres livrables J7 restent ouverts. Le [suivi](../suivi/MATRICE.md) relie ce contrat aux réalisations et aux preuves.

## Scénarios datés

Le scénario version 2 contient un projet, une graine décimale, deux listes historiques vides (`actions`, `prototype_servers`) et une liste `network_actions`. Chaque entrée porte une identité unique `id`, un instant virtuel `at_us` en microsecondes et une commande réseau. Les instants sont croissants au sens large; l’ordre du tableau départage les égalités. Maximum : 4096 actions. La version 1 conserve exclusivement les échanges de prototype historiques.

Les actions datées utilisent les mêmes commandes transactionnelles que l’interface : applications, sockets, ping, tables, routage, filtrage et entrées enregistrées de modem. Elles ne peuvent pas déléguer au système hôte. Les commandes de contrôle du temps et les messages du prototype sont interdits dans cette liste. Un chargement n’ouvre aucun modem réel.

Les actions restent dans la file du moteur, même lorsqu’elles sont très éloignées dans le temps. À un même instant, leur priorité 0 précède les réceptions réseau de priorité 10, les initialisations réseau de priorité 15 et les minuteries applicatives de priorité 20. La séquence stable départage les égalités. Une action invalide ne consomme pas son entrée et ne modifie pas l’état du moteur; l’exécution signale l’erreur à cette frontière. Les actions précédentes peuvent avoir été exécutées.

Le [schéma de scénario](../../contracts/network-scenario.schema.json) complète les validations sémantiques. L’[exemple LAN daté](../../fixtures/valid/network-scenario.json) émet deux pings, avec une reprise possible avant le second.

## Journal portable

Le facteur standard conserve la version 1 ci-dessous. La politique étendue utilise
le [journal et le checkpoint version 2](ATTENTES_J7.md), avec facteur d’attente
enregistré, profils explicites et le même encodage structurel. Les règles de
reconstruction, générations et continuation restent applicables.

Le [journal version 1](../../contracts/recording.schema.json) porte le scénario initial, la graine effective, la génération initiale, les quotas et une suite d’opérations : commande externe acceptée, nombre d’événements consommés, drainage des sorties du modem, arrêt, pause et renouvellement de génération. Les commandes refusées et les répétitions déjà acceptées ne sont pas réémises. Les actions du scénario sont reproduites par la progression de sa file, sans double enregistrement.

Les entrées du modem conservent leur contenu, ordre et instant d’arrivée virtuel. Le rejeu n’instancie pas d’adaptateur réel. Les commandes restent soumises aux validations de projet, révision, génération et quotas. Une frontière d’événements impossible, une génération non croissante ou une origine incompatible est refusée.

Limites : 65 536 opérations, 1 000 000 d’événements avancés, document de 64 Mio; les files d’exécution restent bornées à 65 536 événements, 100 000 faits et 4096 reçus de commandes. Une saturation du journal interdit son export et la création d’un checkpoint incomplet; elle ne devient pas une perte réseau silencieuse. Le fichier peut contenir les données pédagogiques du projet, y compris les identifiants de ses comptes.

## État complet et reprise

Le [checkpoint version 1](../../contracts/checkpoint.schema.json) contient le journal, l’état complet et une empreinte SHA-256 de cet état canonique. L’état comprend :

- Configuration courante et scénario, révision, génération et identité d’exécution.
- Horloge, file ordonnée, actions futures, séquences et budgets de transit des trames.
- Algorithme et état du générateur aléatoire.
- Tables Ethernet/ARP, routes, RIP, DHCP, NAT, filtrage et transport.
- Sockets, temporisations, applications, fichiers, cache DNS, courrier et P2P.
- Reçus de commandes et trace historique.

L’encodage `structural-json-v1` représente les associations par `{"$entries":[[clé,valeur],…]}`, triées selon leur clé canonique. Il conserve aussi les clés composées. Les entiers larges utilisent les objets `{"$u64":"…"}` ou `{"$i64":"…"}`; les compteurs déjà définis comme décimaux restent des chaînes. Les tableaux gardent leur ordre et les réels finis leur valeur. Cet encodage d’état est distinct de celui des données ordinaires du scénario initial.

Un checkpoint s’exporte en pause. Sa lecture vérifie version, limites, empreinte et atteignabilité : le journal reconstruit un état dont l’empreinte complète doit correspondre. Un état arbitraire muni d’une empreinte recalculée ne suffit pas. Cette stratégie implique une reprise proportionnelle à l’historique validé; elle ne promet pas un chargement en temps constant.

La reprise renouvelle la génération et conserve la trace antérieure. Les commandes tardives de l’ancienne génération restent inertes. L’adoption d’une exécution dans l’interface dépasse également les générations déjà utilisées dans cette session. Les identités des commandes émises ensuite évitent celles déjà consommées ou réservées par une action future.

Un journal complet peut prolonger un checkpoint si son origine et son préfixe correspondent exactement. La dernière plage d’événements du préfixe peut avoir été fusionnée avec sa suite; seule la différence est rejouée. Les commandes de cette continuation explicitement choisie sont appliquées à la nouvelle génération. Cela n’autorise pas les commandes tardives ordinaires.

## Interface et fichiers

La fenêtre Exécutions ouvre un scénario, reprend un checkpoint et rejoue un journal; elle exporte aussi un checkpoint ou un journal. Ces opérations lourdes se déroulent hors de la boucle d’affichage. Un résultat invalide conserve le document et l’exécution précédents. Une ouverture confirmée remplace le document d’un bloc et marque ses données comme non enregistrées dans un projet. Les langues FR EN ES IT PT DE et les vues Icônes/Schéma sont conservées.

Les fichiers d’exécution sont distincts des projets de 8 Mio et des futures autosauvegardes. L’écriture est atomique sur le même volume. Les entrées et destinations du lanceur doivent être différentes; l’interface refuse d’exporter par-dessus son projet courant. La récupération après fermeture forcée et la restauration automatique de session restent à réaliser.
