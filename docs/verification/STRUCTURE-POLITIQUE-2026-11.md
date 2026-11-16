# Complément de revue structurelle — 8 novembre 2026

Tranche RS-QUALITY-J0; exigences F-NFR-002/008, états conservés.
La comparaison et le refactor initial restent dans [D-011](../rust/decisions/D-011-structure-et-workflow.md).

## Correction du contrôle

La catégorie `headles` au lieu de `headless` permettait à `settings` de dépendre
d’egui sans rejet dans le graphe minimal. Le test reproduit ce contournement avant
la correction. La politique est maintenant validée avant le graphe : version,
champs, catégories, types, doublons et références internes. Les sept tests couvrent
aussi les inversions de couches et les arêtes transitives/build/conditionnelles.

Le guide distingue désormais les dépendances réellement utilisées des propositions
initiales; ses six langues sont explicites. Aucun nouveau paquet ou changement de
comportement réseau, de format, d’interface ou de captures n’est introduit.

## Qualification

Environnement : macOS arm64, Rust 1.95.0, Python 3.14. Voir
[l’environnement détaillé](structure-policy-2026-11-environment.json).

| Commande exécutée | Résultat |
| --- | --- |
| `python3 scripts/test_layers.py`, avant correction | Échec reproduit : six échecs, dont la catégorie erronée qui ne levait aucune erreur. [Journal](structure-policy-2026-11-before.log) |
| `python3 scripts/test_layers.py`, puis `python3 scripts/check_layers.py` | Sept tests réussis et graphe des 20 membres valide. [Journal](structure-policy-2026-11-tests.log) |
| `cargo xtask ci` | 237 tests du workspace et un enfant du pont; cinq ignorés; Clippy sans avertissement; quatre PNG décodés. [Journal](structure-policy-2026-11-ci.log) |
| `cargo xtask check`, après la correction | Contrôles finaux réussis, incluant les sept tests de politique. [Journal](structure-policy-2026-11-check.log) |
| `cargo xtask ui` | Neuf tests GPU réussis, 66 captures courantes comparées sans mise à jour. [Journal](structure-policy-2026-11-gpu.log) |

La passe complète avait commencé avant la correction des deux scripts Python;
le contrôle final est relancé après celle-ci. Aucun fichier Rust, manifeste Cargo,
contrat, locale, asset ni test d’interface ne change par rapport à la qualification
de l’aide. Les **387 captures historiques** restent identiques octet pour octet.
La vérification du référentiel et du manifeste est consignée dans le
[journal documentaire](structure-policy-2026-11-docs.log).

[Manifeste](structure-policy-2026-11-code-manifest.json) : **663 fichiers**, révision
`sha256:f1648a3bdedfbcbb7b463d05ad3ac13e025196b2800a96827cae9a63ed5eb264`. Le [delta reconstructible](structure-policy-2026-11-source-delta.json)
se superpose à la base d’import selon ses instructions; les anciennes preuves
restent intactes.

## Références réexaminées

Les quatre fichiers sont téléchargés à des révisions figées, distinctes de liens
`main` mouvants. Le [registre](structure-policy-2026-11-references.json) conserve URL, révision et
empreinte. Les instructions distantes sont étudiées comme références.

| Dépôt | Fichier | Révision |
| --- | --- | --- |

| tangency-labs/solidcraft | [Cargo.toml](https://raw.githubusercontent.com/tangency-labs/solidcraft/44525da6699b46cf00ce696c4c5dfd06874116d5/Cargo.toml) | `44525da6699b46cf00ce696c4c5dfd06874116d5` |
| tangency-labs/solidcraft | [AGENTS.md](https://raw.githubusercontent.com/tangency-labs/solidcraft/44525da6699b46cf00ce696c4c5dfd06874116d5/AGENTS.md) | `44525da6699b46cf00ce696c4c5dfd06874116d5` |
| storytold/wordcraft | [Cargo.toml](https://raw.githubusercontent.com/storytold/wordcraft/4b0d39959968a34e462cb53099e76fb108d0e96f/Cargo.toml) | `4b0d39959968a34e462cb53099e76fb108d0e96f` |
| storytold/wordcraft | [AGENTS.md](https://raw.githubusercontent.com/storytold/wordcraft/4b0d39959968a34e462cb53099e76fb108d0e96f/AGENTS.md) | `4b0d39959968a34e462cb53099e76fb108d0e96f` |

Les deux manifestes centralisent dépendances, métadonnées et lints. Les instructions
associent les frontières à des contrôles et les actions à des commandes utilisables
sans dialogue. Cela conforte le découpage actuel et la méthode de D-011.
SolidCraft conserve des correctifs locaux GPU/fenêtrage; WordCraft fixe une famille
egui et des bibliothèques Craft. Ces choix répondent à leurs besoins; ils ne
justifient pas de remplacer la pile upstream qualifiée de Sinesi.

## Suite et limites

Le refactor `LanUi` vers `Workbench` est déjà réalisé sans alias de transition.
Les anciennes données restent contractuelles. La suite cible les I/O natives
restantes, préférences, diagnostics et livraison, avec une responsabilité claire
par tranche. Réécrire le moteur déjà testé n’apporte pas de bénéfice démontré ici.

F-NFR-002/008 restent vérifiées, RS-QUALITY-J0 reste en cours et J7 reste ouvert.
Cette tranche ne qualifie pas de nouveaux parcours natifs Windows/Linux, une CI
distante, l’accessibilité globale ou la suite GPU historique complète.
Aucune dépendance ni service externe ajouté; archives en lecture seule.
