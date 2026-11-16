# J7 — Qualification partielle de l’import

Cette preuve concerne la tranche initiale d’import. J7 reste en cours; elle ne prouve ni reprise/rejeu, ni récupération, ni exports, ni livraison multiplateforme. Les [limites fonctionnelles](../agnostique/PROJETS_J7.md) et la [réalisation](../rust/J7-EN-COURS.md) distinguent conversion structurelle et équivalence réseau.

| Contrôle | Résultat constaté |
| --- | --- |
| `cargo test --workspace --locked --offline` | 162 tests du workspace réussis, plus un test dans le processus enfant du pont; trois tests ignorés par défaut; [journal](j7-import-workspace.log) |
| `cargo test -p sns-filius-import --test import --locked --offline -- --ignored` | Deux recettes réussies : corpus local de 20 fichiers et ARP/ICMP du LAN historique; [journal](j7-import-reference.log) |
| `cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_background --locked --offline` | Parcours dans six langues et deux vues, comparaison de six captures sans mise à jour; [journal](j7-import-gpu.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi; [journal](j7-import-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-import-fmt.log) |
| Contrôles langues/couches | 570 clés complètes et importeur indépendant de la GUI; [langues](j7-import-i18n.log), [couches](j7-import-layers.log) |

Le [rapport du corpus](j7-import-corpus.json) identifie chaque chemin relatif et son rapport propre. Les tests comparent le nombre d’équipements avec la structure XML et reconstruisent le contenu de chaque entrée conservée. Les projets passent la validation métier et les schémas de projet/rapport. Le [ping historique](j7-import-lan.jsonl) utilise la pile Ethernet/ARP/IPv4/ICMP de production; il ne constitue pas une validation fonctionnelle de tous les autres exemples.

Les tests ordinaires couvrent aussi une fixture synthétique propre à Sinesi, un ping importé, la stabilité de deux conversions, les collisions MAC, les noms dupliqués, références absentes, CRC incorrect, taille excessive, XML à entités, chemins d’archive et de fichiers virtuels dangereux, ainsi que la conservation inactive de sources/binaires inconnus. Le CLI garde l’archive originale et refuse les destinations identiques. L’interface importe hors du fil graphique et conserve le document antérieur après erreur.

Le [manifeste de cette tranche](j7-import-code-manifest.json) couvre 401 fichiers, révision `sha256:5531b30104b58c67a00a500118c1a57372635ab2ab7765e95cc21e74bc7e69f7`. L’[environnement](j7-import-environment.json) est macOS arm64 / Rust 1.95.0. Les six captures `j7-import-*` sont distinctes des références J0–J6; FR et DE ont été inspectées.

La conservation des originaux ne rend pas les données non converties actives. Certains réglages, boîtes mail/contacts, états et styles nécessitent encore conversion ou vérification. Le rapport annonce explicitement `functional_equivalence_verified: false`. Les critères de R-REPRISE et R-RAPPORT, ainsi que les résultats Windows/Linux, sont absents de cette tranche et restent ouverts.

La [copie des sources de qualification](j7-import-source-bundle.tar.gz) conserve le code exact de cette tranche malgré les modifications ultérieures du checkout. Ses [métadonnées](j7-import-source-bundle.json) donnent empreinte et nombre de fichiers; chaque source scellée a été revérifiée dans le bundle sans extraction. L’archive Filius et le cache des dépendances ne sont pas inclus. Pour les deux recettes historiques, fournir le corpus local d’origine; les autres tests utilisent la fixture synthétique distribuée.
