# Export de réseau — J7

F-DOC-003 produit une image du réseau et de ses annotations appliquées. Ce profil complète les [annotations](ANNOTATIONS_J7.md); le [rapport PDF F-DOC-004](RAPPORTS_J7.md) utilise la même scène. La [matrice](../suivi/MATRICE.md) et les [preuves](../verification/J7-PNG.md) relient comportement et adaptateur.

## Instantané et scène

L’export fige projet, vue Icônes/Schéma et libellés confirmés au moment de la demande. Il ne lit pas les widgets pour fabriquer l’image : une scène documentaire contient rectangles, segments, symboles et blocs de texte. Chaque élément garde son identité source; l’inventaire inclut tous les équipements, câbles et annotations. Les positions absentes reçoivent la même disposition initiale que l’éditeur, sans écrire dans le projet.

Les annotations sont peintes dans leur ordre, puis les câbles et associations sans fil, puis les équipements et leurs libellés. Les vingt symboles embarqués sont communs à l’écran et à l’export. Notes/zones, dimensions, couleurs, gras, italique, soulignement et famille de police suivent le projet. Sélection, poignées, panneaux, brouillons non appliqués et animation de paquets ne sont pas dessinés. Les conditions temporaires de simulation ne deviennent pas des annotations du plan.

En simulation graphique, les libellés effectifs sont figés séparément : une IPv4 attribuée par DHCP peut apparaître sur l’image sans transformer la configuration DHCP persistante en adresse statique. Le lanceur qui reçoit seulement un fichier projet utilise ses libellés configurés; il ne simule pas une attribution pour inventer un état courant.

La scène est commune aux adaptateurs de document : coordonnées, couleurs, police, lignes, glyphes et identités sont indépendants des widgets. Le backend PDF et ses sections ne sont pas déclarés réalisés par la seule présence de cette scène.

## PNG et cadrage

Le fichier est un **PNG RGBA 8 bits**, transparent par défaut. Une option choisit le fond opaque du canevas : clair en Schéma, sombre en Icônes. Il n’existe aucun cadre de fenêtre dans l’image. Le cadrage considère l’étendue complète, y compris positions négatives, annotations hors de la vue actuelle et documents sans équipement mais avec notes. Un document entièrement vide est refusé avec diagnostic.

La taille demandée est le côté le plus long, **256 à 4 096 pixels**, par défaut **2 400**. Le rapport largeur/hauteur est conservé, avec au moins un pixel utile par axe. Une marge de scène de 24 unités et une marge supplémentaire de **16 pixels par côté** évitent une coupe en bord d’image, même pour un plan très étendu. La résolution ne définit aucune échelle physique du réseau. Un plan immense peut produire de très petits objets; son cadrage reste complet.

Les couleurs du projet sont sRGB RGBA non prémultipliées. La composition respecte l’alpha. Les pixels transparents autour du plan restent réellement transparents; choisir un fond opaque ne modifie pas le projet. Un symbole noir sur fond transparent nécessite naturellement un support clair pour son affichage; le fond de canevas est disponible dans les options.

## Texte et limites

Les polices sont embarquées; aucun téléchargement ni recherche de police système n’est nécessaire. La mise en forme utilise les substitutions/positions OpenType, dont ligatures et accents combinés. Les quatre variantes proportionnelle/fixe et normale/grasse sont disponibles; l’italique applique une inclinaison et le soulignement suit les lignes.

Le profil de texte est horizontal gauche à droite, qualifié pour FR EN ES IT PT DE. Il ne revendique pas un support universel des écritures. Un caractère absent ou une direction non prise en charge produit un diagnostic avec son code Unicode, avant toute écriture de destination. Aucun carré vide n’est substitué silencieusement.

Les lignes reviennent à la ligne aux espaces/tirets; un mot long est coupé aux frontières des groupes de glyphes. Le texte des annotations est mis en page dans la zone utile avec marge de 6 et découpé au cadre de l’annotation. Les libellés d’équipements occupent une ligne, avec points de suspension si nécessaire. Le nombre de blocs dépassant leur zone utile est annoncé dans le résultat, afin de raccourcir un libellé ou agrandir une annotation. L’exemple documentaire contient désormais assez de hauteur pour afficher ses trois étapes.

Limites du profil : 65 536 octets par bloc de texte, 1 Mio de texte cumulé dans la scène, 200 000 glyphes visibles, 2 000 000 verbes de tracé mis en cache, 4 096 × 4 096 pixels maximum et 64 Mio pour le PNG encodé. Le projet garde ses limites propres. Une saturation refuse l’export; elle ne retire pas silencieusement des objets. Ces bornes ne sont pas une mesure de performance ou de mémoire maximale de l’application entière.

## Écriture et interface

La sélection du fichier et le rendu lourd sont effectués hors du traitement des interactions. Une seule opération PNG est active par instance. Le résultat provient de l’instantané figé, même si l’utilisateur continue à éditer. Le réseau n’est ni arrêté ni avancé par l’export; un scénario déjà en cours continue indépendamment.

Le fichier est entièrement préparé avant son remplacement atomique. Une erreur de rendu conserve une destination existante et le document. La GUI refuse le projet ouvert, le fichier de préférences et le stockage géré des récupérations. Pendant l’écriture, cette destination est réservée contre une autre sauvegarde de l’application. La fermeture normale attend l’opération PNG. Le lanceur refuse aussi de remplacer son projet source.

Les contrôles et diagnostics existent dans les six langues. Les options d’image sont des choix de présentation, séparés du projet. Les dialogues natifs et le parcours sur trois systèmes gardent leur porte de validation propre; les captures du harnais ne valent pas preuve de dialogue natif.

## Vérification

La recette vérifie les deux vues, tous les symboles, les quatre polices, accents et styles, un document sans nœud, les positions négatives et la conservation des données. Les mêmes entrées produisent des octets identiques dans deux exports indépendants sur la plateforme qualifiée. La GUI et le lanceur produisent le même PNG que la scène de référence.

Un lecteur PNG indépendant vérifie CRC, décompression/filtres, dimensions, RGBA, empreinte et marge transparente réelle, ainsi que l’inventaire des identités. Les images sont aussi inspectées visuellement; ce contrôle ne remplace pas les mesures d’accessibilité ni la livraison trois OS. Les anciens schémas/projets et les captures historiques restent conservés.
