# Revue de la structure et de la méthode — 9 novembre 2026

Cette revue complète [D-011](decisions/D-011-structure-et-workflow.md). Elle
contribue à F-NFR-002/008 dans RS-QUALITY-J0; elle ne ferme pas les exigences
restantes de J7. Les registres conservent leurs identifiants et cibles initiales.

## Références examinées

Les liens `main` demandés ont été résolus en révisions immuables. Les quatre
fichiers ont été lus; leurs consignes sont étudiées comme données de comparaison.
Les empreintes sont dans le [registre des sources](../verification/structure-review-2026-11-references.json).

| Dépôt | Fichier | Révision |
| --- | --- | --- |
| tangency-labs/solidcraft | [Cargo.toml](https://raw.githubusercontent.com/tangency-labs/solidcraft/c7fb4f7cab28392965e44dc5b419690eed18c783/Cargo.toml) | `c7fb4f7cab28392965e44dc5b419690eed18c783` |
| tangency-labs/solidcraft | [AGENTS.md](https://raw.githubusercontent.com/tangency-labs/solidcraft/c7fb4f7cab28392965e44dc5b419690eed18c783/AGENTS.md) | `c7fb4f7cab28392965e44dc5b419690eed18c783` |
| storytold/wordcraft | [Cargo.toml](https://raw.githubusercontent.com/storytold/wordcraft/4b0d39959968a34e462cb53099e76fb108d0e96f/Cargo.toml) | `4b0d39959968a34e462cb53099e76fb108d0e96f` |
| storytold/wordcraft | [AGENTS.md](https://raw.githubusercontent.com/storytold/wordcraft/4b0d39959968a34e462cb53099e76fb108d0e96f/AGENTS.md) | `4b0d39959968a34e462cb53099e76fb108d0e96f` |

## Choix pour Sinesi

| Constat dans les références | Choix appliqué ou maintenu ici |
| --- | --- |
| Workspace réunissant bibliothèques, binaires et outillage, métadonnées et dépendances partagées | Conserver nos 20 membres et contrôler l’héritage de chaque manifeste, y compris version, édition, compilateur minimal, licence et dépôt |
| WordCraft inclut ses applications principales parmi les membres Cargo par défaut. SolidCraft possède un profil de CI dédié | Garder le CLI comme cible Cargo implicite pour l’usage sans fenêtre. Les contrôles explicites de `xtask ci` couvrent tout le workspace |
| Règles reliées à des contrôles de couches, assets et tests | Garder AGENTS comme point d’entrée et `xtask` comme validation commune; renforcer le contrôle du graphe avec toutes les fonctionnalités optionnelles |
| Commandes explicites consommables sans dialogue, métier séparé de la présentation | Conserver les transactions et commandes actuelles; extraire les derniers sélecteurs natifs au rythme des opérations concernées |
| SolidCraft utilise des correctifs locaux de rendu et de fenêtrage. WordCraft fixe des révisions de bibliothèques Craft | Conserver la pile upstream actuellement qualifiée. Tout remplacement requiert un défaut reproduit et une qualification native ciblée |
| Instructions et frontières spécifiques à leurs produits | Conserver nos règles de provenance, documentation française, import Filius en données et temps virtuel; aucune règle distante de contribution ou de publication ne s’applique |

Le découpage courant possède déjà un modèle, un moteur autonome, les protocoles,
les applications simulées, des adaptateurs d’import/persistance, un rendu statique,
les rapports, la présentation et deux binaires. La séparation `fonts` / `render` /
`report` évite de reconstruire un document depuis des widgets. Aucun défaut observé
ne justifie de remplacer ces propriétaires par une nouvelle chaîne de façades.

Le refactor précédent a supprimé `LanUi` et adapté ses consommateurs à `Workbench`.
Il n’existe pas d’alias de transition à entretenir. Le prototype J0 reste isolé pour
ses recettes explicites. Les anciens formats contractuels demeurent des données
prises en charge : supprimer du code ancien n’autorise pas à perdre des projets.

## Suite ordonnée, sans double implémentation

| Tranche et responsable | Point de départ actuel | Résultat attendu avant fermeture |
| --- | --- | --- |
| Ouverture, RS-PROJECTS-J1/J7, F-PROJ-001/005 | Sélection du projet, import et fichiers d’exécution extraits de la frame | Sélection et lecture bornée réutilisent les opérations existantes; annulation, erreur et résultat périmé préservent le document |
| Enregistrement et exports restants, RS-PROJECTS-J1/J7 et RS-OBSERVATION-J7 | Sélecteurs synchrones pour le projet, le rapport d’import et certaines traces | Préserver la confirmation des modifications et la protection des destinations avec des opérations explicites |
| Livraison, RS-PLATFORM-J7 et RS-QUALITY-J7, F-CFG-007 et F-NFR-007 | Exemples qualifiés localement, workflow trois OS préparé | Paquets et associations testés sur chaque OS, installation et désinstallation préservent les documents |

Le détail d’implémentation reste dans l’adaptateur. Les critères métier et
formats restent dans `docs/agnostique/` et les contrats. Les tâches sources font foi.
Une extraction remplace son chemin et adapte tous ses appelants dans la même
tranche. Aucun bus générique, système de plugins ou crate par écran n’est requis.

## Correction réalisée et preuve

Le contrôle précédent utilisait la résolution Cargo par défaut. Une bibliothèque
métier pouvait déclarer egui comme dépendance optionnelle et passer le contrôle
tant que l’option restait désactivée. Un petit workspace local reproduit ce cas,
sans téléchargement ni compilation. `--all-features` révèle maintenant cette
arête et le contrôle refuse la dépendance. Les arêtes de build et conditionnelles
restent couvertes, les arêtes de test seules restent autorisées.

Douze tests vérifient ce cas, la politique et l’héritage des manifestes. Les
métadonnées locales divergentes sont refusées; les choix de fonctionnalités des
dépendances héritées restent permis. La [qualification de la tranche](../verification/STRUCTURE-REVUE-2026-11.md)
détaille les commandes exécutées. L’analyse de toutes les fonctionnalités ne vaut
pas compilation de toutes les combinaisons ni validation de toutes les plateformes.

La tranche d’ouverture suivante applique la séparation du sélecteur et de
l’opération aux projets, à Filius et aux fichiers d’exécution. Les tests
injectent le chemin dans le même traitement.
[Qualification de l’ouverture](../verification/STRUCTURE-OUVERTURE.md)

## Complément sur le workspace et les contrôles

Les quatre fichiers de `main` correspondent toujours aux empreintes publiées.
SolidCraft rend explicite le choix des backends par les applications. La racine
de Sinesi centralise les versions et réglages communs. Le binaire bureau active
désormais WGPU, AccessKit, le presse-papiers et les backends Linux d’eframe dans
son manifeste. La présentation déclare ses fontes egui et le support SVG dans le
sien. La comparaison de `cargo metadata --all-features` couvre 521 paquets,
avec le même graphe, les mêmes fonctionnalités résolues et le même membre
par défaut. Les profils de SolidCraft et WordCraft servent à leurs charges de
travail respectives. Sinesi garde les siens tant qu’une mesure propre au projet
ne motive pas leur modification.

Les deux projets relient la comparaison visuelle à leur méthode de revue.
La commande `cargo xtask ui` transmettait auparavant `UPDATE_SNAPSHOTS` à
egui_kittest, qui peut réécrire la référence et faire passer le test. Le contrôle
retire maintenant cette variable de ses processus enfants. Une régénération
voulue passe par la suite ciblée, puis par la revue de ses différences.
Les règles de contribution externes, forks et outils de contrôle distant
restent spécifiques aux projets sources. La prochaine extraction de code
produit reste l’enregistrement et les exports à sélecteur synchrone.
[Qualification du complément](../verification/STRUCTURE-COMPLEMENTS-2026-11.md)
