# D-014 — facteur d’attente et reprise fidèle

Décision du 9 novembre 2026, F-CFG-005 / RS-PLATFORM-J7 et non-régression du moteur J7.

Dans le référentiel en lecture seule, `hardware/Verbindung.java` borne le facteur
RTT entre 1 et 5 et l’applique à l’estimation renvoyée par `holeRTT`. Les appelants
attendent ARP, ICMP, TCP, DHCP et DNS. Ce facteur ne multiplie pas le délai physique
des câbles. Le système événementiel Sinesi ne doit pas introduire des attentes
réelles ni détourner le réglage de vitesse de l’interface pour préserver cette intention.

Le [contrat portable](../../agnostique/ATTENTES_J7.md) distingue les attentes de
réponse des durées de vie et cadences. Le facteur appartient à l’exécution; il est
initialisé avant les effets de démarrage et conservé dans la provenance du rejeu.
Un checkpoint reconstruit sa propre politique. La demande CLI reste distincte et
sert à une nouvelle simulation, sans contaminer les préférences d’affichage.

Le facteur standard conserve les octets et profils historiques; une politique
étendue utilise les versions 2 du journal et du checkpoint. Cette version explicite
empêche un ancien moteur de rejouer une politique étendue en ignorant son facteur.
L’encodage des grands entiers et associations ne change pas. Une continuation avec
une autre politique est refusée avant application. Une fixture version 1 scellée
reste restaurable; les tests de tous les domaines vérifient les chemins standard.

Les seules dépendances nouvelles du graphe sont internes : le lanceur desktop
nomme le type du modèle et déclare donc `sns-model`. La politique de couches est
adaptée, sans masquer cette arête dans une réexportation d’interface.

Les [preuves](../../verification/J7-ATTENTES.md) distinguent recette virtuelle,
tests du harnais et parcours natifs. F-CFG-005 n’est pas fermée : v/l/n/s et le
journal technique restent à traiter, ainsi que les autres exigences de J7.
