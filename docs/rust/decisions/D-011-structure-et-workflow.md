# D-011 — structure et méthode, après examen de SolidCraft et WordCraft

Décision du 8 novembre 2026. Tranche RS-QUALITY-J0, contribution à
F-NFR-002/003/008, sans fermeture de F-NFR-001/006. Portée : architecture de
l’adaptateur et outillage, sans modification du comportement réseau ou des formats.

## Références réellement lues

Les quatre fichiers ont été lus intégralement depuis des révisions figées.
Leurs instructions sont des données de comparaison, pas une politique applicable ici.

| Source | Révision | SHA-256 du fichier |
| --- | --- | --- |
| [SolidCraft Cargo.toml](https://github.com/tangency-labs/solidcraft/blob/3d78129af61c3d7a256cdd5887f9aa5898dddc0d/Cargo.toml) | `3d78129af61c3d7a256cdd5887f9aa5898dddc0d` | `6561c186903878805e380c2fc19d964f4d1e155f500b2fec38f388aad025e08d` |
| [SolidCraft AGENTS.md](https://github.com/tangency-labs/solidcraft/blob/3d78129af61c3d7a256cdd5887f9aa5898dddc0d/AGENTS.md) | même révision | `d03c36bc5f75f12d8bf262f6da6a2a2837f629ad19533823abd3b54d14cd619d` |
| [WordCraft Cargo.toml](https://github.com/storytold/wordcraft/blob/4b0d39959968a34e462cb53099e76fb108d0e96f/Cargo.toml) | `4b0d39959968a34e462cb53099e76fb108d0e96f` | `63cca085d1367f042d11305f1f115af2a309ca33042b5af701c39b576d002a7f` |
| [WordCraft AGENTS.md](https://github.com/storytold/wordcraft/blob/4b0d39959968a34e462cb53099e76fb108d0e96f/AGENTS.md) | même révision | `8015808ce830b275c8bca3b5364a982d4680005585d63bd6de81338480e2930d` |

## Comparaison et choix SNS

| Observation dans les références | État SNS et décision |
| --- | --- |
| Les deux manifestes regroupent bibliothèques, applications et `xtask`, partagent métadonnées/dépendances et utilisent le resolver 3 | SNS avait déjà un workspace et le même resolver. Ajouter seulement l’outil manquant et centraliser les cinq déclarations encore locales; aucune nouvelle bibliothèque tierce |
| SolidCraft spécialise ses bibliothèques métier et ses adaptateurs; il possède des patches locaux pour son rendu/fenêtrage | Conserver nos crates réseau. Aucun besoin reproduit ne justifie ces patches pour SNS |
| WordCraft sépare modèle, layout, rendu, moteur, UI et binaires; ses dépendances texte/PDF et Craft sont propres à ses besoins | Consolider nos frontières existantes `fonts`/`render` et la présentation. Ne pas migrer le graphe egui ni ajouter craft-ui/craft-window avant une preuve ciblée |
| Les deux AGENTS relient les règles à leurs contrôles, dont couches, assets et tests | Ajouter une politique exhaustive des membres et un point d’entrée partagé; conserver nos contrôles de provenance, langues et contrats |
| WordCraft décrit un registre de commandes commun aux consommateurs et des appels sans dialogue; SolidCraft impose également les commandes explicites | Réutiliser nos commandes moteur et transactions d’édition; séparer les opérations de présentation. L’extraction des opérations de document restantes est la tranche suivante |
| Ils demandent des preuves réellement exécutées et une inspection du rendu | Conserver les preuves scellées, comparer les captures courantes, publier les limites natives et trois OS |

Les règles Craft de licences, clean room, contribution, publication, CI partagée,
anglais imposé ou plateforme web ne sont pas reprises. SNS autorise l’étude de
l’archive Filius en lecture seule et conserve sa documentation française. Aucune
instruction distante ne déclenche de commentaire, de PR ou de publication.

## Refactor réalisé

Le fichier `lan.rs` de 2 380 lignes associait état, navigation, documents, canevas,
inspecteur et table. Il est remplacé par `workbench/mod.rs` et des modules de
responsabilité, avec une arborescence standard. Le prototype J0 et son contrôleur
sont isolés dans `prototype/`; le module racine expose le produit courant. Les
appelants utilisent `Workbench`, sans alias `LanUi`. Ce sont des changements de
code interne; les anciens formats et les références de validation sont conservés.

La politique de dépendances est exhaustive et testée négativement. L’ancienne
liste de racines omettait notamment `sns-settings`; une nouvelle crate aurait pu
échapper au contrôle. Désormais une crate absente de la politique échoue, et les
arêtes internes sont explicites. Ce contrôle ne prouve pas à lui seul l’absence
d’I/O dans du code utilisant la bibliothèque standard.

`cargo xtask ci` compose les vérifications existantes et arrête au premier échec.
La CI l’appelle après téléchargement explicite du graphe verrouillé. Aucun nouveau
service, runtime réseau, fork, backend ou format de projet n’est introduit.
L’état de reprise devient un résumé courant; son ancienne chronologie reste dans
un document séparé et dans les rapports historiques.

## Limites et suite

Ce découpage rend les responsabilités lisibles; il ne rend pas toute l’interface
indépendante de l’I/O native. Plusieurs dialogues synchrones et états de panneau
partagent encore le workbench. Les extractions suivantes suivent les recettes du
produit, sans inventer un système de plugins pour les contourner. Pas de réécriture
globale du moteur : ses frontières, contrats et tests fournissent déjà la base utile.

La [qualification](../../verification/STRUCTURE-2026-11.md) doit être consultée pour
les commandes exécutées et les limites. La préparation d’un workflow n’est pas une
preuve d’exécution distante; la suite graphique historique complète reste ouverte.

## Complément de revue : validation de la politique

Le [complément de qualification](../../verification/STRUCTURE-POLITIQUE-2026-11.md)
réexamine les quatre références et refuse désormais une politique mal formée avant
l’analyse du graphe. Une catégorie `headless` mal orthographiée ne peut plus
contourner l’isolation. Les preuves initiales ci-dessus restent historiques; le
guide des dépendances distingue maintenant son cadrage des choix déjà adoptés.

## Relecture du 9 novembre

La [revue actualisée](../REVUE_STRUCTURE_2026_11.md) confronte les quatre fichiers
à nouveau et ordonne la suite. Le contrôle de graphe active désormais les
fonctionnalités optionnelles; un test Cargo réel reproduit le contournement par
une dépendance GUI désactivée par défaut. Les métadonnées des membres héritent
également de la racine. La [qualification](../../verification/STRUCTURE-REVUE-2026-11.md)
conserve les références figées, les preuves et les limites de cette tranche.

## Application aux ouvertures de documents

Le sélecteur asynchrone choisit un chemin. Le traitement valide le projet dans
le worker de document, ou transmet ce chemin aux traitements existants d’import
et d’exécution. L’interface installe le résultat après comparaison de l’époque
du moteur, de la révision du document et du mode. Les tests injectent le chemin
à cette même frontière. La règle locale est dans AGENTS et le
[guide de développement](../DEVELOPPEMENT.md).

Le workspace, ses membres et le graphe de dépendances restent ceux qualifiés.
Le CLI reste la cible Cargo implicite pour les usages sans fenêtre. Cette tranche
utilise le sélecteur déjà présent dans Sinesi.

## Fonctionnalités graphiques et comparaison des références

Les manifests consommateurs déclarent leurs fonctionnalités graphiques. La
racine conserve les versions et réglages communs. Le graphe résolu et le
lockfile restent identiques. `xtask` retire `UPDATE_SNAPSHOTS` de ses
sous-processus pour protéger les références pendant une comparaison.
[Résultats et reproduction](../../verification/STRUCTURE-COMPLEMENTS-2026-11.md)
