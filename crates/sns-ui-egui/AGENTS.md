# Présentation native Sinesi

Lire le [guide de développement](../../docs/rust/DEVELOPPEMENT.md) et la
[carte actuelle](../../docs/rust/ARCHITECTURE.md) avant de changer une frontière.

- `src/workbench/mod.rs` possède la coordination et l’état de présentation.
  Les modules `canvas`, `inspector`, `chrome`, `document`, `commands` et les
  panneaux spécialisés portent leurs responsabilités. Un nouveau panneau va
  dans son module; ne pas reconstituer un fichier central contenant tous les écrans.
- Le worker est l’unique propriétaire du moteur. Un panneau lit la projection
  appliquée et poste des commandes; ni le nombre de frames ni un dessin ne font
  avancer un protocole. La configuration sauvegardée n’est pas le réseau effectif.
- Les actions sans interface et les tests utilisent les mêmes transactions.
  Un sélecteur natif fournit un chemin à cette action; il ne contient pas de logique métier.
- Les sous-modules restent privés. N’élargir la visibilité que jusqu’au parent
  qui compose les panneaux; ne pas exposer les structures internes pour contourner
  leur propriétaire. Pas de nouveau `#[path]` pour simuler une arborescence.
- `src/prototype/` contient l’expérience J0 et son contrôleur. Le produit courant
  utilise `Workbench`; ne pas ajouter de fonctionnalité produit au prototype.
  Ses tests et références restent identifiables, séparés du workbench réseau.
- Rendu statique : réutiliser symboles, fontes et placement communs. L’export
  travaille sur une scène capturée; aucune dépendance de l’export vers les widgets.
- Conserver les captures historiques. Une évolution visuelle documente ses
  références courantes et la raison du changement; un déplacement de fichiers
  doit comparer les références existantes sans les régénérer.
