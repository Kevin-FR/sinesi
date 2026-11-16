# J7 — export PNG qualifié localement

F-DOC-003 est vérifié. RS-REPORTING-J7 reste en cours pour F-DOC-004 (PDF). Le profil portable est [EXPORTS_J7](../agnostique/EXPORTS_J7.md); les choix de réalisation sont dans [D-010](../rust/decisions/D-010-export-scene-j7.md).

La scène indépendante des widgets cadre équipements, liaisons physiques/radio et annotations, y compris aux coordonnées négatives ou hors du viewport. Elle partage positions de repli, SVG et fontes embarquées avec l’éditeur. L’image exclut les panneaux et les poignées. Le fond transparent est proposé par défaut; Icônes/Schéma et fond opaque sont disponibles. Le rendu est borné à 4096 pixels par côté. Les glyphes sont façonnés à partir des quatre fontes Noto, sans lecture de police système ni ressource externe.

Le CLI expose `render-png`; la GUI capture une révision puis rend/écrit en arrière-plan. Elle conserve la simulation et son temps; les libellés DHCP effectifs sont capturés sans convertir la configuration DHCP en statique. Les chemins du projet, des préférences et de la récupération sont protégés. Un second export simultané est refusé et une fermeture normale attend le travail engagé.

## Vérifications exécutées

Environnement : macOS arm64, Rust 1.95.0, egui 0.37.0/wgpu. Détails dans [l’environnement](j7-png-environment.json).

- `SNS_RECORD_J7_PNG="$PWD/docs/verification" cargo test --workspace --locked --offline` : **224 tests du workspace et un enfant du pont**, cinq tests ignorés; deux processus de récupération sont réellement arrêtés. [Journal](j7-png-workspace.log).
- `cargo test -p sns-ui-egui --features ui-snapshots --test png --locked --offline` : **3 tests, 12 nouvelles captures comparées sans mise à jour**, six langues et deux vues. Les captures FR/Icônes et DE/Schéma ont été inspectées. [Journal](j7-png-gpu.log).
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` et `cargo fmt --all --check` : réussis. [Clippy](j7-png-clippy.log), [formatage](j7-png-fmt.log).
- `python3 scripts/check_i18n.py`, `check_network_assets.py`, `check_fonts.py` et `check_layers.py` : réussis; 709 clés × six langues, 20 SVG et quatre fontes.
- Pour chacun des quatre PNG ci-dessous : `python3 scripts/check_png_exports.py IMAGE --metadata METADATA`. Le décodeur indépendant vérifie CRC, décompression, dimensions, empreinte, identifiants et marge transparente de 16 pixels. **4 réussites**. [Journal](j7-png-independent.log).
- `python3 scripts/test_docs.py` et `python3 scripts/test_filius_inspection.py` : deux tests chacun, réussis. `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check --require-reference`. [Contrôle documentaire](j7-png-docs.log).

Tests de référence : `sns-render/tests/png.rs`, `sns-fonts/tests/layout.rs`, `sns-cli/tests/png.rs`, `sns-ui-egui/tests/png.rs`. Ils couvrent cadrage de tous les objets, notes seules, placement absent, déterminisme octet pour octet, transparence/fond opaque, dix symboles, accents/ligatures/retours à la ligne, texte borné, absence de mutation, échecs préservant la destination et capture d’un état appliqué pendant l’édition suivante.

## Artefacts et reconstruction

| Scène | Image | Métadonnées |
| --- | --- | --- |
| Icônes, coordonnées négatives et note éloignée | [PNG](j7-png-icons.png) | [JSON](j7-png-icons.json) |
| Schéma, même cadrage | [PNG](j7-png-diagram.png) | [JSON](j7-png-diagram.json) |
| Tous les symboles et styles, Icônes | [PNG](j7-png-symbols-icons.png) | [JSON](j7-png-symbols-icons.json) |
| Tous les symboles et styles, Schéma | [PNG](j7-png-symbols-diagram.png) | [JSON](j7-png-symbols-diagram.json) |

[Manifeste](j7-png-code-manifest.json) : 568 fichiers, révision `sha256:283a6311478c3e03b56949a1188eecae75604599ed1d3dc45df62e55cf0a01a9`. [Delta de sources](j7-png-source-delta.json) superposable à l’archive initiale d’import; les preuves et captures historiques restent conservées.

## Limites

Le sélecteur natif n’a pas été manipulé manuellement. Aucun résultat Windows/Linux n’est revendiqué. La suite graphique historique complète doit encore être réconciliée avec l’interface J7; seuls les parcours PNG ci-dessus sont qualifiés ici. Le texte exporté couvre les six langues latines prévues; les scripts non pris en charge sont refusés explicitement. Les légendes temporaires de panne et animations de paquets ne font pas partie du plan statique. PDF, préférences complètes, exemples et livraison restent des travaux J7 distincts.
