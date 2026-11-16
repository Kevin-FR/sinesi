# Développer et vérifier l’adaptateur Rust

Lire d’abord [AGENTS.md](../../AGENTS.md), [l’état actuel](../suivi/ETAT.md), les
exigences et le lot `RS-*`. Le [processus portable](../agnostique/EVOLUTION.md)
définit la méthode; [D-011](decisions/D-011-structure-et-workflow.md) explique la
comparaison SolidCraft/WordCraft et les choix retenus.

## Une entrée pour les vérifications

Rust est fixé par `rust-toolchain.toml`. Python **3.11 minimum** est nécessaire
aux outils documentaires et à la lecture des manifestes; aucune dépendance Python
supplémentaire n’est requise par ces contrôles.

```sh
cargo fetch --locked             # préparation explicite du cache
cargo xtask check                # format, couches, catalogues/assets/fontes, outils, registres
cargo xtask layers               # graphe de production et tests négatifs de la politique
cargo xtask ci                   # check + Clippy + workspace + quatre preuves PNG décodées
cargo xtask ui                   # comparaison des captures PNG/PDF, aide, préférences et configuration J7 actuelles sur GPU
```

L’alias lance l’outil local `xtask`, sans bibliothèque tierce ni téléchargement
implicite. Les sous-commandes Cargo utilisent `--locked --offline`. La CI trois OS
prépare le cache puis appelle la **même** commande `ci`. `SNS_PYTHON` permet de
choisir un interpréteur, par exemple `SNS_PYTHON=python`. Le répertoire de travail
est toujours la racine du dépôt; les commandes utilisent des arguments structurés,
sans shell. Un contrôle en échec arrête la suite et renvoie un code non nul.

`ci` ne régénère pas les snapshots et n’exécute pas les corpus ignorés par défaut.
Les sous-processus de `xtask` ignorent `UPDATE_SNAPSHOTS`. Une mise à jour
voulue passe par un appel direct au test graphique concerné, puis par une revue
des images et du diff. Les échecs de comparaison peuvent produire des images de diagnostic.
Il ne qualifie ni un dialogue natif ni trois plateformes par sa seule présence dans
un workflow. `ui` couvre actuellement les tranches PNG/PDF, aide, préférences et configuration; la réconciliation de la suite
graphique historique complète reste ouverte et ne doit pas être masquée par ce nom.

## Choisir les tests

| Changement | Vérification appropriée en plus de `check` |
| --- | --- |
| Protocole ou moteur | Tests du domaine, déterminisme/rejeu et recettes touchées |
| Modèle ou format | Contrats, anciennes fixtures, entrée invalide et écriture sans perte |
| Présentation | Tests d’interaction du panneau et références rendues concernées; inspecter les images |
| Export | Scène/API et CLI, destination préservée sur échec, décodeur indépendant, inspection du résultat |
| Frontière ou refactor transversal | `ci`, graphe de dépendances et comparaison des références courantes |
| Intégration native/livraison | Parcours réel distinct sur les OS visés; joindre les artefacts et limites |

Les appels détaillés restent possibles, par exemple
`cargo test -p sns-engine --test routing --locked --offline` ou
`cargo test -p sns-ui-egui --test png --locked --offline`.
Ajouter une régression qui échoue sans la correction lorsque le changement porte
sur un défaut réel. Un déplacement de méthodes ne nécessite pas de recopier leurs tests.

## Ajouter une dépendance ou un module

Les versions, réglages par défaut et fonctionnalités communes vivent dans
`[workspace.dependencies]`. Les manifestes membres héritent avec
`workspace = true`, y compris pour les tests. Les fonctionnalités propres à un
adaptateur sont choisies par le consommateur. Le binaire bureau choisit les
backends natifs et l’accessibilité eframe. La présentation choisit les fontes
egui et le chargement SVG. Les fonctionnalités du harnais restent liées à
`ui-snapshots`.
Toutes les crates héritent des lints. Le lockfile reste l’autorité de résolution.
Version, édition, version minimale du compilateur, licence et dépôt héritent aussi
de `[workspace.package]`; `cargo xtask layers` refuse une divergence locale.
L’ajout d’une bibliothèque demande un besoin concret, licence, choix de features
et examen du graphe; les forks ne sont retenus que pour un problème reproduit.

La politique [layers.json](layers.json) déclare **tous les membres** et leurs arêtes
internes de production. Un membre inconnu ou une arête différente échoue. Les
dépendances normales, de build, conditionnelles et transitives sont examinées;
une dépendance uniquement de test est exclue de la frontière de production.
La résolution active `--all-features` pour tous les membres : une dépendance
optionnelle désactivée par défaut ne peut pas contourner l’isolation. Cela examine
le graphe; cela ne prétend pas compiler ni tester toutes les combinaisons de features.
La politique elle-même est validée avant le graphe : version, champs, catégories
et références internes inconnus sont refusés. Une faute dans `headless` ne doit
pas transformer une crate métier en adaptateur sans contrôle.
Les couches sans GUI refusent aussi une route indirecte vers UI, pont ou outil.
La politique ne détecte pas toute utilisation de l’API standard : l’absence de
socket réelle implicite reste également un invariant de revue et de recette.

## Refactor sans façade provisoire

Le produit expose désormais `Workbench`. `LanUi` est supprimé de l’API interne,
ses appelants sont adaptés et aucun alias ne le prolonge. `prototype` isole
l’expérience J0; elle reste exécutable par l’option explicite existante pour ses
recettes historiques, sans recevoir les nouvelles fonctions du produit.
La compatibilité des projets 1–9, scénarios, journaux et checkpoints reste couverte;
leur suppression nécessiterait une décision de format, indépendante de ce renommage.

[refactor-paths.json](refactor-paths.json) relie les anciens fichiers aux nouveaux.
Les `actual_paths` sont actualisés; les `planned_paths`, preuves et manifestes
historiques sont préservés. Les anciennes révisions se reconstruisent avec leurs
archives scellées, pas en recréant des fichiers relais dans le code courant.

Pour ouvrir un fichier, `document.rs` reçoit un chemin du sélecteur asynchrone
ou d’un test. Il valide le projet hors frame ou transmet le chemin au traitement
d’import ou d’exécution existant. L’application du résultat vérifie le mode,
l’époque du worker et la révision du projet. Nouveau, Ouvrir, Exemples et Import
partagent l’installation d’un projet validé.

Les prochains sélecteurs à extraire concernent l’enregistrement, les exports
encore synchrones et les fichiers des applications simulées.
La [revue du 9 novembre](REVUE_STRUCTURE_2026_11.md) précise les extractions
restantes, leurs propriétaires et leurs critères de remplacement.
L’aide et les exports PNG/PDF sont désormais qualifiés dans leurs tranches J7.
Ne pas ouvrir un second moteur, un bus générique ou une dépendance Craft pour
remplacer une responsabilité déjà possédée par une crate existante.

## Vérification indépendante des rapports

L’application et `ci` n’ont pas besoin de Python tiers. Pour contrôler aussi les
PDF avec un lecteur indépendant, utiliser un environnement Python de qualification
et `python -m pip install -r scripts/requirements-pdf-verification.txt`, puis :

```sh
python scripts/check_pdf_report.py docs/verification/j7-report-fr-diagram.pdf --metadata docs/verification/j7-report-fr-diagram.json
```

Cette dépendance pypdf est réservée à la vérification. Les tests Rust produisent les
preuves avec `SNS_RECORD_J7_REPORT="$PWD/docs/verification" cargo test -p sns-report --locked --offline`.
Poppler rend les pages pour l’inspection, séparée de l’extraction textuelle.
