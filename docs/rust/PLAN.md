# Plan d’implémentation Rust

Vue générée depuis [taches.json](taches.json). Les chemins sont des cibles proposées; les fichiers effectivement réalisés sont indiqués séparément. Les dépendances expriment les prérequis techniques; les recettes restent celles du catalogue agnostique.

<a id="rs-engine-j0"></a>

## RS-ENGINE-J0 — Moteur et cycle de simulation — J0

- Jalon : `J0`; état : `verified`; taille indicative : `L`.
- Exigences : [F-SIM-005](../agnostique/CATALOGUE.md#f-sim-005).
- Prérequis : aucun.
- Cibles : `crates/sns-engine`.
- Fichiers réalisés : `Cargo.toml`, `crates/sns-engine`, `crates/sns-model`, `docs/decisions/D-001-ordonnancement.md`, `fixtures/valid/trace.jsonl`, `rust-toolchain.toml`.
- Approche : File BinaryHeap avec Reverse; temps entier, ordre total, moteur synchrone à propriétaire unique; aucune minuterie système dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-projects-j0"></a>

## RS-PROJECTS-J0 — Projets et import Filius — J0

- Jalon : `J0`; état : `verified`; taille indicative : `L`.
- Exigences : [F-PROJ-004](../agnostique/CATALOGUE.md#f-proj-004).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-project`, `crates/sns-filius-import`.
- Fichiers réalisés : `contracts`, `crates/sns-project`, `docs/decisions/D-004-format-j0.md`, `fixtures`.
- Approche : DTO Serde pour format neutre JSON/ZIP; XML .fls interprété comme données en liste autorisée; aucune exécution de JavaBeans ou de bytecode.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-quality-j0"></a>

## RS-QUALITY-J0 — Contrats, qualité et outillage — J0

- Jalon : `J0`; état : `in_progress`; taille indicative : `M`.
- Exigences : [F-NFR-001](../agnostique/CATALOGUE.md#f-nfr-001), [F-NFR-002](../agnostique/CATALOGUE.md#f-nfr-002), [F-NFR-003](../agnostique/CATALOGUE.md#f-nfr-003), [F-NFR-006](../agnostique/CATALOGUE.md#f-nfr-006), [F-NFR-008](../agnostique/CATALOGUE.md#f-nfr-008), [F-NFR-009](../agnostique/CATALOGUE.md#f-nfr-009).
- Prérequis : aucun.
- Cibles : `contracts`, `fixtures`, `crates/sns-cli`, `docs/decisions`, `crates/sns-ui-egui`, `apps/sns-desktop`, `crates/sns-html`.
- Fichiers réalisés : `.cargo/config.toml`, `.github/workflows/ci.yml`, `AGENTS.md`, `CONTRIBUTING.md`, `Cargo.lock`, `Cargo.toml`, `LICENSE`, `README.md`, `apps/sns-desktop`, `contracts`, `crates/sns-cli`, `crates/sns-html`, `crates/sns-ui-egui`, `crates/sns-ui-egui/AGENTS.md`, `crates/sns-ui-egui/src/prototype`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/tests/lan.rs`, `docs/README.md`, `docs/agnostique/ARCHITECTURE.md`, `docs/agnostique/EVOLUTION.md`, `docs/rust/ARCHITECTURE.md`, `docs/rust/DEPENDANCES.md`, `docs/rust/DEVELOPPEMENT.md`, `docs/rust/REVUE_STRUCTURE_2026_11.md`, `docs/rust/decisions`, `docs/rust/decisions/D-011-structure-et-workflow.md`, `docs/rust/layers.json`, `docs/rust/refactor-paths.json`, `docs/suivi/ETAT-HISTORIQUE-2026-11-08.md`, `docs/suivi/ETAT.md`, `docs/verification/J0.md`, `docs/verification/STRUCTURE-2026-11.md`, `docs/verification/STRUCTURE-OUVERTURE.md`, `docs/verification/STRUCTURE-POLITIQUE-2026-11.md`, `docs/verification/STRUCTURE-REVUE-2026-11.md`, `docs/verification/structure-2026-11-ci.log`, `docs/verification/structure-2026-11-code-manifest.json`, `docs/verification/structure-2026-11-docs.log`, `docs/verification/structure-2026-11-environment.json`, `docs/verification/structure-2026-11-gpu.log`, `docs/verification/structure-2026-11-initial-failure.log`, `docs/verification/structure-2026-11-keyboard-stress.json`, `docs/verification/structure-2026-11-source-delta.json`, `docs/verification/structure-document-open-ci.log`, `docs/verification/structure-document-open-code-manifest.json`, `docs/verification/structure-document-open-docs.log`, `docs/verification/structure-document-open-source-patch.json`, `docs/verification/structure-document-open-ui.log`, `docs/verification/structure-policy-2026-11-before.log`, `docs/verification/structure-policy-2026-11-check.log`, `docs/verification/structure-policy-2026-11-ci.log`, `docs/verification/structure-policy-2026-11-code-manifest.json`, `docs/verification/structure-policy-2026-11-docs.log`, `docs/verification/structure-policy-2026-11-environment.json`, `docs/verification/structure-policy-2026-11-gpu.log`, `docs/verification/structure-policy-2026-11-references.json`, `docs/verification/structure-policy-2026-11-source-delta.json`, `docs/verification/structure-policy-2026-11-tests.log`, `docs/verification/structure-review-2026-11-before.log`, `docs/verification/structure-review-2026-11-ci.log`, `docs/verification/structure-review-2026-11-code-manifest.json`, `docs/verification/structure-review-2026-11-docs.log`, `docs/verification/structure-review-2026-11-environment.json`, `docs/verification/structure-review-2026-11-initial-ci.log`, `docs/verification/structure-review-2026-11-references.json`, `docs/verification/structure-review-2026-11-source-delta.json`, `docs/verification/structure-review-2026-11-tests.log`, `scripts/check_layers.py`, `scripts/test_docs.py`, `scripts/test_layers.py`, `xtask`, `xtask/src/main.rs`, `apps/sns-desktop/Cargo.toml`, `crates/sns-ui-egui/Cargo.toml`, `docs/verification/STRUCTURE-COMPLEMENTS-2026-11.md`, `docs/verification/j7-qualification-structure.log`, `docs/verification/j7-qualification-ui.log`, `docs/verification/j7-qualification-ci.log`, `docs/verification/j7-qualification-code-manifest.json`, `docs/verification/j7-qualification-docs.log`, `docs/verification/j7-qualification-source-patch.json`.
- Approche : Schémas neutres, runner et moteur sans UI; prototype egui/eframe 0.37 canevas/table/focus, graphe upstream/forks et HTML pédagogique natif lien/image/POST sans sorties réelles; documenter D-002/D-003 et révisions candidates sans prétendre les recettes produit acquises. Revue du 9 novembre : F-NFR-002/008, contrôle des couches avec toutes les fonctionnalités optionnelles et héritage des métadonnées, dépendances et lints; sans réécriture du moteur ni changement des contrats.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j1"></a>

## RS-DEVICES-J1 — Équipements et configuration — J1

- Jalon : `J1`; état : `verified`; taille indicative : `M`.
- Exigences : [F-EQU-001](../agnostique/CATALOGUE.md#f-equ-001), [F-EQU-002](../agnostique/CATALOGUE.md#f-equ-002), [F-EQU-006](../agnostique/CATALOGUE.md#f-equ-006).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : `crates/sns-engine/tests/lan.rs`, `crates/sns-model/src/lib.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-ui-egui/src/workbench`, `fixtures/valid/lan.json`.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-engine-j1"></a>

## RS-ENGINE-J1 — Moteur et cycle de simulation — J1

- Jalon : `J1`; état : `verified`; taille indicative : `L`.
- Exigences : [F-SIM-001](../agnostique/CATALOGUE.md#f-sim-001), [F-SIM-002](../agnostique/CATALOGUE.md#f-sim-002), [F-SIM-003](../agnostique/CATALOGUE.md#f-sim-003), [F-SIM-004](../agnostique/CATALOGUE.md#f-sim-004), [F-SIM-006](../agnostique/CATALOGUE.md#f-sim-006).
- Prérequis : aucun.
- Cibles : `crates/sns-engine`.
- Fichiers réalisés : `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/lan.rs`, `crates/sns-model/src/lib.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/lan.rs`.
- Approche : File BinaryHeap avec Reverse; temps entier, ordre total, moteur synchrone à propriétaire unique; aucune minuterie système dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-ipv4-j1"></a>

## RS-IPV4-J1 — IPv4 et ICMP — J1

- Jalon : `J1`; état : `verified`; taille indicative : `L`.
- Exigences : [F-IP4-001](../agnostique/CATALOGUE.md#f-ip4-001), [F-IP4-002](../agnostique/CATALOGUE.md#f-ip4-002), [F-IP4-004](../agnostique/CATALOGUE.md#f-ip4-004).
- Prérequis : RS-LINK-J1.
- Cibles : `crates/sns-protocols/src/ipv4`.
- Fichiers réalisés : `crates/sns-cli/src/main.rs`, `crates/sns-engine/tests/lan.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-ui-egui/src/workbench`.
- Approche : Adresses numériques typées, masques validés, next-hop explicite, erreurs structurées et corrélation des sondes.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-link-j1"></a>

## RS-LINK-J1 — Ethernet, switch et ARP — J1

- Jalon : `J1`; état : `verified`; taille indicative : `L`.
- Exigences : [F-L2-001](../agnostique/CATALOGUE.md#f-l2-001), [F-L2-002](../agnostique/CATALOGUE.md#f-l2-002), [F-L2-003](../agnostique/CATALOGUE.md#f-l2-003), [F-L2-004](../agnostique/CATALOGUE.md#f-l2-004), [F-L2-005](../agnostique/CATALOGUE.md#f-l2-005).
- Prérequis : RS-DEVICES-J1.
- Cibles : `crates/sns-protocols/src/link`.
- Fichiers réalisés : `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/lan.rs`, `crates/sns-protocols/src/lib.rs`, `fixtures/valid/lan.json`.
- Approche : Machines à états événementielles; clones de données de trace immuables; timers virtuels pour les caches.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-projects-j1"></a>

## RS-PROJECTS-J1 — Projets et import Filius — J1

- Jalon : `J1`; état : `verified`; taille indicative : `L`.
- Exigences : [F-PROJ-001](../agnostique/CATALOGUE.md#f-proj-001), [F-PROJ-002](../agnostique/CATALOGUE.md#f-proj-002).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-project`, `crates/sns-filius-import`.
- Fichiers réalisés : `contracts/filius-known-classes.json`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/lib.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/chrome.rs`, `crates/sns-ui-egui/src/workbench/document.rs`, `crates/sns-ui-egui/src/workbench/import_ui.rs`, `crates/sns-ui-egui/src/workbench/mod.rs`, `crates/sns-ui-egui/src/workbench/runtime_ui.rs`, `docs/verification/STRUCTURE-OUVERTURE.md`, `docs/verification/j1-filius-inventory.json`, `docs/verification/structure-document-open-ci.log`, `docs/verification/structure-document-open-code-manifest.json`, `docs/verification/structure-document-open-docs.log`, `docs/verification/structure-document-open-source-patch.json`, `docs/verification/structure-document-open-ui.log`, `scripts/inspect_filius.py`, `scripts/test_filius_inspection.py`.
- Approche : DTO Serde pour format neutre JSON/ZIP; XML .fls interprété comme données en liste autorisée; aucune exécution de JavaBeans ou de bytecode.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-quality-j1"></a>

## RS-QUALITY-J1 — Contrats, qualité et outillage — J1

- Jalon : `J1`; état : `verified`; taille indicative : `M`.
- Exigences : [F-NFR-004](../agnostique/CATALOGUE.md#f-nfr-004), [F-NFR-005](../agnostique/CATALOGUE.md#f-nfr-005).
- Prérequis : aucun.
- Cibles : `contracts`, `fixtures`, `crates/sns-cli`, `docs/decisions`, `crates/sns-ui-egui/tests`.
- Fichiers réalisés : `.github/workflows/ci.yml`, `contracts/command-lan.schema.json`, `contracts/event-lan.schema.json`, `contracts/project-lan.schema.json`, `crates/sns-cli/tests/runner.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/tests/lan.rs`, `fixtures/lan_load.rs`, `scripts/check_layers.py`, `scripts/check_manifest.py`.
- Approche : Mesures de capacité/réactivité et tests egui_kittest des interactions, rôles accessibles, labels, focus et zoom; essais natifs distincts pour clavier/IME/DPI; aucun sleep réel dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-topology-j1"></a>

## RS-TOPOLOGY-J1 — Éditeur de réseau — J1

- Jalon : `J1`; état : `verified`; taille indicative : `L`.
- Exigences : [F-TOP-001](../agnostique/CATALOGUE.md#f-top-001), [F-TOP-002](../agnostique/CATALOGUE.md#f-top-002), [F-TOP-003](../agnostique/CATALOGUE.md#f-top-003), [F-TOP-004](../agnostique/CATALOGUE.md#f-top-004), [F-TOP-006](../agnostique/CATALOGUE.md#f-top-006), [F-TOP-007](../agnostique/CATALOGUE.md#f-top-007), [F-TOP-008](../agnostique/CATALOGUE.md#f-top-008), [F-TOP-010](../agnostique/CATALOGUE.md#f-top-010).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-model`, `crates/sns-ui-egui/src/topology`.
- Fichiers réalisés : `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/editor.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/snapshots/j1-editor.png`.
- Approche : Commandes métier validées côté cœur; canevas natif sns-ui-egui avec Painter egui, transformations monde/écran, sélection stable et historique réversible; tests egui_kittest.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-applications-j2"></a>

## RS-APPLICATIONS-J2 — Bureau et services génériques — J2

- Jalon : `J2`; état : `verified`; taille indicative : `M`.
- Exigences : [F-OS-001](../agnostique/CATALOGUE.md#f-os-001), [F-OS-002](../agnostique/CATALOGUE.md#f-os-002), [F-OS-003](../agnostique/CATALOGUE.md#f-os-003), [F-OS-005](../agnostique/CATALOGUE.md#f-os-005), [F-OS-006](../agnostique/CATALOGUE.md#f-os-006).
- Prérequis : RS-DEVICES-J1, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps`, `crates/sns-ui-egui/src/hosts`.
- Fichiers réalisés : `contracts/application-command.schema.json`, `contracts/application-event.schema.json`, `contracts/project-hosts.schema.json`, `crates/sns-apps/Cargo.toml`, `crates/sns-apps/src/lib.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/applications.rs`, `crates/sns-model/src/apps.rs`, `crates/sns-model/src/lib.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/lan.rs`, `fixtures/valid/hosts.json`.
- Approche : Registre AppId et événements applicatifs; séparer installation, service actif et vue egui; panneaux par hôte dans sns-ui-egui avec API de socket simulée.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-filesystem-j2"></a>

## RS-FILESYSTEM-J2 — Fichiers virtuels et outils — J2

- Jalon : `J2`; état : `verified`; taille indicative : `M`.
- Exigences : [F-FS-001](../agnostique/CATALOGUE.md#f-fs-001), [F-FS-002](../agnostique/CATALOGUE.md#f-fs-002), [F-FS-003](../agnostique/CATALOGUE.md#f-fs-003), [F-FS-004](../agnostique/CATALOGUE.md#f-fs-004), [F-FS-005](../agnostique/CATALOGUE.md#f-fs-005).
- Prérequis : RS-DEVICES-J1.
- Cibles : `crates/sns-vfs`, `crates/sns-ui-egui/src/files`.
- Fichiers réalisés : `contracts/project-hosts.schema.json`, `crates/sns-engine/tests/applications.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-vfs/src/lib.rs`, `crates/sns-vfs/tests/files.rs`, `docs/verification/j2-native-project.json`, `docs/verification/j2-native-smoke.json`.
- Approche : Arbre isolé à blobs immuables et opérations transactionnelles; codec de dossiers versionné; outils egui et sélection native rfd explicite, lecture bornée dans l’adaptateur GUI; dialogues actuellement modaux.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-localization-j2"></a>

## RS-LOCALIZATION-J2 — Localisation et préférence de langue — J2

- Jalon : `J2`; état : `verified`; taille indicative : `M`.
- Exigences : [F-CFG-001](../agnostique/CATALOGUE.md#f-cfg-001).
- Prérequis : RS-PROJECTS-J1.
- Cibles : `crates/sns-i18n`, `crates/sns-settings`, `crates/sns-ui-egui`.
- Fichiers réalisés : `.github/workflows/ci.yml`, `apps/sns-desktop/src/main.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-i18n/src/lib.rs`, `crates/sns-settings/src/lib.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/tests/lan.rs`, `scripts/check_i18n.py`.
- Approche : Catalogue de clés commun aux six langues, diagnostics structurés, contrôle de complétude et préférence locale séparée du projet.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j2"></a>

## RS-OBSERVATION-J2 — Traces et visualisation — J2

- Jalon : `J2`; état : `verified`; taille indicative : `L`.
- Exigences : [F-OBS-001](../agnostique/CATALOGUE.md#f-obs-001), [F-OBS-002](../agnostique/CATALOGUE.md#f-obs-002), [F-OBS-003](../agnostique/CATALOGUE.md#f-obs-003), [F-OBS-004](../agnostique/CATALOGUE.md#f-obs-004), [F-OBS-005](../agnostique/CATALOGUE.md#f-obs-005), [F-OBS-010](../agnostique/CATALOGUE.md#f-obs-010).
- Prérequis : RS-IPV4-J1, RS-LINK-J1.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : `crates/sns-model/src/transport.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-protocols/src/transport.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/lan.rs`.
- Approche : Faits canoniques immuables; filtres par interface/couche, inspection historique et activité de liens dans sns-ui-egui; tables virtualisées egui_extras, suivi/purge de présentation et exports texte UTF-8/JSONL.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j2"></a>

## RS-TERMINAL-J2 — Terminal pédagogique — J2

- Jalon : `J2`; état : `verified`; taille indicative : `M`.
- Exigences : [F-CLI-001](../agnostique/CATALOGUE.md#f-cli-001), [F-CLI-002](../agnostique/CATALOGUE.md#f-cli-002), [F-CLI-003](../agnostique/CATALOGUE.md#f-cli-003), [F-CLI-004](../agnostique/CATALOGUE.md#f-cli-004), [F-CLI-005](../agnostique/CATALOGUE.md#f-cli-005), [F-CLI-006](../agnostique/CATALOGUE.md#f-cli-006), [F-CLI-007](../agnostique/CATALOGUE.md#f-cli-007), [F-CLI-008](../agnostique/CATALOGUE.md#f-cli-008), [F-CLI-009](../agnostique/CATALOGUE.md#f-cli-009), [F-CLI-011](../agnostique/CATALOGUE.md#f-cli-011), [F-CLI-012](../agnostique/CATALOGUE.md#f-cli-012), [F-CLI-014](../agnostique/CATALOGUE.md#f-cli-014), [F-CLI-016](../agnostique/CATALOGUE.md#f-cli-016), [F-CLI-018](../agnostique/CATALOGUE.md#f-cli-018), [F-CLI-020](../agnostique/CATALOGUE.md#f-cli-020), [F-CLI-021](../agnostique/CATALOGUE.md#f-cli-021).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : `crates/sns-apps/src/terminal.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/tests/applications.rs`, `crates/sns-model/src/apps.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/tests/lan.rs`.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-transport-j2"></a>

## RS-TRANSPORT-J2 — TCP et UDP simulés — J2

- Jalon : `J2`; état : `verified`; taille indicative : `XL`.
- Exigences : [F-TCP-001](../agnostique/CATALOGUE.md#f-tcp-001), [F-TCP-002](../agnostique/CATALOGUE.md#f-tcp-002), [F-TCP-003](../agnostique/CATALOGUE.md#f-tcp-003), [F-TCP-004](../agnostique/CATALOGUE.md#f-tcp-004), [F-TCP-005](../agnostique/CATALOGUE.md#f-tcp-005), [F-UDP-001](../agnostique/CATALOGUE.md#f-udp-001), [F-UDP-002](../agnostique/CATALOGUE.md#f-udp-002).
- Prérequis : RS-IPV4-J1.
- Cibles : `crates/sns-protocols/src/transport`.
- Fichiers réalisés : `contracts/transport-command.schema.json`, `contracts/transport-event.schema.json`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/transport.rs`, `crates/sns-model/src/transport.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-protocols/src/transport.rs`.
- Approche : Machines à états TCP et datagrammes UDP propres au simulateur; sockets simulées, allocation de ports, retransmissions virtuelles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j3"></a>

## RS-DEVICES-J3 — Équipements et configuration — J3

- Jalon : `J3`; état : `verified`; taille indicative : `M`.
- Exigences : [F-EQU-003](../agnostique/CATALOGUE.md#f-equ-003).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : `contracts/project-routing.schema.json`, `crates/sns-engine/tests/routing.rs`, `crates/sns-model/src/lib.rs`, `crates/sns-model/src/routing.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-ui-egui/src/workbench/routing_ui.rs`.
- Approche : DTO de routage version 4; interfaces explicites, validation et commandes réversibles dans sns-topology; configuration séparée des tables apprises.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-ipv4-j3"></a>

## RS-IPV4-J3 — IPv4 et ICMP — J3

- Jalon : `J3`; état : `verified`; taille indicative : `L`.
- Exigences : [F-IP4-003](../agnostique/CATALOGUE.md#f-ip4-003), [F-IP4-005](../agnostique/CATALOGUE.md#f-ip4-005).
- Prérequis : RS-LINK-J1.
- Cibles : `crates/sns-protocols/src/ipv4`.
- Fichiers réalisés : `contracts/routing-event.schema.json`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/routing.rs`, `crates/sns-protocols/src/forwarding.rs`, `crates/sns-protocols/src/lib.rs`.
- Approche : Adresses numériques typées, masques validés, next-hop explicite, erreurs structurées et corrélation des sondes.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-link-j3"></a>

## RS-LINK-J3 — Ethernet, switch et ARP — J3

- Jalon : `J3`; état : `verified`; taille indicative : `L`.
- Exigences : [F-L2-006](../agnostique/CATALOGUE.md#f-l2-006).
- Prérequis : RS-DEVICES-J1.
- Cibles : `crates/sns-protocols/src/link`.
- Fichiers réalisés : `crates/sns-apps/src/terminal.rs`, `crates/sns-engine/tests/routing.rs`, `crates/sns-protocols/src/forwarding.rs`.
- Approche : Machines à états événementielles; clones de données de trace immuables; timers virtuels pour les caches.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j3"></a>

## RS-OBSERVATION-J3 — Traces et visualisation — J3

- Jalon : `J3`; état : `verified`; taille indicative : `L`.
- Exigences : [F-OBS-009](../agnostique/CATALOGUE.md#f-obs-009).
- Prérequis : RS-IPV4-J1, RS-LINK-J1.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : `contracts/routing-event.schema.json`, `crates/sns-protocols/src/forwarding.rs`, `crates/sns-protocols/src/rip.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/routing_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-routing-j3"></a>

## RS-ROUTING-J3 — Routage statique et dynamique — J3

- Jalon : `J3`; état : `verified`; taille indicative : `L`.
- Exigences : [F-ROUT-001](../agnostique/CATALOGUE.md#f-rout-001), [F-ROUT-002](../agnostique/CATALOGUE.md#f-rout-002), [F-ROUT-003](../agnostique/CATALOGUE.md#f-rout-003), [F-ROUT-004](../agnostique/CATALOGUE.md#f-rout-004), [F-ROUT-005](../agnostique/CATALOGUE.md#f-rout-005).
- Prérequis : RS-DEVICES-J3, RS-IPV4-J3.
- Cibles : `crates/sns-protocols/src/routing`.
- Fichiers réalisés : `contracts/project-routing.schema.json`, `contracts/routing-command.schema.json`, `contracts/routing-event.schema.json`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/routing.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/rip.rs`, `crates/sns-protocols/src/rip_tests.rs`, `crates/sns-protocols/src/routing.rs`, `crates/sns-protocols/src/transport.rs`, `crates/sns-ui-egui/src/workbench/routing_ui.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `fixtures/valid/rip-chain.json`, `fixtures/valid/rip.json`, `fixtures/valid/routed.json`.
- Approche : Tables calculées/statiques/RIP et sélection par préfixe/origine/métrique avec départage stable; annonces structurées sur UDP simulé, poison reverse et timers virtuels 30/75/30 secondes; inspection des choix.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j3"></a>

## RS-TERMINAL-J3 — Terminal pédagogique — J3

- Jalon : `J3`; état : `verified`; taille indicative : `M`.
- Exigences : [F-CLI-010](../agnostique/CATALOGUE.md#f-cli-010), [F-CLI-015](../agnostique/CATALOGUE.md#f-cli-015), [F-CLI-017](../agnostique/CATALOGUE.md#f-cli-017).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : `crates/sns-apps/src/lib.rs`, `crates/sns-apps/src/terminal.rs`, `crates/sns-engine/tests/routing.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/tests/lan.rs`.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-topology-j3"></a>

## RS-TOPOLOGY-J3 — Éditeur de réseau — J3

- Jalon : `J3`; état : `verified`; taille indicative : `L`.
- Exigences : [F-TOP-005](../agnostique/CATALOGUE.md#f-top-005), [F-TOP-011](../agnostique/CATALOGUE.md#f-top-011).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-model`, `crates/sns-ui-egui/src/topology`.
- Fichiers réalisés : `.github/workflows/ci.yml`, `assets/network/provenance.json`, `crates/sns-model/src/lib.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/editor.rs`, `crates/sns-ui-egui/src/network_art.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/tests/lan.rs`, `fixtures/valid/classroom.json`, `scripts/check_manifest.py`, `scripts/check_network_assets.py`.
- Approche : Redimensionnement et échange des connexions validés dans sns-topology; SVG fournis embarqués, styles icônes/schéma et rôles persistants; labels accessibles et tests GUI dans les six langues.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-applications-j4"></a>

## RS-APPLICATIONS-J4 — Bureau et services génériques — J4

- Jalon : `J4`; état : `verified`; taille indicative : `M`.
- Exigences : [F-OS-004](../agnostique/CATALOGUE.md#f-os-004).
- Prérequis : RS-DEVICES-J1, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps`, `crates/sns-ui-egui/src/hosts`.
- Fichiers réalisés : `contracts/project-services.schema.json`, `contracts/services-command.schema.json`, `contracts/services-event.schema.json`, `crates/sns-apps/src/lib.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/services.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/dhcp.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `docs/agnostique/SERVICES_J4.md`, `docs/rust/J4.md`, `fixtures/valid/services.json`.
- Approche : Registre AppId et événements applicatifs; séparer installation, service actif et vue egui; panneaux par hôte dans sns-ui-egui avec API de socket simulée.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-dhcp-j4"></a>

## RS-DHCP-J4 — DHCP et link-local — J4

- Jalon : `J4`; état : `verified`; taille indicative : `L`.
- Exigences : [F-DHCP-001](../agnostique/CATALOGUE.md#f-dhcp-001), [F-DHCP-002](../agnostique/CATALOGUE.md#f-dhcp-002), [F-DHCP-003](../agnostique/CATALOGUE.md#f-dhcp-003), [F-DHCP-004](../agnostique/CATALOGUE.md#f-dhcp-004), [F-DHCP-005](../agnostique/CATALOGUE.md#f-dhcp-005), [F-DHCP-006](../agnostique/CATALOGUE.md#f-dhcp-006).
- Prérequis : RS-APPLICATIONS-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/dhcp`.
- Fichiers réalisés : `contracts/project-services.schema.json`, `contracts/services-command.schema.json`, `contracts/services-event.schema.json`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/services.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/services.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/dhcp.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/desktop/services_ui.rs`, `docs/agnostique/SERVICES_J4.md`, `docs/rust/J4.md`, `fixtures/valid/services.json`.
- Approche : États explicites client/serveur; tables d’offres/réservations/attributions et horloge virtuelle; conflits résolus via ARP simulé.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-dns-j4"></a>

## RS-DNS-J4 — DNS pédagogique — J4

- Jalon : `J4`; état : `verified`; taille indicative : `L`.
- Exigences : [F-DNS-001](../agnostique/CATALOGUE.md#f-dns-001), [F-DNS-002](../agnostique/CATALOGUE.md#f-dns-002), [F-DNS-003](../agnostique/CATALOGUE.md#f-dns-003), [F-DNS-004](../agnostique/CATALOGUE.md#f-dns-004), [F-DNS-005](../agnostique/CATALOGUE.md#f-dns-005), [F-DNS-006](../agnostique/CATALOGUE.md#f-dns-006).
- Prérequis : RS-APPLICATIONS-J2, RS-ENGINE-J0, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/dns`.
- Fichiers réalisés : `contracts/project-services.schema.json`, `contracts/services-command.schema.json`, `contracts/services-event.schema.json`, `crates/sns-apps/src/dns.rs`, `crates/sns-apps/src/lib.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/tests/services.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/services.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench/desktop/services_ui.rs`, `docs/agnostique/SERVICES_J4.md`, `docs/rust/J4.md`, `fixtures/valid/services.json`.
- Approche : Enregistrements A/MX/NS typés; caches par nœud et deadlines virtuelles; délégations bornées pour éviter les cycles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j4"></a>

## RS-TERMINAL-J4 — Terminal pédagogique — J4

- Jalon : `J4`; état : `verified`; taille indicative : `M`.
- Exigences : [F-CLI-013](../agnostique/CATALOGUE.md#f-cli-013).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : `contracts/project-services.schema.json`, `contracts/services-command.schema.json`, `contracts/services-event.schema.json`, `crates/sns-apps/src/dns.rs`, `crates/sns-apps/src/lib.rs`, `crates/sns-apps/src/terminal.rs`, `crates/sns-engine/tests/services.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `docs/agnostique/SERVICES_J4.md`, `docs/rust/J4.md`, `fixtures/valid/services.json`.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-web-j4"></a>

## RS-WEB-J4 — Serveur et navigateur simulés — J4

- Jalon : `J4`; état : `verified`; taille indicative : `L`.
- Exigences : [F-WEB-001](../agnostique/CATALOGUE.md#f-web-001), [F-WEB-002](../agnostique/CATALOGUE.md#f-web-002), [F-WEB-003](../agnostique/CATALOGUE.md#f-web-003), [F-WEB-004](../agnostique/CATALOGUE.md#f-web-004), [F-WEB-005](../agnostique/CATALOGUE.md#f-web-005), [F-WEB-006](../agnostique/CATALOGUE.md#f-web-006).
- Prérequis : RS-DNS-J4, RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/http`, `crates/sns-ui-egui/src/browser`, `crates/sns-html`.
- Fichiers réalisés : `contracts/project-services.schema.json`, `contracts/services-command.schema.json`, `contracts/services-event.schema.json`, `crates/sns-apps/src/web.rs`, `crates/sns-engine/tests/services.rs`, `crates/sns-html/src/lib.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/services.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/src/workbench/desktop/services_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/service_images.rs`, `docs/agnostique/SERVICES_J4.md`, `docs/rust/J4.md`, `fixtures/valid/services.json`.
- Approche : HTTP pédagogique sur TCP simulé; document HTML borné sans I/O dans sns-html et vue egui pour texte, liens, images et formulaires; toutes les ressources et actions GET/POST passent par le moteur.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j5"></a>

## RS-DEVICES-J5 — Équipements et configuration — J5

- Jalon : `J5`; état : `verified`; taille indicative : `M`.
- Exigences : [F-EQU-004](../agnostique/CATALOGUE.md#f-equ-004).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : `contracts/private-event.schema.json`, `contracts/project-private.schema.json`, `contracts/security-command.schema.json`, `crates/sns-engine/tests/private.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/lib.rs`, `crates/sns-model/src/security.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/editor.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/RESEAUX_PRIVES_J5.md`, `docs/rust/J5.md`, `fixtures/valid/private.json`.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-firewall-j5"></a>

## RS-FIREWALL-J5 — Filtrage pédagogique — J5

- Jalon : `J5`; état : `verified`; taille indicative : `L`.
- Exigences : [F-FW-001](../agnostique/CATALOGUE.md#f-fw-001), [F-FW-002](../agnostique/CATALOGUE.md#f-fw-002), [F-FW-003](../agnostique/CATALOGUE.md#f-fw-003), [F-FW-004](../agnostique/CATALOGUE.md#f-fw-004), [F-FW-005](../agnostique/CATALOGUE.md#f-fw-005), [F-FW-006](../agnostique/CATALOGUE.md#f-fw-006), [F-FW-007](../agnostique/CATALOGUE.md#f-fw-007).
- Prérequis : RS-ROUTING-J3, RS-TRANSPORT-J2, RS-WEB-J5.
- Cibles : `crates/sns-protocols/src/firewall`.
- Fichiers réalisés : `contracts/private-event.schema.json`, `contracts/project-private.schema.json`, `contracts/security-command.schema.json`, `crates/sns-apps/src/admin.rs`, `crates/sns-apps/src/lib.rs`, `crates/sns-engine/tests/private.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/security.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/firewall.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-ui-egui/src/workbench/security_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/RESEAUX_PRIVES_J5.md`, `docs/rust/J5.md`, `fixtures/valid/private.json`.
- Approche : Règles typées ordonnées, politique par défaut et modes historiques explicites; un résultat de décision détaillé par paquet.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-nat-j5"></a>

## RS-NAT-J5 — NAT et routeur domestique — J5

- Jalon : `J5`; état : `verified`; taille indicative : `XL`.
- Exigences : [F-NAT-001](../agnostique/CATALOGUE.md#f-nat-001), [F-NAT-002](../agnostique/CATALOGUE.md#f-nat-002), [F-NAT-003](../agnostique/CATALOGUE.md#f-nat-003), [F-NAT-004](../agnostique/CATALOGUE.md#f-nat-004), [F-NAT-005](../agnostique/CATALOGUE.md#f-nat-005), [F-NAT-006](../agnostique/CATALOGUE.md#f-nat-006).
- Prérequis : RS-DEVICES-J5, RS-DHCP-J4, RS-ROUTING-J3, RS-TRANSPORT-J2.
- Cibles : `crates/sns-protocols/src/nat`.
- Fichiers réalisés : `contracts/private-event.schema.json`, `contracts/project-private.schema.json`, `contracts/security-command.schema.json`, `crates/sns-engine/tests/private.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/security.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/dhcp.rs`, `crates/sns-protocols/src/forwarding.rs`, `crates/sns-protocols/src/nat.rs`, `crates/sns-protocols/src/transport.rs`, `crates/sns-ui-egui/src/workbench/security_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/RESEAUX_PRIVES_J5.md`, `docs/rust/J5.md`, `fixtures/valid/private.json`.
- Approche : Clés de mapping explicites, IPv4/ports/ICMP; politique d’ordre NAT/filtrage figée par fixtures; tables consultables.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j5"></a>

## RS-OBSERVATION-J5 — Traces et visualisation — J5

- Jalon : `J5`; état : `verified`; taille indicative : `L`.
- Exigences : [F-OBS-007](../agnostique/CATALOGUE.md#f-obs-007).
- Prérequis : RS-LINK-J1, RS-NAT-J5.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : `contracts/private-event.schema.json`, `contracts/project-private.schema.json`, `contracts/security-command.schema.json`, `crates/sns-engine/tests/private.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/firewall.rs`, `crates/sns-protocols/src/nat.rs`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench/security_ui.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/RESEAUX_PRIVES_J5.md`, `docs/rust/J5.md`, `fixtures/valid/private.json`.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-routing-j5"></a>

## RS-ROUTING-J5 — Routage statique et dynamique — J5

- Jalon : `J5`; état : `verified`; taille indicative : `L`.
- Exigences : [F-ROUT-006](../agnostique/CATALOGUE.md#f-rout-006).
- Prérequis : RS-DEVICES-J3, RS-IPV4-J3.
- Cibles : `crates/sns-protocols/src/routing`.
- Fichiers réalisés : `contracts/private-event.schema.json`, `contracts/project-private.schema.json`, `contracts/security-command.schema.json`, `crates/sns-apps/src/admin.rs`, `crates/sns-engine/tests/private.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/workbench/routing_ui.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/RESEAUX_PRIVES_J5.md`, `docs/rust/J5.md`, `fixtures/valid/private.json`.
- Approche : Tables distinctes calculées/statiques/RIP; choix explicables; ordre stable de départage et délais virtuels.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-web-j5"></a>

## RS-WEB-J5 — Serveur et navigateur simulés — J5

- Jalon : `J5`; état : `verified`; taille indicative : `L`.
- Exigences : [F-WEB-007](../agnostique/CATALOGUE.md#f-web-007).
- Prérequis : RS-DNS-J4, RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/http`, `crates/sns-ui-egui/src/browser`, `crates/sns-html`.
- Fichiers réalisés : `contracts/private-event.schema.json`, `contracts/project-private.schema.json`, `contracts/security-command.schema.json`, `crates/sns-apps/src/admin.rs`, `crates/sns-apps/src/web.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/private.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/services.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/workbench/desktop/services_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/RESEAUX_PRIVES_J5.md`, `docs/rust/J5.md`, `fixtures/valid/private.json`.
- Approche : HTTP pédagogique sur TCP simulé; document HTML borné sans I/O dans sns-html et vue egui pour texte, liens, images et formulaires; toutes les ressources et actions GET/POST passent par le moteur.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j6"></a>

## RS-DEVICES-J6 — Équipements et configuration — J6

- Jalon : `J6`; état : `verified`; taille indicative : `M`.
- Exigences : [F-EQU-005](../agnostique/CATALOGUE.md#f-equ-005), [F-EQU-007](../agnostique/CATALOGUE.md#f-equ-007).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : `assets/network/diagram/access_point.svg`, `assets/network/diagram/modem.svg`, `assets/network/icon/access_point.svg`, `assets/network/icon/modem.svg`, `assets/network/provenance.json`, `contracts/advanced-command.schema.json`, `contracts/advanced-event.schema.json`, `contracts/project-advanced.schema.json`, `crates/sns-engine/tests/wireless.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/lib.rs`, `crates/sns-model/src/modem.rs`, `crates/sns-model/src/wireless.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-protocols/src/routing.rs`, `crates/sns-protocols/src/transport.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-ui-egui/src/network_art.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/APPLICATIONS_J6.md`, `docs/rust/J6.md`, `docs/verification/J6-REVALIDATION.md`, `docs/verification/J6.md`, `fixtures/valid/modem.json`, `fixtures/valid/wireless.json`, `scripts/check_network_assets.py`.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-extensions-j6"></a>

## RS-EXTENSIONS-J6 — Extensions et modem — J6

- Jalon : `J6`; état : `verified`; taille indicative : `XL`.
- Exigences : [F-EXT-005](../agnostique/CATALOGUE.md#f-ext-005).
- Prérequis : RS-DEVICES-J6, RS-LINK-J1.
- Cibles : `crates/sns-bridge`, `crates/sns-extension-host`, `crates/sns-ui-egui/src/extensions`.
- Fichiers réalisés : `Cargo.lock`, `Cargo.toml`, `contracts/advanced-command.schema.json`, `contracts/advanced-event.schema.json`, `contracts/project-advanced.schema.json`, `crates/sns-bridge`, `crates/sns-engine/src/lib.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/modem.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/bridge.rs`, `crates/sns-ui-egui/Cargo.toml`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/APPLICATIONS_J6.md`, `docs/rust/J6.md`, `docs/verification/J6-REVALIDATION.md`, `docs/verification/J6.md`.
- Approche : J6 : adaptateur réel séparé, fil dédié et sockets non bloquantes, protocole borné/versionné, entrées enregistrées; J8 : contrat d’extensions à prototyper séparément.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-mail-j6"></a>

## RS-MAIL-J6 — SMTP, POP3 et client mail — J6

- Jalon : `J6`; état : `verified`; taille indicative : `L`.
- Exigences : [F-MAIL-001](../agnostique/CATALOGUE.md#f-mail-001), [F-MAIL-002](../agnostique/CATALOGUE.md#f-mail-002), [F-MAIL-003](../agnostique/CATALOGUE.md#f-mail-003), [F-MAIL-004](../agnostique/CATALOGUE.md#f-mail-004), [F-MAIL-005](../agnostique/CATALOGUE.md#f-mail-005), [F-MAIL-006](../agnostique/CATALOGUE.md#f-mail-006), [F-MAIL-007](../agnostique/CATALOGUE.md#f-mail-007), [F-MAIL-008](../agnostique/CATALOGUE.md#f-mail-008).
- Prérequis : RS-APPLICATIONS-J2, RS-DNS-J4, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/mail`, `crates/sns-ui-egui/src/mail`.
- Fichiers réalisés : `contracts/advanced-command.schema.json`, `contracts/advanced-event.schema.json`, `contracts/project-advanced.schema.json`, `crates/sns-apps/src/dns.rs`, `crates/sns-apps/src/lib.rs`, `crates/sns-apps/src/mail.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/tests/mail.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/mail.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/workbench/desktop/mail_ui.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/APPLICATIONS_J6.md`, `docs/rust/J6.md`, `docs/verification/J6-REVALIDATION.md`, `docs/verification/J6.md`, `fixtures/valid/mail.json`.
- Approche : Sessions protocolaires événementielles; boîtes virtuelles, comptes pédagogiques et relais MX; aucune connexion mail réelle.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-p2p-j6"></a>

## RS-P2P-J6 — Gnutella pédagogique — J6

- Jalon : `J6`; état : `verified`; taille indicative : `L`.
- Exigences : [F-P2P-001](../agnostique/CATALOGUE.md#f-p2p-001), [F-P2P-002](../agnostique/CATALOGUE.md#f-p2p-002), [F-P2P-003](../agnostique/CATALOGUE.md#f-p2p-003), [F-P2P-004](../agnostique/CATALOGUE.md#f-p2p-004), [F-P2P-005](../agnostique/CATALOGUE.md#f-p2p-005).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/p2p`, `crates/sns-ui-egui/src/p2p`.
- Fichiers réalisés : `contracts/advanced-command.schema.json`, `contracts/advanced-event.schema.json`, `contracts/project-advanced.schema.json`, `crates/sns-apps/src/p2p.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/tests/p2p.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/p2p.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/src/workbench/desktop/p2p_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `docs/agnostique/APPLICATIONS_J6.md`, `docs/rust/J6.md`, `docs/verification/J6-REVALIDATION.md`, `docs/verification/J6.md`, `fixtures/valid/p2p.json`.
- Approche : Messages typés Ping/Pong/Query/QueryHit, identifiants et TTL; transferts dans le VFS avec quotas et annulation.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j7"></a>

## RS-DEVICES-J7 — Équipements et configuration — J7

- Jalon : `J7`; état : `verified`; taille indicative : `M`.
- Exigences : [F-EQU-008](../agnostique/CATALOGUE.md#f-equ-008).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : `assets/network/diagram/cloud.svg`, `assets/network/icon/cloud.svg`, `assets/network/provenance.json`, `contracts/README.md`, `contracts/checkpoint.schema.json`, `contracts/link-conditions-command.schema.json`, `contracts/link-conditions-event.schema.json`, `contracts/network-scenario.schema.json`, `contracts/presentation-command.schema.json`, `contracts/presentation-event.schema.json`, `contracts/project-presentation.schema.json`, `contracts/recording.schema.json`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/tests/presentation.rs`, `crates/sns-filius-import/src/convert.rs`, `crates/sns-filius-import/tests/import.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/lib.rs`, `crates/sns-model/src/presentation.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/editor.rs`, `crates/sns-ui-egui/src/network_art.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/presentation_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-cloud-icons-pt.png`, `docs/agnostique/PRESENTATION_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/verification/J7-PRESENTATION.md`, `fixtures/valid/cloud.json`, `scripts/check_network_assets.py`.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-engine-j7"></a>

## RS-ENGINE-J7 — Moteur et cycle de simulation — J7

- Jalon : `J7`; état : `verified`; taille indicative : `L`.
- Exigences : [F-SIM-007](../agnostique/CATALOGUE.md#f-sim-007), [F-SIM-008](../agnostique/CATALOGUE.md#f-sim-008).
- Prérequis : aucun.
- Cibles : `crates/sns-engine`.
- Fichiers réalisés : `Cargo.toml`, `apps/sns-desktop/tests/startup.rs`, `contracts/README.md`, `contracts/checkpoint.schema.json`, `contracts/link-conditions-command.schema.json`, `contracts/link-conditions-event.schema.json`, `contracts/network-scenario.schema.json`, `contracts/presentation-command.schema.json`, `contracts/presentation-event.schema.json`, `contracts/project-documentation.schema.json`, `contracts/project-presentation.schema.json`, `contracts/recording.schema.json`, `contracts/startup-report.schema.json`, `crates/sns-apps/src/dns.rs`, `crates/sns-apps/src/lib.rs`, `crates/sns-apps/src/mail.rs`, `crates/sns-apps/src/p2p.rs`, `crates/sns-apps/src/web.rs`, `crates/sns-bridge/src/driver.rs`, `crates/sns-bridge/tests/bridge.rs`, `crates/sns-cli/src/main.rs`, `crates/sns-cli/src/runtime_cli.rs`, `crates/sns-cli/tests/runner.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/src/checkpoint.rs`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/src/recording.rs`, `crates/sns-engine/src/state.rs`, `crates/sns-engine/src/state_value.rs`, `crates/sns-engine/tests/checkpoint.rs`, `crates/sns-engine/tests/link_conditions.rs`, `crates/sns-engine/tests/timing.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/lib.rs`, `crates/sns-model/src/link_conditions.rs`, `crates/sns-model/src/timing.rs`, `crates/sns-project/src/lib.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/bridge.rs`, `crates/sns-protocols/src/dhcp.rs`, `crates/sns-protocols/src/firewall.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-protocols/src/link_conditions.rs`, `crates/sns-protocols/src/nat.rs`, `crates/sns-protocols/src/routing.rs`, `crates/sns-protocols/src/transport.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/import_ui.rs`, `crates/sns-ui-egui/src/workbench/link_conditions_ui.rs`, `crates/sns-ui-egui/src/workbench/runtime_ui.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/configuration.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/snapshots/j7-links-blocked-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-blocked-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-blocked-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-blocked-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-blocked-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-blocked-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-repaired-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-repaired-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-repaired-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-repaired-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-repaired-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-links-repaired-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-restored-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-restored-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-restored-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-restored-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-restored-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-runtime-restored-pt.png`, `crates/sns-vfs/src/lib.rs`, `docs/agnostique/ATTENTES_J7.md`, `docs/agnostique/PANNES_J7.md`, `docs/agnostique/PROJETS_J7.md`, `docs/agnostique/REPRISE_J7.md`, `docs/rust/ATTENTES_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/verification/J7-ATTENTES.md`, `docs/verification/J7-PANNES.md`, `docs/verification/J7-REPRISE-EN-COURS.md`, `docs/verification/j7-timing-checkpoint.json`, `docs/verification/j7-timing-ci.log`, `docs/verification/j7-timing-cli-checkpoint.json`, `docs/verification/j7-timing-cli-default.jsonl`, `docs/verification/j7-timing-cli-full.jsonl`, `docs/verification/j7-timing-cli-recording.json`, `docs/verification/j7-timing-cli-replay.jsonl`, `docs/verification/j7-timing-cli-resume.jsonl`, `docs/verification/j7-timing-code-manifest.json`, `docs/verification/j7-timing-docs.log`, `docs/verification/j7-timing-environment.json`, `docs/verification/j7-timing-gpu.log`, `docs/verification/j7-timing-help-fr.txt`, `docs/verification/j7-timing-recording.json`, `docs/verification/j7-timing-source-delta.json`, `docs/verification/j7-timing-startup-de-icons.json`, `docs/verification/j7-timing-startup-fr-diagram.json`, `docs/verification/j7-timing-targeted.log`, `fixtures/valid/checkpoint-v1-unconfigured.json`, `fixtures/valid/link-conditions-scenario.json`, `fixtures/valid/network-scenario.json`, `fixtures/valid/response-wait-scenario.json`.
- Approche : File BinaryHeap avec Reverse; temps entier, ordre total, moteur synchrone à propriétaire unique; aucune minuterie système dans les protocoles. Politique d’attente J7 incluse dans la provenance du journal et de la continuation; le facteur standard conserve les anciennes versions.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j7"></a>

## RS-OBSERVATION-J7 — Traces et visualisation — J7

- Jalon : `J7`; état : `verified`; taille indicative : `L`.
- Exigences : [F-OBS-006](../agnostique/CATALOGUE.md#f-obs-006), [F-OBS-008](../agnostique/CATALOGUE.md#f-obs-008).
- Prérequis : RS-IPV4-J1, RS-LINK-J1.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : `Cargo.lock`, `Cargo.toml`, `crates/sns-i18n/locales/messages.json`, `crates/sns-trace/Cargo.toml`, `crates/sns-trace/src/journey.rs`, `crates/sns-trace/src/lib.rs`, `crates/sns-trace/tests/observation.rs`, `crates/sns-ui-egui/Cargo.toml`, `crates/sns-ui-egui/src/traffic.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/trace_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/snapshots/j7-observation-global-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-global-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-global-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-global-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-global-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-global-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-local-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-local-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-local-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-local-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-local-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-observation-local-pt.png`, `docs/agnostique/OBSERVATION_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/verification/J7-OBSERVATION.md`, `scripts/check_i18n.py`, `scripts/check_layers.py`, `scripts/check_trace_exports.py`.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-platform-j7"></a>

## RS-PLATFORM-J7 — Intégration desktop et préférences — J7

- Jalon : `J7`; état : `in_progress`; taille indicative : `L`.
- Exigences : [F-CFG-002](../agnostique/CATALOGUE.md#f-cfg-002), [F-CFG-003](../agnostique/CATALOGUE.md#f-cfg-003), [F-CFG-004](../agnostique/CATALOGUE.md#f-cfg-004), [F-CFG-005](../agnostique/CATALOGUE.md#f-cfg-005), [F-CFG-006](../agnostique/CATALOGUE.md#f-cfg-006), [F-CFG-007](../agnostique/CATALOGUE.md#f-cfg-007), [F-CFG-008](../agnostique/CATALOGUE.md#f-cfg-008).
- Prérequis : RS-PROJECTS-J1.
- Cibles : `apps/sns-desktop`, `crates/sns-settings`.
- Fichiers réalisés : `.github/workflows/packages.yml`, `Cargo.lock`, `apps/sns-desktop/Cargo.toml`, `apps/sns-desktop/src/launch.rs`, `apps/sns-desktop/src/main.rs`, `apps/sns-desktop/tests/startup.rs`, `contracts/README.md`, `contracts/checkpoint.schema.json`, `contracts/configuration.schema.json`, `contracts/diagnostic.schema.json`, `contracts/link-conditions-command.schema.json`, `contracts/link-conditions-event.schema.json`, `contracts/network-scenario.schema.json`, `contracts/preferences.schema.json`, `contracts/presentation-command.schema.json`, `contracts/presentation-event.schema.json`, `contracts/project-documentation.schema.json`, `contracts/project-presentation.schema.json`, `contracts/recording.schema.json`, `contracts/startup-report.schema.json`, `crates/sns-apps/src/lib.rs`, `crates/sns-bridge/tests/bridge.rs`, `crates/sns-cli/src/main.rs`, `crates/sns-cli/src/runtime_cli.rs`, `crates/sns-cli/tests/runner.rs`, `crates/sns-engine/src/applications.rs`, `crates/sns-engine/src/checkpoint.rs`, `crates/sns-engine/src/lib.rs`, `crates/sns-engine/src/recording.rs`, `crates/sns-engine/src/state.rs`, `crates/sns-engine/tests/lan.rs`, `crates/sns-engine/tests/routing.rs`, `crates/sns-engine/tests/timing.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/annotations.rs`, `crates/sns-model/src/lib.rs`, `crates/sns-model/src/timing.rs`, `crates/sns-project/src/destination.rs`, `crates/sns-project/src/lib.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-protocols/src/dhcp.rs`, `crates/sns-protocols/src/lib.rs`, `crates/sns-protocols/src/transport.rs`, `crates/sns-settings/Cargo.toml`, `crates/sns-settings/src/configuration.rs`, `crates/sns-settings/src/diagnostics.rs`, `crates/sns-settings/src/display.rs`, `crates/sns-settings/src/help.rs`, `crates/sns-settings/src/lib.rs`, `crates/sns-settings/tests/configuration.rs`, `crates/sns-settings/tests/display.rs`, `crates/sns-ui-egui/src/workbench/annotations.rs`, `crates/sns-ui-egui/src/workbench/canvas.rs`, `crates/sns-ui-egui/src/workbench/chrome.rs`, `crates/sns-ui-egui/src/workbench/commands.rs`, `crates/sns-ui-egui/src/workbench/configuration_ui.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/src/workbench/document.rs`, `crates/sns-ui-egui/src/workbench/export_ui.rs`, `crates/sns-ui-egui/src/workbench/help_ui.rs`, `crates/sns-ui-egui/src/workbench/import_ui.rs`, `crates/sns-ui-egui/src/workbench/inspector.rs`, `crates/sns-ui-egui/src/workbench/mod.rs`, `crates/sns-ui-egui/src/workbench/preferences_ui.rs`, `crates/sns-ui-egui/src/workbench/runtime_ui.rs`, `crates/sns-ui-egui/src/workbench/session_ui.rs`, `crates/sns-ui-egui/src/workbench/worker.rs`, `crates/sns-ui-egui/tests/configuration.rs`, `crates/sns-ui-egui/tests/help.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/png.rs`, `crates/sns-ui-egui/tests/preferences.rs`, `crates/sns-ui-egui/tests/report.rs`, `crates/sns-ui-egui/tests/session.rs`, `crates/sns-ui-egui/tests/snapshots`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-about-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-about-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-about-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-about-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-about-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-about-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-design-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-documentation-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-help-simulation-icons-pt.png`, `docs/README.md`, `docs/agnostique/AFFICHAGE_J7.md`, `docs/agnostique/AIDE_J7.md`, `docs/agnostique/ARCHITECTURE.md`, `docs/agnostique/ATTENTES_J7.md`, `docs/agnostique/CONFIGURATION_J7.md`, `docs/agnostique/CONTRATS.md`, `docs/agnostique/DIAGNOSTIC_J7.md`, `docs/agnostique/EXEMPLES_J7.md`, `docs/agnostique/PROJETS_J7.md`, `docs/agnostique/REPRISE_J7.md`, `docs/rust/AFFICHAGE_J7.md`, `docs/rust/AIDE_J7.md`, `docs/rust/ARCHITECTURE.md`, `docs/rust/ATTENTES_J7.md`, `docs/rust/CONFIGURATION_J7.md`, `docs/rust/DEVELOPPEMENT.md`, `docs/rust/DIAGNOSTIC_J7.md`, `docs/rust/EXEMPLES_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/rust/LIVRAISON_J7.md`, `docs/rust/decisions/D-013-configuration-j7.md`, `docs/rust/decisions/D-014-attentes-j7.md`, `docs/rust/layers.json`, `docs/suivi/ETAT.md`, `docs/suivi/features.json`, `docs/verification/J7-AFFICHAGE.md`, `docs/verification/J7-AIDE.md`, `docs/verification/J7-ATTENTES.md`, `docs/verification/J7-CONFIGURATION.md`, `docs/verification/J7-DIAGNOSTIC.md`, `docs/verification/J7-LANCEMENT-EXEMPLES.md`, `docs/verification/J7-LIVRAISON-PAQUETS.md`, `docs/verification/j7-configuration-ci.log`, `docs/verification/j7-configuration-cli-de-diagram.json`, `docs/verification/j7-configuration-cli-de-icons.json`, `docs/verification/j7-configuration-cli-en-diagram.json`, `docs/verification/j7-configuration-cli-en-icons.json`, `docs/verification/j7-configuration-cli-es-diagram.json`, `docs/verification/j7-configuration-cli-es-icons.json`, `docs/verification/j7-configuration-cli-fr-diagram.json`, `docs/verification/j7-configuration-cli-fr-icons.json`, `docs/verification/j7-configuration-cli-it-diagram.json`, `docs/verification/j7-configuration-cli-it-icons.json`, `docs/verification/j7-configuration-cli-pt-diagram.json`, `docs/verification/j7-configuration-cli-pt-icons.json`, `docs/verification/j7-configuration-cli-tests.log`, `docs/verification/j7-configuration-code-manifest.json`, `docs/verification/j7-configuration-docs.log`, `docs/verification/j7-configuration-environment.json`, `docs/verification/j7-configuration-gpu-initial.log`, `docs/verification/j7-configuration-gpu.log`, `docs/verification/j7-configuration-help-de.txt`, `docs/verification/j7-configuration-help-en.txt`, `docs/verification/j7-configuration-help-es.txt`, `docs/verification/j7-configuration-help-fr.txt`, `docs/verification/j7-configuration-help-it.txt`, `docs/verification/j7-configuration-help-pt.txt`, `docs/verification/j7-configuration-source-delta.json`, `docs/verification/j7-configuration-window-gesture-after.log`, `docs/verification/j7-configuration-window-gesture-before.log`, `docs/verification/j7-delivery-code-manifest.json`, `docs/verification/j7-delivery-docs.log`, `docs/verification/j7-delivery-environment.json`, `docs/verification/j7-delivery-linux-smoke.log`, `docs/verification/j7-delivery-macos-smoke.log`, `docs/verification/j7-delivery-packages.sha256.txt`, `docs/verification/j7-delivery-repository.log`, `docs/verification/j7-delivery-source-patch.json`, `docs/verification/j7-diagnostic-ci.log`, `docs/verification/j7-diagnostic-code-manifest.json`, `docs/verification/j7-diagnostic-docs.log`, `docs/verification/j7-diagnostic-environment.json`, `docs/verification/j7-diagnostic-export.json`, `docs/verification/j7-diagnostic-failure.jsonl`, `docs/verification/j7-diagnostic-gpu.log`, `docs/verification/j7-diagnostic-launch.jsonl`, `docs/verification/j7-diagnostic-snapshots.log`, `docs/verification/j7-diagnostic-source-patch.json`, `docs/verification/j7-display-ci.log`, `docs/verification/j7-display-code-manifest.json`, `docs/verification/j7-display-docs.log`, `docs/verification/j7-display-environment.json`, `docs/verification/j7-display-gpu-initial.log`, `docs/verification/j7-display-gpu.log`, `docs/verification/j7-display-initial-ci-failure.log`, `docs/verification/j7-display-initial-gpu-failure.log`, `docs/verification/j7-display-runtime-stress.json`, `docs/verification/j7-display-runtime-stress.log`, `docs/verification/j7-display-source-delta.json`, `docs/verification/j7-display-terminal-bounds-after.log`, `docs/verification/j7-display-terminal-bounds-before.log`, `docs/verification/j7-display-terminal-gpu-initial.log`, `docs/verification/j7-help-ci.log`, `docs/verification/j7-help-code-manifest.json`, `docs/verification/j7-help-docs.log`, `docs/verification/j7-help-environment.json`, `docs/verification/j7-help-gpu-final.log`, `docs/verification/j7-help-source-delta.json`, `docs/verification/j7-launch-examples-ci.log`, `docs/verification/j7-launch-examples-code-manifest.json`, `docs/verification/j7-launch-examples-direct.jsonl`, `docs/verification/j7-launch-examples-docs.log`, `docs/verification/j7-launch-examples-environment.json`, `docs/verification/j7-launch-examples-gpu.log`, `docs/verification/j7-launch-examples-snapshots.log`, `docs/verification/j7-launch-examples-source-patch.json`, `docs/verification/j7-launch-examples-startup.json`, `docs/verification/j7-timing-checkpoint.json`, `docs/verification/j7-timing-ci.log`, `docs/verification/j7-timing-cli-checkpoint.json`, `docs/verification/j7-timing-cli-default-summary.json`, `docs/verification/j7-timing-cli-default.jsonl`, `docs/verification/j7-timing-cli-full-summary.json`, `docs/verification/j7-timing-cli-full.jsonl`, `docs/verification/j7-timing-cli-partial-summary.json`, `docs/verification/j7-timing-cli-recording.json`, `docs/verification/j7-timing-cli-replay-summary.json`, `docs/verification/j7-timing-cli-replay.jsonl`, `docs/verification/j7-timing-cli-resume-summary.json`, `docs/verification/j7-timing-cli-resume.jsonl`, `docs/verification/j7-timing-code-manifest.json`, `docs/verification/j7-timing-default.jsonl`, `docs/verification/j7-timing-docs.log`, `docs/verification/j7-timing-environment.json`, `docs/verification/j7-timing-extended.jsonl`, `docs/verification/j7-timing-gpu.log`, `docs/verification/j7-timing-help-de.txt`, `docs/verification/j7-timing-help-en.txt`, `docs/verification/j7-timing-help-es.txt`, `docs/verification/j7-timing-help-fr.txt`, `docs/verification/j7-timing-help-it.txt`, `docs/verification/j7-timing-help-pt.txt`, `docs/verification/j7-timing-recording.json`, `docs/verification/j7-timing-restored.jsonl`, `docs/verification/j7-timing-source-delta.json`, `docs/verification/j7-timing-startup-de-diagram.json`, `docs/verification/j7-timing-startup-de-icons.json`, `docs/verification/j7-timing-startup-en-diagram.json`, `docs/verification/j7-timing-startup-en-icons.json`, `docs/verification/j7-timing-startup-es-diagram.json`, `docs/verification/j7-timing-startup-es-icons.json`, `docs/verification/j7-timing-startup-fr-diagram.json`, `docs/verification/j7-timing-startup-fr-icons.json`, `docs/verification/j7-timing-startup-it-diagram.json`, `docs/verification/j7-timing-startup-it-icons.json`, `docs/verification/j7-timing-startup-pt-diagram.json`, `docs/verification/j7-timing-startup-pt-icons.json`, `docs/verification/j7-timing-targeted.log`, `fixtures/valid/arp.json`, `fixtures/valid/configuration.json`, `fixtures/valid/direct.json`, `fixtures/valid/modem-connect.json`, `fixtures/valid/modem-listen.json`, `fixtures/valid/preferences-v1.json`, `fixtures/valid/preferences-v2.json`, `fixtures/valid/response-wait-scenario.json`, `packaging/build.py`, `packaging/linux/fr.sinesi.Sinesi.desktop`, `packaging/linux/fr.sinesi.Sinesi.xml`, `packaging/macos/Launcher.swift`, `packaging/smoke-deb.sh`, `packaging/smoke-windows.ps1`, `packaging/smoke.py`, `packaging/windows/Sinesi.iss`, `scripts/check_i18n.py`, `scripts/check_manifest.py`, `xtask/src/main.rs`, `docs/verification/J7-LIVRAISON-CLAVIER-AMD64.md`, `docs/verification/j7-qualification-repository.log`, `docs/verification/j7-qualification-arm64-keyboard-final.log`, `docs/verification/j7-qualification-amd64-keyboard.log`, `docs/verification/j7-qualification-amd64-build.log`, `docs/verification/j7-qualification-amd64-smoke.log`, `docs/verification/j7-qualification-windows-static.log`, `docs/verification/j7-qualification-packages.sha256.txt`, `docs/verification/j7-qualification-code-manifest.json`, `docs/verification/j7-qualification-docs.log`, `docs/verification/j7-qualification-source-patch.json`, `packaging/smoke-linux-accessibility.py`, `docs/verification/J7-BUREAU-NATIF-DEBIAN.md`, `docs/verification/j7-native-input-shortcut-before.log`, `docs/verification/j7-native-input-shortcut-after.log`, `docs/verification/j7-native-input-repository.log`, `docs/verification/j7-native-input-ui.log`, `docs/verification/j7-native-input-macos.log`, `docs/verification/j7-native-input-packaging.log`, `docs/verification/j7-native-input-packages.sha256.txt`, `docs/verification/j7-native-input-debian-arm64.log`, `docs/verification/j7-native-input-code-manifest.json`, `docs/verification/j7-native-input-docs.log`, `docs/verification/j7-native-input-source-patch.json`, `docs/verification/j7-native-input-debian-amd64.log`, `docs/verification/J7-WINDOWS-COMPATIBILITE.md`, `docs/verification/j7-windows-compatibility-build.log`, `docs/verification/j7-windows-compatibility-inno.log`, `docs/verification/j7-windows-compatibility-pe.log`, `docs/verification/j7-windows-compatibility-smoke.log`, `docs/verification/j7-windows-compatibility-smoke.sh`, `docs/verification/j7-windows-compatibility-package.sha256.txt`, `docs/verification/j7-windows-compatibility-code-manifest.json`, `docs/verification/j7-windows-compatibility-source-patch.json`, `docs/verification/j7-windows-compatibility-docs.log`, `docs/verification/j7-windows-compatibility-keyboard.log`, `packaging/macos/SmokeAssociation.swift`, `packaging/smoke-macos.sh`, `docs/verification/J7-MACOS-LAUNCHSERVICES.md`, `docs/verification/j7-macos-launchservices-smoke.log`, `docs/verification/j7-macos-launchservices-negative.log`, `docs/verification/j7-macos-launchservices-package.sha256.txt`, `docs/verification/j7-macos-launchservices-code-manifest.json`, `docs/verification/j7-macos-launchservices-source-patch.json`, `docs/verification/j7-macos-launchservices-docs.log`, `packaging/windows/KeyboardSmoke.ps1`, `docs/verification/J7-WINDOWS-CLAVIER.md`, `docs/verification/j7-windows-keyboard-local.log`, `docs/verification/j7-windows-keyboard-config.log`, `docs/verification/j7-windows-keyboard-code-manifest.json`, `docs/verification/j7-windows-keyboard-source-patch.json`, `docs/verification/j7-windows-keyboard-docs.log`, `packaging/macos/SmokeKeyboard.swift`, `docs/verification/J7-MACOS-CLAVIER.md`, `docs/verification/j7-macos-keyboard-local.log`, `docs/verification/j7-macos-keyboard-code-manifest.json`, `docs/verification/j7-macos-keyboard-source-patch.json`, `docs/verification/j7-macos-keyboard-docs.log`.
- Approche : Application native sns-desktop avec eframe/egui 0.37 et backend wgpu; rfd et presse-papiers via services dédiés; évaluation craft-ui/craft-window et forks cohérents; profils, langues et packaging trois OS. craft-update reste une option distincte. Tranche RTT du 9 novembre : F-CFG-005, facteur 1–5 des attentes de réponse, sérialisé pour reprise/rejeu; cadences et durées de vie inchangées. F-SIM-007/008 en non-régression. Journal technique J7 avec événements bornés, sortie CLI et export séparé du réseau. Options d’apparence système et de présentation compacte de F-CFG-005. Application temporaire des réglages et validation à 840 × 600. F-CFG-008 couvre un catalogue embarqué avec liaison directe et une paire de projets modem qualifiée par le pont en boucle locale.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-projects-j7"></a>

## RS-PROJECTS-J7 — Projets et import Filius — J7

- Jalon : `J7`; état : `verified`; taille indicative : `L`.
- Exigences : [F-PROJ-003](../agnostique/CATALOGUE.md#f-proj-003), [F-PROJ-005](../agnostique/CATALOGUE.md#f-proj-005), [F-PROJ-006](../agnostique/CATALOGUE.md#f-proj-006).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-project`, `crates/sns-filius-import`.
- Fichiers réalisés : `Cargo.lock`, `Cargo.toml`, `apps/sns-desktop/src/main.rs`, `contracts/README.md`, `contracts/import-report.schema.json`, `contracts/session.schema.json`, `crates/sns-cli/Cargo.toml`, `crates/sns-cli/src/main.rs`, `crates/sns-cli/tests/runner.rs`, `crates/sns-filius-import`, `crates/sns-filius-import/src/annotations.rs`, `crates/sns-filius-import/src/convert.rs`, `crates/sns-filius-import/src/lib.rs`, `crates/sns-filius-import/src/mail.rs`, `crates/sns-filius-import/src/mail_content.rs`, `crates/sns-filius-import/src/network.rs`, `crates/sns-filius-import/src/security.rs`, `crates/sns-filius-import/src/xml.rs`, `crates/sns-filius-import/tests/import.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-project/Cargo.toml`, `crates/sns-project/src/lib.rs`, `crates/sns-project/src/session.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-project/tests/session.rs`, `crates/sns-settings/src/lib.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-ui-egui/Cargo.toml`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/annotations.rs`, `crates/sns-ui-egui/src/workbench/chrome.rs`, `crates/sns-ui-egui/src/workbench/desktop/mail_ui.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-ui-egui/src/workbench/desktop/p2p_ui.rs`, `crates/sns-ui-egui/src/workbench/document.rs`, `crates/sns-ui-egui/src/workbench/import_ui.rs`, `crates/sns-ui-egui/src/workbench/link_conditions_ui.rs`, `crates/sns-ui-egui/src/workbench/mod.rs`, `crates/sns-ui-egui/src/workbench/routing_ui.rs`, `crates/sns-ui-egui/src/workbench/runtime_ui.rs`, `crates/sns-ui-egui/src/workbench/security_ui.rs`, `crates/sns-ui-egui/src/workbench/session_ui.rs`, `crates/sns-ui-egui/src/workbench/trace_ui.rs`, `crates/sns-ui-egui/tests/annotations.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/session.rs`, `crates/sns-ui-egui/tests/snapshots/j7-session-offer-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-offer-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-offer-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-offer-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-offer-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-offer-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-restored-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-restored-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-restored-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-restored-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-restored-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-session-restored-pt.png`, `docs/agnostique/PROJETS_J7.md`, `docs/agnostique/SESSION_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/rust/decisions/D-009-session-j7.md`, `docs/suivi/ETAT.md`, `docs/verification/J7-ANNOTATIONS.md`, `docs/verification/J7-IMPORT-COURRIER.md`, `docs/verification/J7-IMPORT-DHCP-ROUTES.md`, `docs/verification/J7-IMPORT-EN-COURS.md`, `docs/verification/J7-IMPORT-NAT-FILTRAGE.md`, `docs/verification/J7-IMPORT-RIP-DOCUMENTATION.md`, `docs/verification/J7-SESSION-EN-COURS.md`, `docs/verification/J7-SESSION.md`, `docs/verification/STRUCTURE-OUVERTURE.md`, `docs/verification/j7-import-mail-code-manifest.json`, `docs/verification/j7-import-mail-corpus.json.gz`, `docs/verification/j7-import-mail-docs.log`, `docs/verification/j7-import-mail-entry.log`, `docs/verification/j7-import-mail-linux-package.log`, `docs/verification/j7-import-mail-macos-package.log`, `docs/verification/j7-import-mail-packages.sha256.txt`, `docs/verification/j7-import-mail-pop3.json`, `docs/verification/j7-import-mail-source-patch.json`, `docs/verification/j7-import-mail-tests.log`, `docs/verification/j7-import-network-code-manifest.json`, `docs/verification/j7-import-network-corpus-summary.json`, `docs/verification/j7-import-network-dhcp.jsonl`, `docs/verification/j7-import-network-docs.log`, `docs/verification/j7-import-network-entry.log`, `docs/verification/j7-import-network-routes.jsonl`, `docs/verification/j7-import-network-source-patch.json`, `docs/verification/j7-import-network-tests.log`, `docs/verification/j7-import-rip-doc-code-manifest.json`, `docs/verification/j7-import-rip-doc-diagram.json`, `docs/verification/j7-import-rip-doc-diagram.png`, `docs/verification/j7-import-rip-doc-docs.log`, `docs/verification/j7-import-rip-doc-entry.log`, `docs/verification/j7-import-rip-doc-icons.json`, `docs/verification/j7-import-rip-doc-icons.png`, `docs/verification/j7-import-rip-doc-source-patch.json`, `docs/verification/j7-import-rip-doc-summary.json`, `docs/verification/j7-import-rip-doc-tests.log`, `docs/verification/j7-import-rip-doc-trace.jsonl.gz`, `docs/verification/j7-import-security-code-manifest.json`, `docs/verification/j7-import-security-docs.log`, `docs/verification/j7-import-security-entry.log`, `docs/verification/j7-import-security-firewall.jsonl.gz`, `docs/verification/j7-import-security-nat.jsonl.gz`, `docs/verification/j7-import-security-source-patch.json`, `docs/verification/j7-import-security-summary.json`, `docs/verification/j7-import-security-tests.log`, `docs/verification/structure-document-open-ci.log`, `docs/verification/structure-document-open-code-manifest.json`, `docs/verification/structure-document-open-docs.log`, `docs/verification/structure-document-open-source-patch.json`, `docs/verification/structure-document-open-ui.log`, `fixtures/legacy/lan.fls`, `fixtures/legacy/lan.xml`, `fixtures/valid/session.json`, `scripts/check_i18n.py`, `scripts/check_layers.py`, `scripts/check_manifest.py`.
- Approche : DTO Serde pour format neutre JSON/ZIP; XML .fls interprété comme données en liste autorisée; aucune exécution de JavaBeans ou de bytecode.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-quality-j7"></a>

## RS-QUALITY-J7 — Contrats, qualité et outillage — J7

- Jalon : `J7`; état : `in_progress`; taille indicative : `M`.
- Exigences : [F-NFR-007](../agnostique/CATALOGUE.md#f-nfr-007).
- Prérequis : aucun.
- Cibles : `contracts`, `fixtures`, `crates/sns-cli`, `docs/decisions`, `crates/sns-ui-egui/tests`.
- Fichiers réalisés : `.github/workflows/packages.yml`, `crates/sns-i18n/locales/messages.json`, `crates/sns-ui-egui/src/workbench/document.rs`, `docs/agnostique/PROJETS_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/rust/LIVRAISON_J7.md`, `docs/suivi/ETAT.md`, `docs/verification/J7-LIVRAISON-PAQUETS.md`, `docs/verification/j7-delivery-code-manifest.json`, `docs/verification/j7-delivery-docs.log`, `docs/verification/j7-delivery-environment.json`, `docs/verification/j7-delivery-linux-smoke.log`, `docs/verification/j7-delivery-macos-smoke.log`, `docs/verification/j7-delivery-packages.sha256.txt`, `docs/verification/j7-delivery-repository.log`, `docs/verification/j7-delivery-source-patch.json`, `packaging/build.py`, `packaging/linux/fr.sinesi.Sinesi.desktop`, `packaging/linux/fr.sinesi.Sinesi.xml`, `packaging/macos/Launcher.swift`, `packaging/smoke-deb.sh`, `packaging/smoke-windows.ps1`, `packaging/smoke.py`, `packaging/windows/Sinesi.iss`, `scripts/check_manifest.py`, `crates/sns-ui-egui/tests/lan.rs`, `docs/verification/J7-LIVRAISON-CLAVIER-AMD64.md`, `docs/verification/j7-qualification-repository.log`, `docs/verification/j7-qualification-arm64-keyboard-final.log`, `docs/verification/j7-qualification-amd64-keyboard.log`, `docs/verification/j7-qualification-amd64-build.log`, `docs/verification/j7-qualification-amd64-smoke.log`, `docs/verification/j7-qualification-windows-static.log`, `docs/verification/j7-qualification-packages.sha256.txt`, `docs/verification/j7-qualification-code-manifest.json`, `docs/verification/j7-qualification-docs.log`, `docs/verification/j7-qualification-source-patch.json`, `crates/sns-ui-egui/src/workbench/commands.rs`, `packaging/smoke-linux-accessibility.py`, `docs/verification/J7-BUREAU-NATIF-DEBIAN.md`, `docs/verification/j7-native-input-shortcut-before.log`, `docs/verification/j7-native-input-shortcut-after.log`, `docs/verification/j7-native-input-repository.log`, `docs/verification/j7-native-input-ui.log`, `docs/verification/j7-native-input-macos.log`, `docs/verification/j7-native-input-packaging.log`, `docs/verification/j7-native-input-packages.sha256.txt`, `docs/verification/j7-native-input-debian-arm64.log`, `docs/verification/j7-native-input-code-manifest.json`, `docs/verification/j7-native-input-docs.log`, `docs/verification/j7-native-input-source-patch.json`, `docs/verification/j7-native-input-debian-amd64.log`, `docs/verification/J7-WINDOWS-COMPATIBILITE.md`, `docs/verification/j7-windows-compatibility-build.log`, `docs/verification/j7-windows-compatibility-inno.log`, `docs/verification/j7-windows-compatibility-pe.log`, `docs/verification/j7-windows-compatibility-smoke.log`, `docs/verification/j7-windows-compatibility-smoke.sh`, `docs/verification/j7-windows-compatibility-package.sha256.txt`, `docs/verification/j7-windows-compatibility-code-manifest.json`, `docs/verification/j7-windows-compatibility-source-patch.json`, `docs/verification/j7-windows-compatibility-docs.log`, `docs/verification/j7-windows-compatibility-keyboard.log`, `packaging/macos/SmokeAssociation.swift`, `packaging/smoke-macos.sh`, `docs/verification/J7-MACOS-LAUNCHSERVICES.md`, `docs/verification/j7-macos-launchservices-smoke.log`, `docs/verification/j7-macos-launchservices-negative.log`, `docs/verification/j7-macos-launchservices-package.sha256.txt`, `docs/verification/j7-macos-launchservices-code-manifest.json`, `docs/verification/j7-macos-launchservices-source-patch.json`, `docs/verification/j7-macos-launchservices-docs.log`, `packaging/windows/KeyboardSmoke.ps1`, `docs/verification/J7-WINDOWS-CLAVIER.md`, `docs/verification/j7-windows-keyboard-local.log`, `docs/verification/j7-windows-keyboard-config.log`, `docs/verification/j7-windows-keyboard-code-manifest.json`, `docs/verification/j7-windows-keyboard-source-patch.json`, `docs/verification/j7-windows-keyboard-docs.log`, `packaging/macos/SmokeKeyboard.swift`, `docs/verification/J7-MACOS-CLAVIER.md`, `docs/verification/j7-macos-keyboard-local.log`, `docs/verification/j7-macos-keyboard-code-manifest.json`, `docs/verification/j7-macos-keyboard-source-patch.json`, `docs/verification/j7-macos-keyboard-docs.log`.
- Approche : Matrice Windows/Linux/macOS, parcours clavier et langues; snapshots egui_kittest avec wgpu/snapshot et polices de production; essais réels dialogues, presse-papiers et multi-écran; inspection visuelle PNG/PDF.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-reporting-j7"></a>

## RS-REPORTING-J7 — Documentation et exports — J7

- Jalon : `J7`; état : `verified`; taille indicative : `M`.
- Exigences : [F-DOC-001](../agnostique/CATALOGUE.md#f-doc-001), [F-DOC-002](../agnostique/CATALOGUE.md#f-doc-002), [F-DOC-003](../agnostique/CATALOGUE.md#f-doc-003), [F-DOC-004](../agnostique/CATALOGUE.md#f-doc-004).
- Prérequis : RS-OBSERVATION-J2, RS-TOPOLOGY-J1.
- Cibles : `crates/sns-report`, `crates/sns-ui-egui/src/annotations`, `crates/sns-render`, `crates/sns-fonts`.
- Fichiers réalisés : `.github/workflows/ci.yml`, `Cargo.lock`, `Cargo.toml`, `assets/fonts/NotoSans-Bold.ttf`, `assets/fonts/NotoSans-Regular.ttf`, `assets/fonts/NotoSansMono-Bold.ttf`, `assets/fonts/NotoSansMono-Regular.ttf`, `assets/fonts/OFL.txt`, `assets/fonts/README.md`, `assets/fonts/provenance.json`, `contracts/checkpoint.schema.json`, `contracts/network-scenario.schema.json`, `contracts/project-documentation.schema.json`, `contracts/recording.schema.json`, `contracts/session.schema.json`, `crates/sns-cli/Cargo.toml`, `crates/sns-cli/src/main.rs`, `crates/sns-cli/src/render_cli.rs`, `crates/sns-cli/src/report_cli.rs`, `crates/sns-cli/src/runtime_cli.rs`, `crates/sns-cli/tests/png.rs`, `crates/sns-cli/tests/report.rs`, `crates/sns-filius-import/src/convert.rs`, `crates/sns-fonts/Cargo.toml`, `crates/sns-fonts/src/layout.rs`, `crates/sns-fonts/src/lib.rs`, `crates/sns-fonts/tests/layout.rs`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/annotations.rs`, `crates/sns-model/src/lib.rs`, `crates/sns-project/src/session.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-render`, `crates/sns-render/Cargo.toml`, `crates/sns-render/src/assets.rs`, `crates/sns-render/src/lib.rs`, `crates/sns-render/src/png.rs`, `crates/sns-render/tests/png.rs`, `crates/sns-report`, `crates/sns-report/Cargo.toml`, `crates/sns-report/src/content.rs`, `crates/sns-report/src/lib.rs`, `crates/sns-report/src/pdf.rs`, `crates/sns-report/src/scene.rs`, `crates/sns-report/tests/report.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/annotations.rs`, `crates/sns-topology/tests/duplication.rs`, `crates/sns-ui-egui/Cargo.toml`, `crates/sns-ui-egui/src/lib.rs`, `crates/sns-ui-egui/src/network_art.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/annotations.rs`, `crates/sns-ui-egui/src/workbench/chrome.rs`, `crates/sns-ui-egui/src/workbench/export_ui.rs`, `crates/sns-ui-egui/src/workbench/import_ui.rs`, `crates/sns-ui-egui/src/workbench/mod.rs`, `crates/sns-ui-egui/src/workbench/runtime_ui.rs`, `crates/sns-ui-egui/src/workbench/session_ui.rs`, `crates/sns-ui-egui/tests/annotations.rs`, `crates/sns-ui-egui/tests/png.rs`, `crates/sns-ui-egui/tests/report.rs`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-annotations-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-png-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-pdf-icons-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-diagram-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-diagram-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-diagram-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-diagram-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-diagram-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-diagram-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-icons-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-icons-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-icons-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-icons-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-icons-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-report-png-icons-pt.png`, `docs/agnostique/ANNOTATIONS_J7.md`, `docs/agnostique/ARCHITECTURE.md`, `docs/agnostique/EXPORTS_J7.md`, `docs/agnostique/RAPPORTS_J7.md`, `docs/agnostique/SESSION_J7.md`, `docs/rust/ARCHITECTURE.md`, `docs/rust/DEPENDANCES.md`, `docs/rust/DEVELOPPEMENT.md`, `docs/rust/J7-EN-COURS.md`, `docs/rust/decisions/D-009-session-j7.md`, `docs/rust/decisions/D-010-export-scene-j7.md`, `docs/rust/decisions/D-012-rapport-pdf-j7.md`, `docs/rust/layers.json`, `docs/verification/J7-ANNOTATIONS.md`, `docs/verification/J7-PNG.md`, `docs/verification/J7-RAPPORT.md`, `docs/verification/J7-SESSION.md`, `docs/verification/j7-png-clippy.log`, `docs/verification/j7-png-code-manifest.json`, `docs/verification/j7-png-docs.log`, `docs/verification/j7-png-environment.json`, `docs/verification/j7-png-gpu.log`, `docs/verification/j7-png-independent.log`, `docs/verification/j7-png-source-delta.json`, `docs/verification/j7-png-workspace.log`, `docs/verification/j7-report-ci.log`, `docs/verification/j7-report-code-manifest.json`, `docs/verification/j7-report-docs.log`, `docs/verification/j7-report-environment.json`, `docs/verification/j7-report-fr-diagram.pdf`, `docs/verification/j7-report-glyph-failure.log`, `docs/verification/j7-report-gpu.log`, `docs/verification/j7-report-independent.log`, `docs/verification/j7-report-layout.json`, `docs/verification/j7-report-pagination.pdf`, `docs/verification/j7-report-review-fr.png`, `docs/verification/j7-report-source-delta.json`, `fixtures/valid/annotations.json`, `fixtures/valid/report-network.json`, `scripts/check_fonts.py`, `scripts/check_i18n.py`, `scripts/check_layers.py`, `scripts/check_manifest.py`, `scripts/check_pdf_report.py`, `scripts/check_png_exports.py`, `scripts/requirements-pdf-verification.txt`, `xtask/src/main.rs`.
- Approche : Scène documentaire sans widgets; PNG resvg/tiny-skia verrouillés, rapport paginé sns-report avec Krilla 0.8.2 et glyphes SNS explicites; instantané/configuration effective, écriture atomique, quotas et validation indépendante/visuelle.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j7"></a>

## RS-TERMINAL-J7 — Terminal pédagogique — J7

- Jalon : `J7`; état : `verified`; taille indicative : `M`.
- Exigences : [F-CLI-019](../agnostique/CATALOGUE.md#f-cli-019).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : `crates/sns-apps/src/lib.rs`, `crates/sns-apps/src/terminal.rs`, `crates/sns-engine/tests/applications.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `docs/agnostique/DIAGNOSTIC_TERMINAL_J7.md`, `docs/agnostique/FIDELITE.md`, `docs/rust/J7-EN-COURS.md`, `docs/verification/J7-TERMINAL.md`.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-topology-j7"></a>

## RS-TOPOLOGY-J7 — Éditeur de réseau — J7

- Jalon : `J7`; état : `verified`; taille indicative : `L`.
- Exigences : [F-TOP-009](../agnostique/CATALOGUE.md#f-top-009).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-model`, `crates/sns-ui-egui/src/topology`.
- Fichiers réalisés : `Cargo.lock`, `crates/sns-i18n/locales/messages.json`, `crates/sns-model/src/lib.rs`, `crates/sns-project/src/lib.rs`, `crates/sns-topology/Cargo.toml`, `crates/sns-topology/src/duplication.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/duplication.rs`, `crates/sns-ui-egui/src/workbench`, `crates/sns-ui-egui/src/workbench/import_ui.rs`, `crates/sns-ui-egui/src/workbench/runtime_ui.rs`, `crates/sns-ui-egui/src/workbench/topology_ui.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/snapshots/j7-topology-aligned-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-aligned-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-aligned-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-aligned-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-aligned-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-aligned-pt.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-copy-de.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-copy-en.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-copy-es.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-copy-fr.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-copy-it.png`, `crates/sns-ui-egui/tests/snapshots/j7-topology-copy-pt.png`, `docs/agnostique/EDITION_J7.md`, `docs/rust/J7-EN-COURS.md`, `docs/verification/J7-TOPOLOGIE.md`.
- Approche : Commandes métier validées côté cœur; canevas natif sns-ui-egui avec Painter egui, transformations monde/écran, sélection stable et historique réversible; tests egui_kittest.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-extensions-j8"></a>

## RS-EXTENSIONS-J8 — Extensions et modem — J8

- Jalon : `J8`; état : `not_started`; taille indicative : `XL`.
- Exigences : [F-EXT-001](../agnostique/CATALOGUE.md#f-ext-001), [F-EXT-002](../agnostique/CATALOGUE.md#f-ext-002), [F-EXT-003](../agnostique/CATALOGUE.md#f-ext-003), [F-EXT-004](../agnostique/CATALOGUE.md#f-ext-004), [F-EXT-006](../agnostique/CATALOGUE.md#f-ext-006).
- Prérequis : RS-APPLICATIONS-J2.
- Cibles : `crates/sns-bridge`, `crates/sns-extension-host`, `crates/sns-ui-egui/src/extensions`.
- Fichiers réalisés : aucun.
- Approche : J6: pont réel Tokio séparé et entrées enregistrées; J8: contrat versionné, adaptateur Wasmtime à prototyper, quotas et capacités minimales.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-future-j9"></a>

## RS-FUTURE-J9 — Études optionnelles — J9

- Jalon : `J9`; état : `deferred`; taille indicative : `XL`.
- Exigences : [F-ADV-001](../agnostique/CATALOGUE.md#f-adv-001), [F-ADV-002](../agnostique/CATALOGUE.md#f-adv-002), [F-ADV-003](../agnostique/CATALOGUE.md#f-adv-003), [F-ADV-004](../agnostique/CATALOGUE.md#f-adv-004), [F-ADV-005](../agnostique/CATALOGUE.md#f-adv-005), [F-ADV-006](../agnostique/CATALOGUE.md#f-adv-006).
- Prérequis : aucun.
- Cibles : `docs/decisions`.
- Fichiers réalisés : aucun.
- Approche : Maintenir ces lots différés; écrire une décision et des contrats avant d’ajouter dépendances ou implémentation.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-reporting-j9"></a>

## RS-REPORTING-J9 — Documentation et exports — J9

- Jalon : `J9`; état : `deferred`; taille indicative : `M`.
- Exigences : [F-DOC-005](../agnostique/CATALOGUE.md#f-doc-005).
- Prérequis : RS-OBSERVATION-J2, RS-TOPOLOGY-J1.
- Cibles : `crates/sns-report`, `crates/sns-ui-egui/src/annotations`, `crates/sns-render`, `crates/sns-fonts`.
- Fichiers réalisés : aucun.
- Approche : Scène d’export indépendante des widgets; tiny-skia 0.12 pour PNG, évaluation craft-pdf/Krilla pour PDF; pagination SNS et chaîne de glyphes explicite si nécessaire; validation visuelle et quotas.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-ux-j8"></a>

## RS-UX-J8 — Interface élève — niveaux, espaces, palette, animation

- Jalon : `J8`; état : `in_progress`; taille indicative : `L`.
- Exigences : [F-UX-001](../agnostique/CATALOGUE.md#f-ux-001), [F-UX-002](../agnostique/CATALOGUE.md#f-ux-002), [F-UX-003](../agnostique/CATALOGUE.md#f-ux-003), [F-UX-004](../agnostique/CATALOGUE.md#f-ux-004), [F-UX-005](../agnostique/CATALOGUE.md#f-ux-005), [F-UX-006](../agnostique/CATALOGUE.md#f-ux-006), [F-ADV-004](../agnostique/CATALOGUE.md#f-adv-004).
- Prérequis : aucun.
- Cibles : `crates/sns-settings/src/display.rs`, `contracts/preferences.schema.json`, `crates/sns-i18n/locales/messages.json`, `crates/sns-ui-egui/src/workbench/chrome.rs`, `crates/sns-ui-egui/src/workbench/preferences_ui.rs`, `crates/sns-ui-egui/src/workbench/canvas.rs`, `crates/sns-ui-egui/src/workbench/inspector.rs`, `crates/sns-ui-egui/src/workbench/desktop`, `crates/sns-ui-egui/tests`, `docs/agnostique/EXPERIENCE_ELEVE.md`, `docs/rust/decisions/D-015-interface-eleve.md`, `crates/sns-missions`, `contracts/mission.schema.json`.
- Fichiers réalisés : `crates/sns-settings/src/display.rs`, `crates/sns-settings/src/configuration.rs`, `crates/sns-settings/tests/display.rs`, `contracts/preferences.schema.json`, `contracts/configuration.schema.json`, `contracts/startup-report.schema.json`, `crates/sns-i18n/locales/messages.json`, `crates/sns-ui-egui/src/workbench/toolbar.rs`, `crates/sns-ui-egui/src/workbench/chrome.rs`, `crates/sns-ui-egui/src/workbench/mod.rs`, `crates/sns-ui-egui/src/workbench/preferences_ui.rs`, `crates/sns-ui-egui/tests/student.rs`, `crates/sns-ui-egui/tests/annotations.rs`, `crates/sns-ui-egui/tests/configuration.rs`, `crates/sns-ui-egui/tests/help.rs`, `crates/sns-ui-egui/tests/lan.rs`, `crates/sns-ui-egui/tests/png.rs`, `crates/sns-ui-egui/tests/preferences.rs`, `crates/sns-ui-egui/tests/report.rs`, `crates/sns-ui-egui/tests/session.rs`, `crates/sns-ui-egui/tests/snapshots`, `crates/sns-ui-egui/tests/snapshots-historique/atelier-j7`, `xtask/src/main.rs`, `docs/agnostique/EXPERIENCE_ELEVE.md`, `docs/rust/decisions/D-015-interface-eleve.md`, `docs/verification/J8-UX-NIVEAUX.md`, `crates/sns-ui-egui/src/workbench/inspector.rs`, `crates/sns-ui-egui/src/workbench/canvas.rs`, `crates/sns-ui-egui/src/workbench/commands.rs`, `crates/sns-ui-egui/src/lib.rs`, `crates/sns-ui-egui/src/story.rs`, `crates/sns-ui-egui/src/workbench/packets.rs`, `crates/sns-ui-egui/src/workbench/story_ui.rs`, `crates/sns-ui-egui/src/workbench/trace_table.rs`, `crates/sns-ui-egui/src/workbench/desktop/terminal_text.rs`, `crates/sns-ui-egui/src/workbench/desktop/mod.rs`, `crates/sns-topology/src/advice.rs`, `crates/sns-topology/src/lib.rs`, `crates/sns-topology/tests/advice.rs`, `crates/sns-missions`, `crates/sns-cli/src/mission_cli.rs`, `crates/sns-cli/src/main.rs`, `crates/sns-cli/Cargo.toml`, `crates/sns-cli/tests/missions.rs`, `crates/sns-ui-egui/src/workbench/missions_ui.rs`, `crates/sns-ui-egui/src/workbench/document.rs`, `crates/sns-ui-egui/Cargo.toml`, `crates/sns-settings/src/lib.rs`, `contracts/mission.schema.json`, `contracts/mission-report.schema.json`, `contracts/README.md`, `docs/agnostique/MISSIONS.md`, `docs/rust/decisions/D-016-missions.md`, `docs/rust/layers.json`, `docs/rust/ARCHITECTURE.md`, `Cargo.toml`.
- Approche : Une seule présentation Workbench; niveau Élève/Expert en préférence d’affichage facultative du profil v2; commandes avancées inchangées et réservées au niveau Expert; nouvelles références graphiques après archivage des captures J0–J7 (D-015).
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.
