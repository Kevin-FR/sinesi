# Diagnostic technique J7

La tranche couvre F-CFG-006 et les intentions v et l de F-CFG-005. Le journal situe
un incident par une étape et un numéro d’ordre. Le journal et son export ne
reçoivent que des codes d’événements prédéfinis. Les messages d’erreur libres, les
arguments, les chemins, les identifiants d’équipements, les secrets, les courriers,
les fichiers virtuels et les trames restent hors de ces fichiers.

## Lancement et export

- `-v` ou `--verbose` ajoute les étapes de réussite détaillées et les affiche sur
  la sortie d’erreur. Les résultats automatiques restent sur la sortie standard.
- `-l` ou `--log` crée un journal nommé par lancement dans le répertoire de travail
  effectif. `--log-file PATH` choisit un nom précis et prime sur le nom automatique.
  Le lanceur annonce le chemin absolu. Il réclame un fichier neuf et refuse un
  chemin réservé au projet, aux profils ou à la récupération.
- Dans **Exécutions**, **Exporter le diagnostic technique…** enregistre les
  événements récents. Cette action est disponible en conception et en simulation.
  Les protections de destination s’appliquent aussi au journal déjà ouvert.

Les événements sont numérotés dans l’ordre où l’application les observe.
Le journal de lancement conserve les 256 premières entrées au format JSONL.
L’export manuel conserve jusqu’aux 256 dernières entrées dans un objet JSON versionné
et indique combien d’entrées antérieures ont été écartées. Les lignes du fichier
de lancement ont la même forme que les entrées de l’export. Le
[schéma](../../contracts/diagnostic.schema.json) définit leurs champs et codes.

## Recette

Lancer un contrôle de configuration avec journal explicite dans un répertoire de
travail différent. Vérifier le chemin annoncé, le journal et le rapport sur la sortie
standard. Relancer avec la même destination pour vérifier la conservation du fichier.
Ouvrir un projet avec courrier puis tenter de restaurer un checkpoint invalide.
Exporter le diagnostic depuis Exécutions. L’export situe l’échec de lecture ou de
reconstruction sans recopier le fichier, le projet ou le contenu des messages.
