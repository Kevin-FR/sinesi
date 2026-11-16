# J7 — annotations et zones qualifiées localement

Passe du **8 novembre 2026**, macOS arm64, Rust 1.95.0, egui 0.37/wgpu. **F-DOC-001 et F-DOC-002 sont vérifiés**. RS-REPORTING-J7 reste en cours pour PNG/PDF. Le [profil portable](../agnostique/ANNOTATIONS_J7.md), le [contrat](../../contracts/project-documentation.schema.json), l’[exemple](../../fixtures/valid/annotations.json) et l’[adaptateur](../rust/J7-EN-COURS.md#annotations-et-zones) fixent le périmètre.

## Commandes et résultats

- `cargo test --workspace --locked --offline` : **215 tests du workspace et 1 test enfant du pont réussis**, 5 ignorés par défaut. Deux helpers de récupération sont réellement lancés puis arrêtés de force par leurs parents; ils ne sont pas comptés comme tests réussis autonomes. Les corpus Filius et la mesure de capacité restent ignorés dans cette passe. [Journal final](j7-annotations-workspace.log).
- `cargo test -p sns-ui-egui --features ui-snapshots --test annotations --locked --offline` : **2 tests réussis**, dont parcours paramétré dans les six langues et deux vues, **12 PNG comparés sans mise à jour**. FR Icônes et DE Schéma inspectés après la comparaison : texte et accents lisibles, zone derrière le réseau, câbles contrastés, note entièrement cadrée et poignée visible. [Journal](j7-annotations-gpu.log).
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`, `cargo fmt --all --check` : réussis. [Clippy](j7-annotations-clippy.log), [formatage](j7-annotations-fmt.log).
- `python3 scripts/check_i18n.py`, `python3 scripts/check_network_assets.py`, `python3 scripts/check_fonts.py`, `python3 scripts/check_layers.py` : **693 clés × six langues**, **20 SVG**, **4 fontes fixées avec licence/poids/glyphes contrôlés**, frontières réussies. [Langues](j7-annotations-i18n.log), [assets](j7-annotations-assets.log), [fontes](j7-annotations-fonts.log), [couches](j7-annotations-layers.log).
- `python3 scripts/docs.py render`, `python3 scripts/docs.py check --require-reference`, `python3 scripts/check_manifest.py check docs/verification/j7-annotations-code-manifest.json` : vues et inventaire validés; [journal documentaire](j7-annotations-docs.log).

## Recettes

| Critère | Résultat et chemin de test |
| --- | --- |
| Texte/dimensions/styles durables | Géométrie, couleurs, gras, italique, soulignement et famille conservés par sérialisation/enregistrement/réouverture; schéma 9, session et checkpoint validés dans `sns-project/tests/contracts.rs` |
| Édition et ordre | Ajout avec identités distinctes, modification en place, suppression réversible, transaction refusée sans mutation pour dimensions invalides et identité déjà occupée : `sns-topology/tests/annotations.rs` |
| Réseau conservé | Équipements, interfaces, câbles, configurations et disposition identiques; états réseau/applications/file/aléa/temps comparés avant et après l’ajout; fusion du runtime conserve les annotations |
| Duplication | Styles et texte conservés avec identité renouvelée et déplacement attendu des notes copiées |
| Gestes et clavier | Déplacement et redimensionnement à deux zooms, conversion pixels/document, annuler/rétablir, Échap sans commit et modification numérique accessible : `sns-ui-egui/tests/annotations.rs` |
| Empilement | Zones peintes avant câbles/nœuds, visibles en Icônes et Schéma; contour des câbles conservant leur contraste sur un fond coloré |
| Brouillon non appliqué | Récupération du texte d’annotation avec projet explicite inchangé; reprise sans application, puis application seulement sur commande de l’utilisateur |
| Compatibilité | Projets 1 à 8 sans champ style inchangés, nouveau profil 9 explicite; schémas courants de scénario/journal/checkpoint/session acceptent le profil documentaire |

Les douze nouvelles images sont `crates/sns-ui-egui/tests/snapshots/j7-annotations-{icons,diagram}-{fr,en,es,it,pt,de}.png`. Les références des tranches précédentes n’ont pas été remplacées. Les fontes Noto, leur licence et leurs empreintes sont [embarquées](../../assets/fonts/README.md); seules les familles des annotations les utilisent. L’italique est une inclinaison de présentation. Aucun nouveau paquet tiers ni changement de version externe : le lockfile ajoute seulement `sns-fonts` et son lien depuis la GUI.

## Échec intermittent de récupération et correctif

La première [passe intégrale en échec](j7-annotations-initial-failure.log) a révélé un scan temporairement vide pour un document corrompu dont le propriétaire venait d’être fermé. Ses octets étaient conservés, mais le test attendait un diagnostic. L’hypothèse de référence de verrou héritée pendant un lancement concurrent est documentée comme inférence dans [D-009](../rust/decisions/D-009-session-j7.md#complément-pendant-la-tranche-annotations).

`LockedFile` libère maintenant explicitement le verrou avant fermeture du fichier, pour session et garde d’allocation. Le test de stockage passe **40 fois de suite**, chaque répétition comprenant trois tests et un arrêt forcé réel. [Journal](j7-annotations-session-stress.log), [résultat](j7-annotations-session-stress.json). Reproduction dans un shell zsh/bash sur cet environnement :

```sh
for attempt in {1..40}; do
  cargo test -p sns-project --test session --locked --offline || exit 1
done
```

La passe finale complète et les tests graphiques ci-dessus sont exécutés après ce correctif. Les quarante répétitions sont une vérification supplémentaire, pas des tests ajoutés au total du workspace. F-PROJ-003/006 gardent leurs preuves historiques et reçoivent ce complément.

## Sources, limites et suite

Le [manifeste](j7-annotations-code-manifest.json) couvre **544 fichiers**, révision `sha256:020669e66fe9209d8e5899823d06db615a66f9e6b283df391a1eace825573084`. Le [delta source](j7-annotations-source-delta.json) se superpose au bundle initial d’import, avec reconstruction de l’inventaire vérifiée. Les preuves/sources antérieures sont conservées. L’[environnement](j7-annotations-environment.json) détaille les tests, processus et limites.

Le texte est uniforme par note, revient à la ligne et est découpé à ses dimensions; agrandir montre la suite. Pas de rotation ni d’ancrage automatique. Les six langues latines requises sont contrôlées; aucun support universel des écritures n’est annoncé. La conversion des styles historiques d’import reste partielle. La qualification utilise le harnais rendu sur macOS : aucune nouvelle preuve native manuelle, Windows ou Linux.

La suite graphique historique complète, la lisibilité générale de l’inspecteur et la livraison trois OS restent ouvertes. **F-DOC-003/004 ne sont pas implémentés par cette tranche** : prochaine action, scène commune et export PNG, puis rapport PDF paginé. J7 reste ouvert avec import partiel, plateforme/configuration/lancement/aide/diagnostics, exemples et portes de livraison.
