# J7 en cours — import, scénarios, rejeu et livraison

Le périmètre intégral reste celui de [PROJETS_J7](../agnostique/PROJETS_J7.md) et de la roadmap. Les sections suivantes décrivent les tranches réalisées : import `.fls`, scénarios/checkpoints/rejeu, pannes, duplication/alignement et présentation. La récupération de session est également qualifiée localement. Les exports PDF/PNG et la livraison multiplateforme restent ouverts.

## Import

`sns-filius-import` prend des octets et un nom de source, puis renvoie `Project` et `ImportReport`. Le crate ne fait aucune I/O hôte. `archive.rs` borne le ZIP, ses noms et la décompression; `xml.rs` construit un arbre en lecture seule et résout les références; `convert.rs` et `mail.rs` contiennent les correspondances explicites. Les propriétés non utilisées restent dans le rapport et les octets des entrées sont conservés dans les ressources.

Le lockfile fixe `zip 9.4.1` (sans fonctionnalités par défaut, Deflate) et `roxmltree 0.20.0`, déjà présent dans le graphe de rendu. La configuration de l’analyseur refuse les DTD et borne les nœuds; le préfiltre refuse aussi les déclarations d’entités. Les décisions de noms/tailles/chemins et références restent dans SNS, indépendamment des protections de bibliothèque. Voir la [documentation ZIP](https://docs.rs/zip/9.4.1/zip/struct.ZipArchive.html) et les [options XML](https://docs.rs/roxmltree/0.20.0/roxmltree/struct.ParsingOptions.html).

`Editor::mark_unsaved` distingue une importation encore sans fichier utilisateur d’un projet enregistré. `import_ui.rs` utilise un canal borné et un fil de traitement; le résultat validé est publié d’un bloc, les erreurs conservent le projet antérieur. Le rapport affiche écarts, champs, classes et pièces dans les six langues et peut être exporté en JSON. Les vues Icônes/Schéma restent des projections.

```sh
cargo run -p sns-cli --locked --offline -- import-filius source.fls --output converted.sinesi.json --report coverage.json
cargo test -p sns-filius-import --locked --offline
SNS_RECORD_J7_IMPORT="$PWD/docs/verification/j7-import-corpus.json" SNS_RECORD_J7_IMPORT_DIR="$PWD/docs/verification" cargo test -p sns-filius-import --test import --locked --offline -- --ignored
cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_background --locked --offline
```

Les deux tests ignorés par défaut nécessitent `_archive/filius-master` : corpus des 20 fichiers et ping du LAN historique. Ils échouent si l’archive manque lorsqu’on les demande explicitement. Les tests ordinaires utilisent une fixture ZIP/XML synthétique propre à Sinesi et couvrent import, références, champs inconnus, pièces inactives, corruption, chemins, entités, quotas et vrai ping simulé. Le corpus, ses sources et son runtime ne sont pas redistribués.

## Suites requises

Finaliser la couverture sémantique d’import et ses écarts. Puis réaliser état complet/checkpoint/rejeu et pannes, récupération/session, annotations et scène commune d’export, filtres structurés/parcours, duplication/alignement, préférences/aide/CLI/diagnostics, exemples et packaging. Ne pas fermer RS-PROJECTS-J7 sur la seule présence de l’importeur. La validation Windows/Linux/macOS doit être fondée sur des résultats réellement obtenus.

La [fabrication des paquets et les parcours de livraison](LIVRAISON_J7.md)
séparent les preuves automatisées des ouvertures et manipulations natives.

## Qualification DHCP et routes statiques

`sns-filius-import/src/network.rs` convertit les plages DHCP, les réservations
valides et les lignes littérales des tables statiques. Le validateur du modèle
reste l’autorité. `xml.rs` permet de consulter un champ avant de le déclarer
converti. L’archive et le schéma du rapport version 1 sont conservés.
Les recettes passent par l’importeur, puis par les automates DHCP, ARP, IPv4
et ICMP du moteur.
[Résultats et traces](../verification/J7-IMPORT-DHCP-ROUTES.md)

## Qualification NAT et filtrage

`sns-filius-import/src/security.rs` traduit les tables NAT et le pare-feu
sérialisé par l’application ou le configurateur web historique. Le parseur lit
les arguments littéraux de `getField` et `set` pour les attributs connus d’une
règle. Le modèle valide chaque destination de redirection et la politique
complète. Les traces vérifient les échanges et leur rejeu.
[Résultats et traces](../verification/J7-IMPORT-NAT-FILTRAGE.md)

## Qualification RIP et documentation

`network.rs` lit les indicateurs RIP et de transfert IPv4. Le mode RIP garde
les routes statiques et la passerelle par défaut historiques inactives.
`annotations.rs` interprète les types, géométries, couleurs et styles
historiques connus et produit le profil de projet 9. Les références JavaBeans
sont des données et les formes exportées proviennent de la scène commune.
[Recettes et limites](../verification/J7-IMPORT-RIP-DOCUMENTATION.md)

## Qualification du courrier historique

`mail.rs` lit les comptes JavaBeans et les fichiers virtuels de Filius.
`mail_content.rs` convertit les dossiers, boîtes et contacts sous les quotas
du modèle. Une relève POP3 simulée utilise les messages importés et la
recette compare la trace après restauration et rejeu.
[Rapports du corpus et preuves](../verification/J7-IMPORT-COURRIER.md)

## Tranche moteur et fichiers d’exécution

Le [contrat portable de reprise](../agnostique/REPRISE_J7.md) décrit les formats. `sns-engine/src/recording.rs`, `checkpoint.rs`, `state.rs` et `state_value.rs` exposent `Recording`, `Checkpoint` et l’état structurel. Les états privés des protocoles, applications et du VFS implémentent `Serialize`; les entiers larges et les cartes à clés composées passent par un sérialiseur dédié. Le budget de transit ignoré par la sérialisation de trace des trames est sauvegardé séparément pour les réceptions en attente.

La restauration reconstruit l’état depuis les entrées validées, compare le hash complet, puis renouvelle la génération. Elle ne désérialise pas les structures privées depuis un arbre arbitraire. Son coût est O(historique), avec limites explicites; ce choix permet la validation d’atteignabilité dans ce profil pédagogique. Les octets estimés du journal sont bornés en mémoire et recalculés après lecture. Une exportation saturée est refusée.

`Scenario::network_actions` est activé au format 2. Le moteur conserve les actions comme `Event::NetworkCommand`, consommées transactionnellement par un état candidat; leur progression est enregistrée une seule fois. L’ancien format 1 garde sa trace de prototype inchangée. `replay_remaining` traite aussi une frontière d’événements fusionnée et remappe seulement les commandes d’une continuation explicitement validée.

`sns-project` ajoute des I/O atomiques bornées à 64 Mio pour les fichiers d’exécution, sans augmenter la limite des projets. `sns-cli/src/runtime_cli.rs` porte les exports et les sous-commandes ci-dessous. Tous les chemins sont contrôlés avant l’exécution et tous les exports sont encodés avant leur première écriture; chaque fichier est remplacé atomiquement, sans transaction commune entre plusieurs destinations.

```sh
cargo run -p sns-cli --locked --offline -- run fixtures/valid/network-scenario.json --until-us 1500 --checkpoint point.json
cargo run -p sns-cli --locked --offline -- resume point.json --until-us 10000000 --trace resumed.jsonl
cargo run -p sns-cli --locked --offline -- run fixtures/valid/network-scenario.json --until-us 10000000 --recording inputs.json
cargo run -p sns-cli --locked --offline -- replay inputs.json --trace replayed.jsonl
cargo run -p sns-cli --locked --offline -- resume point.json --continuation inputs.json --until-us 10000000
```

`runtime_ui.rs` utilise un canal borné et un fil pour lire, reconstruire ou écrire. Le worker moteur produit les captures et publie un nouvel état après validation. Les commandes ont un compteur d’acquittement distinct de leur identité; les reçus conservés et les actions futures réservent leurs noms. Le pilote du modem renouvelle sa session et évite ses anciens reçus lors d’une activation explicite après reprise. Aucun tunnel n’est ouvert par le chargement ou le rejeu.

Le [rapport de cette tranche](../verification/J7-REPRISE-EN-COURS.md) ne ferme pas le jalon : pannes par liaison, documentation/exports, session/récupération, préférences, exemples complets et livraison trois OS restent requis.

## Pannes par liaison

Le [profil portable](../agnostique/PANNES_J7.md) fixe l’algorithme et les unités. `sns-model/src/link_conditions.rs` définit `LinkConditions` et sa validation; `ConfigureLink` reste dans l’enveloppe de commande API 1. `sns-protocols/src/link_conditions.rs` porte les flux SplitMix64 séparés, leurs paramètres et les faits 9. Le plan réseau décide perte et délai sur un état candidat; le commit conserve l’atomicité et la cause de la réception.

`Network::link_conditions` est une carte vide par défaut, omise de l’état sérialisé tant qu’elle est vide. Cela maintient la compatibilité des checkpoints non configurés de la tranche précédente, vérifiée sur une fixture figée. Les liaisons configurées utilisent des `Delay::Timer` avec délai calculé; la politique historique `Delay::Link` reste inchangée pour les autres. Aucun module de sockets réelles n’est impliqué.

Le worker publie graine et configurations pour la vue. `link_conditions_ui.rs` propose une sélection de câble, des champs numériques et une application en simulation; le document de topologie n’est pas modifié. Le canvas montre le taux et colore les conditions en Icônes et Schéma. Les écritures de checkpoint utilisent le chemin déjà qualifié de `runtime_ui.rs`.

```sh
cargo run -p sns-cli --locked --offline -- run fixtures/valid/link-conditions-scenario.json --until-us 5000000 --checkpoint links-point.json
cargo run -p sns-cli --locked --offline -- resume links-point.json --until-us 50000000 --trace links-resumed.jsonl
cargo run -p sns-cli --locked --offline -- run fixtures/valid/link-conditions-scenario.json --until-us 50000000 --recording links-inputs.json --trace links-full.jsonl
cargo run -p sns-cli --locked --offline -- replay links-inputs.json --trace links-replayed.jsonl
```

## Duplication et alignement

Le [profil portable](../agnostique/EDITION_J7.md) fixe F-TOP-009. `sns-topology/src/duplication.rs` prépare un projet candidat et un rapport d’identités/créations/liens omis, sans I/O. L’allocateur réserve les identités existantes, les MAC normalisées, les noms et SSID; les références typées sont remappées explicitement. Une carte distincte conserve la correspondance des messages partagés entre boîtes copiées.

`Edit::Duplicate` et `Edit::Align` passent par la même transaction et l’historique. `Editor::apply` renvoie désormais `EditResult` : le client peut sélectionner les créations et expliquer les câbles extérieurs omis. Toute erreur de préparation restaure aussi le compteur d’identités de l’éditeur. Les alignements modifient les positions existantes en place pour éviter une modification d’ordre sans déplacement.

La borne de document de 8 Mio est déclarée dans `sns-model::MAX_PROJECT_DOCUMENT_BYTES` et réutilisée par la persistance; sa valeur ne change pas. La préparation d’une copie vérifie la taille sérialisée avant publication. Le seul changement de dépendance est un lien de test de `sns-topology` vers `sns-engine`, déjà présent dans le workspace; aucune version de paquet n’a changé.

`topology_ui.rs` porte bouton/raccourci, menu d’alignement et diagnostic IPv4 non bloquant. Les identités sélectionnées sont filtrées après annulation/rétablissement afin d’éviter des références de vue périmées. Le moteur de simulation n’est pas remplacé par une opération d’alignement; les changements de topologie restent réservés à la conception.

Les tests exécutent un ping sur le LAN copié et vérifient que ses réceptions Ethernet restent dans ce LAN. Ils couvrent aussi NAT/DHCP/routes, association radio, courrier, fichiers, noms Unicode, quotas et les huit alignements. Le parcours d’interface utilise les six langues, les deux vues, le raccourci D et l’annulation/rétablissement.

## Apparence nuage

Le [profil portable](../agnostique/PRESENTATION_J7.md) fixe F-EQU-008. `sns-model/src/presentation.rs` prépare un `Project::with_symbol` validé; `DeviceSymbol::Cloud` promeut le projet au profil 8, uniquement pour un switch. `Command::SetSymbol` est traité dans le moteur sans recréer `Network` ni `Applications`, sans consommer d’aléa ou avancer le temps. Un fait 10 accompagne la modification effective. Les reçus, révisions et limites suivent les règles habituelles.

`Edit::SetSymbol` fournit l’historique de conception. `Editor::accept_runtime` fusionne les symboles des seules identités explicitement modifiées, en conservant les valeurs de topologie locales. `presentation_ui.rs` envoie la commande en simulation et l’édition en conception. L’interface attend l’acquittement pour refléter l’état confirmé; les autres modifications du document sont verrouillées pendant cette attente. Le champ numérique de rétention SAT possède désormais un libellé accessible associé.

Les deux `cloud.svg` de `assets/network` sont copiés sans modification depuis les assets utilisateur; provenance et empreintes sont vérifiées. L’importeur convertit le booléen historique `cloud`, avec diagnostic si sa valeur est invalide. Les schémas autonomes de présentation et les schémas courants de scénario/journal/checkpoint couvrent le nouveau profil; les anciens schémas de projet restent inchangés.

```sh
cargo test -p sns-engine --test presentation --locked --offline
cargo test -p sns-filius-import --test import --locked --offline -- --ignored
cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_cloud --locked --offline
```

Le [rapport de qualification](../verification/J7-PRESENTATION.md) relie les recettes aux preuves locales. Il ne ferme pas les autres lots J7.

## Recherche de traces et parcours causal

Le [profil portable](../agnostique/OBSERVATION_J7.md) fixe F-OBS-006/008. Le nouveau crate pur `sns-trace` dépend du modèle, des faits du moteur et de `serde_json`; il ne possède ni horloge ni I/O. `Query` combine les critères, `export` encode les mêmes références en JSONL/CSV avec une borne de 64 Mio. Aucun paquet tiers n’est ajouté au lockfile : seuls le crate local et ses liens sont nouveaux. `scripts/check_layers.py` inclut ce crate dans les racines sans GUI ni sockets.

`Journey` construit un index des couples génération/événement et des causes. Une reprise conserve les anciens faits : la résolution préfère une cause de même génération, puis accepte une identité unique dans la trace complète. Les ancêtres et les descendants de la sélection sont ordonnés; les autres branches des ancêtres ne sont pas ajoutées. Les causes ambiguës/manquantes sont signalées comme parcours partiel. `layer_parts` expose les enveloppes enregistrées et laisse les couches absentes vides.

`TrafficFilter` compose la requête avec interface, couches, recherche textuelle et borne d’effacement. La table et les trois formats d’export partagent son prédicat. Les colonnes redimensionnables ont des minima lisibles; le défilement non animé évite la boucle de recalcul lors d’une réduction de sélection. `trace_ui.rs` gère les contrôles, la sélection figée, un fil d’export et l’animation locale/globale. L’origine du worker est distincte de l’identité historique du fait sélectionné; une nouvelle exécution ferme le parcours, une reprise permet d’inspecter ses faits antérieurs.

La fenêtre expose les quatre couches, la cause courante et une chronologie des équipements; le canevas souligne l’équipement courant en Icônes et Schéma. Les étapes de 600 ms utilisent seulement le temps de présentation d’egui. Le réseau ne reçoit aucune commande pour cette animation. Le parcours peut être ouvert, arrêté, repris, avancé/reculé et consulté par équipement.

```sh
SNS_RECORD_J7_OBSERVATION="$PWD/docs/verification" cargo test --workspace --locked --offline
cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_trace_queries --locked --offline
python3 scripts/check_trace_exports.py docs/verification/j7-observation-selected.jsonl docs/verification/j7-observation-selected.csv --source docs/verification/j7-observation-web.jsonl
python3 scripts/check_trace_exports.py docs/verification/j7-observation-escaping.jsonl docs/verification/j7-observation-escaping.csv
```

Le [rapport](../verification/J7-OBSERVATION.md) distingue les preuves de ce lot et les portes de sortie J7 encore ouvertes. Les schémas d’événements ne changent pas : JSONL reprend leur sérialisation et le CSV porte leur contenu complet.

## Diagnostic terminal interne

Le [profil portable](../agnostique/DIAGNOSTIC_TERMINAL_J7.md) fixe F-CLI-019. La lecture couvre `Terminal.test`, `terminalEingabeAuswerten` puis `Anwendung.ausfuehren` et son invocation : la méthode publique est effectivement sélectionnable malgré son absence de `usageInfo`. Aucune exécution du code archivé n’est nécessaire.

`sns-apps::COMMAND_HELP` centralise le texte de la liste pédagogique, auparavant dupliqué dans `desktop.rs`. Le rendu utilise cette même constante avec l’aide traduite; son contenu visible ne change pas. `usage("test")` reste absent de la liste autorisée, ce qui refuse la saisie et `help test` avant tout effet. La recette interne s’exécute seulement dans le test de développement sur un clone de `Vfs`, avec écriture/lecture de `test.txt` et sans mutation du VFS réel de l’hôte simulé.

```sh
SNS_RECORD_J7_TERMINAL="$PWD/docs/verification" cargo test -p sns-engine --test applications j7_historical --locked --offline
```

Le test passe par le moteur et le terminal de production pour vérifier les trois refus, les fichiers et le réseau inchangés, l’absence de fait de fichier ou Ethernet, l’inventaire d’aide et le diagnostic isolé. Le [rapport](../verification/J7-TERMINAL.md) relie cette preuve au registre. Aucun nouveau protocole, format ou logiciel installable n’est ajouté.

## Session et récupération

Le [profil portable](../agnostique/SESSION_J7.md), la [décision D-009](decisions/D-009-session-j7.md) et le [rapport](../verification/J7-SESSION.md) couvrent F-PROJ-003/006. `sns-project/src/session.rs` valide et publie atomiquement des documents indépendants du projet explicite, avec verrou compagnon et garde d’allocation. `session_ui.rs` gère les travaux en arrière-plan, le cycle de deux secondes, le cache plein, les propositions et la publication avant fermeture normale.

`DesktopState` sérialise des listes d’associations plutôt que des clés JSON composées; son décodeur vérifie les propriétaires d’applications. Inspecteur, routage, sécurité, conditions et formulaires applicatifs restent des données inactives. Les comptes d’acquittement du courrier sont réinitialisés pour préserver le nouveau composeur. Le contrôle des sorties est partagé entre sauvegarde de projet, exports d’exécution, de traces et de rapport d’import.

`apps/sns-desktop/src/main.rs` accepte un projet positionnel, les chemins Unicode et la frontière `--`; il refuse plusieurs projets ou la combinaison prototype/projet. Ce parseur n’implémente pas encore le lot complet F-CFG-005. Les préférences restent limitées à la langue dans cette tranche.

```sh
SNS_RECORD_J7_SESSION="$PWD/docs/verification" cargo test --workspace --locked --offline
cargo test -p sns-ui-egui --features ui-snapshots --test session recovery_offer --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

Les helpers ignorés des tests `sns-project/tests/session.rs` et `sns-ui-egui/tests/session.rs` sont lancés par leurs parents puis arrêtés de force. Le premier confirme la libération du verrou et l’intégrité du projet explicite; le second utilise le vrai cycle automatique et restaure nom appliqué, texte non appliqué, éditeur et saisie de terminal sans l’exécuter. La fermeture normale, les chemins protégés et un cache plein sont aussi testés. Les six langues et les vues Icônes/Schéma sont conservées. RS-PROJECTS-J7 reste en cours pour F-PROJ-005; la preuve sur trois OS et la suite graphique globale restent des portes de livraison.

## Annotations et zones

Le [profil portable](../agnostique/ANNOTATIONS_J7.md) fixe F-DOC-001/002. `sns-model/src/annotations.rs` porte les styles et la transaction `with_annotation`; `Edit::AddAnnotation` attribue l’identité, `Edit::Annotate` met à jour en place sans modifier l’empilement. Les validations de dimensions, texte, identités et taille sont appliquées avant commit. La sérialisation omet `style: None`, préservant les anciens projets et leur état de checkpoint.

`sns-ui-egui/src/annotations.rs` gère inspecteur, aperçu de déplacement/redimensionnement et peinture avant câbles/nœuds. La poignée ne s’enregistre que si elle rencontre le canevas. L’édition numérique utilise des libellés accessibles; son application et le relâchement de glisser restent une transaction chacun. Cadrer prend les limites des annotations en compte. Le mode documentation protège les équipements contre les mutations de topologie. Les brouillons rejoignent la récupération inactive avec `annotation_draft` optionnel dans la version 1, sans nouvelle commande moteur.

Le nouveau `sns-fonts`, sans dépendance tierce, fournit quatre TTF Noto Regular/Bold embarqués. Leurs familles egui sont nommées exclusivement pour les annotations; les menus gardent leurs polices. L’italique utilise `TextFormat::italics`. Les sources sont fixées au commit `ffebf8c1ee449e544955a7e813c54f9b73848eac`, licence OFL-1.1 et empreintes dans `assets/fonts`. Voir [la documentation officielle des licences Noto](https://notofonts.github.io/noto-docs/website/use/). Aucun service de police n’est utilisé à l’exécution et aucune version de bibliothèque externe n’est modifiée.

```sh
cargo test -p sns-project -p sns-topology --locked --offline
cargo test -p sns-ui-egui --features ui-snapshots --test annotations --locked --offline
python3 scripts/check_fonts.py
```

Les [preuves](../verification/J7-ANNOTATIONS.md) distinguent cette tranche des exports PNG/PDF encore ouverts. Les fontes appartiennent au chemin réalisé prévu pour les futurs exports, sans annoncer un shaper ou backend de rapport déjà présent.

## Scène documentaire et PNG

Le [profil portable](../agnostique/EXPORTS_J7.md) et [D-010](decisions/D-010-export-scene-j7.md) fixent F-DOC-003. `sns-render` contient scène, assets communs et raster CPU; `sns-fonts/src/layout.rs` fournit shaping, lignes/groupes et contours. `Scene::build_with_labels` accepte une projection confirmée des noms/IPv4/MAC sans écrire les paramètres effectifs dans le projet. Le backend PDF reste à ajouter.

`export_ui.rs` orchestre sélection native, rendu et écriture atomique sur des fils bornés; `begin_png_export` est partagé par le dialogue et les tests. La destination active est protégée contre les autres sauvegardes. L’erreur conserve projet et fichier antérieur. Les résultats indiquent dimensions, vue capturée et textes dépassant leur zone; le nom court est affiché, le chemin complet reste disponible au survol.

```sh
cargo run -p sns-cli --locked --offline -- render-png fixtures/valid/annotations.json --output network.png --view diagram --size 2400 --background transparent
SNS_RECORD_J7_PNG="$PWD/docs/verification" cargo test --workspace --locked --offline
cargo test -p sns-ui-egui --features ui-snapshots --test png --locked --offline
python3 scripts/check_png_exports.py docs/verification/j7-png-diagram.png --metadata docs/verification/j7-png-diagram.json
```

Le CLI accepte `--view icons|diagram`, `--size 256..4096`, `--background transparent|canvas`, avec refus des options invalides/doublons et de la source comme destination. Il renvoie les dimensions et comptes en JSON sur stdout. La GUI conserve les six langues et la vue courante. Ses contrôles utilisent le même service d’export; le dialogue natif n’est pas simulé par les tests du harnais.

Les [preuves](../verification/J7-PNG.md) incluent quatre PNG indépendants, vingt symboles, quatre variantes de police et étiquettes DHCP, ainsi que conservation du réseau/projet et captures du dialogue dans les six langues. Le PNG est préparé sans fenêtre par le CLI. Aucun SVG exportable, backend PDF ou prise en charge universelle des écritures n’est activé par cette tranche.


## Préférences d’affichage

F-CFG-003 / RS-PLATFORM-J7 : voir le [profil portable](../agnostique/AFFICHAGE_J7.md)
et l’[adaptateur](AFFICHAGE_J7.md). La version 2 du profil utilisateur conserve langue,
fonte, taille, représentation, panneaux et géométrie des fenêtres. Les projets et
protocoles restent indépendants. La priorité de configuration et la livraison restent
à traiter séparément; l’aide et les exports PNG/PDF décrits par leurs rapports sont
qualifiés après les premières sections historiques ci-dessus.


## Configuration de démarrage

F-CFG-004 dispose du [profil portable](../agnostique/CONFIGURATION_J7.md), de
l’[adaptateur](CONFIGURATION_J7.md) et de [D-013](decisions/D-013-configuration-j7.md).
La résolution et les diagnostics sont communs au lanceur et à l’interface. Les
options temporaires ne sont pas copiées dans le profil lors d’une modification
sans rapport. Les options h/w, la version longue et le contrôle sans fenêtre
sont raccordés. La [politique d’attente RTT](ATTENTES_J7.md) complète r avec
reprise/rejeu. Le [journal technique](DIAGNOSTIC_J7.md) couvre v/l et F-CFG-006.
Les options n/s appliquent le thème système et le profil compact de lancement.
Le [catalogue pédagogique](EXEMPLES_J7.md) couvre les familles de F-CFG-008.
