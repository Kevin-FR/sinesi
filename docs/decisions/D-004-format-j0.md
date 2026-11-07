# D-004 — Format et persistance J0

État : socle JSON implémenté; emballage d’archives et import Filius futurs.

Le projet initial est un JSON UTF-8 `.sinesi.json` avec version de schéma 1 et profil `sns-educational-v1`. Le scénario de recette a sa propre version et contient projet, graine, actions et services de prototype. Les schémas formels sont maintenus dans `contracts/`, séparément des modèles d’implémentation.

Les lectures sont limitées à 8 Mio, y compris en cas de croissance du fichier après consultation de sa taille. Le document est validé avant utilisation. Les objets exécutables, les classes et l’évaluation XML n’existent pas dans ce lecteur. Une extension déclarée active est refusée; les pièces inactives doivent référencer une ressource présente.

Les limites de structure du socle sont 1 024 équipements, 4 096 liens, 128 interfaces par équipement et 32 768 actions initiales. Les corps binaires décodés sont bornés à 1 Mio. Les limites de présentation sont distinctes des limites d’événements de D-001.

Une sauvegarde valide d’abord le projet, écrit un temporaire sur le même volume, synchronise son contenu puis remplace atomiquement la cible. Sur les systèmes qui le permettent, le répertoire parent est synchronisé. Une erreur ne publie pas un fichier partiel. Le binaire CLI applique aussi une limite de 8 Mio à l’export de trace, annoncée comme limite initiale d’écriture.

La migration de qualification v0 renomme `title` en `name` et avance la version à 1; tous les autres champs doivent déjà respecter le profil. Le document source est conservé dans un fichier `*.v0.backup.json` créé sans écraser une sauvegarde antérieure. Une version inconnue est refusée. Cette fixture permet de recetter la conservation d’un original; elle n’affirme pas l’existence d’un ancien format Sinesi distribué et ne convertit pas `.fls`.

L’archive locale de Filius reste un corpus de lecture non embarqué dans le dépôt. Ses empreintes et l’inventaire sont conservés. Le contrôle documentaire annonce explicitement quand ce corpus manque et ne prétend alors pas revérifier son contenu.
