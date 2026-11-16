# Qualification J7 — duplication et alignement

Qualification locale du **8 novembre 2026**, macOS arm64, Rust 1.95.0. F-TOP-009 et RS-TOPOLOGY-J7 sont vérifiés selon le [profil portable](../agnostique/EDITION_J7.md) et l’[adaptateur](../rust/J7-EN-COURS.md). Le jalon J7 complet reste ouvert.

## Contrôles exécutés

| Commande | Résultat |
| --- | --- |
| `SNS_RECORD_J7_TOPOLOGY="$PWD/docs/verification" cargo test --workspace --locked --offline` | **186 tests du workspace**, plus **1 test enfant** du pont; trois ignorés par défaut; [journal](j7-topology-workspace.log) |
| `cargo test -p sns-ui-egui --features ui-snapshots --test lan j7_duplicate --locked --offline` | 1 parcours rendu dans six langues et deux vues, **12 captures comparées sans mise à jour**; [journal](j7-topology-gpu.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Réussi sans avertissement; [journal](j7-topology-clippy.log) |
| `cargo fmt --all --check` | Réussi; [journal](j7-topology-fmt.log) |
| `python3 scripts/check_i18n.py`; `check_network_assets.py`; `check_layers.py` | **620 clés × six langues**, 18 SVG et frontières vérifiés; [langues](j7-topology-i18n.log), [assets](j7-topology-assets.log), [couches](j7-topology-layers.log) |
| `python3 scripts/docs.py render`; `python3 scripts/docs.py check --require-reference` | Registres, dépendances et références vérifiés; [journal](j7-topology-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-topology-code-manifest.json` | **459 fichiers source**, révision `sha256:dd924eaa4a3c9543750f5ab8546d0749d3bf302e17a8a13d7afb8e0c198b9fb1` |

L’[environnement](j7-topology-environment.json), le [manifeste](j7-topology-code-manifest.json) et le [complément de sources](j7-topology-source-delta.json) identifient cette qualification, sans remplacement des preuves historiques.

## Couverture des critères

`sns-topology/tests/duplication.rs` vérifie cinq recettes :

- Deux copies successives d’un LAN : identités et MAC distinctes, câbles remappés, ressources communes conservées, annotation copiée, avertissements IPv4 et réouverture. Un ping émis par un poste copié reçoit sa réponse dans la copie; toutes les réceptions Ethernet restent dans ce LAN.
- Rôles LAN/WAN, routes et interface DHCP remappés; réservation vers la MAC du client copié; nouveau SSID de point d’accès et réassociation du client copié, avec texte Unicode borné.
- Courrier, contacts et fichiers conservés; identités de messages/contacts renouvelées; correspondance serveur/client d’un même message maintenue; noms longs valides après suffixe.
- Huit alignements/répartitions, sans mutation des équipements ou connexions; annulation/rétablissement exacts et absence d’étape d’historique pour un alignement sans déplacement.
- Refus complet de sélection invalide, décalage non fini ou copie dépassant 8 Mio, sans modification du document, de sa révision ou de son historique.

Le parcours `j7_duplicate_align_undo_and_ipv4_warnings_preserve_all_languages_and_views` sélectionne un réseau par rectangle, le duplique, vérifie les avertissements, répartit les équipements, annule/rétablit, change de vue puis emploie le raccourci D. Les opérations sont verrouillées en simulation. Les six langues FR EN ES IT PT DE utilisent les mêmes données et contrôles.

Les captures FR de la copie en Icônes et DE de la répartition en Schéma ont été inspectées : groupes séparés, créations sélectionnées, paramètres MAC/IP visibles et avertissements lisibles. Les limites d’affichage générales de l’inspecteur restent à traiter dans les préférences/accessibilité et la qualification de livraison.

## Artefacts

Le [projet copié](j7-topology-copy.json), ses [avertissements](j7-topology-conflicts.json) et le [ping moteur](j7-topology-copy-ping.jsonl) sont enregistrés par la recette. La [qualification CLI](j7-topology-cli.json) ouvre ensuite ce document et lance un autre [ping](j7-topology-cli-ping.jsonl), avec une réponse et sans modification du fichier source. Les identifiants de commande des deux recettes diffèrent; leurs traces ne sont pas présentées comme identiques entre elles.

## Limites et suite

Les adresses sont conservées et les IPv4 identiques sont des avertissements de conflits possibles. Les ressources du projet restent partagées; les configurations, fichiers d’hôtes et données de courrier copiés sont indépendants. Les annotations sont prises en charge par la transaction; leurs interactions complètes de sélection, dimensionnement et style restent dans F-DOC-001/002.

La suite graphique historique complète doit encore être réconciliée avec le produit courant avant livraison. Aucun résultat Windows/Linux ou parcours manuel autonome n’est revendiqué. Le prochain lot est l’apparence nuage du switch; observation, rapports/exports, session/récupération, configuration, exercices et livraison trois OS restent également ouverts.
