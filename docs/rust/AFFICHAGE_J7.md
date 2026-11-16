# Adaptateur des préférences d’affichage

F-CFG-003 / RS-PLATFORM-J7. Le [profil portable](../agnostique/AFFICHAGE_J7.md)
et son schéma restent l’autorité de comportement.

`sns-settings::display` porte les énumérations et données bornées; `Settings` écrit
la version 2 et lit la version 1 sans changement du fichier. La validation précède
la création de dossiers et l’écriture atomique. Aucun nouveau paquet tiers :
`jsonschema`, déjà utilisé pour les contrats, devient aussi dépendance de test de
`sns-settings`.

`workbench/preferences_ui.rs` possède brouillon, réglages appliqués et un worker
unique d’écriture. Un canal de capacité 1 transporte le résultat; une seule valeur
complète en attente coalesce les changements suivants. Changer la langue conserve
l’ensemble des préférences. La fermeture attend les écritures demandées, y compris
sans récupération de session. Le worker ne possède ni projet mutable ni moteur.

Le facteur de zoom egui est appliqué à la passe suivante, indépendamment du zoom
monde/écran du canevas. La famille `sns-ui-noto` réutilise Noto Sans embarquée puis
les replis proportionnels d’egui; les familles d’annotation restent en place. Les
styles à chasse fixe gardent leur famille. Les fontes du système ne sont pas chargées.

Le recadrage suit les dimensions réellement appliquées après une commande de fenêtre;
un déplacement ou zoom manuel et une vue récupérée arrêtent ce suivi. Les actions
d’annotation précèdent les champs de style. La ligne de saisie du terminal peut
revenir à la ligne à forte échelle.

La disposition adaptative consomme les mêmes panneaux; aucun second inspecteur ou
éditeur réseau n’est créé. Les fenêtres utilisent des identifiants stables et des
rectangles extérieurs, enregistrés après relâchement du pointeur, dans le profil.
La géométrie est partagée par catégorie de logiciel et adaptée à la zone visible.
Les dimensions principales proviennent du panneau; les déplacements manuels de la
fenêtre native principale ne sont pas enregistrés automatiquement.

Le bouton Préférences et Cmd/Ctrl+, ouvrent le panneau, même avec une saisie active.
Appliquer utilise la même méthode publique `set_display_preferences` que les tests;
Annuler et Échap préservent les brouillons du document. Les outils sans frame peuvent
attendre une écriture via `wait_for_preferences`; la boucle de rendu la sonde.

## Validation

- `cargo test -p sns-settings --locked --offline` : migration, bornes, contrat,
  conservation des originaux et de la destination sur échec.
- `cargo test -p sns-ui-egui --test preferences --locked --offline` : six langues,
  deux vues, tailles 150/200 %, non-mutation réseau, redémarrage, saisie active,
  géométrie et terminal agrandi.
- `cargo xtask ci`, puis `cargo xtask ui` : contrôles communs et captures actuelles.

Le bouton Préférences et les fenêtres adaptatives changent volontairement le rendu
courant. Les nouvelles captures portent le préfixe `j7-display-`; les références
PNG/PDF/aide antérieures sont conservées. Leur génération initiale est séparée de
la comparaison finale sans mise à jour. Les preuves sont dans le rapport de tranche;
aucune manipulation de sélecteur natif ou preuve Windows/Linux n’est déduite du harnais.
