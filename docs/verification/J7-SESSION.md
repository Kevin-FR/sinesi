# J7 — session et récupération qualifiées localement

Passe du **8 novembre 2026**, macOS arm64, Rust 1.95.0, egui 0.37/wgpu. **F-PROJ-003 et F-PROJ-006 sont vérifiés**. RS-PROJECTS-J7 reste en cours pour l’import F-PROJ-005. Le [profil portable](../agnostique/SESSION_J7.md), la [décision D-009](../rust/decisions/D-009-session-j7.md), le [contrat](../../contracts/session.schema.json) et la [fixture](../../fixtures/valid/session.json) fixent le périmètre.

## Commandes exécutées

- `SNS_RECORD_J7_SESSION="$PWD/docs/verification" cargo test --workspace --locked --offline` : **210 tests du workspace et 1 test enfant du pont réussis**; 5 tests ignorés par défaut. Deux helpers de récupération sont lancés explicitement par leur parent et arrêtés de force. Les deux recettes de corpus historiques et la mesure de capacité ne sont pas lancées dans cette passe. [Journal](j7-session-workspace.log).
- `cargo test -p sns-ui-egui --features ui-snapshots --test session recovery_offer --locked --offline` : **1 parcours paramétré, 12 PNG comparés sans mise à jour**, proposition et reprise dans les six langues. Les vues FR/DE ont été inspectées : proposition centrée lisible, vue Schéma et sélection restaurées. [Journal](j7-session-gpu.log).
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` et `cargo fmt --all --check` : réussis. [Clippy](j7-session-clippy.log), [formatage](j7-session-fmt.log).
- `python3 scripts/check_i18n.py`, `python3 scripts/check_network_assets.py`, `python3 scripts/check_layers.py` : **668 clés × six langues**, **20 SVG** et frontières réussies. [Langues](j7-session-i18n.log), [assets](j7-session-assets.log), [couches](j7-session-layers.log).
- `python3 scripts/docs.py render`, `python3 scripts/docs.py check --require-reference` et `python3 scripts/check_manifest.py check docs/verification/j7-session-code-manifest.json` : contrôles de suivi et d’inventaire reproduisibles, [journal](j7-session-docs.log).

## Recettes et résultats

| Critère | Preuve de production |
| --- | --- |
| Séparation préférences/projet/session | Six langues restaurées depuis des préférences indépendantes; un fichier explicite gagne sur l’offre automatique; la reprise retire sa destination et marque le document non enregistré |
| Autosauvegarde et arrêt forcé | Le test d’interface utilise le cycle automatique de deux secondes, attend la publication physique puis tue réellement le processus; projet d’origine identique octet pour octet |
| Saisies non appliquées | Nom d’équipement appliqué et nom brut distinct, texte d’éditeur, saisie de terminal non exécutée et vue Schéma reviennent; aucune commande de terminal n’est exécutée par la reprise |
| Fermeture normale | Le bouton de conservation publie les dernières éditions appliquées/non appliquées avant la commande de fermeture, sans attendre le timer et sans écrire le projet |
| Concurrence et erreurs | Sessions actives invisibles; verrou exclusif; un arrêt libère la possession; données invalides, version inconnue et mauvaises références refusées sans remplacer le document |
| Cache borné | Nettoyage des réservations vides inactives, conservation des octets non vides invalides; un cache plein permet de reprendre une session existante sans supprimer les autres |
| Chemins protégés | Préférences confondues avec un projet explicite désactivées en écriture; sauvegarde de projet refusée dans le cache ou les préférences; exports également protégés |
| Contrat | Enveloppe 1 autonome, projet validé, vue bornée, sélection existante/sans doublon; anciennes sessions sans brouillons acceptées |
| Courrier | Un ancien acquittement d’envoi n’efface pas le nouveau composeur après reprise; flags procéduraux exclus/réinitialisés |

Tests : `crates/sns-project/tests/session.rs`, `crates/sns-project/tests/contracts.rs`, `crates/sns-ui-egui/tests/session.rs`, `desktop::recovery_tests` et tests du parseur desktop. Les preuves d’arrêt sont [stockage](j7-session-crash.json), [document récupéré](j7-session-recovered.json) et [interface automatique](j7-session-ui-crash.json). Les chemins temporaires y sont des métadonnées d’affichage de la recette. Les helpers ignorés ne comptent pas comme tests réussis autonomes.

Les nouvelles images sont `crates/sns-ui-egui/tests/snapshots/j7-session-{offer,restored}-{fr,en,es,it,pt,de}.png`. Les captures historiques J0–J7 restent inchangées. Cette comparaison ciblée ne déclare pas verte la suite graphique historique complète; les colonnes de routage de l’inspecteur restent à traiter dans la qualification de lisibilité.

## Sources et limites

Le [manifeste](j7-session-code-manifest.json) couvre **516 fichiers**, révision `sha256:0d3628a1e9288b3327679b8e575ca5d0f01c7c9ae98de968d49a5c4600360e77`. Le [delta source](j7-session-source-delta.json) se superpose au bundle initial d’import; son empreinte et la reconstruction vérifiée préservent les preuves antérieures. L’archive Filius n’est pas redistribuée. L’[environnement](j7-session-environment.json) distingue tests ordinaires, processus enfants et preuves absentes.

Une interruption peut perdre les frappes suivant la dernière publication. La session récupère un document et des brouillons inactifs, pas les files réseau ni les connexions réelles. La langue est le seul réglage global persistant de cette tranche. Police/icônes/disposition, CLI complet et diagnostics restent dans le lot plateforme. Aucun nouveau parcours natif manuel ni preuve Windows/Linux n’est revendiqué.

J7 reste ouvert : import partiel, annotations et PNG/PDF, configuration/lancement/aide/diagnostics, exemples, accessibilité et livraison trois OS. Les preuves de récupération n’en ferment aucune autre exigence.

## Complément ultérieur

La tranche [annotations](J7-ANNOTATIONS.md) étend la récupération au brouillon d’annotation et au projet 9. Elle révèle aussi un diagnostic de cache temporairement absent sous lancement concurrent de processus, puis ajoute la libération explicite du verrou avant fermeture et 40 répétitions réussies de la recette. Les preuves de cette page restent historiques à leur révision; le complément et le correctif sont rattachés aux mêmes exigences dans le registre.
