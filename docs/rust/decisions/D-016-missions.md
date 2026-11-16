# D-016 — missions évaluables dans un crate sans interface

Décision du 16 novembre 2026, F-ADV-004 / RS-UX-J8.
[Comportement portable](../../agnostique/MISSIONS.md).

Le choix « visuel + missions » de [D-015](D-015-interface-eleve.md) demande un format
d’exercice, des contrôles et une correction exportable. L’évaluation doit servir à
l’atelier, aux tests et à la ligne de commande sans dépendre d’un widget.

**Propriétaire.** Le nouveau crate `sns-missions` (couche sans interface) possède le
format version 1, la validation, les contrôles, les étoiles, la correction et les
missions intégrées. Il dépend seulement de `sns-model` (projet, validation) et de
`sns-engine` (type des faits de trace). Il ne lance ni n’avance de simulation : les
contrôles lisent un projet et une trace. `docs/rust/layers.json` déclare le crate et
ses arêtes; `sns-cli` et `sns-ui-egui` en dépendent.

**Présentation.** `workbench/missions_ui.rs` affiche la carte et la carte de mission;
le lancement passe par le chemin de remplacement de document existant (confirmation
des modifications non enregistrées comprise), comme les exemples. Les réparations
passent par les transactions ordinaires de l’éditeur.

**Progression.** Les étoiles sont une préférence utilisateur : champ facultatif
`missions` du profil 2, omis tant qu’il est vide, pour que les profils existants
gardent leurs octets. Le résolveur l’accepte dans le profil utilisateur seulement.

**Service DNS.** Le modèle ne persiste pas l’état démarré d’un service, comme le
référentiel. La mission DNS en fait un objectif (`service_listening`) plutôt que
d’ajouter un démarrage automatique au format de projet.

Les missions de l’enseignant sont des fichiers au même format. Leur ouverture
réutilise le chargement de document asynchrone : sélection, lecture et validation
hors de la frame, puis application seulement si l’époque du moteur, la révision et
le mode n’ont pas changé.
