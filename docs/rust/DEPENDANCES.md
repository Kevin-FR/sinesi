# Pile native egui — sélection et adoption

## État courant de l’adaptateur

Le [manifeste du workspace](../../Cargo.toml) et son lockfile font autorité.
Les tableaux de sélection ci-dessous conservent le cadrage initial; une proposition
ne signifie pas une dépendance adoptée. Au 8 novembre 2026 :

- UI : famille upstream egui/eframe/egui_extras/egui_kittest **0.37.0**, rfd **0.17.2**;
  aucun fork egui/winit ni module Craft activé.
- Scène et PNG : resvg **0.45.1**, tiny-skia transitif **0.11.4**;
  fontes embarquées, HarfRust **0.12.0** et Skrifa **0.44.0** dans `sns-fonts`.
- Rapports : Krilla **0.8.2** dans `sns-report`, indépendant des widgets.
- Qualité : **20 membres**, dépendances/lints partagés et politique de couches
  exhaustive; `cargo xtask ci` est commun au poste de développement et à la CI.
- Produit : **FR EN ES IT PT DE**, vues **Icônes/Schéma**. Leur présence ne qualifie
  pas à elle seule l’accessibilité ni un parcours natif sur trois OS.

Les décisions D-010 à D-012 et les sections de réalisation en fin de document
décrivent les choix effectifs. Une dépendance Craft ou un patch observé chez
SolidCraft/WordCraft exige toujours un besoin reproduit dans Sinesi.

## Cadrage initial conservé

Révision du cadrage : **7 novembre 2026**, à partir de la structure proposée par l’utilisateur et des manifestes publics consultés. Ce document conserve la sélection cible et ses évaluations. Le socle J0 utilise désormais la famille upstream 0.37.0, avec graphe compilé localement et verrouillé; voir [D-002](decisions/D-002-egui-j0.md). J1 adopte rfd 0.17.2 pour les dialogues natifs, verrouillé dans Cargo.lock. Les briques Craft, tiny-skia et le texte avancé restent des candidats des prochains lots.

La direction graphique est **egui/eframe 0.37**. Les numéros ci-dessous désignent des familles de versions cibles. Un lockfile et des commits Git immuables devront fixer les versions effectivement testées. Le catalogue agnostique et les identifiants `F-*` restent l’autorité fonctionnelle.

## Interface et services natifs

| Brique | Sélection SNS | Emplacement proposé | Exigences et lot |
| --- | --- | --- | --- |
| egui 0.37 | Retenu pour l’UI; `default-features = false`, `default_fonts`, `monochrome_emoji_fonts` | `crates/sns-ui-egui` | TOP, OS, OBS; RS-TOPOLOGY-J1, RS-APPLICATIONS-J2, RS-OBSERVATION-J2 |
| eframe 0.37 | Retenu pour application, fenêtre et boucle; sélection explicite du backend wgpu et des intégrations natives | `apps/sns-desktop` | F-NFR-005/007, F-CFG-007; prototype RS-QUALITY-J0, livraison RS-PLATFORM-J7 |
| egui_extras 0.37 | Retenu pour tables à colonnes redimensionnables; pas de chargeurs d’images/réseau activés globalement | `crates/sns-ui-egui` | F-OBS-001/004/008, tables MAC/NAT/DNS et pare-feu |
| egui_kittest 0.37 | Dépendance de développement; interactions et arbre accessible, snapshots dans un job de rendu dédié | Tests de `sns-ui-egui` | F-NFR-005, parcours TOP/OS/OBS; RS-QUALITY-J1/J7 |
| rfd 0.17 | Retenu pour ouvrir/enregistrer/importer/exporter des fichiers réels choisis par l’utilisateur | Adaptateur système de `sns-desktop` | F-PROJ-001/005, F-FS-003, F-DOC-003/004 |
| arboard | Candidat pour accès direct supplémentaire; commencer avec le support presse-papiers d’eframe pour les champs | Adaptateur presse-papiers de `sns-desktop` | Saisie et copie de texte, F-CLI-018, F-FS-004 |

Les deux fonctionnalités de police sont déclarées dans le [manifeste upstream egui 0.37](https://github.com/emilk/egui/blob/0.37.0/crates/egui/Cargo.toml). L’application doit choisir explicitement ce qu’elle embarque. Les fonctions d’accessibilité, presse-papiers et backends Linux sont documentées dans le [manifeste eframe 0.37](https://github.com/emilk/egui/blob/0.37.0/crates/eframe/Cargo.toml).

`rfd` propose des API synchrones et asynchrones, avec backends Linux GTK ou portail XDG. Le choix doit être testé sur la cible; le thread de rendu ne doit pas attendre une opération longue. [Documentation rfd](https://docs.rs/rfd/0.17.0/rfd/). L’accès direct au presse-papiers dispose de son propre adaptateur si nécessaire. [Documentation arboard](https://docs.rs/arboard/latest/arboard/).

## Rendu et fenêtrage

| Brique | Sélection SNS | Limite d’adoption | Exigences et lot |
| --- | --- | --- | --- |
| egui-wgpu 0.37 | Backend via eframe; dépendance directe seulement pour un callback GPU spécifique | Même famille/version/source que l’eframe choisi; aucune boucle GPU parallèle | F-TOP-008, F-OBS-005/006; RS-TOPOLOGY-J1/J7, RS-OBSERVATION-J7 |
| winit | Intégration transitive d’eframe; évaluation du fork `storytold/winit` si la pile Craft l’exige | Pas de fork ajouté isolément; cohérence de la famille egui et tests natifs | F-NFR-007, F-CFG-003/007; RS-PLATFORM-J7 |
| tiny-skia 0.12 | Cible pour rastériser la scène exportable et produire le PNG | Pas de capture de widgets comme format métier; limites de dimensions/mémoire explicites | F-DOC-003; RS-REPORTING-J7 |
| lyon_tessellation | Candidat pour remplissage/contour de tracés plus complexes | Pas requis pour dessiner les premiers équipements et câbles avec egui | F-TOP-008, F-DOC-003; évaluation dans les lots de rendu |

tiny-skia fournit des primitives 2D dont l’application gère transformations, clipping et styles; lyon tesselle les tracés. La scène et sa politique de coordonnées sont donc à développer dans SNS. [Documentation tiny-skia](https://docs.rs/tiny-skia/0.12.0/tiny_skia/), [documentation lyon_tessellation](https://docs.rs/lyon_tessellation/latest/lyon_tessellation/).

Le canevas de J1 commence avec les formes egui, un index spatial de sélection si les mesures le justifient, et une transformation monde/écran unique. Les câbles animés reflètent les événements; la fréquence de dessin ne détermine jamais l’évolution du réseau.

## Texte

| Brique | Rôle envisagé | Politique SNS |
| --- | --- | --- |
| skrifa | Métriques et contours des polices pour le rendu de document/export | Évaluer dans `sns-fonts`; ne pas exporter par copie approximative des pixels de l’UI |
| harfrust | Shaping OpenType : transformation du texte en glyphes positionnés | Évaluer avec le backend PDF et les annotations; versions alignées avec les API de police réellement utilisées |
| unicode-bidi | Ordre d’affichage des séquences mêlant écritures gauche/droite | Candidat pour documents multilingues; conserver les positions logiques pour sélection/édition |
| icu_segmenter | Opportunités de césure de lignes, mots et graphèmes | Ajouter les données nécessaires si le corpus le demande; la couverture thaï est une évaluation, pas une nouvelle exigence de parité |

Sources : [skrifa](https://docs.rs/skrifa/latest/skrifa/), [harfrust](https://docs.rs/harfrust/latest/harfrust/), [unicode-bidi](https://docs.rs/unicode-bidi/latest/unicode_bidi/), [icu_segmenter](https://docs.rs/icu_segmenter/latest/icu_segmenter/).

FR/EN/ES/IT/PT/DE sont les six langues conservées pour F-CFG-001. Les textes libres peuvent contenir d’autres écritures : documenter le support constaté et les caractères manquants. Une future extension de langue exige ses recettes. Installer un shaper ou un moteur bidi ne les branche pas automatiquement aux champs egui : le pipeline, le curseur, la sélection, l’IME, le fallback et les métriques d’export doivent être intégrés et testés explicitement.

## Briques craft-libs

| Brique | Position pour SNS | Frontière et validation |
| --- | --- | --- |
| craft-ui, fonctionnalité `serde` | Candidat prioritaire pour composants, panneaux et docking | Uniquement dans `sns-ui-egui`; les contenus des panneaux, commandes et modèles restent SNS. Tester sauvegarde/restauration d’un agencement sans effet sur le scénario |
| craft-window, fonctionnalité `egui` | Candidat pour gestion cohérente des cadres, viewports et fenêtres | Éviter de doubler la responsabilité d’eframe; tester fermeture, focus, DPI, multi-écran et changement d’écran |
| craft-update | Option ultérieure de distribution | Peut contacter un service réel : activation et politique séparées; son absence ne bloque ni le démarrage ni la simulation hors ligne |
| craft-pdf, backend Krilla | Candidat prioritaire pour l’écriture du rapport | Commencer par `default-features = false`, `write`; mise en page, pagination, tables et polices restent dans `sns-report` |
| craft-docpath | Candidat pour contrôler des chemins externes déclarés | Une autorisation de chemin ne donne pas accès au disque à une page simulée. Conserver les règles VFS, la validation ZIP/XML, les quotas et le choix explicite des fichiers |

Sources primaires : [craft-ui](https://github.com/storytold/craft-libs/blob/92d392cff132eaf53e13afff77f38b5c11b73aa0/crates/ui/Cargo.toml), [craft-window](https://github.com/storytold/craft-libs/blob/main/crates/window/Cargo.toml), [craft-update](https://github.com/storytold/craft-libs/blob/92d392cff132eaf53e13afff77f38b5c11b73aa0/crates/update/Cargo.toml), [craft-pdf](https://github.com/storytold/craft-libs/blob/92d392cff132eaf53e13afff77f38b5c11b73aa0/crates/pdf/README.md), [craft-docpath](https://github.com/storytold/craft-libs/blob/92d392cff132eaf53e13afff77f38b5c11b73aa0/crates/docpath/src/lib.rs).

Pour craft-pdf, choisir `write-simple-text` seulement si l’adaptateur utilise son shaper/`draw_text`; un pipeline fournissant ses glyphes n’en a pas nécessairement besoin. Le consommateur doit déclarer au workspace racine le patch Krilla correspondant à sa révision; les patches d’une dépendance ne se propagent pas au workspace appelant. L’écriture seule ne justifie pas de reprendre toute la chaîne de lecture/rastérisation PDF.

craft-docpath encadre l’ouverture de chemins stockés dans des documents, y compris certaines ressources locales/partages. SNS doit appliquer une politique plus restrictive aux scénarios : un fichier choisi pour import est copié dans le VFS; une référence embarquée ne devient pas une permission générale d’accès hôte. Cette brique ne constitue pas une sandbox d’extension ni un extracteur d’archive sûr à elle seule.

## Compatibilité : versions et sources

La référence CADCraft consultée patche **toute la famille egui** et winit, et pas seulement winit. Son manifeste indique également des révisions distinctes pour certaines briques Craft. Ces révisions sont des points d’étude, pas un verrouillage SNS :

| Source de référence | Révision observée | Usage dans le cadrage |
| --- | --- | --- |
| `storytold/egui` | `6d518fc799681571963842f95b55a6b68431b15a` | Évaluer la pile cohérente si craft-ui/fenêtres nécessitent ses API |
| `storytold/winit` | `a517b3846ee8457d84ee4975110bfda6797c44dc` | Évaluer avec cette même pile native |
| `storytold/craft-libs` | `92d392cff132eaf53e13afff77f38b5c11b73aa0` | Référence composants/fenêtres/PDF/chemins |
| `storytold/craft-libs` pour craft-update | `c5a1cbfb5f9049f9270955caffb5bee5997aebf7` | Référence distincte de l’option de mise à jour |

Source : [manifeste CADCraft consulté](https://github.com/storytold/cadcraft/blob/main/Cargo.toml). Les manifestes CADCraft et craft-libs annoncent une base Rust 1.95; le compilateur minimum réel de SNS devra être déterminé sur le graphe finalement choisi. Le nom « 0.37 » ne suffit pas à garantir la compatibilité entre deux sources Git et crates.io.

**Protocole J0 :**

1. Construire un petit adaptateur avec egui/eframe upstream 0.37 et les fonctionnalités explicites; vérifier police, tableaux, presse-papiers et backend de fenêtre.
2. Étudier les modules craft-ui/craft-window utiles à SNS et leurs API. Si des API du fork sont nécessaires, adopter et tester la famille complète sur des commits cohérents.
3. Déclarer les éventuels `[patch.crates-io]` au workspace racine. Un `rev` sur craft-ui seul ne force pas toutes ses dépendances à employer le fork attendu.
4. Inspecter `cargo tree -d` et `cargo tree -e features`, puis les inverses des familles concernées. Expliquer les doubles versions légitimes des bibliothèques de texte; rejeter les types egui incompatibles à une frontière commune.
5. Verrouiller `Cargo.lock`, le toolchain, les révisions et les plateformes testées; conserver la décision D-002 avec les résultats réels.

Le graphe upstream du socle est résolu et compilé localement. Les compatibilités des forks, des modules Craft et des dépendances proposées pour les prochains lots restent à démontrer.

## Schéma de configuration initiale

Extrait de sélection cible pour le workspace upstream; seule la sélection réellement déclarée dans Cargo.toml est compilée dans J0. Les dépendances Git Craft et leurs patches attendent la décision de compatibilité. Les fonctionnalités eframe devront être adaptées aux plateformes effectivement compilées.

```toml
[workspace.dependencies]
egui = { version = "0.37", default-features = false, features = ["default_fonts", "monochrome_emoji_fonts"] }
eframe = { version = "0.37", default-features = false, features = ["wgpu", "accesskit", "clipboard", "x11", "wayland"] }
egui_extras = { version = "0.37", default-features = false }
rfd = "0.17"
tiny-skia = { version = "0.12", default-features = false, features = ["std", "png-format"] }

# Dans les dev-dependencies de sns-ui-egui :
# egui_kittest = { version = "0.37", features = ["wgpu", "snapshot"] }
# Dans une intégration GPU spécifique seulement :
# egui-wgpu = "0.37"
```

Une table de dépendances au workspace ne les installe pas partout : chaque crate déclare ce qu’il utilise. Les polices de production et de snapshot doivent être identiques; les tests ne doivent pas activer silencieusement une chaîne de glyphes absente du binaire livré.

## Portes de validation

- **J0 — RS-QUALITY-J0 :** petit canevas natif, table de trace, saisie/focus, graphe de dépendances et preuve HTML lien/image/POST alimentée par la simulation. Consigner les limites plutôt que remplacer un formulaire par une simple vue texte.
- **J1/J2 :** intégrer `sns-ui-egui` aux commandes de topologie et d’hôtes; tests d’interaction egui_kittest et labels accessibles, cohérence zoom/sélection, arrêt/reprise sans bloquer l’UI.
- **J4 — RS-WEB-J4 :** renderer HTML pédagogique natif, ressources du VFS via HTTP simulé, aucun chargeur hôte ou ouverture de navigateur réel implicite.
- **J7 — RS-REPORTING-J7 / RS-PLATFORM-J7 :** PNG transparent, PDF paginé et polices cohérentes, dialogues/presse-papiers, fenêtres et accessibilité sur les trois OS.

Le [guide egui_kittest](https://github.com/emilk/egui/blob/0.37.0/crates/egui_kittest/README.md) distingue interactions et snapshots GPU. Les tests sans fenêtre réelle sont complétés par les essais natifs; les diffs d’image ne sont pas acceptés automatiquement par mise à jour des références.

## Dépendances réellement utilisées à J2

Le [lockfile](../../Cargo.lock) fixe notamment egui/eframe 0.37.0, rfd 0.17.2 et image 0.25.10. J2 active PNG/JPEG/GIF/BMP pour les images virtuelles, avec quotas de décodage. Les crates de domaine sns-apps/sns-vfs/sns-i18n restent sans GUI et sns-settings conserve la préférence de langue hors projet. Voir [J2](J2.md) pour le graphe et les limites des dialogues modaux. Aucun fork winit ni brique Craft supplémentaire n’a été introduit pour ce jalon.

## Dépendances J3

Le rendu des SVG fournis active uniquement `egui_extras/svg` et ses helpers CPU, sans image loader réseau. Le lockfile fixe resvg/usvg 0.45.1 et tiny-skia 0.11.4 sur ce chemin. Les graphismes sont intégrés au binaire avec leurs empreintes; les protocoles ne reçoivent aucune dépendance de rendu supplémentaire. Voir [J3](J3.md).

## Réalisation J6

Le courrier et le P2P réutilisent le TCP/DNS/VFS et SHA-256 déjà disponibles. Le nouvel adaptateur `sns-bridge` utilise uniquement les sockets et canaux bornés de la bibliothèque standard, sur un fil dédié. Le moteur ne dépend pas de cet adaptateur. Aucun runtime asynchrone ni dépendance réseau externe n’a été ajouté. Les SVG modem/point d’accès fournis par l’utilisateur portent le catalogue embarqué à 18 fichiers.

## Polices documentaires J7

`sns-fonts` est une bibliothèque locale de données, sans dépendance externe. Quatre fichiers Noto Sans/Noto Sans Mono Regular/Bold, non modifiés, sont fixés au commit `ffebf8c1ee449e544955a7e813c54f9b73848eac` de Noto. Leur [provenance et licence OFL-1.1](../../assets/fonts/README.md) accompagnent les fontes; les polices ne sont pas couvertes par la licence MIT du code SNS. La GUI les inscrit dans des familles d’annotation dédiées. `scripts/check_fonts.py` vérifie empreintes, poids et corpus latin des six langues hors ligne. L’ajout au lockfile concerne uniquement ce crate local; aucun nouveau paquet tiers ni changement de version externe. Les backends PNG/PDF et shaping restent à qualifier dans RS-REPORTING-J7.

## Rendu PNG J7

`sns-render` réutilise resvg/usvg **0.45.1** et tiny-skia **0.11.4**, sans fonctionnalités par défaut de resvg. HarfRust **0.12.0** et skrifa **0.44.0** deviennent des dépendances directes de `sns-fonts`; pollster **0.4.0** est utilisé pour attendre le dialogue asynchrone sur un fil GUI dédié. Ces versions existaient déjà dans Cargo.lock : aucun nouveau paquet tiers ni changement de version, seulement le crate local et les liens directs. Les images externes SVG et les fontes système ne sont pas résolues. La proposition tiny-skia 0.12 n’est pas présentée comme réalisée; [D-010](decisions/D-010-export-scene-j7.md) consigne le choix vérifié. Le backend CPU reste sans egui/wgpu, runtime réseau ni I/O implicite. Les bibliothèques et le backend PDF restent distincts.

## Réexamen de structure du 8 novembre 2026

[D-011](decisions/D-011-structure-et-workflow.md) compare les manifestes SolidCraft/WordCraft à des révisions figées. Les dépendances encore locales (XML, ZIP, validation de schéma et attente asynchrone) héritent désormais du workspace, sans changer leur résolution. `xtask` utilise uniquement la bibliothèque standard. Les forks et crates Craft des références restent des candidats motivés par un besoin, pas une nouvelle pile activée. Le [guide](DEVELOPPEMENT.md) et la [politique](layers.json) sont les points d’entrée courants.

## Rapport PDF J7

Krilla 0.8.2, défauts désactivés et `raster-images`, rejoint le graphe pour l’écriture
PDF dans `sns-report`. Les glyphes restent façonnés par SNS; pas de `simple-text`,
lecture de PDF, rayon ou patch Craft activé. Les versions internes de Skrifa et
tiny-skia-path sont séparées de celles de l’export PNG. [D-012](decisions/D-012-rapport-pdf-j7.md)
détaille le choix, les données qui traversent la frontière et les dépendances de
qualification indépendantes de l’application.
