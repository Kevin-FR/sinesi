# Conditions et pannes par liaison — J7

F-SIM-008 complète le [contrat de reprise](REPRISE_J7.md). Une condition agit sur une liaison filaire précise dans les deux sens, sans transformer une perte locale en panne globale. Le projet conserve ses paramètres physiques; les conditions d’exécution sont conservées par les scénarios, journaux et checkpoints.

## Paramètres et application

La commande API 1 `simulation.link.configure` porte une identité `link` et les conditions suivantes :

| Champ | Valeur | Interprétation |
| --- | --- | --- |
| `delay_us` | Chaîne décimale non négative | Délai fixe de transmission, en microsecondes |
| `jitter_us` | Chaîne décimale non négative | Gigue supplémentaire entre zéro et cette borne, incluse |
| `loss_ppm` | Entier de 0 à 1 000 000 | Seuil de pertes en parties par million; 10 000 vaut 1 % |

La somme délai + gigue ne dépasse pas 60 000 000 µs. Une identité inconnue, un dépassement ou une perte hors limites est refusé sans mutation. Les liens radio calculés et les sockets du modem ne sont pas des liaisons configurables par cette commande.

Les conditions remplacent les valeurs physiques pour les prochains envois sur ce câble. Une modification ne reprogramme pas les trames déjà en vol. À 100 %, toutes les nouvelles émissions de la liaison sont perdues; les protocoles réagissent avec leurs délais et tentatives habituels. À 0 %, aucune émission n’est perdue par ce mécanisme. Le bouton historique de perte manuelle reste une action globale distincte.

Les commandes datées du scénario peuvent appliquer puis réparer une panne. L’interface propose les mêmes réglages depuis Exécutions et l’inspecteur d’un câble sélectionné, en simulation. La graine du scénario est affichée. Les champs délai/gigue utilisent les millisecondes et les pertes un pourcentage; le contrat reste entier et exact. Dans les deux vues, une liaison configurée affiche son taux de pertes; une perte positive utilise l’orange, et 100 % le rouge. Ces indications disparaissent en conception.

Les réglages ne sont pas ajoutés silencieusement au fichier projet : exporter le journal ou le checkpoint conserve l’exercice en cours. Le scénario de l’enseignant peut déclarer les commandes initiales. Un nouveau démarrage depuis un projet seul reprend ses paramètres physiques; rejouer le même scénario avec sa graine reproduit l’exercice.

## Tirages reproductibles et isolés

Chaque liaison explicitement configurée possède son propre générateur et son nombre d’essais. Les émissions d’une autre liaison ne consomment pas ce flux. Réappliquer des conditions conserve sa position; un démarrage neuf réinitialise les flux selon la graine et les identités. Les liaisons sans conditions explicites gardent la politique historique de délai/gigue.

Le profil `link-splitmix64-v1` est indépendant de la plateforme :

1. Calculer FNV-1a sur les octets de `sns-link-v1`, un octet nul, puis l’identité de liaison. Valeur initiale : `0xcbf29ce484222325`; multiplication par `0x100000001b3` après chaque XOR. Les opérations se font modulo 2^64.
2. L’état initial est cette empreinte XOR la graine du scénario.
3. Pour chaque émission configurée, produire deux valeurs SplitMix64, y compris lorsque la perte ou la gigue vaut zéro. Chaque sortie incrémente l’état par `0x9e3779b97f4a7c15`, applique XOR/décalage de 30, multiplication par `0xbf58476d1ce4e5b9`, XOR/décalage de 27, multiplication par `0x94d049bb133111eb`, puis XOR/décalage de 31; toujours sur 64 bits.
4. Le premier résultat modulo 1 000 000 est comparé strictement à `loss_ppm`. Le second modulo (`jitter_us` + 1) est ajouté au délai fixe. Le compteur d’essais est incrémenté une fois.

La décision est prise à l’émission. Une perte produit un abandon explicite; une transmission conserve le délai choisi et sa cause jusqu’à la réception. Une saturation de file ou de trace n’avance pas le flux aléatoire : la transaction est entièrement refusée.

## Événements et persistance

Les faits version 9 sont : `simulation.link.configured`, `simulation.link.transit` et `ethernet.dropped` avec raison `simulation.link_loss`. Ils portent la liaison, les paramètres ou le tirage/délai choisi, et les identifiants de trame lorsque pertinents. Les formats antérieurs restent distincts.

Le [contrat de commande](../../contracts/link-conditions-command.schema.json), le [contrat de faits](../../contracts/link-conditions-event.schema.json) et les schémas [scénario](../../contracts/network-scenario.schema.json), [journal](../../contracts/recording.schema.json) et [checkpoint](../../contracts/checkpoint.schema.json) sont autonomes. La somme délai/gigue et les références sont contrôlées sémantiquement en plus du schéma.

Le checkpoint inclut conditions, position du générateur et essais de chaque liaison. Sa reprise et son rejeu reproduisent les pertes et les délais après la frontière sauvegardée. Les checkpoints précédents sans conditions restent lisibles; leur absence garde exactement la politique antérieure. Une nouvelle commande ou un fait 9 nécessite un lecteur connaissant cette extension.

## Recette

L’[exercice de deux liaisons indépendantes](../../fixtures/valid/link-conditions-scenario.json) configure 50 % de pertes et une gigue sur une paire de postes; l’autre paire reste à 0 %. Les deux séries de pings sont datées. La recette vérifie aussi 100 %, réparation à 0 %, identité des tirages de la paire témoin, changement de graine, ordre des équipements, reprise en présence d’actions futures, refus transactionnels et compatibilité d’un ancien checkpoint. La [matrice](../suivi/MATRICE.md) relie les exigences à leurs preuves.
