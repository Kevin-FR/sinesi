# Faire évoluer le logiciel

Une évolution conserve les identifiants fonctionnels, les contrats portables et les
recettes. Le découpage d’un adaptateur et ses bibliothèques peuvent changer. La
compatibilité des données ne se déduit pas de celle de son API interne.

## Tranche de travail

1. Partir de l’état de reprise, choisir les exigences et vérifier leurs prérequis.
2. Lire le chemin utilisé et identifier la responsabilité qui possède le comportement.
3. Définir l’effet observable et la recette qui distingue une réussite d’un échec.
4. Remplacer le chemin courant et adapter ses appelants ensemble. Éviter deux
   implémentations du même comportement ou une chaîne de façades provisoires.
5. Vérifier les contrats, l’absence de mutation sur échec et les régressions utiles.
   Une modification visuelle conserve langues, représentations et parcours clavier.
6. Enregistrer résultats et limites dans les deux registres, puis régénérer leurs vues.

Une opération métier est accessible par une commande ou transaction explicite.
Un dialogue collecte des paramètres; il n’est pas nécessaire à l’exécution de cette
opération. L’interface et le lanceur automatisé consomment les mêmes comportements.
Une commande de présentation, telle que changer de panneau, n’est pas un événement
réseau et ne doit pas encombrer le journal de simulation.

## Frontières à préserver

| Responsabilité | Règle vérifiable |
| --- | --- |
| Modèle et simulation | Aucun besoin de fenêtre ou d’accès réseau réel pour exécuter un exercice |
| Présentation | Lit un état appliqué; les résultats périmés sont refusés par identité de génération/révision |
| Persistance et import | Données bornées et validées, original préservé, aucun code embarqué exécuté |
| Rendu et rapports | Instantané explicite, indépendant du viewport et du cycle des protocoles |
| Pont réel | Activation explicite et entrées enregistrées; rejeu sans connexion réelle |
| Qualité | Toute nouvelle unité rejoint la politique de dépendances et les contrôles communs |

Les règles d’un autre produit servent de comparaison; ses contraintes de licence,
de plateforme ou de contribution ne deviennent pas automatiquement celles de Sinesi.

## Suite du chantier

La première tranche structure les frontières et les validations existantes sans
modifier les formats. Les suivantes extraient les opérations de document encore
couplées à la présentation, puis consolident préférences, livraison et parcours réels.
Chaque extraction doit avoir au moins deux consommateurs utiles ou simplifier une
frontière existante; une bibliothèque par bouton n’est pas un objectif.

Ces travaux contribuent à F-NFR-002/003/008. Ils ne suffisent pas à qualifier seuls
le fonctionnement réseau réel coupé (F-NFR-001), la totalité des contenus importés
(F-NFR-006), l’accessibilité ou la livraison sur trois systèmes.
