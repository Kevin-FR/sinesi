# J7 — configuration de démarrage

**F-CFG-004 vérifié; F-CFG-005 et RS-PLATFORM-J7 restent en cours.** Voir le
[profil portable](../agnostique/CONFIGURATION_J7.md), les
[contrats](../../contracts/configuration.schema.json), l’[adaptateur](../rust/CONFIGURATION_J7.md)
et [D-013](../rust/decisions/D-013-configuration-j7.md).

## Résultat

La priorité est déterministe, champ par champ : valeurs intégrées, administrateur,
profil utilisateur, puis CLI. Les valeurs administrateur sont des défauts. Une valeur
invalide garde la valeur valide de priorité inférieure, avec diagnostic traduit.
Les champs valides restent appliqués. Les fichiers malformés, futurs ou dépassant
65536 octets sont préservés; un profil invalide n’est pas réécrit. Les nombres
intégraux JSON décimaux/exponentiels suivent le contrat entier portable.

L’instantané avant CLI est distinct de l’état effectif. Les sauvegardes copient les
champs explicitement modifiés; changer la langue ou la fonte ne fige pas les autres
options temporaires. Un reset explicite rétablit également des valeurs utilisateur
masquées. Les rôles de fichier sont contrôlés, y compris après parents absents,
traversée `..` et liens symboliques. Les entrées administrateur et le projet restent
protégés des sauvegardes de préférences et des exports.

Les diagnostics ne contiennent pas les valeurs rejetées ou les clés inconnues. Le
panneau expose les chemins dans une section repliée et peut se rouvrir depuis les
préférences. Le rapport JSON omet ces chemins et les données pédagogiques.
L’initialisation du projet est partagée par le lancement et le contrôle sans fenêtre;
les lectures ont lieu avant la création de la fenêtre native.

## Validation exécutée

Environnement : macOS arm64, Rust 1.95.0, Python 3.14.4, egui 0.37.0/wgpu.
[L’environnement détaillé](j7-configuration-environment.json) conserve les différences
au manifeste d’affichage et le contrôle du graphe tiers.

| Commande | Résultat et preuve |
| --- | --- |
| `cargo xtask ci` | **265 tests workspace et un enfant du pont**, cinq ignorés; Clippy sans avertissement; contrôles de format, couches, contrats, catalogues, provenance et quatre PNG décodés. [Journal](j7-configuration-ci.log) |
| `cargo xtask ui` | **21 tests de la suite graphique**, **108 captures comparées sans mise à jour**. [Journal](j7-configuration-gpu.log) |
| `cargo test -p sns-desktop --locked --offline` | Parseur natif et quatre tests de processus : couches, projet réel, aide/version/repli dans les six langues, répertoire de travail, limites de sérialisation des chemins. [Journal](j7-configuration-cli-tests.log) |
| `cargo test -p sns-ui-egui --test configuration ordinary_help_clicks_do_not_persist_geometry_derived_from_temporary_display_options --locked --offline -- --exact` | Régression reproduite avant correction. [Avant](j7-configuration-window-gesture-before.log); après correction, les six tests de configuration et six de préférences passent. [Après](j7-configuration-window-gesture-after.log) |
| `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check --require-reference` | Vues, registres, références et empreintes du corpus contrôlés. [Journal](j7-configuration-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-configuration-code-manifest.json` | Inventaire courant identique au manifeste |

Les sept tests de `sns-settings/tests/configuration.rs` couvrent priorité, origines,
formes numériques intégrales, bornes, fichiers absents/malformés/futurs/volumineux,
quota de diagnostics, anonymisation et copie des seuls champs modifiés. Les six
tests UI couvrent invalidité dans six langues et deux vues, absence de mutation du
projet/temps/trace/exécution, persistance sans contamination CLI, reset/annulation,
chemins protégés et validation de la frontière typée. Les recettes antérieures de
session, annotations, protocoles et exports continuent de passer.

Le vrai binaire est exécuté par les tests de processus. Le résumé du projet est
comparé au fichier choisi (version, équipements et liens), pas seulement à un chemin
annoncé. Le rapport est validé par son schéma autonome. Des valeurs invalides sont
diagnostiquées avec sortie 0 et réglages sûrs; syntaxe ou projet invalide donnent 1.
Le contrôle ne crée ni profil ni cache. Les fixtures originales restent identiques.

## Artefacts et rendu

Douze nouveaux PNG `j7-configuration-*` montrent les diagnostics, à 125 %, dans
FR EN ES IT PT DE et Icônes/Schéma. Les **483 images précédentes restent identiques
octet pour octet**; les 96 références d’affichage/aide/exports restent comparées.
Inspection visuelle FR/Schéma et DE/Icônes : messages lisibles, champs identifiés,
état de lecture seule explicite et fermeture accessible. Les chemins repliés ne
rendent pas les références dépendantes du dossier temporaire de la machine.
La [génération initiale](j7-configuration-gpu-initial.log) des douze captures est
séparée de la comparaison finale sans `UPDATE_SNAPSHOTS`.

Six aides textuelles et douze rapports réels sont enregistrés avec le binaire local :
`j7-configuration-help-{fr,en,es,it,pt,de}.txt` et
`j7-configuration-cli-{fr,en,es,it,pt,de}-{icons,diagram}.json` dans ce dossier.
Ils proviennent de `--config fixtures/valid/configuration.json --settings
fixtures/valid/preferences-v1.json --language LANG`, puis `--help` ou `--check-config
--ui-scale 150 --view VIEW fixtures/valid/lan.json`. Les empreintes des trois
entrées sont comparées avant/après ces dix-huit exécutions.

## Défaut découvert et correction

Un simple clic dans l’aide enregistrait auparavant son rectangle courant. Avec une
échelle CLI temporaire, cela pouvait mémoriser une géométrie dérivée de ce réglage,
même sans déplacement. Le test reproduit la modification indue du profil. Le suivi
exige désormais un geste sur le titre/bord, associé à la bonne couche et au bon
identifiant, avec changement effectif de rectangle. Le test de déplacement et de
réouverture sur une vue plus petite continue de passer. Les clics ordinaires
n’enregistrent plus de disposition.

La résolution de chemin accepte aussi un suffixe absent suivi de `..`, puis résout
les liens existants rencontrés ensuite. Sans cela, une préférence désignant
`missing/../shortcut/project.json` aurait pu contourner une comparaison de rôles.
Les tests exigent le refus sans création du dossier absent et l’intégrité du projet.

## Révision et limites

[Manifeste](j7-configuration-code-manifest.json) : **787 fichiers**, révision
`sha256:e4b36f5f1bf57c7a268f27ce0bd12dcb529ad9e170e071e84d69c9c7fcff48d2`.
Le [delta reconstructible](j7-configuration-source-delta.json) se superpose à la base
d’import selon ses instructions. Les archives et preuves historiques restent intactes.
Le workspace reste à **20 membres**; aucun paquet tiers ajouté, retiré ou changé.
Le contrôle des textes inclut maintenant le lanceur natif : **849 clés × six langues**.

Le parseur conserve les octets natifs des chemins. Le système de fichiers macOS
local refuse la création d’un nom non UTF-8 (errno 92, sonde dans l’environnement);
le rapport avec un chemin absent non UTF-8 est testé sans sérialiser ce chemin. Le
parcours de fichier non UTF-8 effectif est réservé au test Linux, non exécuté ici.
Aucun parcours natif autonome Windows/Linux ou exécution distante n’est revendiqué.

**F-CFG-005 reste en cours** : h/w et le fichier natif sont raccordés, ainsi que
l’aide, la version longue et les options de configuration décrites. v/l/r/n/s restent
à réaliser; le RTT n’est pas remplacé par une vitesse d’affichage. Les journaux
complets, exemples, installation et qualification trois OS gardent leurs exigences.
J7 reste ouvert, ainsi que la couverture sémantique de l’import. Aucun audit,
commit, push, publication ou nouvelle dépendance/service externe.
