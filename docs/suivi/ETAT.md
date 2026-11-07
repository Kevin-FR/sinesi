# État de reprise — Sinesi, 7 novembre 2026

## Situation réelle

La première tranche J0 est exécutable : workspace Rust 1.95.0, moteur déterministe, format JSON portable, sauvegarde atomique, CLI et prototypes egui/eframe 0.37.0. Le code neuf est sous MIT. Git est initialisé localement avec `https://github.com/Kevin-FR/sinesi.git` comme origine.

Les composants réalisés sont sns-model, sns-engine, sns-project, sns-html, sns-cli, sns-ui-egui et sns-desktop. Les protocoles réseau complets restent à développer. L’interface indique « Atelier J0 » et les faits portent le préfixe `prototype.message.*`.

## Registres

| Référence | État actuel |
| --- | --- |
| 177 exigences | 6 verified, 2 in_progress, 169 not_started |
| 49 lots Rust | 2 verified, 1 in_progress, 44 not_started, 2 deferred |
| RS-ENGINE-J0 | Vérifié : ordre total, temps virtuel, graine, bornes, quotas et commandes |
| RS-PROJECTS-J0 | Vérifié : contrat initial, validation, sauvegarde atomique et migration de qualification |
| RS-QUALITY-J0 | En cours : fondations présentes, recettes transversales complètes encore ouvertes |
| Référence Filius | 1 061 fichiers indexés; archive locale inchangée et hors Git |

Exigences vérifiées : F-SIM-005, F-PROJ-004, F-NFR-002, F-NFR-003, F-NFR-008, F-NFR-009. Les preuves sont dans les registres et la [matrice](MATRICE.md). Le [rapport J0](../verification/J0.md), ses journaux et son manifeste d’empreintes identifient le code testé.

## Contrôles réellement exécutés

- `cargo fmt --all --check` et `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- `cargo test --workspace --locked` : 23 tests réussis.
- `cargo test -p sns-ui-egui --features ui-snapshots --test workbench --locked` : 5 tests réussis, dont captures GPU comparées aux références inspectées.
- Deux processus CLI donnent des traces identiques; bornes et codes de sortie sont testés.
- `scripts/check_layers.py` vérifie les frontières du cœur; deux tests de `scripts/test_docs.py` refusent les états vérifiés sans preuve/prérequis.
- Le binaire desktop a été construit; le parcours page, image et POST a été vérifié dans la fenêtre native macOS, avec résultat `message=Bonjour`.
- `python3 scripts/docs.py check --require-reference` contrôle suivi, liens, ancres et empreintes historiques locales.

Environnement : macOS arm64, Rust 1.95.0. La CI Windows/Linux/macOS est configurée; ses jobs distants ne sont pas annoncés comme exécutés. Aucun protocole complet n’est déclaré vérifié à partir des messages du prototype.

## Décisions et limites

Voir [D-001](../decisions/D-001-ordonnancement.md), [D-004](../decisions/D-004-format-j0.md), [D-002](../rust/decisions/D-002-egui-j0.md), [D-003](../rust/decisions/D-003-html-j0.md) et [D-007](../rust/decisions/D-007-provenance.md).

F-NFR-001 reste en cours : le prototype sans dépendance réseau ne satisfait pas encore les recettes LAN/DNS/HTTP d’une pile complète. F-NFR-006 reste en cours : le lecteur initial refuse ZIP/XML; l’import historique sûr avec rapport reste à développer. Le jalon J0 complet n’est donc pas déclaré clos par cette tranche de démarrage.

Les modules Craft, forks egui/winit, rfd, exports produit PNG/PDF, texte avancé et plugins ne sont pas intégrés. Le graphe upstream suffit au prototype; leurs évaluations restent dans [DEPENDANCES.md](../rust/DEPENDANCES.md).

## Prochaine tranche

1. Commencer J1 : édition du graphe, équipements/interfaces, puis Ethernet/commutation/ARP/IPv4/ICMP simulés.
2. Faire évoluer explicitement les contrats et la matrice de fidélité, sans confondre messages du prototype et protocoles.
3. Compléter les recettes transversales ouvertes lorsque leur pile est disponible.
4. Exécuter la CI distante et les parcours natifs des autres OS avant d’affirmer une livraison multiplateforme.

## Maintenance du suivi

features.json est l’autorité fonctionnelle; ../rust/taches.json distingue cibles proposées et fichiers réalisés. `verified` exige recette, commande, résultat, environnement, révision et artefacts, ainsi que des prérequis vérifiés.

Après modification : `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check`. Sans le corpus `_archive`, le contrôle annonce qu’il ne revalide pas le contenu historique. Ne pas régénérer l’audit sans sa référence.
