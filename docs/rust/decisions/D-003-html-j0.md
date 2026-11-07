# D-003 — Document HTML natif de prototype

Le module `sns-html` construit un document de données avec texte/titres, liens, images et formulaires POST. Il n’a ni chargeur, ni moteur de script, ni accès hôte. `sns-ui-egui` transforme les actions en messages du moteur. Le serveur de qualification fournit ses routes depuis une fixture embarquée; aucune connexion HTTP réelle n’est ouverte.

Le sous-ensemble accepté est : html/body/main, h1/h2, p/br, a, img, form/input/label/button et strong/em/b/i. Le style typographique riche des balises inline n’est pas encore reproduit. Les attributs sont en liste autorisée, leurs valeurs sont obligatoirement citées, les images ont une source et un texte alternatif, les formulaires emploient POST et les champs sont textuels. Les balises non reconnues produisent une erreur; le document ne lance jamais JavaScript ou un navigateur externe.

Tous les chemins sont virtuels absolus. Les schémas d’URL, références réseau `//`, traversées `..`, antislashs, caractères NUL et chemins encodés susceptibles de contourner la politique sont rejetés. Les entités sont décodées avant cette validation. La limite est 64 Kio de source HTML, 2 048 éléments de lecture et 16 niveaux; un formulaire est borné à 32 champs. Le parseur n’est pas présenté comme un parseur HTML5 généraliste.

Les images du prototype sont des PNG obtenus par requête événementielle puis décodés sous quota : 1 Mio de corps décodé, 1 024 × 1 024 pixels et 8 Mio d’allocation du décodeur. Les textures proviennent uniquement de ces réponses. Les autres formats requis par Filius, le CSS et l’ensemble des pages d’administration doivent être implémentés/recettés à J4/J5.

Preuves de prototype : lien aller/retour, image effectivement décodée, champ initial et POST retourné par le service événementiel, plus tests de rejet des scripts, ressources externes, entités piégées et balisages invalides. Ces échanges portent la famille `prototype.message.*`; ils ne prouvent pas une pile TCP/IP ni un HTTP filaire conforme.
