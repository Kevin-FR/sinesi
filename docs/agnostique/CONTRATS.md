# Contrats portables — socle version 1 et cible

Le socle version 1 possède désormais des schémas formels et des fixtures dans [contracts/](../../contracts/README.md). Les exemples projet, commande et événement ci-dessous correspondent à ce socle. Les familles applicatives complètes, archives, checkpoints et extensions décrites ensuite restent la cible des jalons suivants; elles ne sont pas annoncées comme implémentées.

## Types et invariants

| Type logique | Représentation externe proposée | Invariant |
| --- | --- | --- |
| Identifiant d’entité | Chaîne opaque stable | Unique dans le projet, jamais déduit du libellé ou d’une IP |
| Identifiant d’exécution | Chaîne opaque | Nouveau à chaque démarrage normal |
| Temps virtuel | Entier non négatif encodé en chaîne décimale | Unité commune : microseconde; comparaison numérique, pas lexicographique |
| Séquence | Entier non négatif encodé en chaîne décimale | Strictement croissante dans une exécution |
| IPv4 | Chaîne décimale pointée canonique | Quatre octets valides; calcul numérique interne |
| Préfixe IPv4 | Adresse réseau canonique et longueur 0–32 | Les bits hôte de l’adresse réseau sont nuls |
| MAC | Six octets, affichés en hexadécimal canonique | Unicité générée; collision importée explicitement diagnostiquée |
| Port transport | Entier 0–65535 | Politique de réservation explicite; écoute sur 0 traitée comme allocation ou refus selon l’API |
| Contenu binaire | Référence de ressource ou base64 pour un petit message | Taille bornée; encodage déclaré |
| Énumération | Chaîne stable non localisée | Une valeur inconnue est rejetée ou conservée selon un mécanisme de compatibilité annoncé |

Les temps et séquences sont des chaînes décimales à la frontière pour éviter les pertes de précision chez les clients. Les libellés traduits ne servent jamais de discriminants de sérialisation.

## Projet

Un projet décrit la configuration persistante, ses ressources, ses applications et ses annotations. Il n’embarque pas implicitement les paquets en vol ou les caches d’une ancienne exécution.

```json
{
  "schema_version": 1,
  "semantic_profile": "sns-educational-v1",
  "project_id": "project-01",
  "name": "LAN de démonstration",
  "devices": [
    {
      "id": "host-a",
      "kind": "workstation",
      "label": "Poste A",
      "interfaces": [
        {
          "id": "if-a",
          "mac": "02:00:00:00:00:01",
          "medium": "ethernet",
          "ipv4": {"mode": "static", "address": "192.168.1.2", "prefix_length": 24}
        }
      ],
      "network": {"default_gateway": null, "dns_server": null},
      "applications": [{"id": "terminal-a", "kind": "terminal", "config": {}}],
      "files": []
    }
  ],
  "links": [],
  "annotations": [],
  "layout": {"devices": [{"device_id": "host-a", "x": 120, "y": 80}]},
  "extensions": [],
  "resources": []
}
```

Le schéma formel doit préciser chaque variante d’équipement, les politiques applicatives persistantes et les références de fichiers. Il doit contrôler les références vers nœuds, ports, interfaces et ressources. Une extension de données optionnelle peut être conservée dans un espace de noms dédié; elle ne doit pas permettre des propriétés arbitraires exécutables.

## Emballage et sauvegarde

Le socle J0 enregistre un document JSON UTF-8 `.sinesi.json`, avec remplacement atomique et migration de qualification conservant un original. Le futur emballage proposé est une archive contenant `manifest.json`, `project.json`, `assets/` et les pièces d’extensions déclarées. Le suffixe de cette archive future reste à décider; `.fls` reste celui du format historique.

Le manifeste contient versions, liste de ressources, tailles, empreintes et métadonnées de provenance. Une ouverture n’active aucune extension ni connexion externe. Une sauvegarde écrit un temporaire dans le même volume, vérifie les données puis remplace atomiquement la destination. L’autosauvegarde a un chemin différent et ne remplace pas le fichier utilisateur.

L’import doit borner le nombre d’entrées, la taille décompressée, le taux de compression, la profondeur et la taille des textes. Les chemins absolus, traversées de répertoires, liens symboliques et références non déclarées sont refusés. Les valeurs de quotas seront fixées et testées avant mise en production.

## Commandes

```json
{
  "api_version": 1,
  "command_id": "cmd-step",
  "project_id": "project-j0",
  "expected_revision": "0",
  "run_id": "run-42-0",
  "command": {
    "type": "simulation.step"
  }
}
```

J0 implémente pause, reprise, pas à pas, arrêt et envoi de message de prototype. Le catalogue ci-dessous décrit les familles cibles; il ne constitue pas la liste des commandes déjà disponibles.

| Famille | Commandes proposées | Préconditions |
| --- | --- | --- |
| Projet | `project.new`, `project.load`, `project.save`, `project.import_legacy` | Révision connue et écriture hôte via adaptateur |
| Édition | `device.add/update/remove`, `link.add/update/remove`, `annotation.*`, `edit.undo/redo` | Conception pour changement de topologie; références valides |
| Exécution | `simulation.start/pause/resume/step/stop`, `simulation.set_speed` | Génération active correcte et état compatible |
| Applications | `app.install/uninstall/start/stop/configure`, `terminal.execute/cancel` | Hôte et application existants; capacité autorisée |
| Fichiers | `file.create/read/write/copy/move/remove/import` | Chemin virtuel valide; accès hôte limité au fichier choisi |
| Observation | `trace.query/export/clear`, `table.query/clear` | Séparer purge de trace et purge de table réseau |
| Avancé | `checkpoint.save/restore`, `bridge.connect/disconnect`, `extension.enable/disable` | Action explicite et capacités annoncées |

Une commande reçoit une réponse d’acceptation ou de rejet; les effets différés sont des événements. Réessayer le même `command_id` retourne le résultat déjà connu dans la fenêtre de déduplication publiée et ne déclenche pas un second envoi. Une révision de projet périmée produit un conflit explicite.

## Événements

```json
{
  "event_schema_version": 1,
  "run_id": "run-42-0",
  "event_id": "evt-1",
  "time_us": "0",
  "sequence": "1",
  "type": "prototype.message.sent",
  "device_id": "client",
  "caused_by": null,
  "correlation_id": "flow-0",
  "payload": {
    "source": "client",
    "target": "server",
    "message": {
      "kind": "signal",
      "text": "Bonjour Sinesi"
    }
  }
}
```

La version 1 publie les faits de prototype envoyés, transférés, livrés ou abandonnés. La cible complète ajoute les événements suivants : état d’exécution, configuration modifiée, trame émise/reçue, couche traversée, paquet transformé/abandonné, route choisie, table modifiée, timer expiré, sortie de terminal, état de service et diagnostic d’adaptateur.

La trace complète est append-only pendant une génération. Chaque transformation conserve un avant/après ou une référence vers des instantanés immuables. Les projections UI peuvent être regroupées, mais le flux canonique utilisé pour les recettes conserve son ordre et sa causalité.

Le hash canonique exclut les horodatages réels, durées de rendu et identifiants aléatoires non dérivés de la graine. Le socle J0 trie récursivement les clés, normalise l’identité d’exécution en `run`, conserve l’ordre des faits et applique SHA-256 aux octets JSONL UTF-8 terminés par LF. Le même objet réordonné ne change pas la comparaison sémantique.

## Requêtes et projections

Les requêtes de topologie, configuration, applications, fichiers, tables et traces retournent des valeurs sans référence mutable vers le moteur. Une réponse volumineuse utilise une pagination avec curseur stable, génération et borne de révision. Un curseur expiré produit une erreur plutôt qu’une lecture incohérente.

Un filtre de traces porte sur intervalle virtuel, interfaces, appareils, protocoles, adresses et corrélation. Les exports indiquent les filtres, versions, exécution, borne de capture et éventuelles plages perdues. L’export texte reproduit la vue agrégée; les exports structurés conservent les identifiants et les causes.

## Checkpoints et rejeu

Un checkpoint comprend configuration de départ et révision, profil de sémantique, état des équipements/applications, horloge, file d’événements, compteurs, état du générateur aléatoire, timers, sockets, caches, tables et position dans le journal des entrées. Il possède sa propre version, indépendante du format de projet.

Une reprise sur une version incompatible est refusée avec explication ou migrée par un chemin testé. Un rejeu n’utilise ni horloge réelle ni socket réelle : les entrées du pont sont lues dans le journal. Les comparaisons couvrent les états intermédiaires et les traces après reprise.

## Erreurs

Une erreur comporte `code`, `message_key`, paramètres de traduction, cible et détails structurés. Le code est stable et indépendant de la langue.

Exemples de catégories : `validation.invalid_ipv4`, `topology.port_busy`, `routing.no_route`, `transport.port_in_use`, `protocol.timeout`, `project.unsupported_version`, `project.invalid_reference`, `project.import_loss`, `resource.limit_exceeded`, `extension.capability_denied` et `state.stale_run`.

Les abandons réseau attendus sont des faits de simulation, pas des exceptions fatales de l’application. Une panne interne est un diagnostic distinct avec contexte technique et possibilité d’export; elle ne doit pas divulguer les contenus pédagogiques par défaut.

## Contrats d’extension et du pont

Une extension déclare identifiant, version, version d’API, point d’entrée, capacités et quotas. L’API permet des commandes sur les ressources virtuelles autorisées, des événements et des données de configuration. La compilation de sources est une opération d’outillage séparée de l’exécution du scénario.

Un pont déclare version de protocole, identités d’instances et liaison virtuelle associée. Ses messages sont cadrés et bornés; une connexion incompatible est rejetée. L’ouverture et la fermeture sont explicites. Le format natif historique et ses objets sérialisés ne constituent pas un contrat d’interopérabilité à conserver.
