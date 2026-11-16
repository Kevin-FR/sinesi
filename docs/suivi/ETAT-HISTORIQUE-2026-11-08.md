# État de reprise — Sinesi

## J7 ouvert — import, scénarios, rejeu et livraison

Objectif intégral : les 22 exigences et 9 lots J7 de la roadmap, en conservant les six langues et les vues Icônes/Schéma. F-SIM-007/008, F-TOP-009, F-EQU-008, F-OBS-006/008, F-CLI-019, F-PROJ-003/006 et F-DOC-001/002 sont vérifiés localement : scénarios réseau datés, état complet, rejeu, pannes par liaison, duplication/alignement, nuage, observation, diagnostic interne, récupération et annotations. RS-ENGINE-J7, RS-TOPOLOGY-J7, RS-DEVICES-J7, RS-OBSERVATION-J7 et RS-TERMINAL-J7 sont qualifiés. L’import F-PROJ-005 est implémenté avec rapport de pertes, et RS-PROJECTS-J7 reste en cours. Les autres lots, notamment rapports, configuration et livraison trois OS, restent ouverts. J7 n’est pas terminé.

## Socle J6 qualifié — 7 novembre 2026

J1 à J6 sont réalisés localement. J6 ajoute le courrier SMTP/POP3 avec relais MX/A, les comptes/messages/contacts persistants, le P2P avec recherche et téléchargement, les associations Wi-Fi par SSID et le modem explicite entre instances réelles. Les six langues **FR EN ES IT PT DE** et les vues **Icônes/Schéma** sont conservées.

| Registre | État |
| --- | --- |
| 178 exigences | **154 verified**, 1 implemented, 2 in_progress, 21 not_started |
| 50 lots techniques | **42 verified**, 3 in_progress, 3 not_started, 2 deferred |
| J1 | 28 exigences / 7 lots vérifiés |
| J2 | 40 exigences / 6 lots vérifiés |
| J3 | 15 exigences / 7 lots vérifiés |
| J4 | 20 exigences / 5 lots vérifiés |
| J5 | 17 exigences / 6 lots vérifiés |
| J6 | **16 exigences / 4 lots vérifiés**; R-MAIL, R-P2P et R-PONT qualifiées |
| Transversal | F-NFR-001/F-NFR-006 et RS-QUALITY-J0 restent en cours |

[features.json](features.json) et [taches.json](../rust/taches.json) sont les registres de référence. Chaque critère J6 possède une preuve reproductible avec commande, environnement, artefacts et révision. Les chemins prévus sont conservés; les fichiers réalisés figurent dans `actual_paths`. Les vues ont été régénérées sans recalculer l’audit Filius.

## Réalisation J6

Le [profil portable](../agnostique/APPLICATIONS_J6.md) définit comportements, contrats et limites. L’[adaptateur Rust](../rust/J6.md) décrit les modules; le [rapport J6](../verification/J6.md) relie les tests et captures aux exigences.

Fichiers principaux : `sns-model/src/mail.rs`, `p2p.rs`, `wireless.rs`, `modem.rs`; `sns-apps/src/mail.rs`, `p2p.rs`, `dns.rs`, `lib.rs`; `sns-protocols/src/bridge.rs`, `lib.rs`, `routing.rs`, `transport.rs`; `sns-engine/src/applications.rs`, `lib.rs`; nouveau `sns-bridge`; fenêtres `mail_ui.rs`/`p2p_ui.rs`, configuration, rendu et worker de la GUI. Trois schémas autonomes J6 et quatre fixtures complètent les tests. Le catalogue contient **545 clés × six langues**, avec **18 SVG** embarqués dont modem et point d’accès fournis par l’utilisateur.

Le projet 7 / `sns-educational-applications-v1` garde les données et réglages. Les sessions TCP, recherches et tunnels actifs restent transitoires. Les commandes restent API 1; les faits supplémentaires ont la version 8. Les données mail et les fichiers téléchargés passent par les mêmes transactions que le réseau. Les sockets système restent réservées à l’adaptateur de modem explicitement activé.

## Qualification exécutée

- `SNS_J6_ARTIFACT_DIR="$PWD/docs/verification" cargo test --workspace --locked --offline` : **155 tests du workspace**, plus **1 test enfant** dans le second processus; une mesure historique ignorée.
- `cargo test -p sns-ui-egui --features ui-snapshots --test lan --test workbench --locked --offline` : **38 tests rendus**, comparaison des **93 captures J6** sans mise à jour.
- `cargo fmt --all --check`, Clippy avec `-D warnings` et compilation `sns-desktop` : réussis.
- Contrôles des langues, assets/provenance, couches, documentation et inspection Filius : réussis.
- `docs.py render`, `docs.py check --require-reference` et vérification du manifeste : réussis.

Le [manifeste J6](../verification/j6-code-manifest.json) couvre **384 fichiers**, révision `sha256:3991dc6ed962fbd61a6259ac9c53185b19cc7b2f8e9b409453a3205283dfda28`. Les [traces](../verification/j6-artifacts.json) comprennent courrier local/interdomaines, P2P et les deux processus du pont. Le ping a traversé le tunnel réel et chaque côté a enregistré deux entrées externes. Les preuves [J1](../verification/J1.md), [J2](../verification/J2.md), [J3](../verification/J3.md), [J4](../verification/J4.md) et [J5](../verification/J5.md) restent historiques.

## Limites et reprise

Courrier textuel sans TLS/pièces jointes; contacts complets par l’API. P2P pédagogique avec quotas et sans interopérabilité historique. Wi-Fi logique sans propagation radio. Modem réel sans chiffrement/authentification, activé manuellement et fermé à l’arrêt/retour en conception. Aucun service mail ou pair Internet réel n’est contacté.

La qualification a été exécutée sur macOS arm64 avec le harnais rendu. Aucun nouveau parcours manuel en fenêtre autonome ni résultat Windows/Linux/CI distante n’est revendiqué. Les limites et recettes détaillées sont dans le rapport; la capacité globale reste à qualifier dans les exigences transversales.

**Prochaine tranche : J7**, selon la roadmap : projets partageables/récupérables, import des données historiques avec rapport, scénarios/rejeu des entrées externes, packaging et documentation de livraison. J6 enregistre les entrées du pont; il ne déclare pas encore leur rejeu réalisé. Conserver les contrats et preuves des versions précédentes.

## Tranche initiale J7 — import présent, jalon en cours

Le [profil portable J7](../agnostique/PROJETS_J7.md), l’[adaptateur en cours](../rust/J7-EN-COURS.md) et le [rapport partiel](../verification/J7-IMPORT-EN-COURS.md) décrivent précisément le périmètre. `sns-filius-import` lit les octets ZIP/XML sans extraction ni exécution, conserve les originaux et rapporte les champs non convertis. L’import est disponible dans le CLI (`import-filius`) et en arrière-plan dans la GUI. Les paramètres, comptes mail, logiciels connus, fichiers et DNS sont partiellement convertis; les pertes restantes sont explicites. Le projet précédent est conservé en cas d’erreur et le projet importé est marqué non enregistré.

Contrôles réellement exécutés : **162 tests du workspace plus un test enfant**, trois tests ignorés par défaut dont les deux recettes de corpus ensuite exécutées explicitement (**20 imports et ping historique**). Clippy sans avertissement, formatage, contrôles des couches et des **570 clés × six langues** réussis. Le parcours graphique d’import et d’échec est testé dans les six langues et les deux vues; six captures d’import ont été comparées sans mise à jour, les captures FR/DE inspectées. Les PNG J0–J6 restent historiques.

La tranche ne ferme pas J7 : couverture sémantique de l’import à approfondir, checkpoint complet/rejeu/pannes, autosauvegarde/session, annotations/styles, PNG/PDF, recherche/parcours/export structurés, duplication/alignement, préférences/aide/CLI/diagnostics, exemples et livraison/validation sur trois OS restent requis. La prochaine tranche est RS-ENGINE-J7, avec état complet des files, protocoles et applications; un simple rechargement du projet ne vaut pas reprise.

## Tranche moteur J7 ouverte

F-SIM-007/008 et RS-ENGINE-J7 sont en cours. Le format de reprise doit porter l’état complet et le journal des commandes/progressions/entrées externes, avec vérification d’intégrité et génération renouvelée. Les tables à clés composées seront représentées par des entrées structurées afin de conserver un contrat indépendant du langage. Aucun résultat de checkpoint/rejeu/panne n’est encore annoncé.

## Revalidation ciblée J6 dans le checkout J7

Le [complément de preuve](../verification/J6-REVALIDATION.md) conserve la qualification historique et ajoute une passe courante : 16 tests réseau/applications/pont plus un test enfant, 4 parcours graphiques fonctionnels et 1 contrat réussis. Les six langues et les vues Icônes/Schéma sont conservées; 575 clés et 18 SVG sont contrôlés. Les seize exigences et quatre lots J6 gardent leur état vérifié et leurs anciennes preuves.

La comparaison rendue échoue sur les quatre tests J6 : le bouton d’import J7 change la barre d’outils par rapport aux images historiques. Ces références restent intactes; cette passe n’annonce pas une suite globale verte. Les vues générées ont été régénérées. Aucun nouveau comportement de production ni critère J7 n’est déclaré terminé par cette revalidation.

## Tranche J7 — scénarios réseau, checkpoints et rejeu

F-SIM-007 est vérifié localement; RS-ENGINE-J7 reste en cours pour F-SIM-008. Le [contrat portable](../agnostique/REPRISE_J7.md), l’[adaptateur](../rust/J7-EN-COURS.md) et le [rapport de preuve](../verification/J7-REPRISE-EN-COURS.md) couvrent l’état complet, les actions réseau datées, les générations, les limites mémoire du journal et les entrées du pont rejouées hors ligne. La restauration reconstruit l’historique validé et compare l’état complet; elle a un coût proportionnel à cet historique.

Le CLI propose `run` avec scénario 2 et exports, `resume` et `replay`; la fenêtre Exécutions ouvre et exporte ces fichiers en arrière-plan. Une erreur conserve le document précédent; une reprise publie une nouvelle génération. Six langues et deux vues sont conservées. La qualification comprend 174 tests du workspace plus un test enfant, 2 recettes de référence réexécutées, Clippy sans avertissement et 12 captures nouvelles comparées sans mise à jour. Le catalogue contient 588 clés × six langues; 18 SVG restent embarqués. Les références historiques ne sont pas remplacées; la suite graphique historique complète n’est pas annoncée verte.

Le manifeste de cette tranche couvre 424 fichiers (`sha256:02f6cb9f761bb56561017fd99620884d1d4ddcc53d749a68beced8c7d6e54e1b`). Les traces et checkpoints synthétiques sont fournis. Le périmètre complet J7 reste ouvert : prochaine tranche, pannes par liaison; puis observations, édition/documentation/rapports, session et récupération, préférences/CLI/diagnostics, exemples et livraison avec preuves réelles sur les trois OS.

## J7 — pannes et lot moteur qualifiés le 8 novembre 2026

F-SIM-008 et RS-ENGINE-J7 sont vérifiés : délais, gigue et pertes par liaison, flux aléatoires séparés, commandes de scénario, interface et représentation dans les deux vues, checkpoint et rejeu. Le [profil portable](../agnostique/PANNES_J7.md) fixe la sémantique; le [rapport](../verification/J7-PANNES.md) relie les critères à la pile de production. Un checkpoint non configuré de la tranche précédente reste lisible.

Contrôles courants : 180 tests du workspace plus un test enfant, Clippy sans avertissement, formatage, 604 clés × six langues, 18 SVG et frontières vérifiés; 12 captures nouvelles comparées sans mise à jour. Les invocations CLI produisent trois traces identiques de 388 événements. La paire témoin reçoit ses 16 réponses; la liaison à 50 % en reçoit 2. Le manifeste porte 444 fichiers, révision `sha256:1da9f21252dc17f6397dbb1b485c34835a22e4e0c630e48102152efa8607ae88`.

Les anciennes captures et leurs sources qualifiées sont préservées. La suite graphique globale doit être réconciliée avec le produit courant avant la livraison; elle n’est pas annoncée verte par cette passe ciblée. Aucun résultat Windows/Linux ou packaging final n’est revendiqué.

Prochaine tranche : F-TOP-009 / RS-TOPOLOGY-J7 (duplication et alignement), puis F-EQU-008 / RS-DEVICES-J7 (apparence nuage). Les prérequis F-SIM-005/F-TOP-001 sont vérifiés. Le périmètre restant comprend aussi observation, documentation et exports, récupération/session, préférences/aide/diagnostics/CLI, exercices complets et preuve de livraison sur trois OS.

## Tranche topologie J7 ouverte

F-TOP-009 / RS-TOPOLOGY-J7 sont en cours, avec F-SIM-005 vérifié. La copie doit renouveler identités et MAC, remapper les références internes et câbles inclus, préserver les données applicatives et signaler les IPv4 identiques comme avertissements. L’alignement doit être réversible et ne pas modifier le réseau. Aucun résultat de cette tranche n’est encore revendiqué.

## J7 — duplication et alignement qualifiés le 8 novembre 2026

F-TOP-009 / RS-TOPOLOGY-J7 sont vérifiés selon le [profil portable](../agnostique/EDITION_J7.md) et le [rapport de preuve](../verification/J7-TOPOLOGIE.md). Les copies renouvellent identités/MAC et remappent câbles, routes, rôles LAN/WAN, réservations DHCP, SSID et données de courrier. Les IPv4 identiques sont signalées comme conflits possibles selon les segments. Huit opérations d’alignement/répartition restent réversibles; un refus conserve entièrement le document.

Qualification : 186 tests du workspace plus un test enfant, Clippy et formatage réussis; 620 clés × six langues, 18 SVG et frontières vérifiés; 12 captures nouvelles comparées sans mise à jour. Le LAN copié communique sur ses propres câbles, et son document enregistré est validé/pingé dans le CLI sans mutation. Le manifeste porte 459 fichiers, révision `sha256:dd924eaa4a3c9543750f5ab8546d0749d3bf302e17a8a13d7afb8e0c198b9fb1`.

Le prochain lot est F-EQU-008 / RS-DEVICES-J7 : apparence nuage du switch, avec ses ports et sa table MAC conservés. Les interactions complètes des annotations restent dans le lot documentation. J7 reste ouvert, notamment pour observation, rapports/exports, récupération/session, configuration et livraison trois OS. La suite graphique globale et la lisibilité générale de l’inspecteur doivent être qualifiées sur le produit final.

## Tranche apparence J7 ouverte

F-EQU-008 / RS-DEVICES-J7 sont en cours, avec F-TOP-001 vérifié. Le changement de représentation sera une mutation de présentation, conservant les ports, la SAT et les événements en attente, y compris en simulation. Les deux SVG nuage proviennent des assets utilisateur; aucun résultat de qualification de ce lot n’est encore annoncé.

## J7 — apparence nuage qualifiée le 8 novembre 2026

F-EQU-008 / RS-DEVICES-J7 sont vérifiés selon le [profil portable](../agnostique/PRESENTATION_J7.md) et le [rapport](../verification/J7-PRESENTATION.md). Le changement d’apparence est une commande de présentation sans reconstruction du réseau : ports, SAT, événements en attente et temps sont conservés. Le projet 8 garde le symbole; journal et checkpoint le restaurent. L’import lit le booléen historique sans exécution. Les deux SVG proviennent des assets fournis, avec provenance vérifiée.

Fichiers principaux : `sns-model/src/presentation.rs`, `sns-engine/src/lib.rs`, `sns-topology/src/lib.rs`, `sns-ui-egui/src/presentation_ui.rs`, `lan.rs`, `network_art.rs`, importeur, contrats et tests associés; liste complète dans `actual_paths`. Qualification exécutée : 192 tests du workspace plus un enfant, deux tests de corpus explicites, Clippy sans avertissement, formatage, 625 clés × six langues, 20 SVG et frontières vérifiés. Douze nouvelles captures Icônes/Schéma sont comparées sans mise à jour; les vues FR/DE ont été inspectées. Les anciennes captures restent conservées.

J7 demeure ouvert. Prochaine tranche : observation F-OBS-006/008, avec parcours causal local/global, filtres et exports structurés; puis documentation/rapports, récupération/session, configuration, exemples et livraison trois OS. La suite graphique complète et la lisibilité générale de l’inspecteur restent à qualifier sur le produit final. Aucun résultat Windows/Linux n’est revendiqué.

## Tranche observation J7 ouverte

F-OBS-006/008 et RS-OBSERVATION-J7 sont en cours; leurs prérequis Ethernet et IPv4 sont vérifiés. Les filtres combinés et les exports partageront une même sélection d’identifiants. Le parcours suivra les relations causales enregistrées, avec quatre couches et animations locale/globale, sans exécuter ni modifier le réseau. Aucun résultat de qualification de cette tranche n’est encore annoncé.

## J7 — observation qualifiée le 8 novembre 2026

F-OBS-006/008 et RS-OBSERVATION-J7 sont vérifiés selon le [profil portable](../agnostique/OBSERVATION_J7.md) et le [rapport](../verification/J7-OBSERVATION.md). Le nouveau module pur de trace combine équipement, bornes de temps, protocole, adresse et corrélation avec les filtres existants. JSONL et CSV figent la sélection et conservent les identités et contenus, avec écriture atomique en arrière-plan. Le parcours suit les causes, y compris entre générations d’une reprise, et montre quatre couches, étapes locales/globales et équipement courant dans les deux vues. L’animation ne modifie pas le moteur.

Fichiers : nouveau `sns-trace`, `sns-ui-egui/src/trace_ui.rs`, `traffic.rs`, `lan.rs`, tests, langues et vérificateur CSV; liste dans `actual_paths`. Qualification : 197 tests du workspace plus un enfant, Clippy sans avertissement, formatage, 651 clés × six langues, 20 SVG et frontières vérifiés; 12 captures comparées sans mise à jour et vues FR/DE inspectées. L’analyse CSV indépendante confirme 2 faits filtrés identiques à la source et un cas Unicode/échappement. Le manifeste porte 498 fichiers, révision `sha256:5b4404f0b49216e26c6f6b9139362436650aba12bb458fb49e31e17d7dfcf41f`.

La prochaine tranche est F-CLI-019 / RS-TERMINAL-J7 : documenter le diagnostic historique hors commandes élèves. Puis restent annotations, PNG/PDF, session/récupération, préférences/aide/CLI/diagnostics, exemples et livraison trois OS. La couverture sémantique de l’import reste partielle. La suite graphique complète et la lisibilité générale de l’inspecteur restent à qualifier avant livraison; aucun résultat Windows/Linux n’est revendiqué.

## Tranche diagnostic terminal J7 ouverte

F-CLI-019 / RS-TERMINAL-J7 sont en cours avec prérequis VFS/TCP vérifiés. La lecture du chemin historique confirme que `test` écrit `test.txt` et reste appelable par sélection dynamique malgré son absence de l’aide. La nouvelle interface doit réserver cette écriture à une recette interne de diagnostic et refuser la commande élève.

## J7 — diagnostic terminal qualifié le 8 novembre 2026

F-CLI-019 / RS-TERMINAL-J7 sont vérifiés selon le [profil portable](../agnostique/DIAGNOSTIC_TERMINAL_J7.md) et le [rapport](../verification/J7-TERMINAL.md). Le helper historique est documenté avec son chemin effectif : absent de l’aide, mais appelable par le sélecteur dynamique. Sinesi refuse `test`, `test ignored` et `help test`; la recette interne écrit/lit uniquement dans un clone isolé du VFS. La liste d’aide est centralisée dans le module terminal et utilisée par le rendu, avec le même contenu visible.

Fichiers : `sns-apps/src/terminal.rs`, `lib.rs`, `sns-ui-egui/src/desktop.rs`, test moteur et documentation; `actual_paths` recense les changements. Qualification exécutée : 1 recette ciblée, 198 tests du workspace plus un enfant, Clippy sans avertissement, formatage, 651 clés × six langues, 20 SVG et frontières vérifiés. Aucun nouveau PNG : le texte visible de l’aide ne change pas. Révision des 498 fichiers : `sha256:2da4353ff9c13f5286afddda344a163279be35a42741bf54cf1ee201c7b043c0`.

Prochaine tranche : F-PROJ-003/006, session et récupération atomique séparées du projet explicite, avec fermeture forcée réelle pour la recette. Restent aussi annotations/PNG/PDF, configuration et lancement, exemples, import partiel, qualification graphique globale et livraison sur trois OS. J7 n’est pas terminé.

## Tranche session et récupération J7 ouverte

F-PROJ-003/006 sont en cours dans RS-PROJECTS-J7 (import encore partiel). Les prérequis topologie sont vérifiés. La récupération conservera un document atomique distinct par instance, avec proposition explicite de reprise, priorité au fichier fourni au lancement et restauration en document non enregistré. Les verrous de fichiers doivent être validés entre processus puis par arrêt forcé; aucun résultat de récupération n’est encore annoncé.

## J7 — socle de récupération, intégration encore requise

F-PROJ-003/006 restent en cours. `sns-project/src/session.rs` et `tests/session.rs` ajoutent un document par instance, ses limites de taille/vue et un verrou compagnon. La [décision D-009](../rust/decisions/D-009-session-j7.md) et le [rapport partiel](../verification/J7-SESSION-EN-COURS.md) documentent les deux tests réussis et Clippy ciblé sans avertissement. Le test force réellement l’arrêt d’un processus, puis récupère le document publié sans modifier le projet enregistré. Un helper est ignoré par défaut et n’est lancé que par ce parent.

Les nouveaux diagnostics existent dans les six langues (655 clés présentes). Ce socle n’est pas encore branché au produit : contrat formel, autosauvegarde en arrière-plan, proposition de reprise, priorité au projet explicite, brouillons et cycle de vie du cache restent à réaliser avant qualification. Le dernier workspace intégral qualifié est la tranche terminal (198 tests plus un enfant); les deux tests de récupération sont une passe ciblée supplémentaire. Aucun jalon complet ni résultat trois OS n’est annoncé.

Pour la livraison ultérieure, le dépôt distant est joignable avec le compte déjà configuré et aucun run Actions n’a été trouvé lors de la lecture. Aucun commit, push ou lancement distant n’a été effectué. La préparation et les preuves de distribution restent dans les lots plateforme/qualité.

## J7 — session et récupération qualifiées le 8 novembre 2026

F-PROJ-003/006 sont vérifiés selon le [profil portable](../agnostique/SESSION_J7.md) et le [rapport](../verification/J7-SESSION.md). L’interface publie une copie atomique distincte toutes les deux secondes de temps réel, puis propose les sessions inactives au lancement. Un fichier explicite garde la priorité. La reprise revient en conception, sans destination de sauvegarde, avec langue, vue, sélection et brouillons non exécutés. La fermeture normale peut conserver ce dernier état avant de fermer. Les chemins de projet, préférences et cache sont protégés contre les écritures croisées.

Fichiers principaux : `sns-project/src/session.rs`, `session.schema.json`, fixture/contrats, `sns-ui-egui/src/session_ui.rs`, sérialisation inactive des brouillons et configurations, `sns-settings`, parseur desktop et tests; liste complète dans `actual_paths`. Qualification : **210 tests du workspace plus un enfant du pont**, deux processus de récupération réellement arrêtés, cinq tests ignorés par défaut; Clippy sans avertissement et formatage réussis. **12 captures nouvelles** sont comparées sans mise à jour, FR/DE inspectées; **668 clés × six langues**, **20 SVG** et frontières vérifiés. Manifeste : **516 fichiers**, `sha256:0d3628a1e9288b3327679b8e575ca5d0f01c7c9ae98de968d49a5c4600360e77`. Les preuves et sources historiques sont conservées.

L’arrêt forcé récupère la dernière publication durable, avec projet d’origine identique et saisies d’inspecteur/éditeur/terminal préservées sans exécution. Cache plein, réservations vides, corruption conservée, ancienne session sans brouillons et ancien acquittement de courrier sont testés. Le statut de récupération ne garantit pas la durabilité de la dernière frappe; les checkpoints réseau restent distincts.

RS-PROJECTS-J7 reste en cours pour l’import partiel. Prochaine tranche : F-DOC-001/002 dans RS-REPORTING-J7, notes et zones stylées durables, puis scène PNG/PDF. Restent aussi préférences complètes/aide/CLI/diagnostics, exemples, lisibilité et suite graphique globale, livraison et parcours réels sur trois OS. Aucun résultat Windows/Linux ni nouveau parcours natif manuel n’est revendiqué. J7 n’est pas terminé.

## Tranche annotations J7 ouverte

F-DOC-001/002 et RS-REPORTING-J7 sont en cours; F-TOP-001/F-OBS-001 et les lots prérequis sont vérifiés. La tranche ajoute dimensions, zones et styles persistants, édition réversible et interactions de documentation. Le profil de projet doit être versionné, les anciens documents et checkpoints rester lisibles et les annotations rester indépendantes des protocoles. Aucun résultat de qualification n’est encore annoncé.

## J7 — annotations et zones qualifiées le 8 novembre 2026

F-DOC-001/002 sont vérifiés selon le [profil portable](../agnostique/ANNOTATIONS_J7.md) et le [rapport](../verification/J7-ANNOTATIONS.md). Le projet 9 conserve texte, dimensions, couleurs, taille, famille, gras, italique et soulignement. Notes et zones se déplacent/redimensionnent avec aperçu puis transaction, se suppriment et s’annulent/rétablissent. Les zones restent derrière câbles et équipements dans Icônes/Schéma; Cadrer tient compte de leur géométrie. Un brouillon non appliqué rejoint la récupération de session sans modifier le projet explicite.

Fichiers principaux : `sns-model/src/annotations.rs`, transactions de topologie, `sns-ui-egui/src/annotations.rs`, canevas/session, contrats/fixture et nouveau `sns-fonts`. Quatre fontes Noto Regular/Bold sont embarquées sans changement de dépendance tierce; licence, provenance, poids et glyphes des six langues sont vérifiés. La CI préparée inclut ce contrôle, sans prétendre avoir exécuté un run distant.

Qualification finale : **215 tests du workspace plus un enfant du pont**, cinq tests ignorés par défaut; deux tests graphiques et **12 captures nouvelles comparées sans mise à jour**, FR/DE inspectées. Clippy sans avertissement, formatage, **693 clés × six langues**, **20 SVG**, quatre fontes et frontières réussis. Manifeste : **544 fichiers**, `sha256:020669e66fe9209d8e5899823d06db615a66f9e6b283df391a1eace825573084`. Les sources et références antérieures restent conservées.

Un échec intermittent de diagnostic de cache a été conservé dans le rapport. Le propriétaire de fichier libère désormais explicitement son verrou avant fermeture; **40 répétitions** de la recette, chacune avec un arrêt forcé, passent ensuite, ainsi que la suite complète finale. Ce complément est rattaché à F-PROJ-003/006 et RS-PROJECTS-J7.

RS-REPORTING-J7 reste en cours : prochaine tranche **F-DOC-003**, scène commune et PNG transparent, puis **F-DOC-004**, rapport PDF. J7 compte 11 exigences vérifiées sur 22; l’import reste implémenté avec couverture partielle. Restent plateforme/configuration/aide/CLI/diagnostics, exemples, suite graphique globale/lisibilité et livraison avec parcours sur trois OS. Aucun résultat Windows/Linux ou nouveau parcours natif manuel n’est revendiqué. J7 n’est pas terminé.

## Tranche PNG J7 ouverte

F-DOC-003 est en cours dans RS-REPORTING-J7, avec F-TOP-001/F-OBS-001 et les annotations qualifiés. La tranche construit une scène indépendante des widgets pour image transparente, cadrage de tous les objets et texte issu des polices embarquées. PNG et futur PDF partageront cette scène; aucun résultat d’export n’est encore annoncé.

## J7 — PNG qualifié; amélioration de structure ouverte

F-DOC-003 est vérifié selon [le rapport PNG](../verification/J7-PNG.md) : scène complète indépendante des widgets, glyphes embarqués, export transparent en arrière-plan et CLI, six langues et Icônes/Schéma. **224 tests du workspace plus un enfant**, cinq ignorés; trois tests graphiques et **12 captures nouvelles comparées**, quatre exports décodés indépendamment, Clippy/formatage et contrôles réussis. Manifeste : 568 fichiers, `sha256:283a6311478c3e03b56949a1188eecae75604599ed1d3dc45df62e55cf0a01a9`. Les références historiques restent conservées; le sélecteur natif et Windows/Linux ne sont pas qualifiés ici.

J7 compte **12 exigences vérifiées sur 22**; RS-REPORTING-J7 reste en cours pour PDF. À la demande de l’utilisateur, la prochaine tranche compare SolidCraft/WordCraft et améliore frontières, organisation du workbench et commandes de validation. Elle relève de RS-QUALITY-J0 et des exigences transversales F-NFR-001/002/003/006/008/009; elle ne ferme pas les validations encore manquantes. Les contrats, données utilisateur, six langues et vues restent le point de comparaison.
