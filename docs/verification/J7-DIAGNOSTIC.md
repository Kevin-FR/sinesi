# J7 — journal et export de diagnostic

F-CFG-006 et v/l de F-CFG-005 relèvent de RS-PLATFORM-J7.
[Contrat portable](../agnostique/DIAGNOSTIC_J7.md) · [Adaptateur Rust](../rust/DIAGNOSTIC_J7.md)

Le contrôle sans fenêtre confirme le chemin annoncé et le rapport JSON sur la
sortie standard. L’essai détaillé enregistre `configuration_resolved` et
`project_ready`. Un projet illisible produit `project_load_failed`. Le lanceur
refuse le remplacement d’un journal existant et les collisions avec le projet,
le profil, la configuration administrative ou la récupération.

- [Journal détaillé de lancement](j7-diagnostic-launch.jsonl)
- [Journal d’un échec de chargement](j7-diagnostic-failure.jsonl)
- [Export d’une erreur de restauration](j7-diagnostic-export.json)

Le test du harnais charge le projet de courrier, injecte un checkpoint invalide
et exporte l’échec. Il compare le projet, le temps et la trace avant et après.
Il protège les trois destinations exercées et vérifie le contenu exact de
l’export. Le test du journal vérifie aussi le plafond du fichier, la conservation
des événements récents et le schéma JSON.

Le bouton **Exporter le diagnostic technique…** et la confirmation sont
contrôlés dans les six langues et les vues Icônes et Schéma. Douze références
propres à cette fenêtre ont été créées. Les captures Schéma FR, Schéma DE et
Icônes PT ont été inspectées visuellement.

- [Schéma FR](../../crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-fr.png)
- [Schéma DE](../../crates/sns-ui-egui/tests/snapshots/j7-diagnostic-diagram-de.png)
- [Icônes PT](../../crates/sns-ui-egui/tests/snapshots/j7-diagnostic-icons-pt.png)

La vérification porte sur macOS arm64. Le harnais exerce l’opération d’export
avec une destination explicite. L’utilisation du sélecteur de fichiers natif
et les parcours Windows et Linux restent à qualifier.

## Validation locale

| Commande exécutée | Résultat |
| --- | --- |
| `SNS_RECORD_J7_DIAGNOSTICS="$PWD/docs/verification" cargo xtask ci` | Clippy sans avertissement, 278 tests du workspace et un enfant du pont réussis, cinq ignorés, quatre exports PNG contrôlés. [Journal](j7-diagnostic-ci.log) |
| `env -u UPDATE_SNAPSHOTS cargo xtask ui` | 23 tests et 120 références comparées. Les 496 PNG antérieurs correspondent à leur manifeste. [Journal](j7-diagnostic-gpu.log) |
| `python3 scripts/docs.py render`, `python3 scripts/docs.py check --require-reference` | Registres, vues et empreintes du référentiel vérifiés. [Journal](j7-diagnostic-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-diagnostic-code-manifest.json` | Inventaire scellé de 805 fichiers. [Manifeste](j7-diagnostic-code-manifest.json) |

L’[environnement](j7-diagnostic-environment.json) répertorie les fichiers modifiés.
La révision est `sha256:72eaec6602cecd73da604d6d77c519fb30c838dfa8d225be8308dcbf080908bb`. Le
[correctif d’inventaire](j7-diagnostic-source-patch.json) complète la révision RTT
pour reconstruire les sources qualifiées.
