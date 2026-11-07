# Guide de lecture et de maintenance

## Organisation

| Dossier | Autorité | À emporter pour un autre langage |
| --- | --- | --- |
| `agnostique/` | Besoins, comportements, recettes, découpage et jalons | Oui, intégralement |
| `suivi/features.json` | Exigences identifiées, dépendances, provenance et état fonctionnel | Oui; réévaluer les preuves d’implémentation |
| `suivi/domaines.json` | Familles et composants logiques | Oui |
| `suivi/` autres fichiers | Situation de travail et vues de liaison | Adapter aux nouveaux lots |
| `rust/` | Réalisation proposée dans le langage choisi | Remplacer par un autre adaptateur |
| `audit/` | Référentiel historique, preuves et empreintes | Oui pour conserver la traçabilité |

Le code historique expose des comportements pédagogiques simplifiés. Le catalogue ne prétend pas transformer Filius en émulateur complet d’un système d’exploitation ou en implémentation exhaustive des protocoles Internet.

## Ordre de lecture

1. [Cahier des charges](agnostique/CAHIER_DES_CHARGES.md).
2. [Architecture](agnostique/ARCHITECTURE.md) et [contrats](agnostique/CONTRATS.md).
3. [Roadmap](agnostique/ROADMAP.md) et [recettes intégrées](agnostique/RECETTE.md).
4. [Catalogue détaillé](agnostique/CATALOGUE.md), par domaine.
5. [Architecture Rust](rust/ARCHITECTURE.md), [pile native et dépendances](rust/DEPENDANCES.md), puis [lots techniques](rust/PLAN.md).
6. [État de reprise](suivi/ETAT.md) et [matrice](suivi/MATRICE.md).

## Sources de vérité

Les registres JSON sont les sources de vérité pour les identifiants, états et liaisons. Les textes expliquent l’intention et les décisions. `CATALOGUE.md`, `MATRICE.md`, `rust/PLAN.md` et `audit/SOURCES.md` sont des vues générées.

Une correction de comportement peut nécessiter une mise à jour du cahier des charges et du registre. Une simple évolution de bibliothèque doit rester dans le volet d’implémentation.

```sh
# Après modification des exigences ou des tâches
python3 scripts/docs.py render
python3 scripts/docs.py check

# Seulement pour recalculer l’index de référence
python3 scripts/docs.py audit
```

## Réutilisation avec un autre langage

Copier la partie agnostique, les contrats, les recettes, les registres fonctionnels et les preuves de l’audit. Créer un dossier d’adaptateur et son registre de tâches portant les mêmes `F-*`. Remplacer la matrice technique et l’état de reprise. Ne pas copier les états `verified` sans rejouer leurs recettes sur la nouvelle réalisation. Le script fourni génère actuellement les vues de l’adaptateur Rust; il faudra lui ajouter le nouvel adaptateur ou produire une matrice équivalente.
