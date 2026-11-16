# Duplication et alignement — J7

F-TOP-009 complète l’édition réversible de topologie. Les prérequis de conception et d’historique restent ceux du [LAN](LAN_J1.md); la [matrice](../suivi/MATRICE.md) relie ce comportement aux réalisations et preuves.

## Copie d’une sélection

La copie est une transaction de conception. Elle conserve les paramètres et données de la sélection et attribue de nouvelles identités aux équipements, interfaces, logiciels, câbles internes et annotations. Les nouvelles MAC sont locales, unicast et distinctes de toutes celles déjà présentes, y compris après plusieurs copies.

Les câbles dont les deux extrémités appartiennent aux équipements copiés sont inclus automatiquement. Les câbles sélectionnés vers un équipement extérieur sont recensés comme omis; une sélection composée uniquement de câbles est refusée. Les ports originaux restent occupés par leurs propres câbles. Le nombre et l’ordre des interfaces des équipements copiés sont conservés.

Les références internes sont remappées : interfaces des routes, rôles LAN/WAN, interface du serveur DHCP, réservations visant les MAC de clients copiés et clés d’association radio. Une copie de point d’accès reçoit un SSID disponible, borné à 32 octets et valide en texte. Les clients copiés avec ce point d’accès utilisent son nouveau SSID. Les références vers un équipement extérieur restent attachées à leur cible d’origine.

Les logiciels installés, leurs réglages et les fichiers virtuels sont copiés. Le courrier durable garde corps, destinataires, dossiers et contacts, avec de nouvelles identités de messages et contacts. Un même message présent dans un serveur et un client copiés conserve une identité logique commune dans la copie. Les ressources et pièces d’extension du projet sont partagées par référence. La copie de réglages de modem n’ouvre aucun tunnel réel.

Le nom de chaque équipement reçoit un suffixe numérique disponible. Les noms et SSID longs sont raccourcis à une frontière de texte valide pour réserver la place du suffixe. Le contenu de l’original est conservé. Les coordonnées copiées reçoivent un décalage; l’interface place le groupe à droite du réseau et le rend visible par un cadrage.

La transaction refuse une référence inconnue, une position invalide, un dépassement du nombre d’équipements ou un résultat dépassant 8 Mio sérialisés. Un refus conserve projet, révision et historique. Annuler/rétablir restitue les mêmes identités, MAC, liens et données; les copies ne sont pas régénérées lors du rétablissement.

## IPv4 conservées et avertissements

La copie conserve les adresses configurées. Les IPv4 statiques identiques sont regroupées par adresse avec leurs équipements/interfaces. Elles sont signalées comme conflits **possibles**, car des segments isolés peuvent légitimement réutiliser une adresse. Ce contrôle ne modifie pas le document et ne bloque pas l’ouverture ou la simulation.

L’interface explique cette distinction, permet de développer chaque groupe et de sélectionner l’équipement concerné. Modifier une adresse met le diagnostic à jour. Une attribution DHCP future n’est pas assimilée à une adresse statique déjà configurée.

## Alignement et répartition

Les outils concernent les équipements sélectionnés :

- Gauche, centre horizontal et droite : même coordonnée X, choisie au minimum, au milieu des bornes ou au maximum.
- Haut, centre vertical et bas : mêmes règles sur Y.
- Répartition horizontale ou verticale : positions régulièrement espacées entre les deux bornes; l’autre coordonnée est conservée.

Les boîtes d’équipement ont les mêmes dimensions; ces règles alignent également leurs bords ou centres visibles. Deux équipements sont requis pour aligner et trois pour répartir. Les égalités sont départagées par identité stable. Les identités, MAC, configurations et connexions restent inchangées. Un alignement sans changement ne crée pas d’étape d’historique.

La duplication est disponible par bouton et raccourci de commande D; les outils d’alignement/répartition sont dans la palette de conception. Les opérations de topologie restent verrouillées en simulation. La transaction accepte aussi une sélection d’annotations seules; leurs interactions de sélection et d’édition dans le mode de documentation relèvent de F-DOC-001/002.

FR EN ES IT PT DE et les vues Icônes/Schéma utilisent les mêmes données. Le changement de vue n’est pas une édition de topologie. Le format du projet reste inchangé pour cette tranche.
