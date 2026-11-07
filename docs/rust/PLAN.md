# Plan d’implémentation Rust

Vue générée depuis [taches.json](taches.json). Les chemins sont des cibles proposées; les fichiers effectivement réalisés sont indiqués séparément. Les dépendances expriment les prérequis techniques; les recettes restent celles du catalogue agnostique.

<a id="rs-engine-j0"></a>

## RS-ENGINE-J0 — Moteur et cycle de simulation — J0

- Jalon : `J0`; état : `verified`; taille indicative : `L`.
- Exigences : [F-SIM-005](../agnostique/CATALOGUE.md#f-sim-005).
- Prérequis : aucun.
- Cibles : `crates/sns-engine`.
- Fichiers réalisés : `Cargo.toml`, `rust-toolchain.toml`, `crates/sns-model`, `crates/sns-engine`, `fixtures/valid/trace.jsonl`, `docs/decisions/D-001-ordonnancement.md`.
- Approche : File BinaryHeap avec Reverse; temps entier, ordre total, moteur synchrone à propriétaire unique; aucune minuterie système dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-projects-j0"></a>

## RS-PROJECTS-J0 — Projets et import Filius — J0

- Jalon : `J0`; état : `verified`; taille indicative : `L`.
- Exigences : [F-PROJ-004](../agnostique/CATALOGUE.md#f-proj-004).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-project`, `crates/sns-filius-import`.
- Fichiers réalisés : `crates/sns-project`, `contracts`, `fixtures`, `docs/decisions/D-004-format-j0.md`.
- Approche : DTO Serde pour format neutre JSON/ZIP; XML .fls interprété comme données en liste autorisée; aucune exécution de JavaBeans ou de bytecode.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-quality-j0"></a>

## RS-QUALITY-J0 — Contrats, qualité et outillage — J0

- Jalon : `J0`; état : `in_progress`; taille indicative : `M`.
- Exigences : [F-NFR-001](../agnostique/CATALOGUE.md#f-nfr-001), [F-NFR-002](../agnostique/CATALOGUE.md#f-nfr-002), [F-NFR-003](../agnostique/CATALOGUE.md#f-nfr-003), [F-NFR-006](../agnostique/CATALOGUE.md#f-nfr-006), [F-NFR-008](../agnostique/CATALOGUE.md#f-nfr-008), [F-NFR-009](../agnostique/CATALOGUE.md#f-nfr-009).
- Prérequis : aucun.
- Cibles : `contracts`, `fixtures`, `crates/sns-cli`, `docs/decisions`, `crates/sns-ui-egui`, `apps/sns-desktop`, `crates/sns-html`.
- Fichiers réalisés : `crates/sns-cli`, `crates/sns-html`, `crates/sns-ui-egui`, `apps/sns-desktop`, `contracts`, `scripts/check_layers.py`, `scripts/test_docs.py`, `.github/workflows/ci.yml`, `LICENSE`, `docs/rust/decisions`, `docs/verification/J0.md`.
- Approche : Schémas neutres, runner et moteur sans UI; prototype egui/eframe 0.37 canevas/table/focus, graphe upstream/forks et HTML pédagogique natif lien/image/POST sans sorties réelles; documenter D-002/D-003 et révisions candidates sans prétendre les recettes produit acquises.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j1"></a>

## RS-DEVICES-J1 — Équipements et configuration — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-EQU-001](../agnostique/CATALOGUE.md#f-equ-001), [F-EQU-002](../agnostique/CATALOGUE.md#f-equ-002), [F-EQU-006](../agnostique/CATALOGUE.md#f-equ-006).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : aucun.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-engine-j1"></a>

## RS-ENGINE-J1 — Moteur et cycle de simulation — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-SIM-001](../agnostique/CATALOGUE.md#f-sim-001), [F-SIM-002](../agnostique/CATALOGUE.md#f-sim-002), [F-SIM-003](../agnostique/CATALOGUE.md#f-sim-003), [F-SIM-004](../agnostique/CATALOGUE.md#f-sim-004), [F-SIM-006](../agnostique/CATALOGUE.md#f-sim-006).
- Prérequis : aucun.
- Cibles : `crates/sns-engine`.
- Fichiers réalisés : aucun.
- Approche : File BinaryHeap avec Reverse; temps entier, ordre total, moteur synchrone à propriétaire unique; aucune minuterie système dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-ipv4-j1"></a>

## RS-IPV4-J1 — IPv4 et ICMP — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-IP4-001](../agnostique/CATALOGUE.md#f-ip4-001), [F-IP4-002](../agnostique/CATALOGUE.md#f-ip4-002), [F-IP4-004](../agnostique/CATALOGUE.md#f-ip4-004).
- Prérequis : RS-LINK-J1.
- Cibles : `crates/sns-protocols/src/ipv4`.
- Fichiers réalisés : aucun.
- Approche : Adresses numériques typées, masques validés, next-hop explicite, erreurs structurées et corrélation des sondes.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-link-j1"></a>

## RS-LINK-J1 — Ethernet, switch et ARP — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-L2-001](../agnostique/CATALOGUE.md#f-l2-001), [F-L2-002](../agnostique/CATALOGUE.md#f-l2-002), [F-L2-003](../agnostique/CATALOGUE.md#f-l2-003), [F-L2-004](../agnostique/CATALOGUE.md#f-l2-004), [F-L2-005](../agnostique/CATALOGUE.md#f-l2-005).
- Prérequis : RS-DEVICES-J1.
- Cibles : `crates/sns-protocols/src/link`.
- Fichiers réalisés : aucun.
- Approche : Machines à états événementielles; clones de données de trace immuables; timers virtuels pour les caches.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-projects-j1"></a>

## RS-PROJECTS-J1 — Projets et import Filius — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-PROJ-001](../agnostique/CATALOGUE.md#f-proj-001), [F-PROJ-002](../agnostique/CATALOGUE.md#f-proj-002).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-project`, `crates/sns-filius-import`.
- Fichiers réalisés : aucun.
- Approche : DTO Serde pour format neutre JSON/ZIP; XML .fls interprété comme données en liste autorisée; aucune exécution de JavaBeans ou de bytecode.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-quality-j1"></a>

## RS-QUALITY-J1 — Contrats, qualité et outillage — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-NFR-004](../agnostique/CATALOGUE.md#f-nfr-004), [F-NFR-005](../agnostique/CATALOGUE.md#f-nfr-005).
- Prérequis : aucun.
- Cibles : `contracts`, `fixtures`, `crates/sns-cli`, `docs/decisions`, `crates/sns-ui-egui/tests`.
- Fichiers réalisés : aucun.
- Approche : Mesures de capacité/réactivité et tests egui_kittest des interactions, rôles accessibles, labels, focus et zoom; essais natifs distincts pour clavier/IME/DPI; aucun sleep réel dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-topology-j1"></a>

## RS-TOPOLOGY-J1 — Éditeur de réseau — J1

- Jalon : `J1`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-TOP-001](../agnostique/CATALOGUE.md#f-top-001), [F-TOP-002](../agnostique/CATALOGUE.md#f-top-002), [F-TOP-003](../agnostique/CATALOGUE.md#f-top-003), [F-TOP-004](../agnostique/CATALOGUE.md#f-top-004), [F-TOP-006](../agnostique/CATALOGUE.md#f-top-006), [F-TOP-007](../agnostique/CATALOGUE.md#f-top-007), [F-TOP-008](../agnostique/CATALOGUE.md#f-top-008), [F-TOP-010](../agnostique/CATALOGUE.md#f-top-010).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-model`, `crates/sns-ui-egui/src/topology`.
- Fichiers réalisés : aucun.
- Approche : Commandes métier validées côté cœur; canevas natif sns-ui-egui avec Painter egui, transformations monde/écran, sélection stable et historique réversible; tests egui_kittest.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-applications-j2"></a>

## RS-APPLICATIONS-J2 — Bureau et services génériques — J2

- Jalon : `J2`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-OS-001](../agnostique/CATALOGUE.md#f-os-001), [F-OS-002](../agnostique/CATALOGUE.md#f-os-002), [F-OS-003](../agnostique/CATALOGUE.md#f-os-003), [F-OS-005](../agnostique/CATALOGUE.md#f-os-005), [F-OS-006](../agnostique/CATALOGUE.md#f-os-006).
- Prérequis : RS-DEVICES-J1, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps`, `crates/sns-ui-egui/src/hosts`.
- Fichiers réalisés : aucun.
- Approche : Registre AppId et événements applicatifs; séparer installation, service actif et vue egui; panneaux par hôte dans sns-ui-egui avec API de socket simulée.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-filesystem-j2"></a>

## RS-FILESYSTEM-J2 — Fichiers virtuels et outils — J2

- Jalon : `J2`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-FS-001](../agnostique/CATALOGUE.md#f-fs-001), [F-FS-002](../agnostique/CATALOGUE.md#f-fs-002), [F-FS-003](../agnostique/CATALOGUE.md#f-fs-003), [F-FS-004](../agnostique/CATALOGUE.md#f-fs-004), [F-FS-005](../agnostique/CATALOGUE.md#f-fs-005).
- Prérequis : RS-DEVICES-J1.
- Cibles : `crates/sns-vfs`, `crates/sns-ui-egui/src/files`.
- Fichiers réalisés : aucun.
- Approche : Chemins virtuels normalisés et contenu binaire; vues egui indépendantes des imports hôte; dialogues natifs rfd via adaptateur sans bloquer eframe.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j2"></a>

## RS-OBSERVATION-J2 — Traces et visualisation — J2

- Jalon : `J2`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-OBS-001](../agnostique/CATALOGUE.md#f-obs-001), [F-OBS-002](../agnostique/CATALOGUE.md#f-obs-002), [F-OBS-003](../agnostique/CATALOGUE.md#f-obs-003), [F-OBS-004](../agnostique/CATALOGUE.md#f-obs-004), [F-OBS-005](../agnostique/CATALOGUE.md#f-obs-005), [F-OBS-010](../agnostique/CATALOGUE.md#f-obs-010).
- Prérequis : RS-IPV4-J1, RS-LINK-J1.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : aucun.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j2"></a>

## RS-TERMINAL-J2 — Terminal pédagogique — J2

- Jalon : `J2`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-CLI-001](../agnostique/CATALOGUE.md#f-cli-001), [F-CLI-002](../agnostique/CATALOGUE.md#f-cli-002), [F-CLI-003](../agnostique/CATALOGUE.md#f-cli-003), [F-CLI-004](../agnostique/CATALOGUE.md#f-cli-004), [F-CLI-005](../agnostique/CATALOGUE.md#f-cli-005), [F-CLI-006](../agnostique/CATALOGUE.md#f-cli-006), [F-CLI-007](../agnostique/CATALOGUE.md#f-cli-007), [F-CLI-008](../agnostique/CATALOGUE.md#f-cli-008), [F-CLI-009](../agnostique/CATALOGUE.md#f-cli-009), [F-CLI-011](../agnostique/CATALOGUE.md#f-cli-011), [F-CLI-012](../agnostique/CATALOGUE.md#f-cli-012), [F-CLI-014](../agnostique/CATALOGUE.md#f-cli-014), [F-CLI-016](../agnostique/CATALOGUE.md#f-cli-016), [F-CLI-018](../agnostique/CATALOGUE.md#f-cli-018), [F-CLI-020](../agnostique/CATALOGUE.md#f-cli-020), [F-CLI-021](../agnostique/CATALOGUE.md#f-cli-021).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : aucun.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-transport-j2"></a>

## RS-TRANSPORT-J2 — TCP et UDP simulés — J2

- Jalon : `J2`; état : `not_started`; taille indicative : `XL`.
- Exigences : [F-TCP-001](../agnostique/CATALOGUE.md#f-tcp-001), [F-TCP-002](../agnostique/CATALOGUE.md#f-tcp-002), [F-TCP-003](../agnostique/CATALOGUE.md#f-tcp-003), [F-TCP-004](../agnostique/CATALOGUE.md#f-tcp-004), [F-TCP-005](../agnostique/CATALOGUE.md#f-tcp-005), [F-UDP-001](../agnostique/CATALOGUE.md#f-udp-001), [F-UDP-002](../agnostique/CATALOGUE.md#f-udp-002).
- Prérequis : RS-IPV4-J1.
- Cibles : `crates/sns-protocols/src/transport`.
- Fichiers réalisés : aucun.
- Approche : Machines à états TCP et datagrammes UDP propres au simulateur; sockets simulées, allocation de ports, retransmissions virtuelles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j3"></a>

## RS-DEVICES-J3 — Équipements et configuration — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-EQU-003](../agnostique/CATALOGUE.md#f-equ-003).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : aucun.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-ipv4-j3"></a>

## RS-IPV4-J3 — IPv4 et ICMP — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-IP4-003](../agnostique/CATALOGUE.md#f-ip4-003), [F-IP4-005](../agnostique/CATALOGUE.md#f-ip4-005).
- Prérequis : RS-LINK-J1.
- Cibles : `crates/sns-protocols/src/ipv4`.
- Fichiers réalisés : aucun.
- Approche : Adresses numériques typées, masques validés, next-hop explicite, erreurs structurées et corrélation des sondes.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-link-j3"></a>

## RS-LINK-J3 — Ethernet, switch et ARP — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-L2-006](../agnostique/CATALOGUE.md#f-l2-006).
- Prérequis : RS-DEVICES-J1.
- Cibles : `crates/sns-protocols/src/link`.
- Fichiers réalisés : aucun.
- Approche : Machines à états événementielles; clones de données de trace immuables; timers virtuels pour les caches.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j3"></a>

## RS-OBSERVATION-J3 — Traces et visualisation — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-OBS-009](../agnostique/CATALOGUE.md#f-obs-009).
- Prérequis : RS-IPV4-J1, RS-LINK-J1.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : aucun.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-routing-j3"></a>

## RS-ROUTING-J3 — Routage statique et dynamique — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-ROUT-001](../agnostique/CATALOGUE.md#f-rout-001), [F-ROUT-002](../agnostique/CATALOGUE.md#f-rout-002), [F-ROUT-003](../agnostique/CATALOGUE.md#f-rout-003), [F-ROUT-004](../agnostique/CATALOGUE.md#f-rout-004), [F-ROUT-005](../agnostique/CATALOGUE.md#f-rout-005).
- Prérequis : RS-DEVICES-J3, RS-IPV4-J3.
- Cibles : `crates/sns-protocols/src/routing`.
- Fichiers réalisés : aucun.
- Approche : Tables distinctes calculées/statiques/RIP; choix explicables; ordre stable de départage et délais virtuels.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j3"></a>

## RS-TERMINAL-J3 — Terminal pédagogique — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-CLI-010](../agnostique/CATALOGUE.md#f-cli-010), [F-CLI-015](../agnostique/CATALOGUE.md#f-cli-015), [F-CLI-017](../agnostique/CATALOGUE.md#f-cli-017).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : aucun.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-topology-j3"></a>

## RS-TOPOLOGY-J3 — Éditeur de réseau — J3

- Jalon : `J3`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-TOP-005](../agnostique/CATALOGUE.md#f-top-005).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-model`, `crates/sns-ui-egui/src/topology`.
- Fichiers réalisés : aucun.
- Approche : Commandes métier validées côté cœur; canevas natif sns-ui-egui avec Painter egui, transformations monde/écran, sélection stable et historique réversible; tests egui_kittest.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-applications-j4"></a>

## RS-APPLICATIONS-J4 — Bureau et services génériques — J4

- Jalon : `J4`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-OS-004](../agnostique/CATALOGUE.md#f-os-004).
- Prérequis : RS-DEVICES-J1, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps`, `crates/sns-ui-egui/src/hosts`.
- Fichiers réalisés : aucun.
- Approche : Registre AppId et événements applicatifs; séparer installation, service actif et vue egui; panneaux par hôte dans sns-ui-egui avec API de socket simulée.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-dhcp-j4"></a>

## RS-DHCP-J4 — DHCP et link-local — J4

- Jalon : `J4`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-DHCP-001](../agnostique/CATALOGUE.md#f-dhcp-001), [F-DHCP-002](../agnostique/CATALOGUE.md#f-dhcp-002), [F-DHCP-003](../agnostique/CATALOGUE.md#f-dhcp-003), [F-DHCP-004](../agnostique/CATALOGUE.md#f-dhcp-004), [F-DHCP-005](../agnostique/CATALOGUE.md#f-dhcp-005), [F-DHCP-006](../agnostique/CATALOGUE.md#f-dhcp-006).
- Prérequis : RS-APPLICATIONS-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/dhcp`.
- Fichiers réalisés : aucun.
- Approche : États explicites client/serveur; tables d’offres/réservations/attributions et horloge virtuelle; conflits résolus via ARP simulé.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-dns-j4"></a>

## RS-DNS-J4 — DNS pédagogique — J4

- Jalon : `J4`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-DNS-001](../agnostique/CATALOGUE.md#f-dns-001), [F-DNS-002](../agnostique/CATALOGUE.md#f-dns-002), [F-DNS-003](../agnostique/CATALOGUE.md#f-dns-003), [F-DNS-004](../agnostique/CATALOGUE.md#f-dns-004), [F-DNS-005](../agnostique/CATALOGUE.md#f-dns-005), [F-DNS-006](../agnostique/CATALOGUE.md#f-dns-006).
- Prérequis : RS-APPLICATIONS-J2, RS-ENGINE-J0, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/dns`.
- Fichiers réalisés : aucun.
- Approche : Enregistrements A/MX/NS typés; caches par nœud et deadlines virtuelles; délégations bornées pour éviter les cycles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j4"></a>

## RS-TERMINAL-J4 — Terminal pédagogique — J4

- Jalon : `J4`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-CLI-013](../agnostique/CATALOGUE.md#f-cli-013).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : aucun.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-web-j4"></a>

## RS-WEB-J4 — Serveur et navigateur simulés — J4

- Jalon : `J4`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-WEB-001](../agnostique/CATALOGUE.md#f-web-001), [F-WEB-002](../agnostique/CATALOGUE.md#f-web-002), [F-WEB-003](../agnostique/CATALOGUE.md#f-web-003), [F-WEB-004](../agnostique/CATALOGUE.md#f-web-004), [F-WEB-005](../agnostique/CATALOGUE.md#f-web-005), [F-WEB-006](../agnostique/CATALOGUE.md#f-web-006).
- Prérequis : RS-DNS-J4, RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/http`, `crates/sns-ui-egui/src/browser`, `crates/sns-html`.
- Fichiers réalisés : aucun.
- Approche : HTTP pédagogique sur TCP simulé; document HTML borné sans I/O dans sns-html et vue egui pour texte, liens, images et formulaires; toutes les ressources et actions GET/POST passent par le moteur.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j5"></a>

## RS-DEVICES-J5 — Équipements et configuration — J5

- Jalon : `J5`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-EQU-004](../agnostique/CATALOGUE.md#f-equ-004).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : aucun.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-firewall-j5"></a>

## RS-FIREWALL-J5 — Filtrage pédagogique — J5

- Jalon : `J5`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-FW-001](../agnostique/CATALOGUE.md#f-fw-001), [F-FW-002](../agnostique/CATALOGUE.md#f-fw-002), [F-FW-003](../agnostique/CATALOGUE.md#f-fw-003), [F-FW-004](../agnostique/CATALOGUE.md#f-fw-004), [F-FW-005](../agnostique/CATALOGUE.md#f-fw-005), [F-FW-006](../agnostique/CATALOGUE.md#f-fw-006), [F-FW-007](../agnostique/CATALOGUE.md#f-fw-007).
- Prérequis : RS-ROUTING-J3, RS-TRANSPORT-J2, RS-WEB-J5.
- Cibles : `crates/sns-protocols/src/firewall`.
- Fichiers réalisés : aucun.
- Approche : Règles typées ordonnées, politique par défaut et modes historiques explicites; un résultat de décision détaillé par paquet.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-nat-j5"></a>

## RS-NAT-J5 — NAT et routeur domestique — J5

- Jalon : `J5`; état : `not_started`; taille indicative : `XL`.
- Exigences : [F-NAT-001](../agnostique/CATALOGUE.md#f-nat-001), [F-NAT-002](../agnostique/CATALOGUE.md#f-nat-002), [F-NAT-003](../agnostique/CATALOGUE.md#f-nat-003), [F-NAT-004](../agnostique/CATALOGUE.md#f-nat-004), [F-NAT-005](../agnostique/CATALOGUE.md#f-nat-005), [F-NAT-006](../agnostique/CATALOGUE.md#f-nat-006).
- Prérequis : RS-DEVICES-J5, RS-DHCP-J4, RS-ROUTING-J3, RS-TRANSPORT-J2.
- Cibles : `crates/sns-protocols/src/nat`.
- Fichiers réalisés : aucun.
- Approche : Clés de mapping explicites, IPv4/ports/ICMP; politique d’ordre NAT/filtrage figée par fixtures; tables consultables.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j5"></a>

## RS-OBSERVATION-J5 — Traces et visualisation — J5

- Jalon : `J5`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-OBS-007](../agnostique/CATALOGUE.md#f-obs-007).
- Prérequis : RS-LINK-J1, RS-NAT-J5.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : aucun.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-routing-j5"></a>

## RS-ROUTING-J5 — Routage statique et dynamique — J5

- Jalon : `J5`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-ROUT-006](../agnostique/CATALOGUE.md#f-rout-006).
- Prérequis : RS-DEVICES-J3, RS-IPV4-J3.
- Cibles : `crates/sns-protocols/src/routing`.
- Fichiers réalisés : aucun.
- Approche : Tables distinctes calculées/statiques/RIP; choix explicables; ordre stable de départage et délais virtuels.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-web-j5"></a>

## RS-WEB-J5 — Serveur et navigateur simulés — J5

- Jalon : `J5`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-WEB-007](../agnostique/CATALOGUE.md#f-web-007).
- Prérequis : RS-DNS-J4, RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/http`, `crates/sns-ui-egui/src/browser`, `crates/sns-html`.
- Fichiers réalisés : aucun.
- Approche : HTTP pédagogique sur TCP simulé; document HTML borné sans I/O dans sns-html et vue egui pour texte, liens, images et formulaires; toutes les ressources et actions GET/POST passent par le moteur.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j6"></a>

## RS-DEVICES-J6 — Équipements et configuration — J6

- Jalon : `J6`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-EQU-005](../agnostique/CATALOGUE.md#f-equ-005), [F-EQU-007](../agnostique/CATALOGUE.md#f-equ-007).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : aucun.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-extensions-j6"></a>

## RS-EXTENSIONS-J6 — Extensions et modem — J6

- Jalon : `J6`; état : `not_started`; taille indicative : `XL`.
- Exigences : [F-EXT-005](../agnostique/CATALOGUE.md#f-ext-005).
- Prérequis : RS-DEVICES-J6, RS-LINK-J1.
- Cibles : `crates/sns-bridge`, `crates/sns-extension-host`, `crates/sns-ui-egui/src/extensions`.
- Fichiers réalisés : aucun.
- Approche : J6: pont réel Tokio séparé et entrées enregistrées; J8: contrat versionné, adaptateur Wasmtime à prototyper, quotas et capacités minimales.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-mail-j6"></a>

## RS-MAIL-J6 — SMTP, POP3 et client mail — J6

- Jalon : `J6`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-MAIL-001](../agnostique/CATALOGUE.md#f-mail-001), [F-MAIL-002](../agnostique/CATALOGUE.md#f-mail-002), [F-MAIL-003](../agnostique/CATALOGUE.md#f-mail-003), [F-MAIL-004](../agnostique/CATALOGUE.md#f-mail-004), [F-MAIL-005](../agnostique/CATALOGUE.md#f-mail-005), [F-MAIL-006](../agnostique/CATALOGUE.md#f-mail-006), [F-MAIL-007](../agnostique/CATALOGUE.md#f-mail-007), [F-MAIL-008](../agnostique/CATALOGUE.md#f-mail-008).
- Prérequis : RS-APPLICATIONS-J2, RS-DNS-J4, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/mail`, `crates/sns-ui-egui/src/mail`.
- Fichiers réalisés : aucun.
- Approche : Sessions protocolaires événementielles; boîtes virtuelles, comptes pédagogiques et relais MX; aucune connexion mail réelle.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-p2p-j6"></a>

## RS-P2P-J6 — Gnutella pédagogique — J6

- Jalon : `J6`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-P2P-001](../agnostique/CATALOGUE.md#f-p2p-001), [F-P2P-002](../agnostique/CATALOGUE.md#f-p2p-002), [F-P2P-003](../agnostique/CATALOGUE.md#f-p2p-003), [F-P2P-004](../agnostique/CATALOGUE.md#f-p2p-004), [F-P2P-005](../agnostique/CATALOGUE.md#f-p2p-005).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/p2p`, `crates/sns-ui-egui/src/p2p`.
- Fichiers réalisés : aucun.
- Approche : Messages typés Ping/Pong/Query/QueryHit, identifiants et TTL; transferts dans le VFS avec quotas et annulation.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-devices-j7"></a>

## RS-DEVICES-J7 — Équipements et configuration — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-EQU-008](../agnostique/CATALOGUE.md#f-equ-008).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-model`, `crates/sns-devices`.
- Fichiers réalisés : aucun.
- Approche : Identifiants typés, arènes indexées, interfaces explicites; états de configuration distincts de l’état courant.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-engine-j7"></a>

## RS-ENGINE-J7 — Moteur et cycle de simulation — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-SIM-007](../agnostique/CATALOGUE.md#f-sim-007), [F-SIM-008](../agnostique/CATALOGUE.md#f-sim-008).
- Prérequis : aucun.
- Cibles : `crates/sns-engine`.
- Fichiers réalisés : aucun.
- Approche : File BinaryHeap avec Reverse; temps entier, ordre total, moteur synchrone à propriétaire unique; aucune minuterie système dans les protocoles.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-observation-j7"></a>

## RS-OBSERVATION-J7 — Traces et visualisation — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-OBS-006](../agnostique/CATALOGUE.md#f-obs-006), [F-OBS-008](../agnostique/CATALOGUE.md#f-obs-008).
- Prérequis : RS-IPV4-J1, RS-LINK-J1.
- Cibles : `crates/sns-trace`, `crates/sns-ui-egui/src/trace`.
- Fichiers réalisés : aucun.
- Approche : Événements append-only bornés, projections paginées vers sns-ui-egui; tables egui_extras à colonnes redimensionnables, inspection native et tests egui_kittest; export texte/JSONL/CSV.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-platform-j7"></a>

## RS-PLATFORM-J7 — Intégration desktop et préférences — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-CFG-001](../agnostique/CATALOGUE.md#f-cfg-001), [F-CFG-002](../agnostique/CATALOGUE.md#f-cfg-002), [F-CFG-003](../agnostique/CATALOGUE.md#f-cfg-003), [F-CFG-004](../agnostique/CATALOGUE.md#f-cfg-004), [F-CFG-005](../agnostique/CATALOGUE.md#f-cfg-005), [F-CFG-006](../agnostique/CATALOGUE.md#f-cfg-006), [F-CFG-007](../agnostique/CATALOGUE.md#f-cfg-007), [F-CFG-008](../agnostique/CATALOGUE.md#f-cfg-008).
- Prérequis : RS-PROJECTS-J1.
- Cibles : `apps/sns-desktop`, `crates/sns-settings`.
- Fichiers réalisés : aucun.
- Approche : Application native sns-desktop avec eframe/egui 0.37 et backend wgpu; rfd et presse-papiers via services dédiés; évaluation craft-ui/craft-window et forks cohérents; profils, langues et packaging trois OS. craft-update reste une option distincte.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-projects-j7"></a>

## RS-PROJECTS-J7 — Projets et import Filius — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-PROJ-003](../agnostique/CATALOGUE.md#f-proj-003), [F-PROJ-005](../agnostique/CATALOGUE.md#f-proj-005), [F-PROJ-006](../agnostique/CATALOGUE.md#f-proj-006).
- Prérequis : RS-TOPOLOGY-J1.
- Cibles : `crates/sns-project`, `crates/sns-filius-import`.
- Fichiers réalisés : aucun.
- Approche : DTO Serde pour format neutre JSON/ZIP; XML .fls interprété comme données en liste autorisée; aucune exécution de JavaBeans ou de bytecode.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-quality-j7"></a>

## RS-QUALITY-J7 — Contrats, qualité et outillage — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-NFR-007](../agnostique/CATALOGUE.md#f-nfr-007).
- Prérequis : aucun.
- Cibles : `contracts`, `fixtures`, `crates/sns-cli`, `docs/decisions`, `crates/sns-ui-egui/tests`.
- Fichiers réalisés : aucun.
- Approche : Matrice Windows/Linux/macOS, parcours clavier et langues; snapshots egui_kittest avec wgpu/snapshot et polices de production; essais réels dialogues, presse-papiers et multi-écran; inspection visuelle PNG/PDF.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-reporting-j7"></a>

## RS-REPORTING-J7 — Documentation et exports — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-DOC-001](../agnostique/CATALOGUE.md#f-doc-001), [F-DOC-002](../agnostique/CATALOGUE.md#f-doc-002), [F-DOC-003](../agnostique/CATALOGUE.md#f-doc-003), [F-DOC-004](../agnostique/CATALOGUE.md#f-doc-004).
- Prérequis : RS-OBSERVATION-J2, RS-TOPOLOGY-J1.
- Cibles : `crates/sns-report`, `crates/sns-ui-egui/src/annotations`, `crates/sns-render`, `crates/sns-fonts`.
- Fichiers réalisés : aucun.
- Approche : Scène d’export indépendante des widgets; tiny-skia 0.12 pour PNG, évaluation craft-pdf/Krilla pour PDF; pagination SNS et chaîne de glyphes explicite si nécessaire; validation visuelle et quotas.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-terminal-j7"></a>

## RS-TERMINAL-J7 — Terminal pédagogique — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `M`.
- Exigences : [F-CLI-019](../agnostique/CATALOGUE.md#f-cli-019).
- Prérequis : RS-FILESYSTEM-J2, RS-TRANSPORT-J2.
- Cibles : `crates/sns-apps/src/terminal`, `crates/sns-ui-egui/src/terminal`.
- Fichiers réalisés : aucun.
- Approche : Parseur de commandes à liste autorisée et alias explicites; vue egui à jobs annulables via commandes; historique/focus et presse-papiers de l’intégration native; aucun shell OS.
- Validation : implémenter les recettes de chaque exigence liée, ajouter leurs preuves dans les deux registres et vérifier les contrats communs.

<a id="rs-topology-j7"></a>

## RS-TOPOLOGY-J7 — Éditeur de réseau — J7

- Jalon : `J7`; état : `not_started`; taille indicative : `L`.
- Exigences : [F-TOP-009](../agnostique/CATALOGUE.md#f-top-009).
- Prérequis : RS-ENGINE-J0.
- Cibles : `crates/sns-model`, `crates/sns-ui-egui/src/topology`.
- Fichiers réalisés : aucun.
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
