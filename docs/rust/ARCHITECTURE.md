# Réalisation Rust — proposition technique

Ce volet adapte les exigences `F-*` à Rust. Il ne remplace pas leur définition agnostique. La direction retenue pour le cadrage est une application native **egui/eframe 0.37**, inspirée de la structure CADCraft proposée par l’utilisateur. Le socle J0 est initialisé avec les versions upstream 0.37.0 et Rust 1.95.0, verrouillées dans Cargo.lock et rust-toolchain.toml. Les autres modules de cette architecture restent des cibles proposées. Voir les décisions [D-002](decisions/D-002-egui-j0.md) et [D-003](decisions/D-003-html-j0.md) pour les choix effectivement réalisés.

## Recommandation

Construire un cœur Rust synchrone et déterministe, exposé sous forme de bibliothèque, puis deux adaptateurs : CLI et application desktop native. Le crate **`sns-ui-egui`** porte les écrans et les interactions; **`sns-desktop`** possède la fenêtre et la boucle eframe. L’UI dessine les projections du moteur et lui adresse des commandes, sans posséder l’état des protocoles.

Le [manifeste CADCraft](https://github.com/storytold/cadcraft/blob/main/Cargo.toml) et son [crate UI](https://github.com/storytold/cadcraft/blob/main/crates/ui-egui/Cargo.toml) servent de références de découpage. La sélection SNS et les réserves de compatibilité sont détaillées dans le [plan des dépendances](DEPENDANCES.md). Leurs crates de géométrie et formats CAD restent propres à leur produit.

| Couche | Direction SNS | Première validation |
| --- | --- | --- |
| Interface native | egui/eframe 0.37, egui_extras, egui_kittest | Canevas, table de trace, clavier, noms accessibles et multi-écran |
| Rendu écran | Backend wgpu d’eframe; Painter egui pour le canevas initial | Zoom, sélection et animations sur la charge de référence |
| Rendu export | Scène indépendante des widgets; tiny-skia pour PNG | Transparence, cadrage et cohérence du texte |
| Rapport PDF | Adaptateur `sns-report`; évaluation de craft-pdf/Krilla | Pagination, texte sélectionnable, polices et tableaux |
| Système | rfd et presse-papiers eframe, adaptateurs de fenêtres | Ouverture/enregistrement asynchrones et focus sur trois OS |
| Composants partagés | Évaluation ciblée de craft-ui et craft-window | Compatibilité des sources egui et persistance des panneaux |
| Texte avancé | Pipeline de glyphes séparé, ajouté selon besoin de rendu/export | Accents, retours à la ligne et tests de scripts complexes si activés |

La [documentation egui](https://docs.rs/egui/latest/egui/) décrit un modèle immediate mode. Les identifiants des widgets doivent donc rester stables quand une liste est triée, filtrée ou paginée. Le choix de cette famille graphique ne constitue pas une preuve d’accessibilité, de fidélité HTML ou de performance : ces points restent des validations explicites de J0 puis des recettes des jalons concernés.

## Organisation réalisée et cible

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
crates/
  sns-model/          # identifiants, configuration et validation
  sns-engine/         # horloge, file d’événements et orchestration
  sns-devices/        # équipements et ports
  sns-protocols/      # link, ipv4, routing, transport, nat, firewall
  sns-vfs/            # fichiers virtuels
  sns-apps/           # terminal, echo, dhcp, dns, http, mail, p2p
  sns-trace/          # événements historiques et projections
  sns-project/        # format portable, migrations et récupération
  sns-filius-import/  # lecture ZIP/XML comme données
  sns-settings/      # configuration utilisateur/administrateur
  sns-ui-egui/       # panneaux, canevas, bureaux et terminal natifs
  sns-render/        # scène d’export et raster PNG, sans dépendance egui
  sns-fonts/         # politique de polices et glyphes pour exports, si nécessaire
  sns-html/          # document HTML pédagogique borné, sans accès réseau/hôte
  sns-report/        # scène exportable et adaptateurs de rapport
  sns-bridge/        # sockets réelles entre instances, option explicite
  sns-extension-host/# contrat et runtime des extensions
  sns-cli/            # runner sans UI et commandes d’automatisation
apps/
  sns-desktop/        # application eframe et adaptateurs système
contracts/            # schémas neutres, pas générés depuis Rust comme seule autorité
fixtures/             # scénarios, traces et imports
```

J0 contient sns-model, sns-engine, sns-project, sns-html, sns-cli, sns-ui-egui et sns-desktop; les autres crates de l’arborescence restent proposées. Conserver les règles de dépendance lors de leur ajout. Les Cargo workspaces permettent de regrouper les packages et de partager le lockfile. [Livre officiel Rust](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

## Dépendances autorisées entre couches

- `model` et les contrats de données ne dépendent ni d’egui, ni de wgpu, ni de Tokio, ni de craft-ui/craft-window.
- `engine`, `devices`, `protocols`, `vfs` et `apps` ne font pas d’I/O hôte implicite.
- `trace` stocke des données indépendantes des objets UI; l’affichage les lit par curseurs.
- `project`, `bridge`, `report`, `extension-host`, `cli` et `desktop` sont des adaptateurs.
- `sns-ui-egui` dépend des projections et commandes; `sns-desktop` assemble eframe et les services natifs. Seule la couche de présentation utilise craft-ui/craft-window.
- `sns-render`, `sns-fonts` et `sns-html` manipulent des données explicites. Ils ne dépendent pas d’un contexte egui ni d’une boucle de simulation pour calculer une scène, des glyphes ou un document.
- Les besoins des applications sont exprimés par des interfaces internes explicites pour les sockets simulées, fichiers et timers.
- Éviter les dépendances cycliques entre crates en plaçant les types communs et les ports abstraits dans une couche interne réduite; ne pas créer une interface publique générique à chaque petite fonction.

## Moteur

Implémenter une file de priorité avec `BinaryHeap<Reverse<ScheduledEvent>>` et un ordre total `(time_us, priority, sequence)`. Le tas standard est un max-heap; `Reverse` permet l’ordre minimum souhaité. [Documentation BinaryHeap](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html).

Utiliser des nouveaux types pour `DeviceId`, `InterfaceId`, `RunId`, `SimTime`, `PortNumber` et les identifiants de paquets. Représenter les références par identifiants et collections indexées. Un identifiant supprimé ne doit pas désigner accidentellement un nouvel objet; prévoir générations ou allocation monotone.

Le moteur possède son état et expose une méthode d’application de commande et une méthode d’avancement. Les automates utilisent des enums d’événements/états. Les transitions retournent des effets explicites, sans créer un thread par hôte, socket ou protocole. Les itérations sur collections non ordonnées ne peuvent pas déterminer l’ordre des émissions.

Les timers sont des événements; leur annulation emploie un identifiant ou une génération. Les calculs de temps sont vérifiés contre les dépassements. La graine seule ne suffit pas à un checkpoint : conserver l’état exact du générateur et sa version. Choisir puis figer l’algorithme PRNG et l’ordre de ses appels par fixture avant de promettre le rejeu inter-version.

Une première stratégie d’exécution est un thread moteur dédié avec canal borné de commandes, canal de résultats et lots de projections. `sns-ui-egui` reçoit les résultats puis demande un repaint; une frame ne fait pas avancer un protocole. La synchronisation avec l’UI ne doit pas exiger un `Arc<Mutex<World>>` partagé partout. Toute concurrence dans les adaptateurs s’arrête à la file d’entrées du moteur.

## Sérialisation et contrats

Proposer `serde`/`serde_json` pour les DTO externes et des modèles internes séparés. Serde fournit la sérialisation de structures vers plusieurs formats; la définition du schéma métier reste une responsabilité du projet. [Documentation Serde](https://serde.rs/).

Les enums sérialisées utilisent des discriminants stables; aucun nom de type Rust, `Debug`, `usize` ou représentation mémoire ne constitue un format externe. Encoder les grands compteurs comme prévu dans les contrats. Les données d’extension inconnues sont conservées seulement dans des zones prévues, sans désérialisation polymorphe exécutable.

Les schémas et fixtures neutres sont l’autorité. L’UI Rust peut employer directement les DTO de commandes/projections, sans sérialiser pour chaque frame. JSON reste la frontière de fichier, de test et d’interopérabilité. Les formats compacts éventuels des checkpoints ne sont pas automatiquement des formats portables de projet. La sérialisation d’un agencement craft-ui reste une préférence de présentation, séparée du scénario réseau.

## Protocoles et applications

Développer des protocoles pédagogiques sur le moteur, plutôt qu’utiliser les sockets TCP/UDP de l’OS pour donner artificiellement l’illusion d’une simulation. Les applications utilisent `SimSocket`/`SimListener`, VFS et timers virtuels. L’API peut commencer par des opérations explicites à événements; un runtime async virtuel complet n’est pas requis pour J2.

Les paquets de trace sont des copies immuables ou des objets partagés immuables; les transformations créent de nouveaux instantanés. Un appel NAT ne peut pas modifier l’ancien événement référencé par l’inspecteur. Les buffers de contenu sont binaires, avec encodage texte déclaré seulement aux frontières applicatives.

Pour TCP, implémenter d’abord les états, l’établissement, la segmentation pédagogique, ACK, réessais et fermeture attendus. Fenêtres avancées, contrôle de congestion et options non présentes dans la matrice ne sont pas implicites. Les tables de routage/NAT/pare-feu exposent une décision structurée réutilisée par les traces et les inspecteurs.

## UI et navigateur simulé

`sns-ui-egui` envoie des commandes versionnées et ne connaît pas l’organisation mémoire des protocoles. Le canevas possède sa sélection, ses transformations écran/monde et une projection du graphe. Le rendu commence par le Painter d’egui : équipements, étiquettes, câbles, sélection et surbrillances. Les tables egui_extras sont paginées ou virtualisées; les animations consomment les événements sans piloter les timers réseau. Les glyphes/icônes seuls doivent avoir un nom accessible et une alternative clavier.

egui et egui_extras ne fournissent pas un navigateur HTML complet. Prévoir un parseur borné dans `sns-html` et un sous-ensemble de mise en page : texte, titres, paragraphes, liens, images et formulaires nécessaires aux exemples et à l’administration des routeurs. Les actions de lien/formulaire deviennent des commandes HTTP simulées. Les ressources images sont obtenues par le moteur, décodées sous quota puis chargées comme textures locales. Aucun chargeur HTTP générique ni ouverture automatique d’URL externe ne participe à cette vue.

Le prototype J0 doit afficher une page et une image issues d’un serveur simulé et soumettre un POST, sans sortie réelle. Ne pas réduire silencieusement F-WEB-005 à du texte/Markdown : les exercices et formulaires restent requis. JavaScript, iframes et liens de fichiers hôte n’ont pas de chemin d’exécution dans ce document. La liste de balises/styles pris en charge, les limites de taille/profondeur et les messages pour contenu non pris en charge sont à publier avant J4.

Les dialogues rfd sont des actions d’application, distinctes des fichiers virtuels des hôtes. Éviter de bloquer la boucle eframe pendant un dialogue ou un export. Le presse-papiers doit avoir un seul propriétaire d’intégration : support eframe pour les champs, adaptateur arboard direct seulement si un besoin supplémentaire est établi. La vérification de mises à jour est une option de distribution distincte, qui ne doit pas conditionner le fonctionnement hors ligne.

## I/O réelles et pont

Tokio est proposé pour les connexions réelles du pont, leur cadrage, leur annulation et leur arrêt. Il ne doit pas servir d’horloge de protocole au moteur. Son runtime fournit l’exécution asynchrone et les primitives d’I/O de l’adaptateur. [Tutoriel officiel Tokio](https://tokio.rs/tokio/tutorial).

Toute trame entrante est validée, bornée et horodatée à la frontière virtuelle puis inscrite au journal. L’adaptateur ne modifie pas les tables du moteur directement. Un rejeu remplace le pont par un lecteur d’entrées enregistrées. La sérialisation Java historique ne sera pas réutilisée pour ce transport.

## Import et exports

L’import .fls est une bibliothèque de lecture ZIP/XML à liste autorisée. Traiter explicitement identifiants, références et valeurs de configuration connues. Ne pas tenter de recréer un `XMLDecoder` ni de charger les classes `.class` embarquées. Sélectionner les bibliothèques ZIP/XML après vérification de leur politique d’entités, limites de décompression, maintenance et licence.

Générer les exports à partir d’une scène et d’un instantané de données neutres. `sns-render` vise tiny-skia 0.12 pour le PNG; `sns-report` évalue craft-pdf avec le backend Krilla pour le PDF. La mise en page des rapports, les tableaux et la pagination restent à développer dans SNS. lyon_tessellation n’est ajouté que si les tracés ou un callback GPU personnalisé le justifient; le canevas simple peut utiliser les formes egui.

Pour les exports textuels, évaluer une chaîne `skrifa`/`harfrust`, puis `unicode-bidi` et `icu_segmenter` selon les scripts nécessaires. L’ajout de ces crates ne change pas automatiquement le rendu ou l’édition de texte d’egui : il faut une intégration explicite de métriques et glyphes, cohérente avec l’export. Le moteur réseau ne dépendra pas de ces bibliothèques ni d’une session graphique pour exécuter ses recettes.

## Extensions

Proposer un contrat portable fondé sur WebAssembly, avec un hôte Wasmtime à prototyper pour la distribution desktop. Le format exact de l’interface et la chaîne de compilation des modèles sont des décisions de J8, précédées d’une étude courte. La reprise binaire des logiciels Java historiques n’est pas promise.

Wasmtime fournit des mécanismes d’isolation, mais l’application reste responsable de ses fonctions hôte, des capacités exposées et des limites. Limiter mémoire, exécution et ressources; n’ouvrir ni WASI filesystem ni réseau par défaut. Une extension terminée par quota produit un diagnostic et libère ses sockets virtuelles. [Modèle de sécurité Wasmtime](https://docs.wasmtime.dev/security.html).

L’assistant peut compiler un projet utilisateur via un adaptateur d’outillage contrôlé. Le lancement du compilateur doit être distinct de l’ouverture d’un scénario. Les modèles d’exemple sont écrits dans une chaîne supportée et utilisent le contrat virtuel; le langage de création d’extension ne doit pas devenir une contrainte de la spécification agnostique.

## Validation technique

Lorsqu’un workspace existe, la CI doit commencer par formatage, lint, tests de contrats et recettes sans interface. Les commandes proposées sont `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace`; préciser les features effectivement supportées plutôt que supposer que toutes leurs combinaisons sont compatibles.

Ajouter des tests de propriétés ciblés sur calcul de sous-réseaux, validité des tables, unicité des ports et absence de références orphelines; fuzzing ciblé sur import, framing et parseurs exposés. Tester les automates via temps virtuel, sans `sleep` réel. Les tests de contrats et traces sont réutilisables par un autre adaptateur.

Les tests d’interaction utilisent egui_kittest 0.37 sur les widgets réels : rôles/libellés accessibles, focus, commandes, sélection, tables et panneaux. Les snapshots de rendu ajoutent `wgpu` et `snapshot` dans un job adapté; les références d’images sont revues avant remplacement. [Guide officiel egui_kittest](https://github.com/emilk/egui/blob/0.37.0/crates/egui_kittest/README.md).

Les contrôles natifs couvrent séparément dialogues, presse-papiers, IME, fenêtres/multi-écran, DPI et backend Linux. Un test de widgets sans fenêtre réelle ne valide pas ces intégrations. Les exports PNG/PDF ont leurs propres recettes de cadrage, glyphes, pagination et transparence.

## Décisions à enregistrer lors du démarrage

| Décision | Proposition et preuve attendue | Liens |
| --- | --- | --- |
| D-001 Ordre des événements et PRNG | Priorités explicites, format canonique et état PRNG checkpointable; double exécution identique | F-SIM-005/007 |
| D-002 Application et UI natives | Direction egui/eframe 0.37 retenue; résolution du graphe upstream/forks, prototype canevas, trace et clavier sur trois OS | F-TOP-008, F-NFR-005/007 |
| D-003 Rendu HTML | Sous-ensemble HTML natif dans sns-html; lien, image et POST via pile simulée, sans chargement réel | F-WEB-005/006, F-NFR-006 |
| D-004 Format et migrations | Schémas projet/événements/checkpoints distincts, quotas et suffixe final | F-PROJ-004/005 |
| D-005 Sémantique réseau | Matrices TCP/RIP/DHCP/DNS et ordre NAT/filtrage, écarts pédagogiques documentés | F-TCP-003, F-ROUT-004, F-NAT-001, F-FW-004 |
| D-006 Extensions | ABI, outil de construction, quotas et stratégie des modèles portés | F-EXT-001 à 004/006 |
| D-007 Distribution et provenance | Cibles, notices, actifs réutilisés et versions verrouillées | F-CFG-007, F-NFR-009 |
| D-008 Briques de rendu et de texte | tiny-skia, évaluation craft-pdf/Krilla, scène commune et intégration explicite du shaping | F-DOC-003/004, F-FS-004, F-CFG-001/003 |

La direction graphique de D-002 dispose maintenant d’un prototype upstream compilé et de tests locaux; les validations de livraison sur les autres plateformes et les détails des lots ultérieurs restent ouverts. L’agent peut concrétiser ces décisions dans le périmètre autorisé et documenter les résultats, sans présenter le cadrage actuel comme une implémentation existante.
