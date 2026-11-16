# Aide native — F-CFG-002

Le [profil portable](../agnostique/AIDE_J7.md) définit le comportement. Cette tranche
relève de RS-PLATFORM-J7, dont les autres exigences restent ouvertes.

- `sns-settings/src/help.rs` possède le catalogue de rubriques et d’exemples,
  la version du produit issue du workspace et les notices embarquées. Il ne
  dépend pas d’egui, n’ouvre aucun fichier et n’exécute aucune action.
- `workbench/help_ui.rs` possède la fenêtre, la navigation et les projections
  de version. Les actions d’exemple sont routées vers les `Replacement` existants;
  aucune seconde implémentation du cycle de document n’est ajoutée.
- `commands.rs` traite F1 avant le garde de focus texte; Échap est consommé quand
  il ferme l’aide, pour préserver les saisies et interactions derrière elle.
- `HelpUi` remplace l’ancien booléen et le bloc de texte de `chrome.rs`. Les widgets
  ont des identités stables, avec des valeurs localisées distinctes des enums internes.
- `DOCUMENTATION_SCHEMA_VERSION` est partagé avec la validation et l’évolution des
  annotations du modèle; il reste 9. Les contrats de données ne changent pas.

Les textes proviennent du catalogue six langues. Le test du catalogue parcourt
toutes les rubriques et les dix exemples; `check_i18n.py` inclut désormais aussi
`sns-settings`, ce qui a permis de traduire le diagnostic préexistant de version
de préférences non prise en charge. Aucune bibliothèque tierce, réseau ou fork n’est ajouté.

Commandes : `cargo test -p sns-settings --locked --offline`,
`cargo test -p sns-ui-egui --test help --locked --offline`, puis `cargo xtask ci`
et `cargo xtask ui`. Le dernier comprend désormais PNG, PDF et aide. Les nouvelles
captures `j7-help-*` complètent les références; celles des exports restent comparées
et conservées. Les résultats réellement obtenus figurent dans le
[rapport de vérification](../verification/J7-AIDE.md).

Les options CLI, polices agrandies persistantes, priorité de configuration,
diagnostics et packaging relèvent des prochaines exigences du lot plateforme.

## Repères pour la tranche de lancement suivante

Lecture en archive seule : `FiliusArgs.java` déclare h (aide), v (verbose), w
(répertoire de travail), l (fichier de log), r (facteur RTT), n (apparence native)
et s (petit écran). `Main.java:253` appelle le parseur, initialise le répertoire,
active le log si demandé et transmet le facteur RTT et la petite résolution.
Le champ `verbose` n’a été trouvé que dans sa déclaration et son affectation :
ne pas le présenter comme un comportement exécuté, ni réutiliser `-v` pour version.
`Verbindung.java:184` borne le facteur à 1–5 et l’utilise dans son estimation de RTT.
Suivre ses appelants avant de choisir une correspondance avec le temps virtuel;
ne pas le remplacer silencieusement par la vitesse d’affichage. Ces observations
préparent F-CFG-005 et ne déclarent aucune option supplémentaire réalisée dans SNS.
