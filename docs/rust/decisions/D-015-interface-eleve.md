# D-015 — interface élève à niveaux et nouveau jeu de références

Décision du 16 novembre 2026, F-UX-001 à F-UX-006 / RS-UX-J8.
[Comportement portable](../../agnostique/EXPERIENCE_ELEVE.md).

L’atelier J7 expose une vingtaine de commandes texte sur trois à quatre lignes,
des identifiants d’exécution et un vocabulaire d’ingénierie. Le public visé
(collège et lycée) a besoin d’un parcours court : construire, tester, annoter,
avec des consignes visibles et des explications en langage courant.

**Une seule présentation.** Le `Workbench` reste l’unique interface. Un niveau
d’affichage Élève/Expert est une préférence de présentation, persistée dans le
profil d’affichage comme la représentation Icônes/Schéma. Le niveau ne modifie ni
le projet, ni l’exécution, ni les transactions : il choisit seulement quelles
commandes sont proposées. Les commandes avancées (import Filius et son rapport,
exécutions/rejeu/checkpoints, export du réseau, détail d’exécution) restent les
mêmes transactions, accessibles au niveau Expert et par la CLI. Aucun second écran
élève n’est créé, conformément à la règle « pas de double implémentation ».

**Compatibilité du profil.** Le champ `display.level` est facultatif dans la
version 2 du profil; son absence vaut `student`. Un ancien profil reste lisible
sans réécriture. Un binaire antérieur qui lit un profil contenant ce champ le
refuse comme champ inconnu et applique son repli de démarrage existant, sans
perte du projet.

**Références graphiques.** Toutes les captures de l’atelier changent avec la
nouvelle barre. Les PNG de référence J0–J7 sont déplacés tels quels dans
`crates/sns-ui-egui/tests/snapshots-historique/atelier-j7/` avec un manifeste
SHA-256; ils ne sont plus comparés. Les tests conservent leurs noms et un nouveau
jeu est produit par les suites graphiques ciblées, puis inspecté. Ce déplacement
est un changement d’interface voulu, pas une régénération pour faire passer une
suite.

**Découpage.** Tranche 1 : niveau et barre à trois espaces avec consignes.
Tranches suivantes : palette illustrée et câble guidé, paquets animés et récit du
trajet construits depuis la trace, terminal lisible, fiche pédagogique, puis
missions évaluables (F-ADV-004) avec un format portable à contractualiser.
