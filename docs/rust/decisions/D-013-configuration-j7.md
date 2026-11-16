# D-013 — configuration par couches et options temporaires

Décision du 8 novembre 2026, F-CFG-004, RS-PLATFORM-J7; contribution partielle à
F-CFG-005. [Comportement portable](../../agnostique/CONFIGURATION_J7.md).

La configuration administrateur fournit des défauts, puis le profil et les options
CLI les remplacent champ par champ. Un instantané avant CLI sert aux sauvegardes.
Copier tout l’état effectif lors d’un changement de langue aurait figé des options
temporaires; seules les modifications explicites sont donc appliquées à cet instantané.
Un reset volontaire garde une sémantique complète, y compris pour des valeurs masquées.

Le profil utilisateur reste 1/2; la configuration administrateur et le rapport ont
chacun leur version 1. Un champ invalide conserve la couche inférieure; un profil
invalide n’est pas réécrit. Les diagnostics sont bornés et structurés, sans valeurs
rejetées. Le lecteur strict de préférences garde son contrat de validation de données;
le résolveur possède le repli de démarrage et les origines. Les bornes d’affichage et
la normalisation numérique sont partagées.

Le lanceur et l’interface utilisent la même résolution et le même constructeur de
projet. Les entrées natives restent typées comme chemins; le contrôle sans fenêtre
ne publie ni chemins arbitraires ni données pédagogiques. Les changements techniques
sont confinés à l’adaptateur, aux contrats et à leurs fixtures. Les vingt membres
restent contrôlés; sns-desktop dépend maintenant explicitement de sns-settings.

Cette tranche ne remplace pas le RTT par une vitesse d’affichage et ne fait pas passer
la préparation des options pour leur réalisation. F-CFG-005 reste en cours pour
v/l/r/n/s, puis diagnostics complets et livraison gardent leurs recettes propres.
