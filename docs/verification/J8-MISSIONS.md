# J8 — missions évaluables

F-ADV-004 dans RS-UX-J8. Le [profil portable](../agnostique/MISSIONS.md),
[D-016](../rust/decisions/D-016-missions.md) et les contrats
[mission](../../contracts/mission.schema.json) /
[correction](../../contracts/mission-report.schema.json) fixent la portée.

## Résultat

- Le crate sans interface `sns-missions` lit, valide et évalue les missions version 1
  et produit la correction exportable. Il dépend de `sns-model` et `sns-engine`;
  la politique de couches le déclare.
- Trois missions intégrées, traduites dans les six langues : « Le câble oublié »,
  « La passerelle oubliée » et « L’annuaire en panne ».
- Recette F-ADV-004 sur simulation réelle : sans réparation, les trois objectifs de
  la panne DNS sont incomplets et expliqués; avec le service démarré seulement, un
  seul objectif est atteint; une fois le serveur DNS corrigé, `host www.ecole.test`
  reçoit une réponse et la mission est réussie avec trois étoiles.
- Atelier : bouton « Missions », carte par chapitre avec étoiles et verrouillage,
  carte de mission sous la consigne (objectifs, Vérifier, indice à −1 étoile,
  mission suivante, quitter), étoiles enregistrées dans le profil. « Le voyage du
  paquet » raconte aussi la question DNS, la réponse et l’échec de résolution.
- `sinesi mission-check` produit la même correction (code 0 si réussie, 2 sinon).

## Commandes et tests

Environnement : macOS 27.2 arm64, Apple M2 Max, Rust 1.95.0, egui 0.37.0 et wgpu.

| Recette | Test |
| --- | --- |
| R-MIS-001 | `crates/sns-missions/tests/contracts.rs`, `invalid_missions_are_refused` |
| R-MIS-002 | `repaired_dns_fault_is_validated_and_the_failure_is_explained` |
| R-MIS-003 | `missing_gateway_is_explained_then_repaired_with_a_real_ping`, `forgotten_cable_needs_the_link_and_a_reply` |
| R-MIS-004 | `a_mission_is_started_explained_repaired_scored_and_unlocks_the_next` (six langues, références `j8-mission-*`) |
| R-MIS-005 | `crates/sns-cli/tests/missions.rs` |

Exécutés : `cargo test -p sns-missions`, `cargo test -p sns-cli --test missions`,
`cargo test -p sns-settings` (étoiles facultatives, bornées, profil inchangé si vide),
suite `student`, puis la validation complète : `cargo xtask check`, `cargo xtask layers`,
Clippy workspace avec et sans captures, `cargo test --workspace`, suite graphique
complète et `cargo xtask ui`.

## Constats de mise au point

- Le service DNS n’est pas démarré par le projet (comme dans le référentiel) : la
  première version du test échouait pour cette raison. La mission en fait un
  objectif plutôt que d’étendre le format de projet.
- Passer à la mission suivante après une réparation non enregistrée affiche la
  confirmation habituelle; le test choisit d’abandonner les modifications.
- L’explication d’un objectif restait affichée après sa réparation; elle est
  maintenant liée à l’objectif et disparaît quand il est atteint.

## Corrections associées

- Placement : un équipement ajouté depuis la palette prend la première case libre
  d’une grille alignée sur le réseau (`sns-topology::free_position`, test sur douze
  ajouts successifs sans chevauchement).
- Fenêtre de messagerie : le contenu suit la largeur réelle de la fenêtre; la zone de
  texte ne déborde plus horizontalement (écart antérieur relevé en tranche 1).
- Capture de mission réussie stabilisée : les surbrillances de câble passagères
  expiraient selon le nombre d’images; le test les laisse finir avant la capture.
  La suite `student` avec captures a été rejouée trois fois sans écart.

Validation finale : `cargo xtask check`, `cargo xtask layers`, Clippy workspace avec
et sans captures, `cargo test --workspace`, suite graphique complète et
`cargo xtask ui` (33 tests) en succès; le manifeste de l’archive J0–J7 est intact.

## Missions de l’enseignant

« Ouvrir une mission de l’enseignant… » dans la carte des missions passe par le
chargement de document asynchrone existant. Tests `teacher_mission_is_validated_off_frame_installed_and_started_in_the_class_section`
et `invalid_or_stale_teacher_missions_preserve_the_current_work` (R-MIS-006).

## Limites

Preuves locales macOS par harnais; pas de recette native ni
trois OS. F-ADV-004 est vérifiée localement; la [recette native macOS](J8-NATIF-MACOS.md)
complète le harnais.
