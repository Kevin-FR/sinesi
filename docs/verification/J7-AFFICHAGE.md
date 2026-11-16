# J7 — préférences d’affichage

**F-CFG-003 vérifié; RS-PLATFORM-J7 reste en cours.** Le
[profil portable](../agnostique/AFFICHAGE_J7.md), le
[contrat](../../contracts/preferences.schema.json) et l’[adaptateur](../rust/AFFICHAGE_J7.md)
fixent le périmètre. F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés.

## Résultat

Le profil utilisateur 2 conserve langue, fonte standard/Noto Sans, taille 80–200 %,
Icônes/Schéma, panneaux gauche/droite/onglets, largeurs, dimensions et géométrie
bornée. Le format 1 est migré en mémoire sans réécriture. Les six langues restent
explicites. Un fichier invalide ou futur est préservé; la sauvegarde valide avant
l’écriture atomique. Les préférences, le projet, la récupération et les exports
restent protégés les uns des autres, y compris avec parents absents et lien symbolique.

Un worker d’écriture unique et une valeur suivante complète coalescent les demandes;
la fermeture normale attend leur achèvement. La langue conserve les autres champs.
Les préférences ne commandent pas le moteur. Le test compare projet, fichier source,
trace, temps et identité d’exécution après modification d’affichage.

Les panneaux s’adaptent à la largeur. Les fenêtres d’aide et d’applications sont
mémorisées par catégorie et contraintes à l’espace visible. Les actions d’annotation
précèdent les champs de style; les commandes du terminal reviennent à la ligne à
200 %. Le suivi de taille est arrêté par un déplacement/zoom manuel ou une vue
récupérée. Les familles d’annotation et les ressources des exports sont conservées.

## Vérifications exécutées

Environnement : macOS arm64, Rust 1.95.0, Python 3.14.4, egui 0.37.0/wgpu.
[L’environnement détaillé](j7-display-environment.json) conserve aussi la comparaison
au dernier manifeste et du graphe tiers.

| Commande | Résultat et preuve |
| --- | --- |
| `cargo xtask ci` | **247 tests workspace et un enfant du pont**, cinq ignorés; Clippy sans avertissement; contrôles de format/couches/contrats/catalogues/provenance et quatre PNG décodés indépendamment. [Journal](j7-display-ci.log) |
| `cargo xtask ui` | **15 tests de la suite graphique**, **96 captures comparées sans mise à jour**. [Journal](j7-display-gpu.log) |
| `cargo test -p sns-ui-egui --test lan j7_trace_queries_exports_and_local_global_animation_preserve_six_languages_and_views --locked --offline -- --exact` | **Dix répétitions réussies**, chacune sur six langues. [Journal](j7-display-runtime-stress.log), [résultats](j7-display-runtime-stress.json) |
| `cargo test -p sns-ui-egui --test preferences terminal_remains_usable_at_200_percent_in_all_languages_without_advancing_protocols --locked --offline -- --exact` | Reproduit le débordement avant correction, puis réussit dans les six langues avec une assertion de géométrie. [Avant](j7-display-terminal-bounds-before.log), [après](j7-display-terminal-bounds-after.log) |
| `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check --require-reference` | Registres/vues, références et empreintes du corpus contrôlés. [Journal](j7-display-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-display-code-manifest.json` | Inventaire courant identique au manifeste |

Les tests de `sns-settings` comparent schéma et lecteur, migration des six langues,
champs invalides, quotas, conservation des destinations et toutes les catégories de
fenêtres. Les six tests du panneau couvrent application/annulation, raccourci avec
saisie active, deux vues, redémarrage, tailles 150/200 %, géométrie après changement
de viewport, chemin de projet protégé, création de dossiers et protection de la
récupération, également à travers un lien symbolique Unix. Les recettes de session
et d’export existantes continuent de passer.

## Rendu et échecs corrigés

Le bouton Préférences et les fenêtres adaptatives changent le rendu courant. Les
96 nouvelles références `j7-display-*` comprennent 66 vues PNG/PDF/aide actualisées,
12 panneaux de préférences à 150 %, 12 topologies à 200 % et six terminaux à 200 %.
Les **387 images historiques restent identiques octet pour octet**. La génération
initiale est consignée [ici](j7-display-gpu-initial.log) et la correction finale du
terminal [ici](j7-display-terminal-gpu-initial.log); la comparaison finale n’active
pas `UPDATE_SNAPSHOTS`. Les sauvegardes provisoires des nouvelles captures du
terminal sont retirées après vérification de leur absence du manifeste historique.

Inspection visuelle : préférences FR/Icônes à 150 %, topologie DE/Schéma à 200 %,
aide FR et terminal IT à 200 %, puis terminal IT après correction. Le terminal
avait une action hors fenêtre : une zone de défilement horizontal lui donnait une
largeur de mise en page excessive. La largeur visible borne désormais son retour
à la ligne; le test exige que l’action complète reste dans le rectangle de fenêtre.
Le zoom du réseau reste distinct du zoom d’interface et peut être ajusté pour lire
les libellés après cadrage global.

La [première passe graphique](j7-display-initial-gpu-failure.log) a montré une
annotation hors écran après application différée des dimensions natives. Le
recadrage suit désormais les dimensions réellement appliquées; la recette PNG
originale passe, sans modification de son assertion métier.

Une [passe complète initiale](j7-display-initial-ci-failure.log) a aussi révélé
l’hypothèse de restauration en 400 ms d’une recette. La boucle attend maintenant
l’état appliqué avec un délai maximal de huit secondes et une erreur explicite.
Les dix répétitions et la passe complète finale réussissent. Les temporisations
réseau restent virtuelles; aucun délai de protocole n’est modifié.

## Révision et reconstruction

[Manifeste](j7-display-code-manifest.json) : **766 fichiers**, révision
`sha256:a0dc67c2a795237294f5c51696a0502a879ff20bbe12aafa4a8e1a9388d04ea0`.
Le [delta](j7-display-source-delta.json) se superpose à la base d’import selon ses
instructions. Les anciens rapports, manifestes et sources scellées sont conservés.
Aucun paquet tiers ajouté ou retiré; `jsonschema`, déjà résolu, devient également
une dépendance de test de `sns-settings`. Les 20 membres, 799 clés × six langues,
20 SVG et quatre fontes restent vérifiés.

## Limites et prochaine tranche

Cette qualification porte sur le harnais et le système local. Les déplacements
manuels de la fenêtre native principale ne sont pas enregistrés : ses dimensions
proviennent du panneau. Les fenêtres d’une même catégorie partagent leur géométrie
initiale. La suite GPU historique complète et l’accessibilité globale restent à
réconcilier; aucune qualification Windows/Linux, manipulation native autonome ou
CI distante n’est déduite des captures.

F-CFG-004/005 (priorités et lancement) puis diagnostics, exemples et livraison
restent à traiter, ainsi que l’import sémantique encore partiel. J7 reste ouvert.
Les archives restent en lecture seule; aucun audit, commit, push ou publication.
