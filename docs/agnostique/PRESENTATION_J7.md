# Présentation des commutateurs — J7

F-EQU-008 permet de représenter un commutateur par un nuage. Ce choix est une propriété de présentation : identité, ports, connexions, table MAC, rétention, temps virtuel, paquets en transit et génération restent identiques. Le nuage utilise les illustrations fournies pour chacune des vues Icônes/Schéma. Il ne représente ni un nouveau protocole ni une connexion Internet.

En conception, le changement est réversible par l’historique. En simulation, une commande de présentation modifie le document courant sans reconstruire le réseau. Elle est acquittée, enregistrée et rejouée comme les autres commandes. Un refus ne modifie ni le document ni l’état réseau. La représentation nuage est réservée aux commutateurs; les six langues FR EN ES IT PT DE exposent le même réglage.

## Formats et compatibilité

Le projet **8**, profil `sns-educational-presentation-v1`, ajoute la valeur `cloud` au symbole de présentation des équipements. Les projets 1 à 7 restent lisibles et ne peuvent pas déclarer cette valeur. Le premier choix nuage promeut le projet au format 8; revenir à la représentation ordinaire ne rétrograde pas le format. Le champ de symbole absent ou nul choisit la représentation ordinaire correspondant au type d’équipement.

La commande API 1 `presentation.symbol.set` porte l’identité `device` et le symbole `symbol`, éventuellement nul. Une modification effective produit le fait **10** `presentation.symbol.changed`, avec symbole et version du projet. Une répétition de la même enveloppe reste idempotente. Les scénarios, journaux et checkpoints acceptent cette commande et conservent le document de présentation dans l’état complet. Les anciens profils de faits restent interprétables selon leur version.

L’import Filius lit le booléen `cloud` du commutateur comme une donnée. Une valeur vraie est convertie vers le profil 8; une valeur mal formée est signalée dans le rapport et l’original demeure conservé inactif. Aucun contenu historique n’est exécuté.

## Recette

Faire communiquer deux postes à travers un commutateur pour remplir sa table MAC. Lancer une nouvelle communication, conserver un paquet en transit, puis activer le nuage. Comparer exactement ports, table MAC, file, applications, temps et état aléatoire avant/après. Terminer la communication, revenir au symbole ordinaire, enregistrer/recharger et rejouer le journal : seuls le document de présentation et ses faits doivent refléter ces changements.

La [matrice](../suivi/MATRICE.md) relie cette recette à l’adaptateur; le [rapport local](../verification/J7-PRESENTATION.md) donne les preuves et les limites de qualification.
