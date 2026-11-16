# Préférences d’affichage — J7

F-CFG-003, prérequis F-PROJ-001. Les préférences appartiennent au profil utilisateur;
le projet, l’exécution et les réglages d’une vue récupérée restent distincts.

## Comportement

- Un panneau de préférences permet de choisir taille, fonte, représentation,
  disposition des panneaux, largeurs et dimensions de fenêtre. Appliquer valide et
  enregistre; Annuler ou Échap abandonne le brouillon. Les valeurs par défaut sont
  proposées dans le brouillon et nécessitent aussi Appliquer.
- La taille de l’interface va de 80 à 200 %. Elle agrandit texte et commandes;
  elle est indépendante du zoom du réseau et de la vitesse de simulation. La fonte
  de document embarquée est disponible hors ligne, avec repli pour les symboles;
  le texte technique à chasse fixe garde sa famille dédiée.
- Les vues Icônes et Schéma restent disponibles. Leur choix devient la préférence
  par défaut; une vue de session récupérée peut temporairement choisir l’autre.
- L’inspecteur peut se trouver à droite, à gauche ou dans des onglets. Une largeur
  insuffisante active automatiquement les onglets Réseau/Outils/Inspecteur. Les
  panneaux défilent et le réseau est recadré lors d’un changement d’affichage.
  Le zoom réseau reste disponible pour lire ses libellés après un cadrage global.
- Les dimensions choisies pour la fenêtre principale sont réappliquées. Les
  fenêtres d’applications peuvent être agrandies et centrées à leur ouverture.
  Position et taille de l’aide et des applications sont mémorisées après interaction,
  par catégorie de logiciel. Elles sont contraintes à l’espace disponible sur le
  nouvel écran; plusieurs instances d’une même catégorie partagent ce défaut.
- Une consultation ou modification de préférence n’édite ni le projet, ni un paquet,
  ni un protocole. Une exécution déjà active peut continuer indépendamment.

## Format et limites

Le [contrat](../../contracts/preferences.schema.json) définit deux versions :
1 contient la langue; 2 ajoute le profil d’affichage. Lire une version 1 applique les
valeurs par défaut en mémoire, sans réécrire l’original. Une sauvegarde explicite
produit la version 2. Les six langues FR EN ES IT PT DE sont admises.

| Paramètre | Domaine |
| --- | --- |
| Taille interface | 80–200 % |
| Largeur outils / inspecteur | 160–360 / 240–480 unités logiques |
| Dimensions principales | largeur 840–3840, hauteur 600–2160 unités indépendantes de la densité |
| Taille initiale des applications | 75–150 % |
| Géométrie mémorisée | aide et 13 catégories connues, au plus 16 entrées |
| Coordonnées / dimensions mémorisées | coordonnées −16384 à 16384; largeur 100–7680, hauteur 60–4320 |
| Taille du fichier | 65536 octets maximum |

Les positions mémorisées sont relatives au contenu, en unités indépendantes du
zoom d’interface; les dimensions incluent le cadre et le titre. Le positionnement
réel est adapté à l’écran. Les champs inconnus, valeurs hors domaine et versions
futures sont refusés. Un fichier refusé est préservé et n’est pas remplacé par une
valeur de repli. Les erreurs d’écriture sont visibles et ne modifient pas le projet.
Le chemin de préférences est protégé des sauvegardes/exportations de projet;
les préférences ne peuvent pas écraser le projet ouvert ni la récupération.

## Recettes

1. Dans chaque langue et représentation, agrandir la fonte, appliquer, fermer
   normalement et rouvrir : choix conservés, octets du projet inchangés.
2. Annuler un changement, puis utiliser le raccourci avec une saisie active :
   brouillon et réseau préservés.
3. Déplacer l’aide, relancer sur une vue plus petite : fenêtre accessible, géométrie
   mémorisée et adaptée; terminal utilisable à 200 %.
4. Charger les fixtures 1 et 2 puis des variantes invalides : original intact;
   une sauvegarde invalide conserve la destination précédente.

Ce profil ne définit pas encore la priorité entre paramètres administrateur,
utilisateur et ligne de commande (F-CFG-004), ni les associations de fichiers ou la
qualification native sur trois OS.
