# Qualification du refactor d’ouverture

La [comparaison des quatre sources](../rust/REVUE_STRUCTURE_2026_11.md) conserve
leurs révisions figées. Le sélecteur asynchrone transmet le chemin au chargeur
validé ou aux workers d’import et d’exécution. Un résultat devenu périmé est
abandonné avant installation. Nouveau, Ouvrir, Exemples et Import partagent
l’installation du projet.

Environnement. macOS 27.2 arm64, Rust 1.95.0, egui 0.37.0/wgpu, Python 3.14.4. Chemins injectés par tests. Mac verrouillé, dialogue natif non manipulé.

| Commande | Résultat |
| --- | --- |
| `cargo xtask ci` | Clippy réussi, 287 tests workspace et un enfant du pont, 5 ignorés. Les trois tests d’ouverture couvrent attente, concurrence, installation, annulation, erreur, résultat périmé et aiguillage vers l’import Filius et le scénario. [Journal](structure-document-open-ci.log) |
| `cargo xtask ui` | 25 tests du harnais de rendu. Les 520 PNG de l’inventaire conservent leurs empreintes. [Journal](structure-document-open-ui.log) |
| `python3 scripts/docs.py render` puis `python3 scripts/docs.py check --require-reference` | Registres, vues et empreintes du référentiel contrôlés. [Journal](structure-document-open-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/structure-document-open-code-manifest.json` | 821 fichiers contrôlés. [Manifeste](structure-document-open-code-manifest.json) |

Le catalogue ajoute un libellé dans chacune des six langues. La suite graphique
courante conserve les deux vues. Les sélecteurs d’enregistrement, certains exports
et les sélecteurs des applications simulées restent à traiter séparément.
Les statuts fonctionnels et les cibles prévues restent ceux des registres.
Le [correctif de reconstruction](structure-document-open-source-patch.json)
se superpose à la qualification précédente.
