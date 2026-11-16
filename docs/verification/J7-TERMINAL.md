# Qualification J7 — diagnostic terminal interne

Qualification du **8 novembre 2026**, macOS arm64, Rust 1.95.0. F-CLI-019 et RS-TERMINAL-J7 sont vérifiés selon le [profil portable](../agnostique/DIAGNOSTIC_TERMINAL_J7.md) et l’[adaptateur](../rust/J7-EN-COURS.md). J7 reste ouvert.

## Chemin historique lu

Dans l’archive en lecture seule, `Terminal.java:616` définit l’écriture de `test.txt` avec `blaaa`. L’inventaire `initUsageInfo`, ligne 81, ne contient pas ce helper. Le chemin `terminalEingabeAuswerten`, ligne 1057, vérifie pourtant l’existence de la méthode puis la transmet à la file d’exécution. `Anwendung.java:164` l’enfile et la boucle, ligne 225, l’invoque. Le diagnostic est donc historiquement appelable même sans entrée d’aide. Aucun fichier de l’archive n’a été modifié et aucun runtime historique n’a été lancé.

## Contrôles exécutés

| Commande | Résultat |
| --- | --- |
| `SNS_RECORD_J7_TERMINAL="$PWD/docs/verification" cargo test -p sns-engine --test applications j7_historical --locked --offline` | 1 recette réussie; [journal](j7-terminal-test.log) et [artefact](j7-terminal-diagnostic.json) |
| `SNS_RECORD_J7_TERMINAL="$PWD/docs/verification" cargo test --workspace --locked --offline` | **198 tests du workspace**, plus **1 enfant** du pont; trois ignorés par défaut; [journal](j7-terminal-workspace.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi sans avertissement; [journal](j7-terminal-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-terminal-fmt.log) |
| `python3 scripts/check_i18n.py`; `check_network_assets.py`; `check_layers.py` | 651 clés × six langues, 20 SVG et frontières vérifiés; [langues](j7-terminal-i18n.log), [assets](j7-terminal-assets.log), [couches](j7-terminal-layers.log) |
| `python3 scripts/docs.py render`; `python3 scripts/docs.py check --require-reference` | Registres, preuves et références vérifiés; [journal](j7-terminal-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-terminal-code-manifest.json` | **498 fichiers**, révision `sha256:2da4353ff9c13f5286afddda344a163279be35a42741bf54cf1ee201c7b043c0` |

L’[environnement](j7-terminal-environment.json), le [manifeste](j7-terminal-code-manifest.json) et le [complément de sources](j7-terminal-source-delta.json) identifient les sources de cette passe.

Le test `j7_historical_file_probe_is_diagnostic_only_and_student_test_commands_are_rejected` utilise le moteur et le parseur réels. Les trois saisies produisent trois diagnostics `terminal.unknown_command`; fichiers et réseau restent identiques, sans fait de fichier ou Ethernet. Chaque nom de la liste d’aide est reconnu par le parseur, et `test` est absent de cette liste. Une écriture/lecture directe dans un clone isolé du VFS reproduit le contenu historique sans créer de fichier dans le projet de l’élève.

La GUI utilise la même constante de liste d’aide, avec ses explications traduites. Le texte de la liste est inchangé; aucune nouvelle capture n’est nécessaire pour ce déplacement. Les parcours fonctionnels existants du workspace conservent FR EN ES IT PT DE et les deux vues. La qualification rendue observation précédente reste une preuve distincte; aucune suite graphique globale verte ni résultat Windows/Linux n’est revendiqué ici.

Session/récupération, annotations/PNG/PDF, configuration/aide/diagnostics de lancement, exemples, couverture d’import et livraison trois OS restent à réaliser ou compléter.
