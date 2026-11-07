# D-007 — Provenance et licence du socle Sinesi

Le code neuf de Sinesi est distribué sous licence MIT, indiquée au workspace et dans `LICENSE`. Le dépôt cible est `Kevin-FR/sinesi`; il était vide lors de l’initialisation locale.

Le corpus Filius a servi à établir l’inventaire de capacités. Son moteur, ses sources Java, ses icônes, ses scénarios binaires et son runtime ne sont pas intégrés au nouveau binaire. `_archive/` est exclu du suivi Git. Les inventaires, références et empreintes du cadrage identifient ce corpus séparément; ses fichiers conservent leurs licences et notices d’origine. La licence MIT du code neuf ne réattribue pas ce corpus.

Le découpage CADCraft/craft-libs proposé par l’utilisateur et leurs manifestes publics ont inspiré les frontières de l’adaptateur. Aucun code de ces projets et aucune dépendance Craft ne sont inclus dans J0. Les algorithmes et fonctions Sinesi ont été écrits pour ses contrats.

Les dépendances tierces gardent leurs licences. Le lockfile fixe leurs versions; egui embarque ses polices via `epaint_default_fonts`, qui possède ses propres attributions. Avant une distribution binaire, produire le bundle de notices et vérifier les actifs réellement embarqués pour les plateformes livrées. La déclaration opensource du socle n’annonce pas une distribution binaire déjà publiée.

La fixture d’image du prototype est un PNG géométrique généré dans ce projet; les textes HTML et scénarios J0 sont des fixtures Sinesi. Ils ne reprennent pas d’image ou de contenu pédagogique de l’archive.
