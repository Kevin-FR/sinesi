> Photographie historique de la fondation J2. La réalisation complète et ses preuves sont désormais dans [J2](J2.md); ne pas utiliser les prochaines actions ci-dessous comme état courant.

# J2 — réalisation en cours et reprise

Le périmètre complet reste celui de [ROADMAP](../agnostique/ROADMAP.md), avec la localisation FR/EN/ES/IT/PT/DE demandée pour J2. Le [rapport intermédiaire](../verification/J2-EN-COURS.md) ne ferme pas la porte J2.

## Présent

- `sns-model/src/transport.rs` : commandes Bind/Connect/Send/SendTo/Close/CloseOwner, endpoints, flags, segments, datagrammes, états et notifications. Commande publique `Command::Socket` dans l’enveloppe habituelle.
- `sns-protocols/src/transport.rs` : écoute/démultiplexage, ports éphémères, automates TCP et UDP, segmentation et ACK, retransmission virtuelle bornée, fermeture et libération par propriétaire. `Transport::views()` est la projection destinée au bureau/netstat.
- `sns-protocols/src/lib.rs` : charge `Payload::Ipv4Transport`, événements socket/timer/local, encapsulation/décapsulation via le LAN existant. `Network::socket_events` contient les notifications du dernier traitement et sera consommé par la couche applicative. Elles sont aussi publiées dans les faits de trace. Aucun echo automatique n’est encore implémenté dans un logiciel.
- `sns-engine` : commandes socket synchrones au temps courant, préparation sur clone, contrôle des quotas/dates puis commit, même file de livraisons/minuteries. Les tests bas niveau font le renvoi echo explicitement via les sockets serveur.
- `sns-vfs` : arbre isolé, normalisation, dossiers, données immuables `Arc<[u8]>`, copie/déplacement/suppression transactionnels, codec vers `VirtualFile`. Dossiers explicites en `inode/directory`; finaliser la politique de version/profil avant la recette projet/application complète.
- `sns-i18n` : 193 clés explicitement traduites dans les six langues, contrôle des clés/paramètres, sans fallback vers une autre langue pour les libellés. Les valeurs utilisateur des projets restent inchangées.
- `sns-settings` : préférence JSON versionnée, bornée à 64 Kio, sauvegarde atomique hors projet. `SINESI_SETTINGS_PATH` permet un chemin explicite; sinon dossier de configuration de l’OS. Les tests UI n’écrivent que dans des temporaires.
- `LanUi` : sélecteur de langue, écran de conception/simulation/documentation, aide, inspecteur et tables existantes localisés. Identifiants de widgets indépendants des libellés. `LanUi::native()` charge la préférence; `new()` reste déterministe pour les tests.

## Suite requise pour le même objectif

1. Créer `sns-apps` : registre des 13 logiciels et disponibilité réelle, installation persistante, états de service distincts des vues. Faire réagir un vrai service echo aux `SocketNoticeKind::Data`, via `SocketCommand::Send`, sans raccourci de livraison.
2. Intégrer la couche applicative au commit transactionnel du moteur : consommer les notifications après chaque traitement réseau et conserver les causes des effets ajoutés. Réserver un événement virtuel pour les jobs applicatifs/terminal. Le modèle réseau ne doit pas dépendre des applications.
3. Raccorder bureaux par hôte, client/serveur, explorateur, import natif explicite, éditeur et afficheur. Persister installations/fichiers via le projet sans écraser une édition plus récente de l’UI. Les modifications de topologie restent verrouillées en simulation.
4. Implémenter le terminal (syntaxe/alias, cwd, fichiers, ipconfig, arp ciblé/global, netstat, ping par défaut quatre sondes, capture, historique, interruption, exit, cls/reset). Les noms peuvent être résolus localement depuis la topologie; DNS complet reste J4 et ne doit pas utiliser l’OS.
5. Compléter les vues par interface/couche, filtres, suivi/purge, activité des liens et export texte de la vue. Localiser chaque nouvel écran et les sorties du terminal dans les six langues.
6. Exécuter **R-TCP complet** avec vrais logiciels et cycle de vie, puis **R-FICHIERS complet** dans l’application; ajouter preuves, profil de fidélité final, fixtures/contrats complémentaires et tests UI/natifs. Ne marquer aucune porte globale terminée sur la seule base des tests sockets/VFS actuels.

## Commandes de reprise

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p sns-ui-egui --features ui-snapshots --test lan --test workbench --locked
cargo run -p sns-desktop --locked
python3 scripts/check_layers.py
python3 scripts/docs.py render
python3 scripts/docs.py check --require-reference
```

Les snapshots J0/J1 sont historiques. Les nouvelles références d’éditeur commencent par `j2-editor`; six captures couvrent les six langues. Ne pas régénérer les références historiques pour masquer une régression.
