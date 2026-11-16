# Annotations et zones — J7

Ce profil porte F-DOC-001/002 dans RS-REPORTING-J7. Les annotations appartiennent au projet portable et à son historique d’édition. Elles sont distinctes des états de protocoles, des préférences et des faits de trafic. Les [preuves](../verification/J7-ANNOTATIONS.md) qualifient cette tranche; PNG/PDF restent des exigences séparées.

## Projet et géométrie

Le projet **9**, profil `sns-educational-documentation-v1`, ajoute `style` aux annotations `{id, text, x, y}`. Le [schéma autonome](../../contracts/project-documentation.schema.json) et l’[exemple](../../fixtures/valid/annotations.json) constituent les références. Les projets 1 à 8 gardent leur ancien format : `style` y est refusé et reste absent de leur sérialisation. Ils restent lisibles. Modifier une annotation avec dimensions/styles promeut le document au profil 9, sans modifier les équipements.

`x` et `y` désignent le coin supérieur gauche en coordonnées du document. Ils restent finis dans ±1 000 000. `width` et `height` mesurent la zone entière, chacune entre 24 et 10 000. Le texte comporte au plus 65 536 octets UTF-8; le document garde sa borne de 8 Mio et au plus 4 096 annotations. Identités, dimensions et textes sont validés avant publication d’une édition. Une erreur conserve projet, historique et révision.

L’ajout attribue une nouvelle identité. Une modification conserve identité et position dans l’ordre d’empilement. Un fin contour de contraste conserve la visibilité des câbles sur les fonds colorés. Les zones et notes sont peintes **avant les câbles et équipements**, dans l’ordre du document; elles restent derrière les nœuds dans les deux vues. Une édition n’avance pas le temps virtuel et ne reconfigure aucun protocole. Les contrôles de documentation sont disponibles dans leur mode dédié; un changement de mode conserve la règle générale d’arrêt de la simulation.

## Style portable

| Champ | Sémantique |
| --- | --- |
| `kind` | `text` ou `rectangle`; la zone structurelle a par défaut un fond translucide et un contour |
| `width`, `height` | Dimensions persistantes; note neuve 240 × 100, zone neuve 400 × 260 |
| `font_size` | Taille entre 8 et 96 en unités du document |
| `font` | `proportional` ou `monospace` |
| `bold` | Gras, facultatif avec valeur par défaut fausse |
| `italic`, `underline` | Italique et soulignement, combinables avec le gras |
| `text_color`, `fill_color`, `border_color` | Quatre canaux sRGB RGBA non prémultipliés, chacun entier 0 à 255 |
| `border_width` | Épaisseur de 0 à 20; zéro masque le contour |

Le gras sélectionne une variante de police dédiée. Les ressources de police embarquées évitent un téléchargement ou une installation système à l’exécution; les six langues latines requises ont une recette de couverture. Le support universel des écritures n’est pas revendiqué. L’italique applique une inclinaison de présentation à la police sélectionnée. Le texte revient à la ligne dans la largeur disponible et est mis en page dans la zone intérieure (marge de 6) et découpé au cadre extérieur; un dépassement ne change pas les dimensions enregistrées. L’utilisateur peut agrandir la zone pour voir la suite. Les premiers textes sans style conservent un rendu de note simple à 14 unités avec dimensions de vue par défaut, sans ajout de champ tant qu’ils ne sont pas édités.

## Interactions

Le mode Documentation propose ajout de note ou de zone colorée, puis liste de sélection. L’inspecteur permet de saisir texte, position, dimensions et style. **Appliquer** publie une seule transaction; **Recharger** reprend les valeurs enregistrées dans le document. Supprimer est réversible. Les contrôles numériques et les libellés de sélection sont accessibles au clavier; toutes les commandes sont traduites en FR EN ES IT PT DE.

Le déplacement utilise un aperçu depuis la géométrie initiale; le relâchement publie une seule édition. La poignée inférieure droite redimensionne avec minima et maxima bornés. La transformation utilise le zoom courant : un déplacement de pixels est converti en unités du document. Les équipements gardent la priorité d’interaction au-dessus des zones. Le bouton Cadrer inclut notes et rectangles, y compris un document sans équipement. Une zone hors vue est accessible par la liste; la poignée n’est interactive que lorsqu’elle coupe le canevas.

Chaque changement appliqué passe par annuler/rétablir. Les styles sont copiés avec les annotations lors de la duplication, en renouvelant leur identité. Ouvrir, enregistrer et reprendre un checkpoint ou une session admettent le projet 9. La récupération conserve aussi un brouillon d’inspecteur d’annotation **sans l’appliquer**; il est versionné et borné avec les autres saisies du [profil session](SESSION_J7.md).

## Limites et liens

L’interface courante ne propose ni rotation, ni ancrage automatique des zones aux équipements, ni mise en forme différente par portion de texte. Le gras/italique/soulignement s’applique à la note entière. Les rectangles sont documentaires et ne créent aucun sous-réseau. La conversion des styles historiques d’import reste partielle et ne peut pas être déduite de ces contrôles d’édition.

Les exports PNG et PDF doivent consommer ces mêmes données de scène; leur qualification reste F-DOC-003/004. Les contrôles de lisibilité générale, la suite graphique globale et la livraison sur trois OS restent ouverts dans J7. La [matrice](../suivi/MATRICE.md) relie ce profil à l’adaptateur.
