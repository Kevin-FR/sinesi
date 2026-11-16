# Qualification J7 — pannes par liaison

Qualification locale du **8 novembre 2026**, macOS arm64, Rust 1.95.0. F-SIM-008 est qualifié avec le [profil portable](../agnostique/PANNES_J7.md) et l’[adaptateur](../rust/J7-EN-COURS.md). F-SIM-007 reste vérifié : la reprise conserve aussi les nouveaux flux aléatoires. Le lot RS-ENGINE-J7 est fermé; le jalon J7 complet reste ouvert.

## Commandes exécutées

| Commande | Résultat |
| --- | --- |
| `SNS_RECORD_J7_LINKS="$PWD/docs/verification" cargo test --workspace --locked --offline` | **180 tests du workspace**, plus **1 test enfant** du pont; 3 ignorés par défaut; [journal](j7-links-workspace.log) |
| `cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_link_conditions --locked --offline` | 1 parcours rendu dans **six langues et deux vues**, **12 captures comparées sans mise à jour**; [journal](j7-links-gpu.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi sans avertissement; [journal](j7-links-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-links-fmt.log) |
| `python3 scripts/check_i18n.py`; `check_network_assets.py`; `check_layers.py` | **604 clés × six langues**, 18 SVG et frontières vérifiés; [langues](j7-links-i18n.log), [assets](j7-links-assets.log), [couches](j7-links-layers.log) |
| `python3 scripts/docs.py render`; `python3 scripts/docs.py check --require-reference` | Registres, dépendances, liens et séparation vérifiés; [journal](j7-links-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-links-code-manifest.json` | **444 fichiers source**, révision `sha256:1da9f21252dc17f6397dbb1b485c34835a22e4e0c630e48102152efa8607ae88` |

L’[environnement](j7-links-environment.json), le [manifeste](j7-links-code-manifest.json) et le [complément de sources](j7-links-source-delta.json) identifient la révision. Le paquet initial et le complément permettent de reconstruire les sources qualifiées sans remplacer les preuves précédentes. Le corpus Filius n’est pas redistribué.

## Critères de F-SIM-008

`sns-engine/tests/link_conditions.rs` utilise la pile réelle et deux liaisons indépendantes :

- **100 %** bloque la première paire; la paire témoin reçoit **16 réponses**. À **0 %**, chaque paire reçoit ses **16 réponses**. Les abandons ne concernent que la liaison configurée.
- Les délais choisis sont entre 2000 et 3000 µs; chaque réception causée par une transmission respecte exactement ce délai.
- Les tirages/délais de la paire témoin restent identiques lorsque les pertes de l’autre liaison changent. Une autre graine les modifie; une même graine et un ordre différent des équipements/liens gardent la même trace.
- Un checkpoint pris avec des actions futures et des flux déjà avancés reprend la même trace et le même état réseau. Le journal complet et la continuation du checkpoint reproduisent les décisions.
- Paramètres hors limites et identités inconnues sont refusés sans mutation. Une réparation en cours d’exécution rétablit les réponses. Une saturation de trace ne consomme ni tirage ni entrée de file.
- La fixture d’un checkpoint non configuré de la tranche précédente reste lisible : les anciennes politiques ne sont pas modifiées en l’absence de conditions explicites.

`sns-project/tests/contracts.rs::j7_link_conditions_commands_events_scenarios_and_random_state_match_contracts` valide commande API 1, faits 9, scénario, journal et checkpoint. La somme des durées et les références restent aussi soumises à la validation sémantique.

Le parcours GUI applique 100 %, puis 0 %, par les champs et boutons dans FR EN ES IT PT DE. Il contrôle délai/gigue, conservation du projet, export et restauration des conditions. Les vues Icônes/Schéma sont conservées. Le rendu FR à 100 % et DE à 0 % a été inspecté : textes lisibles, identité complète de liaison, couleur/taux du câble et indication globale cohérente.

## Exercice et artefacts

La [recette CLI](j7-links-cli.json) consigne quatre invocations dans des processus distincts. L’[exercice](../../fixtures/valid/link-conditions-scenario.json) utilise 50 % de pertes sur une paire : **2 réponses** sur 16; l’autre paire reçoit **16 réponses**. La trace comporte **388 événements**, dont **14 pertes configurées**, empreinte `24df46be736d5b10ff09451514822badbde6f00398ffbc34b343736f5ef55600`.

Les traces [complète](j7-links-cli-full.jsonl), [après reprise](j7-links-cli-resumed.jsonl) et [rejouée](j7-links-cli-replayed.jsonl) sont identiques. Le [checkpoint](j7-links-cli-checkpoint.json) et le [journal](j7-links-cli-recording.json) permettent la reproduction. La recette moteur fournit aussi son [checkpoint actif](j7-links-checkpoint.json), son [journal](j7-links-recording.json) et les traces [initiale](j7-links-original.jsonl)/[restaurée](j7-links-restored.jsonl).

## Limites et suite du jalon

Les conditions sont des paramètres d’exécution, pas de nouvelles propriétés implicites du projet. Elles portent sur les prochains envois filaires; elles ne modifient pas une trame déjà en vol, la propagation Wi-Fi ou le réseau OS. Le taux porte sur les trames, donc pas directement sur le pourcentage de réponses ping.

Les captures précédentes restent conservées dans leurs preuves. La suite graphique historique complète doit encore être réconciliée avec l’évolution de la barre d’outils et des fenêtres avant la livraison; cette passe ne la présente pas comme verte. Aucun résultat Windows/Linux ni installation finale n’est revendiqué.

J7 conserve son périmètre entier : import à approfondir, duplication/alignement et nuage du switch, observation, annotations et exports, récupération/session, configuration et diagnostic, exemples complets, packaging et validation des trois systèmes restent à terminer.
