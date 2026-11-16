# J7 — lancement et exemples

RS-PLATFORM-J7 couvre F-CFG-005 et F-CFG-008. Les profils de démarrage, le
facteur d’attente et le journal ont aussi leurs propres recettes.
[Configuration portable](../agnostique/CONFIGURATION_J7.md) ·
[Attentes RTT](J7-ATTENTES.md) · [Diagnostic](J7-DIAGNOSTIC.md)

Le lanceur affiche les alias n/s dans les six langues. Son test injecte le
thème hôte et vérifie le passage au clair puis au sombre. Le contrôle de
configuration vérifie le profil compact effectif, la priorité des options
explicites et les préférences persistantes. Il produit un rapport pour un
projet passé en argument.
[Rapport compact Schéma FR](j7-launch-examples-startup.json)

Le harnais parcourt les onglets à 840 × 600 puis à 640 × 480. Il vérifie
la police, l’échelle, la représentation et l’état de la simulation. Douze
références en thème clair couvrent les six langues et les deux vues.
[Capture Icônes FR](../../crates/sns-ui-egui/tests/snapshots/j7-compact-light-icons-fr.png) ·
[Capture Schéma DE](../../crates/sns-ui-egui/tests/snapshots/j7-compact-light-diagram-de.png)

Le catalogue ouvre 14 projets. Le test du menu exerce la liaison directe,
l’atelier ARP et les deux projets modem dans les six langues. Le test du pont
charge la paire distribuée, transmet un ping entre deux processus en boucle
locale et rejoue la trace enregistrée. Le test de liaison directe observe
ARP et une réponse ICMP sur le câble du projet.
[Trace directe](j7-launch-examples-direct.jsonl) ·
[Recettes exécutables](../rust/EXEMPLES_J7.md)

Le contrôle graphique utilise les signaux de thème du harnais. La
qualification du système de fenêtres et la livraison sur trois OS restent
dans les parcours de plateforme.

## Validation locale

| Commande exécutée | Résultat |
| --- | --- |
| `cargo xtask ci` | Clippy sans avertissement. 284 tests du workspace et un enfant du pont réussis, cinq ignorés. Les treize recettes pédagogiques apparaissent dans le journal. [CI locale](j7-launch-examples-ci.log) |
| `env -u UPDATE_SNAPSHOTS cargo xtask ui` | 25 tests et 132 références comparées. Les 508 fichiers PNG antérieurs correspondent à leur manifeste. [Harnais graphique](j7-launch-examples-gpu.log) |
| `python3 scripts/docs.py render` puis `python3 scripts/docs.py check --require-reference` | Registres, vues et empreintes du référentiel vérifiés. [Journal](j7-launch-examples-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-launch-examples-code-manifest.json` | Inventaire scellé de 821 fichiers. [Manifeste](j7-launch-examples-code-manifest.json) |

Environnement macOS arm64, Rust 1.95.0, egui et wgpu.
[Environnement et périmètre des fichiers](j7-launch-examples-environment.json)

La révision `sha256:dbec2070218bce118ba0d0cb90410a77775e1911d581f340e567a20471bcf59b` est reconstructible
avec le [correctif d’inventaire](j7-launch-examples-source-patch.json).
