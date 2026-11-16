# J6 — Revalidation dans le checkout en cours

Le 7 novembre 2026, macOS arm64, Rust 1.95.0. Le dépôt contient déjà le courrier, le P2P, le Wi-Fi logique et le modem J6. Cette passe contrôle ces fonctionnalités dans le checkout comprenant également des travaux J7. La [qualification initiale J6](J6.md) et ses captures restent historiques.

## Résultats réellement exécutés

| Commande | Résultat |
| --- | --- |
| `cargo test -p sns-engine --test mail --test p2p --test wireless -p sns-bridge --test bridge --locked --offline` | 16 tests réussis, plus un test enfant du pont réel; [journal](j6-revalidation.log) |
| `cargo test -p sns-ui-egui --test lan j6_ --locked --offline` | 4 parcours réussis : courrier, P2P, SSID et modem; FR EN ES IT PT DE et vues Icônes/Schéma; [journal](j6-revalidation-functional-ui.log) |
| `cargo test -p sns-project --test contracts j6_ --locked --offline` | 1 test réussi : projets, commandes, données persistantes et faits J6; [journal](j6-revalidation-contracts.log) |
| `cargo fmt --all --check` | Réussi; [journal](j6-revalidation-fmt.log) |
| `python3 scripts/check_i18n.py` | 575 clés présentes dans chacune des six langues |
| `python3 scripts/check_network_assets.py` | 18 SVG embarqués et provenance vérifiés |
| `python3 scripts/check_layers.py` | Frontières du noyau vérifiées |

Le [manifeste courant](j6-revalidation-code-manifest.json) et l’[environnement](j6-revalidation-environment.json) identifient cette passe. Aucun fichier de production n’a été modifié par cette revalidation.

## Comparaison graphique historique

`cargo test -p sns-ui-egui --features ui-snapshots --test lan j6_ --locked --offline` : **4 échecs de comparaison d’images**; [journal complet](j6-revalidation-ui.log). Les références J6 précèdent le bouton « Importer Filius… » ajouté par J7. Les contrôles fonctionnels passent séparément; le rendu courant ne peut donc pas être annoncé identique aux anciennes captures. La différence française du client courrier a été inspectée : elle concerne la barre d’outils, les contrôles courrier restent lisibles. Les [bornes des différences](j6-revalidation-image-differences.json) consignent les 37 captures concernées : tous les pixels modifiés sont compris entre les lignes 17 et 36 incluses de la barre d’outils.

Les anciennes images n’ont pas été remplacées et aucune mise à jour automatique n’a été activée. La suite graphique globale et la suite complète du workspace ne sont pas déclarées réussies par cette passe ciblée. La qualification initiale reste liée à sa propre révision. Les travaux et tests de reprise J7 demeurent en cours.

## Suivi

Les seize exigences et les quatre lots J6 conservent leurs preuves initiales; une preuve supplémentaire de revalidation fonctionnelle ciblée est ajoutée dans les deux registres. Cette passe ne ferme aucune exigence J7 et ne revendique aucune exécution Windows/Linux.
