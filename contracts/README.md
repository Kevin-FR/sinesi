# Contrats Sinesi — socle J0, version 1

Les fichiers JSON Schema Draft 2020-12 de ce dossier sont l’autorité portable des données effectivement présentes dans J0. Ils sont autonomes et ne téléchargent aucune référence externe. Leur identifiant `https://sinesi.dev/contracts/…` est un espace de noms, pas un service requis.

| Schéma | Fixture | Portée actuelle |
| --- | --- | --- |
| [project.schema.json](project.schema.json) | [project.json](../fixtures/valid/project.json) | Équipements/interfaces, liaisons, disposition et données persistantes |
| [scenario.schema.json](scenario.schema.json) | [scenario.json](../fixtures/valid/scenario.json) | Projet, graine, messages initiaux et routes du serveur de prototype |
| [command.schema.json](command.schema.json) | [command.json](../fixtures/valid/command.json) | Enveloppe et commandes pause/reprise/pas/arrêt/envoi de prototype |
| [event.schema.json](event.schema.json) | [trace.jsonl](../fixtures/valid/trace.jsonl) | Faits immuables du moteur J0 |
| [error.schema.json](error.schema.json) | [error.json](../fixtures/valid/error.json) | Diagnostic stable, cible et paramètres |

La validation structurelle est complétée par des invariants métier : unicité des identités et MAC, références existantes, ports physiques non partagés, chemins virtuels, contenus base64 valides et limites. Un schéma seul ne valide pas les références entre deux listes. Les fixtures négatives couvrent notamment versions inconnues, identité dupliquée et référence inexistante.

Le format initial est un JSON UTF-8 avec suffixe `.sinesi.json`; le scénario de recette inclut le projet avec ses actions. Une sauvegarde de projet ne conserve pas une exécution en cours. Les archives, checkpoints et importeurs historiques restent des livrables distincts de la roadmap. La migration v0 est une fixture de qualification définie par D-004, pas un import Filius.

J0 expose des **messages de prototype**, dont des requêtes/réponses applicatives de démonstration, sur une livraison événementielle déterministe. Les événements `prototype.message.*` ne sont pas des captures Ethernet, TCP ou HTTP standards. Les futures familles de commandes et de protocoles devront faire évoluer les contrats explicitement.

Les compteurs/temps sont des chaînes décimales canoniques de 0 à 2^64−1. La trace canonique JSONL trie récursivement les clés, utilise UTF-8, une fin de ligne LF par événement et normalise l’identifiant d’exécution en `run`. Les faits exposés à l’UI gardent l’identifiant de génération réel. Le hash SHA-256 s’applique aux octets JSONL ainsi normalisés.

Les mêmes fixtures sont consommées par le CLI et le prototype UI, et leur sérialisation réelle est contrôlée par les schémas. Pour exécuter les vérifications :

```sh
cargo test -p sns-project --test contracts --locked
cargo test -p sns-cli --test runner --locked
cargo test -p sns-ui-egui --test workbench --locked
```
