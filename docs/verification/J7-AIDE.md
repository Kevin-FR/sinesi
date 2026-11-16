# J7 — aide contextuelle qualifiée localement

**F-CFG-002 est vérifié. RS-PLATFORM-J7 reste en cours** pour les six autres
exigences du lot. Le [profil portable](../agnostique/AIDE_J7.md) décrit le comportement;
[l’adaptateur](../rust/AIDE_J7.md) décrit le catalogue, les modules et la reprise.

## Réalisation

F1 et le bouton d’aide ouvrent un guide hors ligne dans le contexte du mode actif.
Conception, Simulation et Documentation ont leurs parcours; logiciels, exemples et
À propos sont accessibles sans relancer le logiciel. L’ouverture, la navigation
et la fermeture n’émettent aucune commande réseau et ne modifient pas les données.
Une simulation déjà active peut continuer indépendamment.

À propos affiche la version produit **0.1.0**, le format du projet ouvert (ici 2),
le format documentaire le plus récent (9) et la plage lisible (1–9). Le numéro
récent est partagé avec la validation et l’évolution des annotations; aucun format
persistant n’est modifié. Les crédits séparent code, éléments graphiques fournis,
polices Noto, dépendances et référence Filius. Les notices du code et des fontes
sont embarquées et consultables sans navigateur; aucun nom de personne n’est inféré.

Les dix guides d’exemples réutilisent les chargements existants et leur confirmation
de document non enregistré. Cette navigation ne ferme pas F-CFG-008 : les recettes
complètes de toutes les familles restent à fournir.

F1 est traité avant le garde de focus texte. Échap est consommé quand il ferme
l’aide, afin de conserver le brouillon et les interactions derrière la fenêtre.
Le catalogue et les actions restent distincts : aucune lecture de fichier ou
fonction GUI n’est exécutée par les données de description.

## Vérifications exécutées

Environnement : macOS arm64, Rust 1.95.0, egui 0.37.0/wgpu. Voir
[l’environnement](j7-help-environment.json). Aucune nouvelle dépendance tierce.

| Commande réelle | Résultat / preuve |
| --- | --- |
| `cargo xtask ci` | **237 tests du workspace plus un enfant du pont**, cinq ignorés, Clippy sans avertissement, formatage et contrôles communs réussis. [Journal](j7-help-ci.log) |
| `cargo xtask ui` | **9 tests GPU**, **66 captures comparées sans mise à jour**, dont **42 nouvelles pour l’aide**. [Journal](j7-help-gpu-final.log) |
| `cargo test -p sns-settings --locked --offline` | Trois tests, dont parcours de toutes les rubriques et des dix exemples dans les six langues. [Journal ciblé](j7-help-settings.log); la suite finale les rejoue |
| `cargo test -p sns-ui-egui --test help --locked --offline` | Trois parcours : contexte/versions sans mutation, F1/Échap avec saisie active, protection du document avant un exemple; rejoués dans `ci` et `ui` |
| `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check --require-reference` | Vues, états, liens et empreintes de l’archive vérifiés. [Journal](j7-help-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-help-code-manifest.json` | Inventaire final identique à la révision scellée |

Les modes sont testés dans **les six langues et les deux vues**. La première vue
utilise F1, la seconde le bouton, puis vérifie la même rubrique. Le projet, la
sélection, le run, l’horloge et la trace sont comparés après ouverture. Un champ
modifié mais non appliqué est conservé pendant F1/Échap. Le chargement d’un exemple
est annulé une fois avec conservation du document, puis confirmé avec retour en
Conception au temps 0. Aucun sélecteur natif n’est nécessaire à ces parcours.

Les captures montrent les trois modes dans chaque langue/vue et À propos dans
chaque langue. FR/Conception/Icônes et DE/À propos/Schéma ont été inspectés. Les
**345 images précédentes** restent identiques. Les suites PNG/PDF courantes sont
également comparées. Le contrôle du catalogue porte maintenant **776 clés × six
langues** et inclut `sns-settings`; il a révélé un diagnostic de version de
préférences précédemment non traduit, désormais présent dans les six langues.

## Révision et reconstruction

[Manifeste](j7-help-code-manifest.json) : **663 fichiers**, révision
`sha256:73ca4e8b9d3f973181bd9df48caa71958fa364c41ed0dae170d639332befd48e`.
Le [delta de sources](j7-help-source-delta.json) s’applique à l’archive initiale
d’import, selon ses métadonnées. Les preuves et sources précédentes restent
conservées; les archives Filius/network-kit ne sont ni modifiées ni redistribuées.

La preuve comprend le nouveau test
`crates/sns-ui-egui/tests/help.rs`, le catalogue dans
`crates/sns-settings/src/help.rs` et les références `tests/snapshots/j7-help-*`.
Les chemins prévus des lots restent inchangés; les réalisations sont dans leurs
`actual_paths`. Le code source ne contient plus le booléen d’aide ni le panneau
monolithique précédent.

## Limites et suite

Le harnais vérifie les interactions et le rendu local; il ne qualifie pas une
navigation native autonome, Windows/Linux, IME ou le parcours trois OS. La suite
graphique historique complète reste à réconcilier et la lisibilité avec police
agrandie persistante relève de F-CFG-003. Les crédits ne remplacent pas le bundle
complet de notices de distribution. Configuration globale, options, diagnostics,
exemples complets et packaging restent ouverts. J7 compte **14 exigences vérifiées
sur 22**; son objectif global n’est pas terminé.
