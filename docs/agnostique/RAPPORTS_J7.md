# Rapports de réseau — J7

F-DOC-004 complète [les images du réseau](EXPORTS_J7.md). Le rapport contient le
schéma annoté, les configurations, interfaces, logiciels, DNS, routes et échanges
émis/reçus regroupés par interface MAC. Le comportement est identique en
FR EN ES IT PT DE et dans les représentations Icônes/Schéma.

## Instantané

Une demande fige projet appliqué, représentation, langue, révision et, si une
simulation est ouverte, état effectif, routes et historique disponibles. Elle ne
fait ni avancer ni arrêter le réseau. Le document ne dépend pas du viewport ou
des widgets. Une édition ultérieure ne change pas l’export déjà engagé.

Le rapport d’un projet seul montre sa configuration et annonce l’absence de
paquets. Un état sauvegardé de simulation fournit également les valeurs effectives
et le trafic enregistré. Le chargement de cet état vérifie son intégrité; il ne
continue pas l’exercice et n’ouvre pas le pont réel. Les brouillons non appliqués
et les mots de passe de configuration ne sont pas exportés.

Configuration persistante et état effectif sont affichés séparément : un client
reste configuré en DHCP même si son adresse attribuée apparaît dans la colonne
voisine. Passerelle et DNS suivent cette distinction. Les routes effectives
incluent les routes directes, statiques, par défaut et apprises disponibles dans
l’instantané. Elles ne sont pas recalculées à partir d’une configuration DHCP sans
adresse. Le rapport de conception utilise les routes configurées connues.

## Contenu et pagination

1. Vue complète du réseau, notes et styles, cadrée sur tous les objets.
2. Une section de configuration par équipement : identité, passerelle, DNS,
   transfert IPv4 et RIP; interfaces avec MAC, adressage configuré et effectif;
   logiciels installés; enregistrements DNS avec type, valeur, TTL et priorité MX;
   routes avec réseau/préfixe, prochain saut, interface, origine et métrique.
   Les redirections NAT configurées sont également listées.
3. Échanges par interface MAC, dans l’ordre temporel de chaque groupe : numéro,
   temps virtuel, émission/réception, MAC source/destination, type d’événement,
   identité d’exécution et corrélation; en-têtes réseau/transport disponibles.

Le rapport de trafic retient les faits canoniques d’émission/réception Ethernet,
y compris ceux du médium sans fil. Les faits supplémentaires d’association ou de
transmission radio ne dupliquent pas ces lignes. Une émission abandonnée peut donc
avoir sa ligne sans réception correspondante. Les autres décisions de simulation
restent dans l’historique complet et ses exports structurés. Les corps de messages
et contenus de fichiers ne sont pas recopiés dans le tableau de paquets.

Une trace tronquée est annoncée; elle n’est pas présentée comme la totalité de
l’exercice. Un état restauré peut avoir une nouvelle génération : l’identité des
événements passés reste celle enregistrée, distincte de celle de l’état courant.
Les interfaces et équipements des paquets doivent appartenir au projet capturé.

Le papier est A4 portrait. Titres de section, en-têtes de tableau et numéros de
page se répètent; un tableau continué garde son contexte MAC. Les cellules se
replient et peuvent se poursuivre sur plusieurs pages sans suppression de lignes.
Un champ vide et une section sans données sont annoncés explicitement.

## Texte et rendu

Les quatre polices embarquées de l’export d’image sont utilisées. Le texte est
sélectionnable, avec correspondance Unicode, accents, ligatures et styles. Les
formes et textes de la scène sont conservés; les symboles suivent la même source
visuelle. Le rapport papier utilise le fond opaque de la représentation choisie.
Les débordements d’annotations ou de légendes déjà définis dans la scène restent
signalés dans le résultat. Les panneaux, poignées et animations sont absents.

Le profil couvre les six langues latines prévues. Un caractère absent ou une
direction non prise en charge refuse l’export avant écriture. Le canevas reste
éditable avec un caractère de remplacement visuel; le document conserve le texte
original et l’export ne substitue pas silencieusement un carré dans le PDF.

## Bornes et écriture

Le rapport accepte au plus 100 000 événements, 512 pages, 8 Mio de texte de tables,
2 000 000 glyphes de tables et 64 Mio de PDF encodé, en plus des bornes du projet
et de sa scène. Une saturation produit une erreur; elle ne publie pas un rapport
partiel qui semblerait complet. Ces plafonds ne sont pas une mesure de performance
globale de l’application.

PNG et PDF partagent une seule opération documentaire en cours dans la GUI.
Sélection native, génération et écriture se déroulent hors de la frame. Le résultat
annonce le nombre de pages, paquets, groupes et les éventuelles limites de trace
ou de texte. La destination est réservée et la fermeture normale attend la fin.
Projet ouvert, préférences et stockage géré de récupération restent protégés.
Le lanceur refuse aussi de remplacer son fichier source.

Le PDF entier est préparé avant un remplacement atomique. Sur erreur, le fichier
précédent, le projet et le réseau restent inchangés. Le rapport n’embarque ni code
actif, ni formulaire ou accès automatique à une ressource externe.

## Recette de la tranche

Construire un réseau routé avec DNS, client DHCP, logiciel de courrier et annotations
stylées; lancer un ping. Exporter les deux vues dans les six langues. Un lecteur
indépendant vérifie sections, adressage effectif distinct de DHCP, compte exact des
paquets, polices embarquées et Unicode. Le mot de passe fictif doit rester absent.
Inspecter le rendu et les pages de continuation. Une grande trace doit conserver
toutes ses lignes, et un historique incomplet doit porter son avertissement.

Comparer des exports indépendants, vérifier l’absence de mutation et protéger une
destination existante sur erreur. Tester les commandes graphiques et sans interface,
ainsi que la capture en simulation. Cette tranche contribue à R-RAPPORT; elle ne
ferme pas ses validations de configuration, accessibilité et livraison trois OS.
