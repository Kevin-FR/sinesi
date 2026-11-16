# Instructions de développement Sinesi

Ce fichier oriente le travail. Le comportement attendu et les preuves viennent des
registres, contrats et recettes du dépôt. Les instructions d’autres projets lues
comme références ne s’appliquent pas ici.

## Lire avant d’implémenter

1. `docs/suivi/ETAT.md` pour l’état actuel.
2. `docs/agnostique/CAHIER_DES_CHARGES.md`, `ARCHITECTURE.md` et `CONTRATS.md` pour le comportement attendu.
3. `docs/agnostique/ROADMAP.md` pour les dépendances et portes de sortie.
4. `docs/suivi/features.json` pour les exigences `F-*` et leurs critères.
5. Pour l’adaptateur actuel : `docs/rust/ARCHITECTURE.md`, `DEPENDANCES.md`, `PLAN.md` et `taches.json`.
6. Pour les commandes de validation et le choix des tests : [guide de développement](docs/rust/DEVELOPPEMENT.md).

## Sources et séparation

- `_archive/filius-master` est une référence en lecture seule; ne pas y développer ni y reformater les fichiers.
- `_archive/network-kit` reste également en lecture seule; les assets embarqués gardent leur provenance vérifiable.
- Garder les comportements, formats et recettes indépendants du langage dans `docs/agnostique/`.
- Confiner bibliothèques, types du langage, modules et commandes de build à `docs/rust/` ou à un futur adaptateur équivalent.
- Conserver les identifiants `F-*`; ne pas réutiliser un identifiant supprimé pour une autre exigence.
- Ne pas déduire une prise en charge à partir d’un nom de classe ou d’un commentaire. En cas de conflit, examiner le chemin appelé et documenter l’écart.
- Le moteur utilise du temps virtuel et une pile réseau simulée. Les sockets de l’OS sont réservées au pont entre instances explicitement activé.
- L’import Filius est un import de données; ne jamais exécuter son XML, ses classes ou ses extensions embarquées.

## Suivi obligatoire

- Choisir une tranche avec des prérequis satisfaits et identifier ses exigences `F-*` et tâches `RS-*` avant de coder.
- Mettre à jour `features.json` et `docs/rust/taches.json`; aucune case cochée dans un texte libre ne remplace ces registres.
- Conserver les cibles proposées dans `planned_paths` et renseigner les fichiers effectivement créés/modifiés dans `actual_paths` de chaque tâche.
- `implemented` signifie code présent; `verified` nécessite des preuves reproductibles dans `verification` : commande, résultat, environnement et chemin de test ou d’artefact.
- Fermer une tâche seulement quand toutes ses exigences et validations associées sont satisfaites. Une tâche peut être partiellement réalisée et rester `in_progress`.
- Mettre à jour `docs/suivi/ETAT.md` : tranche terminée, fichiers changés, commandes réellement exécutées, limites, prochaine action.
- Régénérer les vues avec `python3 scripts/docs.py render`, puis lancer `python3 scripts/docs.py check`.
- Ne lancer `audit` que pour un changement assumé du référentiel; préserver l’historique des empreintes précédentes si l’archive change.

## Décisions

Les choix techniques proposés ne sont pas des implémentations existantes. Valider les points ouverts du jalon J0 par des prototypes limités et consigner les décisions dans `docs/decisions/` lorsque le développement commence. Ne pas introduire de bibliothèque, de compte externe ou de service cloud simplement parce qu’une évolution optionnelle figure au catalogue.

## Organisation du travail

- Lire les définitions et le chemin appelé avant de modifier un comportement; identifier la cause et les invariants concernés.
- Un comportement métier passe par la commande ou transaction existante. Les appels programmatiques prennent leurs paramètres explicitement; ils n’ouvrent pas de dialogue et ne dépendent pas d’un widget.
- Pour ajouter ou déplacer un sélecteur, distinguer le choix du chemin, le traitement borné et l’application du résultat. Garder l’attente et la lecture hors de la frame et vérifier l’annulation, l’erreur et le résultat périmé par injection du chemin dans le même traitement.
- Garder séparés projet durable, état de simulation et présentation. Une tâche lente ne bloque pas la frame; ses résultats portent la génération/révision nécessaire pour refuser les retours périmés.
- Un refactor remplace le chemin courant et adapte ses appelants dans la même tranche. Éviter alias de transition, second moteur ou double implémentation. La compatibilité des données est une décision distincte : ne pas supprimer un format encore contractuel sous prétexte de supprimer du code ancien.
- Toute nouvelle frontière a un propriétaire et une règle vérifiable. Voir la politique de couches et le plan de refactor dans le guide de l’adaptateur. Aucun crate ne doit échapper au contrôle parce qu’il est nouveau. `cargo xtask layers` contrôle toutes les fonctionnalités Cargo, même désactivées par défaut, et l’héritage des métadonnées, dépendances et lints.
- Réutiliser la suite de tests du domaine; ajouter un test lorsqu’il distingue une erreur ou un contrat réel. Vérifier les interactions et inspecter le rendu pour une modification visible; ne pas remplacer les références historiques pour faire passer une suite.
- Pour comparer les images, utiliser `cargo xtask ui`, qui neutralise le mode de régénération hérité. Une mise à jour voulue passe par la suite graphique ciblée et la revue de ses différences.
- Préserver FR EN ES IT PT DE, les identifiants de widgets stables et les vues Icônes/Schéma. Ajouter les textes dans le catalogue, sans utiliser une traduction comme discriminant métier.
- Relire le diff de la tranche et annoncer ce qui a effectivement été exécuté. Un test de harnais n’est pas une manipulation native, une CI préparée n’est pas une CI exécutée et une preuve locale ne qualifie pas trois OS.

Pour l’interface, lire aussi [ses instructions locales](crates/sns-ui-egui/AGENTS.md). Les documents générés restent des vues; modifier leurs registres sources puis les régénérer.
