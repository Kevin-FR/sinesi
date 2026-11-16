# Guide de lecture et de maintenance

Le réseau local J1 est réalisé : voir le [profil portable](agnostique/LAN_J1.md), sa [réalisation](rust/J1.md), le [rapport de vérification](verification/J1.md) et l’[état de reprise](suivi/ETAT.md).

Le jalon J2 est également réalisé : [comportements portables](agnostique/HOTES_J2.md), [transport](agnostique/TRANSPORT_J2.md), [réalisation Rust](rust/J2.md) et [preuves J2](verification/J2.md). Le catalogue de logiciels conserve explicitement les fonctions des jalons futurs.

Le jalon J3 est qualifié : [routage portable](agnostique/ROUTAGE_J3.md), [réalisation](rust/J3.md) et [preuves](verification/J3.md). Il ajoute aussi les représentations pédagogiques F-TOP-011 issues des SVG fournis. Les preuves J0/J1/J2 restent historiques.

Le jalon J6 est qualifié : [applications et connexions portables](agnostique/APPLICATIONS_J6.md), [réalisation](rust/J6.md), [preuves](verification/J6.md) et [état actuel](suivi/ETAT.md). Les 13 logiciels du catalogue sont disponibles dans le périmètre pédagogique défini.

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

Le jalon J4 dispose de son [profil de services](agnostique/SERVICES_J4.md), [adaptateur](rust/J4.md) et [rapport de qualification](verification/J4.md).

J5 dispose de son [profil des réseaux privés](agnostique/RESEAUX_PRIVES_J5.md), de son [adaptateur](rust/J5.md) et de sa [qualification](verification/J5.md).

## Structure et travail courant

La [méthode portable](agnostique/EVOLUTION.md), le [guide de développement](rust/DEVELOPPEMENT.md) et [D-011](rust/decisions/D-011-structure-et-workflow.md) complètent la reprise du dossier. La carte Rust décrit désormais les modules réalisés; les propositions futures restent identifiées séparément. Les anciennes preuves et leurs révisions restent disponibles.

Les [préférences d’affichage J7](agnostique/AFFICHAGE_J7.md) ont leur [adaptateur Rust](rust/AFFICHAGE_J7.md) et un contrat utilisateur distinct du projet.

La [configuration de démarrage](agnostique/CONFIGURATION_J7.md) décrit priorités, valeurs sûres et options temporaires; l’[adaptateur](rust/CONFIGURATION_J7.md) liste les commandes actuellement réalisées.
