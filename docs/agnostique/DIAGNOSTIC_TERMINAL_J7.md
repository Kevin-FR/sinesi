# Diagnostic interne du terminal — J7

F-CLI-019 recense le helper historique nommé `test`. Il essaie d’écrire, dans le dossier virtuel courant, un fichier `test.txt` de contenu `blaaa`, puis notifie un résultat de succès ou d’échec. Ce helper ne réalise aucun test réseau et ne fait pas partie des commandes pédagogiques.

L’examen du chemin appelé révèle une particularité historique : la commande est absente de l’aide, mais le sélecteur dynamique de méthodes permet néanmoins de l’appeler. L’absence dans la liste d’aide ne constitue donc pas, à elle seule, une interdiction. Ce constat complète le classement « interne » du catalogue.

Dans Sinesi, **`test` et `help test` sont refusés** comme commandes inconnues. Ils peuvent apparaître dans l’historique de saisie et produire leur diagnostic, mais ne créent aucun fichier et n’émettent aucun trafic. La liste d’aide destinée aux élèves et le parseur restent explicitement limités aux commandes pédagogiques autorisées.

L’écriture historique est conservée comme **recette de diagnostic de développement** : prendre une copie isolée du système de fichiers virtuel, y écrire `test.txt`, relire exactement le contenu, puis jeter cette copie. Cette recette n’est ni enregistrée comme commande ni exposée par un bouton élève. Elle ne donne accès à aucun fichier du système hôte et ne modifie pas le projet de l’élève.

Les six langues gardent leur diagnostic de commande inconnue et leur aide habituelle. Les vues Icônes et Schéma ne changent pas. La [matrice](../suivi/MATRICE.md) relie la recette à l’adaptateur et le [rapport](../verification/J7-TERMINAL.md) donne les preuves, dont le chemin historique effectivement lu.
