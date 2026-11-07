# Instructions de développement SNS

## Lire avant d’implémenter

1. `docs/suivi/ETAT.md` pour l’état actuel.
2. `docs/agnostique/CAHIER_DES_CHARGES.md`, `ARCHITECTURE.md` et `CONTRATS.md` pour le comportement attendu.
3. `docs/agnostique/ROADMAP.md` pour les dépendances et portes de sortie.
4. `docs/suivi/features.json` pour les exigences `F-*` et leurs critères.
5. Pour l’adaptateur actuel : `docs/rust/ARCHITECTURE.md`, `DEPENDANCES.md`, `PLAN.md` et `taches.json`.

## Sources et séparation

- `_archive/filius-master` est une référence en lecture seule; ne pas y développer ni y reformater les fichiers.
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
